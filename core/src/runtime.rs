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
        types::{self, Object, PropertyKey, String as NovaString, Value},
    },
    engine::{
        Global,
        context::{Bindable, GcScope, NoGcScope},
    },
};

use crate::{
    AndromedaError, AndromedaResult, Extension, HostData, MacroTask, exit_with_parse_errors,
};

pub struct RuntimeHostHooks<UserMacroTask> {
    pub(crate) promise_job_queue: RefCell<VecDeque<Job>>,
    pub(crate) host_data: HostData<UserMacroTask>,
    pub(crate) base_path: PathBuf,
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
        }
    }

    pub fn with_base_path(host_data: HostData<UserMacroTask>, base_path: PathBuf) -> Self {
        Self {
            promise_job_queue: RefCell::default(),
            host_data,
            base_path,
        }
    }

    pub fn pop_promise_job(&self) -> Option<Job> {
        self.promise_job_queue.borrow_mut().pop_front()
    }

    pub fn any_pending_macro_tasks(&self) -> bool {
        self.host_data.macro_task_count.load(Ordering::Relaxed) > 0
    }

    /// Resolve a module specifier relative to a referrer path
    fn resolve_module_specifier(&self, specifier: &str, referrer_path: &Path) -> PathBuf {
        if specifier.starts_with("./") || specifier.starts_with("../") {
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

    /// Resolve module file with proper extension handling
    fn resolve_extensions(&self, path: PathBuf) -> Option<PathBuf> {
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

        // For now, let's use the base_path as the referrer directory for relative imports
        // TODO: Properly extract referrer path from host_defined when needed
        let referrer_path = self.base_path.join("script.js");

        // Resolve the module specifier
        let resolved_path = self.resolve_module_specifier(&specifier_str, &referrer_path);
        let resolved_path = match self.resolve_extensions(resolved_path) {
            Some(path) => path,
            None => {
                // Module not found error
                let error = agent.throw_exception(
                    ExceptionType::TypeError,
                    format!("Module not found: {specifier_str}"),
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
        };

        // Check if module is already loaded - let Nova VM handle caching internally

        // Check if this is a JSON module
        let is_json = resolved_path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext == "json")
            .unwrap_or(false);

        // Read the module source
        let source_text = match std::fs::read_to_string(&resolved_path) {
            Ok(content) => content,
            Err(error) => {
                let error = agent.throw_exception(
                    ExceptionType::TypeError,
                    format!(
                        "Failed to read module {}: {}",
                        resolved_path.display(),
                        error
                    ),
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

        // Parse the module
        let module_host_defined = Some(std::rc::Rc::new(resolved_path.clone()) as HostDefined);
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
                let error_msg = format!(
                    "Parse error in module {}: {:?}",
                    resolved_path.display(),
                    errors
                );
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
        _agent: &mut Agent,
        _module_record: SourceTextModule,
        _gc: NoGcScope<'gc, '_>,
    ) -> Vec<(PropertyKey<'gc>, Value<'gc>)> {
        // TODO: Implement import.meta.url when we can properly access host_defined
        // For now, return empty properties
        Vec::new()
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
}

pub struct Runtime<UserMacroTask: 'static> {
    pub config: RuntimeConfig<UserMacroTask>,
    pub agent: GcAgent,
    pub realm_root: RealmRoot,
    pub host_hooks: &'static RuntimeHostHooks<UserMacroTask>,
}

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

        let host_hooks = RuntimeHostHooks::with_base_path(host_data, base_path);

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
                    for extension in &mut config.extensions {
                        extension.load::<UserMacroTask>(agent, global_object, gc.borrow_mut())
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
                match script_evaluation(agent, script.unbind(), gc.reborrow()) {
                    Ok(_) => (),
                    Err(_) => println!("Error in runtime"),
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

            // If both the microtasks and macrotasks queues are empty we can end the event loop
            if !self.host_hooks.any_pending_macro_tasks() {
                break;
            }

            self.handle_macro_task();
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
            Ok(MacroTask::ResolvePromiseWithString(root_value, string_value)) => {
                // First, create the string value in a separate realm call
                let string_global = self
                    .agent
                    .run_in_realm(&self.realm_root, |agent, gc| {
                        let string_val = Value::from_string(agent, string_value, gc.nogc());
                        Some(Global::new(agent, string_val.unbind()))
                    })
                    .unwrap();

                // Then resolve the promise with the pre-created string
                self.agent.run_in_realm(&self.realm_root, |agent, gc| {
                    let promise_value = root_value.take(agent);
                    let string_value = string_global.take(agent);
                    if let Value::Promise(promise) = promise_value {
                        let promise_capability = PromiseCapability::from_promise(promise, false);
                        promise_capability.resolve(agent, string_value, gc);
                    } else {
                        panic!("Attempted to resolve a non-promise value");
                    }
                });
            }
            Ok(MacroTask::ResolvePromiseWithBytes(root_value, _bytes_value)) => {
                self.agent.run_in_realm(&self.realm_root, |agent, gc| {
                    let value = root_value.take(agent);
                    if let Value::Promise(promise) = value {
                        let promise_capability = PromiseCapability::from_promise(promise, false);
                        // TODO: Create Uint8Array from bytes
                        // For now, resolve with undefined
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
}

pub struct RuntimeOutput {
    pub agent: GcAgent,
    pub realm_root: RealmRoot,
    pub result: JsResult<'static, Value<'static>>,
}
