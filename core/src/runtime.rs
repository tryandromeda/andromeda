// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::{
    any::Any,
    borrow::BorrowMut,
    cell::RefCell,
    collections::VecDeque,
    path::{Path, PathBuf},
    str,
    sync::{atomic::Ordering, mpsc::Receiver},
};

use nova_vm::{
    ecmascript::{
        builtins::promise_objects::promise_abstract_operations::promise_capability_records::PromiseCapability,
        execution::{
            Agent, JsResult,
            agent::{ExceptionType, GcAgent, HostHooks, Job, Options, RealmRoot},
        },
        scripts_and_modules::{
            module::module_semantics::{
                ModuleRequest, Referrer,
                abstract_module_records::AbstractModule,
                cyclic_module_records::GraphLoadingStateRecord,
                finish_loading_imported_module,
                source_text_module_records::{SourceTextModule, parse_module},
            },
            script::{HostDefined, parse_script, script_evaluation},
        },
        types::{
            self, InternalMethods, IntoObject, IntoValue, Object, OrdinaryObject,
            PropertyDescriptor, PropertyKey, String as NovaString, Value,
        },
    },
    engine::{
        context::{Bindable, GcScope, NoGcScope},
        rootable::Scopable,
    },
};

use crate::{
    AndromedaError, AndromedaResult, Extension, HostData, MacroTask, exit_with_parse_errors,
    module::ImportMap,
};

pub struct RuntimeHostHooks<UserMacroTask> {
    pub(crate) promise_job_queue: RefCell<VecDeque<Job>>,
    pub(crate) host_data: HostData<UserMacroTask>,
    pub(crate) base_path: PathBuf,
    pub(crate) import_map: Option<ImportMap>,
}

impl<UserMacroTask> std::fmt::Debug for RuntimeHostHooks<UserMacroTask> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Runtime").finish()
    }
}

impl<UserMacroTask> RuntimeHostHooks<UserMacroTask> {
    pub fn new(host_data: HostData<UserMacroTask>) -> Self {
        Self {
            promise_job_queue: RefCell::default(),
            host_data,
            base_path: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            import_map: None,
        }
    }

    pub fn with_base_path(host_data: HostData<UserMacroTask>, base_path: PathBuf) -> Self {
        Self {
            promise_job_queue: RefCell::default(),
            host_data,
            base_path,
            import_map: None,
        }
    }

    pub fn with_import_map(
        host_data: HostData<UserMacroTask>,
        base_path: PathBuf,
        import_map: ImportMap,
    ) -> Self {
        Self {
            promise_job_queue: RefCell::default(),
            host_data,
            base_path,
            import_map: Some(import_map),
        }
    }

    pub fn pop_promise_job(&self) -> Option<Job> {
        self.promise_job_queue.borrow_mut().pop_front()
    }

    pub fn any_pending_macro_tasks(&self) -> bool {
        self.host_data.macro_task_count.load(Ordering::Acquire) > 0
    }

    /// Resolve a module specifier relative to a referrer path
    fn resolve_module_specifier(&self, specifier: &str, referrer_path: &Path) -> String {
        // Try import map resolution first for bare specifiers
        if let Some(import_map) = &self.import_map
            && !specifier.starts_with("./")
            && !specifier.starts_with("../")
            && !specifier.starts_with("/")
            && !specifier.contains("://")
        {
            // This is a bare specifier, try import map resolution
            let base_url = referrer_path.to_string_lossy();
            if let Some(mapped_specifier) = import_map.resolve_specifier(specifier, Some(&base_url))
            {
                // Use the mapped specifier for resolution
                return mapped_specifier;
            }
        }

        // Handle HTTP URLs directly
        if specifier.starts_with("http://") || specifier.starts_with("https://") {
            specifier.to_string()
        } else {
            // Check if referrer is a URL (HTTP/HTTPS)
            let referrer_str = referrer_path.to_string_lossy();
            if referrer_str.starts_with("http://") || referrer_str.starts_with("https://") {
                // Use URL joining for URL-to-URL resolution
                self.resolve_url_specifier(specifier, &referrer_str)
            } else {
                // Use file path resolution for file-to-file resolution
                self.resolve_path_specifier(specifier, referrer_path)
                    .to_string_lossy()
                    .to_string()
            }
        }
    }

    /// Resolve a path-based specifier (internal helper)
    fn resolve_path_specifier(&self, specifier: &str, referrer_path: &Path) -> PathBuf {
        if specifier.starts_with("http://") || specifier.starts_with("https://") {
            // For HTTP URLs, return the URL as a path-like string
            // We'll handle this specially in the loading logic
            PathBuf::from(specifier)
        } else if specifier.starts_with("./") || specifier.starts_with("../") {
            // Relative import
            let referrer_dir = referrer_path.parent().unwrap_or(&self.base_path);
            referrer_dir.join(specifier)
        } else if specifier.starts_with("/") {
            // Absolute import
            PathBuf::from(specifier)
        } else {
            // Relative to base path or bare specifier
            self.base_path.join(specifier)
        }
    }

    /// Resolve a URL-based specifier (internal helper for URL-to-URL resolution)
    fn resolve_url_specifier(&self, specifier: &str, referrer_url: &str) -> String {
        if specifier.starts_with("http://") || specifier.starts_with("https://") {
            // Already a full URL
            specifier.to_string()
        } else {
            // Use URL joining for relative imports
            match url::Url::parse(referrer_url) {
                Ok(base_url) => {
                    match base_url.join(specifier) {
                        Ok(resolved_url) => resolved_url.to_string(),
                        Err(_) => {
                            // If URL joining fails, fall back to simple concatenation
                            // This shouldn't happen with valid URLs, but provides a fallback
                            if let Some(stripped) = specifier.strip_prefix("./") {
                                format!("{}/{}", referrer_url.trim_end_matches('/'), stripped)
                            } else if let Some(stripped) = specifier.strip_prefix("../") {
                                // Simple fallback for parent directory navigation
                                let mut base = referrer_url.trim_end_matches('/');
                                if let Some(last_slash) = base.rfind('/') {
                                    base = &base[..last_slash];
                                }
                                format!("{base}/{stripped}")
                            } else {
                                format!("{}/{}", referrer_url.trim_end_matches('/'), specifier)
                            }
                        }
                    }
                }
                Err(_) => {
                    // If base URL parsing fails, use simple string concatenation as fallback
                    if let Some(stripped) = specifier.strip_prefix("./") {
                        format!("{}/{}", referrer_url.trim_end_matches('/'), stripped)
                    } else if let Some(stripped) = specifier.strip_prefix("../") {
                        // Simple fallback for parent directory navigation
                        let mut base = referrer_url.trim_end_matches('/');
                        if let Some(last_slash) = base.rfind('/') {
                            base = &base[..last_slash];
                        }
                        format!("{base}/{stripped}")
                    } else {
                        format!("{}/{}", referrer_url.trim_end_matches('/'), specifier)
                    }
                }
            }
        }
    }

    /// Resolve module file with proper extension handling
    fn resolve_extensions(&self, path: PathBuf) -> Option<PathBuf> {
        let path_str = path.to_string_lossy();

        // Handle HTTP URLs - they don't need file system extension resolution
        if path_str.starts_with("http://") || path_str.starts_with("https://") {
            return Some(path);
        }

        // First try the path as-is
        if path.exists() {
            return Some(path);
        }

        // Get the base path without extension for trying alternatives
        let path_stem = path.with_extension("");

        // Try different extensions in order of preference
        for ext in &["ts", "js", "mjs", "json"] {
            let candidate = path_stem.with_extension(ext);
            if candidate.exists() {
                return Some(candidate);
            }
        }

        // If the original import had an extension but we didn't find it,
        // try the path without extension (for cases like './math' -> './math.ts')
        if path.extension().is_none() {
            for ext in &["ts", "js", "mjs", "json"] {
                let candidate = path.with_extension(ext);
                if candidate.exists() {
                    return Some(candidate);
                }
            }
        }

        None
    }
}

impl<UserMacroTask: 'static> HostHooks for RuntimeHostHooks<UserMacroTask> {
    fn enqueue_promise_job(&self, job: Job) {
        self.promise_job_queue.borrow_mut().push_back(job);
    }

    fn get_host_data(&self) -> &dyn Any {
        &self.host_data
    }

    fn load_imported_module<'gc>(
        &self,
        agent: &mut Agent,
        referrer: Referrer<'gc>,
        module_request: ModuleRequest<'gc>,
        _host_defined: Option<HostDefined>,
        payload: &mut GraphLoadingStateRecord<'gc>,
        gc: NoGcScope<'gc, '_>,
    ) {
        // Get the module specifier from the module request
        let specifier = module_request.specifier(agent);
        let specifier_str = specifier.to_string_lossy(agent);

        // Extract referrer information properly from Nova VM
        let referrer_info = referrer.host_defined(agent);
        let referrer_str = if let Some(host_defined) = referrer_info {
            // Try to get the referrer module path from host_defined
            if let Some(path_rc) = host_defined.downcast_ref::<std::rc::Rc<String>>() {
                path_rc.as_str().to_string()
            } else if let Some(path_rc) = host_defined.downcast_ref::<std::rc::Rc<PathBuf>>() {
                path_rc.to_string_lossy().to_string()
            } else if let Some(string_val) = host_defined.downcast_ref::<String>() {
                string_val.clone()
            } else {
                // Fallback to default base path
                self.base_path.to_string_lossy().to_string()
            }
        } else {
            // Use runtime's base_path as fallback
            self.base_path.to_string_lossy().to_string()
        };

        // Resolve the module specifier using the proper referrer
        let resolved_specifier =
            if referrer_str.starts_with("http://") || referrer_str.starts_with("https://") {
                // Referrer is a URL, use URL-based resolution
                self.resolve_url_specifier(&specifier_str, &referrer_str)
            } else {
                // Referrer is a file path, use file-based resolution
                let referrer_path = PathBuf::from(&referrer_str);
                self.resolve_module_specifier(&specifier_str, &referrer_path)
            };

        // For HTTP URLs, skip extension resolution; for file paths, try to resolve extensions
        let final_specifier = if resolved_specifier.starts_with("http://")
            || resolved_specifier.starts_with("https://")
        {
            resolved_specifier
        } else {
            let path_buf = PathBuf::from(&resolved_specifier);
            match self.resolve_extensions(path_buf) {
                Some(path) => path.to_string_lossy().to_string(),
                None => {
                    // Module not found error
                    let error = agent.throw_exception(
                        ExceptionType::TypeError,
                        format!("Module not found: {resolved_specifier}"),
                        gc,
                    );
                    finish_loading_imported_module(
                        agent,
                        referrer,
                        module_request,
                        payload,
                        Err(error),
                        gc,
                    );
                    return;
                }
            }
        };

        // Check if this is a JSON module
        let is_json = final_specifier.ends_with(".json");

        // Read the module source - handle both file system and HTTP URLs
        let source_text =
            if final_specifier.starts_with("http://") || final_specifier.starts_with("https://") {
                // HTTP import - fetch from network
                match ureq::get(&final_specifier).call() {
                    Ok(mut response) => match response.body_mut().read_to_string() {
                        Ok(content) => content,
                        Err(error) => {
                            let error = agent.throw_exception(
                                ExceptionType::TypeError,
                                format!("Failed to read HTTP module {final_specifier}: {error}"),
                                gc,
                            );
                            finish_loading_imported_module(
                                agent,
                                referrer,
                                module_request,
                                payload,
                                Err(error),
                                gc,
                            );
                            return;
                        }
                    },
                    Err(error) => {
                        let error = agent.throw_exception(
                            ExceptionType::TypeError,
                            format!("Failed to fetch HTTP module {final_specifier}: {error}"),
                            gc,
                        );
                        finish_loading_imported_module(
                            agent,
                            referrer,
                            module_request,
                            payload,
                            Err(error),
                            gc,
                        );
                        return;
                    }
                }
            } else {
                // File system import - read from local file
                let file_path = PathBuf::from(&final_specifier);
                match std::fs::read_to_string(&file_path) {
                    Ok(content) => content,
                    Err(error) => {
                        let error = agent.throw_exception(
                            ExceptionType::TypeError,
                            format!("Failed to read module {}: {}", file_path.display(), error),
                            gc,
                        );
                        finish_loading_imported_module(
                            agent,
                            referrer,
                            module_request,
                            payload,
                            Err(error),
                            gc,
                        );
                        return;
                    }
                }
            };

        // Handle JSON modules specially
        let final_source = if is_json {
            // For JSON modules, wrap the JSON in a JavaScript module that exports it as default
            format!("export default {source_text};")
        } else {
            source_text
        };

        // Convert to Nova string
        let source_string = NovaString::from_string(agent, final_source, gc);

        // Get the realm from the referrer
        let realm = agent.current_realm(gc);

        // Parse the module with proper host_defined containing the resolved specifier
        let module_host_defined = Some(std::rc::Rc::new(final_specifier.clone()) as HostDefined);
        match parse_module(agent, source_string, realm, module_host_defined, gc) {
            Ok(module) => {
                let abstract_module = AbstractModule::from(module);
                finish_loading_imported_module(
                    agent,
                    referrer,
                    module_request,
                    payload,
                    Ok(abstract_module),
                    gc,
                );
            }
            Err(errors) => {
                // Parse error
                let error_msg = format!("Parse error in module {final_specifier}: {errors:?}");
                let error = agent.throw_exception(ExceptionType::SyntaxError, error_msg, gc);
                finish_loading_imported_module(
                    agent,
                    referrer,
                    module_request,
                    payload,
                    Err(error),
                    gc,
                );
            }
        }
    }

    fn get_import_meta_properties<'gc>(
        &self,
        agent: &mut Agent,
        module_record: SourceTextModule,
        gc: NoGcScope<'gc, '_>,
    ) -> Vec<(PropertyKey<'gc>, Value<'gc>)> {
        let mut properties = Vec::new();

        // Create import.meta.url property
        let url_key = PropertyKey::from_str(agent, "url", gc);

        // Get the module specifier from the module's host_defined data
        // Convert SourceTextModule to Referrer to access host_defined
        let referrer = Referrer::from(module_record);
        let module_url = if let Some(host_defined) = referrer.host_defined(agent) {
            // Try to get the module specifier from host_defined
            if let Some(specifier_rc) = host_defined.downcast_ref::<std::rc::Rc<String>>() {
                let specifier = specifier_rc.as_str();
                // If it's already a full URL, use it as-is; otherwise make it a file:// URL
                if specifier.starts_with("http://") || specifier.starts_with("https://") {
                    specifier.to_string()
                } else {
                    format!(
                        "file://{}",
                        std::fs::canonicalize(specifier)
                            .unwrap_or_else(|_| std::path::PathBuf::from(specifier))
                            .to_string_lossy()
                    )
                }
            } else if let Some(specifier_string) = host_defined.downcast_ref::<String>() {
                // If it's already a full URL, use it as-is; otherwise make it a file:// URL
                if specifier_string.starts_with("http://")
                    || specifier_string.starts_with("https://")
                {
                    specifier_string.clone()
                } else {
                    format!(
                        "file://{}",
                        std::fs::canonicalize(specifier_string)
                            .unwrap_or_else(|_| std::path::PathBuf::from(specifier_string))
                            .to_string_lossy()
                    )
                }
            } else {
                // Fallback to base_path
                format!(
                    "file://{}",
                    self.base_path.join("script.js").to_string_lossy()
                )
            }
        } else {
            // Fallback to base_path
            format!(
                "file://{}",
                self.base_path.join("script.js").to_string_lossy()
            )
        };

        let url_value = Value::from_string(agent, module_url, gc);

        properties.push((url_key, url_value));

        properties
    }

    fn finalize_import_meta<'gc>(
        &self,
        _agent: &mut Agent,
        _import_meta: types::OrdinaryObject<'gc>,
        _module_record: SourceTextModule<'gc>,
        _gc: NoGcScope<'gc, '_>,
    ) {
        // Default implementation - no additional processing needed
    }
}

pub type EventLoopHandler<UserMacroTask> = fn(
    macro_task: UserMacroTask,
    agent: &mut GcAgent,
    realm_root: &RealmRoot,
    host_data: &HostData<UserMacroTask>,
);

pub enum RuntimeFile {
    Embedded {
        path: String,
        content: &'static [u8],
    },
    Local {
        path: String,
    },
}

impl RuntimeFile {
    fn read(&self) -> AndromedaResult<String> {
        match self {
            RuntimeFile::Embedded { path: _, content } => {
                Ok(String::from_utf8_lossy(content).into_owned())
            }
            RuntimeFile::Local { path } => std::fs::read_to_string(path)
                .map_err(|e| Box::new(AndromedaError::fs_error(e, "read", path))),
        }
    }

    fn get_path(&self) -> &str {
        match self {
            RuntimeFile::Embedded { path, content: _ } => path,
            RuntimeFile::Local { path } => path,
        }
    }

    fn validate(&self) -> AndromedaResult<()> {
        match self {
            RuntimeFile::Embedded { .. } => Ok(()),
            RuntimeFile::Local { path } => {
                let path_obj = std::path::Path::new(path);
                if !path_obj.exists() {
                    return Err(Box::new(AndromedaError::fs_error(
                        std::io::Error::new(
                            std::io::ErrorKind::NotFound,
                            format!("File not found: {path}"),
                        ),
                        "validate",
                        path,
                    )));
                }
                if !path_obj.is_file() {
                    return Err(Box::new(AndromedaError::fs_error(
                        std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            format!("Path is not a file: {path}"),
                        ),
                        "validate",
                        path,
                    )));
                }
                std::fs::File::open(path)
                    .map_err(|e| AndromedaError::fs_error(e, "validate", path))?;
                Ok(())
            }
        }
    }
}

pub struct RuntimeConfig<UserMacroTask: 'static> {
    /// Disable or not strict mode.
    pub no_strict: bool,
    /// List of js files to load.
    pub files: Vec<RuntimeFile>,
    /// Enable or not verbose outputs.
    pub verbose: bool,
    /// Collection of Rust Extensions
    pub extensions: Vec<Extension>,
    /// Collection of builtin js sources
    pub builtins: Vec<&'static str>,
    /// User event loop handler.
    pub eventloop_handler: EventLoopHandler<UserMacroTask>,
    /// Macro tasks eventloop receiver.
    pub macro_task_rx: Receiver<MacroTask<UserMacroTask>>,
    /// Import map for module resolution
    pub import_map: Option<ImportMap>,
}

pub struct Runtime<UserMacroTask: 'static> {
    pub config: RuntimeConfig<UserMacroTask>,
    pub agent: GcAgent,
    pub realm_root: RealmRoot,
    pub host_hooks: &'static RuntimeHostHooks<UserMacroTask>,
}

#[cfg_attr(feature = "hotpath", hotpath::measure_all)]
impl<UserMacroTask> Runtime<UserMacroTask> {
    /// Create a new [Runtime] given a [RuntimeConfig]. Use [Runtime::run] to run it.
    pub fn new(
        mut config: RuntimeConfig<UserMacroTask>,
        host_data: HostData<UserMacroTask>,
    ) -> Self {
        // Determine base path from the first file
        let base_path = config
            .files
            .first()
            .and_then(|file| {
                if let RuntimeFile::Local { path } = file {
                    Some(std::path::Path::new(path).parent()?.to_path_buf())
                } else {
                    None
                }
            })
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

        let host_hooks = if let Some(import_map) = config.import_map.clone() {
            RuntimeHostHooks::with_import_map(host_data, base_path, import_map)
        } else {
            RuntimeHostHooks::with_base_path(host_data, base_path)
        };

        let host_hooks: &RuntimeHostHooks<UserMacroTask> = &*Box::leak(Box::new(host_hooks));
        let mut agent = GcAgent::new(
            Options {
                disable_gc: false,
                print_internals: config.verbose,
            },
            host_hooks,
        );
        let create_global_object: Option<for<'a> fn(&mut Agent, GcScope<'a, '_>) -> Object<'a>> =
            None;
        let create_global_this_value: Option<
            for<'a> fn(&mut Agent, GcScope<'a, '_>) -> Object<'a>,
        > = None;
        let realm_root = agent.create_realm(
            create_global_object,
            create_global_this_value,
            Some(
                |agent: &mut Agent, global_object: Object<'_>, mut gc: GcScope<'_, '_>| {
                    let andromeda_obj = OrdinaryObject::create_empty_object(agent, gc.nogc())
                        .scope(agent, gc.nogc());
                    let property_key =
                        PropertyKey::from_static_str(agent, "__andromeda__", gc.nogc());
                    global_object
                        .internal_define_own_property(
                            agent,
                            property_key.unbind(),
                            PropertyDescriptor {
                                value: Some(andromeda_obj.get(agent).into_value()),
                                writable: Some(true),
                                enumerable: Some(false),
                                configurable: Some(true),
                                ..Default::default()
                            },
                            gc.reborrow(),
                        )
                        .unwrap();

                    for extension in &mut config.extensions {
                        extension.load::<UserMacroTask>(
                            agent,
                            global_object,
                            andromeda_obj.get(agent).into_object(),
                            gc.borrow_mut(),
                        );
                    }
                },
            ),
        );

        Self {
            config,
            agent,
            realm_root,
            host_hooks,
        }
    }

    /// Run the Runtime with the specified configuration.
    pub fn run(mut self) -> RuntimeOutput {
        // Load the builtins js sources
        self.agent.run_in_realm(&self.realm_root, |agent, mut gc| {
            for builtin in &self.config.builtins {
                let realm = agent.current_realm(gc.nogc());
                let source_text = types::String::from_str(agent, builtin, gc.nogc());
                let script = match parse_script(
                    agent,
                    source_text,
                    realm,
                    !self.config.no_strict,
                    None,
                    gc.nogc(),
                ) {
                    Ok(script) => script,
                    Err(diagnostics) => exit_with_parse_errors(diagnostics, "<runtime>", builtin),
                };
                let eval_result = script_evaluation(agent, script.unbind(), gc.reborrow()).unbind();
                match eval_result {
                    Ok(_) => (),
                    Err(e) => {
                        let error_value = e.value();
                        let message = error_value
                            .string_repr(agent, gc.reborrow())
                            .as_str(agent)
                            .unwrap_or("<non-string error>")
                            .to_string();
                        println!("Error in runtime: {message}");
                    }
                }
            }
        });
        let mut result = JsResult::Ok(Value::Null);

        // Validate all files before execution
        for file in &self.config.files {
            if let Err(error) = file.validate() {
                eprintln!(
                    "🚨 File validation error for {}: {}",
                    file.get_path(),
                    error
                );
                std::process::exit(1);
            }
        }

        // Fetch the runtime mod.ts file using a macro and add it to the paths
        for file in &self.config.files {
            let file_content = match file.read() {
                Ok(content) => content,
                Err(error) => {
                    eprintln!("🚨 Failed to read file {}: {}", file.get_path(), error);
                    std::process::exit(1);
                }
            };

            if file_content.trim().is_empty() {
                eprintln!("⚠️  Warning: File {} is empty", file.get_path());
                continue;
            }
            result = self.agent.run_in_realm(&self.realm_root, |agent, mut gc| {
                let source_text = types::String::from_string(agent, file_content, gc.nogc());
                let realm = agent.current_realm(gc.nogc());

                let module = match parse_module(
                    agent,
                    source_text,
                    realm,
                    Some(std::rc::Rc::new(file.get_path().to_string()) as HostDefined),
                    gc.nogc(),
                ) {
                    Ok(module) => module,
                    Err(errors) => exit_with_parse_errors(
                        errors,
                        file.get_path(),
                        source_text
                            .as_str(agent)
                            .expect("String is not valid UTF-8"),
                    ),
                };

                agent
                    .run_parsed_module(module.unbind(), None, gc.reborrow())
                    .unbind()
                    .map(|_| Value::Null)
            });
        }

        loop {
            while let Some(job) = self.host_hooks.pop_promise_job() {
                result = self.agent.run_in_realm(&self.realm_root, |agent, mut gc| {
                    job.run(agent, gc.reborrow()).unbind().map(|_| Value::Null)
                });
            }

            // Try to handle a macro task without blocking
            // This handles the case where a task completed so fast that the counter
            // was already decremented but the message is still in the channel
            let has_macro_task = self.try_handle_macro_task();

            // Only exit if there are no pending tasks AND no message was processed
            if !has_macro_task && !self.host_hooks.any_pending_macro_tasks() {
                break;
            }

            // If we saw pending tasks but got no message, block waiting for one
            if !has_macro_task && self.host_hooks.any_pending_macro_tasks() {
                self.handle_macro_task();
            }
        }

        RuntimeOutput {
            agent: self.agent,
            realm_root: self.realm_root,
            result,
        }
    }

    // Listen for pending macro tasks and resolve one by one
    pub fn handle_macro_task(&mut self) {
        match self.config.macro_task_rx.recv() {
            Ok(MacroTask::ResolvePromise(root_value)) => {
                self.agent.run_in_realm(&self.realm_root, |agent, gc| {
                    let value = root_value.take(agent);
                    if let Value::Promise(promise) = value {
                        let promise_capability = PromiseCapability::from_promise(promise, false);
                        promise_capability.resolve(agent, Value::Undefined, gc);
                    } else {
                        panic!("Attempted to resolve a non-promise value");
                    }
                });
            }
            // Let the user runtime handle its macro tasks
            Ok(MacroTask::User(e)) => {
                (self.config.eventloop_handler)(
                    e,
                    &mut self.agent,
                    &self.realm_root,
                    &self.host_hooks.host_data,
                );
            }
            _ => {}
        }
    }

    // Try to handle a macro task without blocking, returns true if a task was handled
    pub fn try_handle_macro_task(&mut self) -> bool {
        match self.config.macro_task_rx.try_recv() {
            Ok(MacroTask::ResolvePromise(root_value)) => {
                self.agent.run_in_realm(&self.realm_root, |agent, gc| {
                    let value = root_value.take(agent);
                    if let Value::Promise(promise) = value {
                        let promise_capability = PromiseCapability::from_promise(promise, false);
                        promise_capability.resolve(agent, Value::Undefined, gc);
                    } else {
                        panic!("Attempted to resolve a non-promise value");
                    }
                });
                true
            }
            // Let the user runtime handle its macro tasks
            Ok(MacroTask::User(e)) => {
                (self.config.eventloop_handler)(
                    e,
                    &mut self.agent,
                    &self.realm_root,
                    &self.host_hooks.host_data,
                );
                true
            }
            _ => false,
        }
    }
}

pub struct RuntimeOutput {
    pub agent: GcAgent,
    pub realm_root: RealmRoot,
    pub result: JsResult<'static, Value<'static>>,
}
