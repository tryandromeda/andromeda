// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::{
    collections::{HashMap, HashSet, VecDeque},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use oxc_allocator::Allocator;
use oxc_ast::ast;
use oxc_parser::{Parser, ParserReturn};
use oxc_span::SourceType;

/// Error type for module-related operations
#[derive(Debug, thiserror::Error)]
pub enum ModuleError {
    #[error("Module not found: {specifier}")]
    NotFound { specifier: String },

    #[error("Parse error in module {path}: {message}")]
    ParseError { path: String, message: String },

    #[error("Resolution error: {message}")]
    ResolutionError { message: String },

    #[error("Runtime error in module {path}: {message}")]
    RuntimeError { path: String, message: String },

    #[error("Circular import detected: {cycle}")]
    CircularImport { cycle: String },

    #[error("Import not found: '{import}' in module '{module}'")]
    ImportNotFound { import: String, module: String },

    #[error("Ambiguous export: '{export}' in module '{module}'")]
    AmbiguousExport { export: String, module: String },

    #[error("Module already loaded: {specifier}")]
    AlreadyLoaded { specifier: String },

    #[error("Invalid module specifier: {specifier}")]
    InvalidSpecifier { specifier: String },

    #[error("IO error: {message}")]
    Io { message: String },
}

/// Result type for module operations
pub type ModuleResult<T> = Result<T, ModuleError>;

/// Module loading state
#[derive(Debug, Clone, PartialEq)]
pub enum ModuleState {
    /// Module is being resolved
    Resolving,
    /// Module source is being fetched
    Fetching,
    /// Module is being parsed
    Parsing,
    /// Module is being instantiated
    Instantiating,
    /// Module is being evaluated
    Evaluating,
    /// Module is fully loaded and evaluated
    Evaluated,
    /// Module failed to load
    Failed(String),
}

/// Export information for a module
#[derive(Debug, Clone)]
pub struct ModuleExport {
    /// Export name (None for default export)
    pub name: Option<String>,
    /// Whether this is a re-export from another module
    pub is_reexport: bool,
    /// Source module for re-exports
    pub source_module: Option<String>,
    /// Original export name in source module
    pub source_name: Option<String>,
}

/// Import information for a module
#[derive(Debug, Clone)]
pub struct ModuleImport {
    /// Import specifier (module path)
    pub specifier: String,
    /// Imported names (None means namespace import)
    pub imports: Option<Vec<String>>,
    /// Local name for default import
    pub default_import: Option<String>,
    /// Local name for namespace import
    pub namespace_import: Option<String>,
}

/// Complete module metadata and content
#[derive(Debug, Clone)]
pub struct ModuleRecord {
    /// Unique identifier for this module
    pub id: String,
    /// Resolved module path/URL
    pub specifier: String,
    /// Module source code
    pub source: String,
    /// Module loading state
    pub state: ModuleState,
    /// Dependencies (modules this module imports)
    pub dependencies: Vec<String>,
    /// Exports provided by this module
    pub exports: Vec<ModuleExport>,
    /// Imports requested by this module
    pub imports: Vec<ModuleImport>,
    /// Whether this module is an ES module
    pub is_es_module: bool,
    /// Module type (js, ts, json, etc.)
    pub module_type: ModuleType,
    /// Error message if loading failed
    pub error: Option<String>,
}

/// Module type classification
#[derive(Debug, Clone, PartialEq)]
pub enum ModuleType {
    JavaScript,
    TypeScript,
    Json,
    Wasm,
    Other(String),
}

impl ModuleType {
    /// Determine module type from file extension
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "js" | "mjs" | "cjs" => ModuleType::JavaScript,
            "ts" | "tsx" | "mts" | "cts" => ModuleType::TypeScript,
            "json" => ModuleType::Json,
            "wasm" => ModuleType::Wasm,
            other => ModuleType::Other(other.to_string()),
        }
    }

    /// Check if this module type requires transpilation
    pub fn needs_transpilation(&self) -> bool {
        matches!(self, ModuleType::TypeScript | ModuleType::Json)
    }
}

/// Trait for module loader implementations
pub trait ModuleLoader: Send + Sync {
    /// Load a module from a specifier
    fn load_module(&self, specifier: &str) -> ModuleResult<String>;

    /// Resolve a module specifier relative to a base path
    fn resolve_specifier(&self, specifier: &str, base: Option<&str>) -> ModuleResult<String>;

    /// Check if a module exists
    fn module_exists(&self, specifier: &str) -> bool;

    /// Get supported extensions for this loader
    fn supported_extensions(&self) -> Vec<&'static str>;
}

/// File system based module loader
pub struct FileSystemModuleLoader {
    pub base_path: PathBuf,
    pub extensions: Vec<&'static str>,
}

impl FileSystemModuleLoader {
    pub fn new(base_path: impl Into<PathBuf>) -> Self {
        Self {
            base_path: base_path.into(),
            extensions: vec!["ts", "js", "mjs", "json"],
        }
    }

    /// Resolve file with extension fallbacks
    fn resolve_with_extensions(&self, path: &Path) -> Option<PathBuf> {
        // Try exact path first
        if path.exists() {
            return Some(path.to_path_buf());
        }

        // Try with extensions
        for ext in &self.extensions {
            let with_ext = path.with_extension(ext);
            if with_ext.exists() {
                return Some(with_ext);
            }
        }

        // Try as directory with index files
        if path.is_dir() {
            for ext in &self.extensions {
                let index_file = path.join(format!("index.{ext}"));
                if index_file.exists() {
                    return Some(index_file);
                }
            }
        }

        None
    }
}

impl ModuleLoader for FileSystemModuleLoader {
    fn load_module(&self, specifier: &str) -> ModuleResult<String> {
        let path = self.base_path.join(specifier);
        let resolved_path =
            self.resolve_with_extensions(&path)
                .ok_or_else(|| ModuleError::NotFound {
                    specifier: specifier.to_string(),
                })?;

        std::fs::read_to_string(&resolved_path).map_err(|e| ModuleError::Io {
            message: format!("Failed to read {}: {}", resolved_path.display(), e),
        })
    }

    fn resolve_specifier(&self, specifier: &str, base: Option<&str>) -> ModuleResult<String> {
        let path = if specifier.starts_with("./") || specifier.starts_with("../") {
            // Relative import
            if let Some(base) = base {
                let base_path = Path::new(base).parent().unwrap_or(Path::new("."));
                base_path.join(specifier)
            } else {
                PathBuf::from(specifier)
            }
        } else if specifier.starts_with('/') {
            // Absolute path
            PathBuf::from(specifier)
        } else {
            // Bare specifier - resolve relative to base_path
            self.base_path.join(specifier)
        };

        // Normalize the path
        let normalized = path.canonicalize().or_else(|_| {
            // If canonicalize fails, try with extensions
            self.resolve_with_extensions(&path)
                .ok_or_else(|| ModuleError::ResolutionError {
                    message: format!("Cannot resolve path: {}", path.display()),
                })
        })?;

        Ok(normalized.to_string_lossy().to_string())
    }

    fn module_exists(&self, specifier: &str) -> bool {
        let path = self.base_path.join(specifier);
        self.resolve_with_extensions(&path).is_some()
    }

    fn supported_extensions(&self) -> Vec<&'static str> {
        self.extensions.clone()
    }
}

/// HTTP-based module loader
pub struct HttpModuleLoader {
    client: ureq::Agent,
    cache: Arc<Mutex<HashMap<String, String>>>,
}

impl HttpModuleLoader {
    pub fn new() -> Self {
        Self {
            client: ureq::Agent::new_with_defaults(),
            cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn with_cache_size(cache_size: usize) -> Self {
        let loader = Self::new();
        // Pre-allocate cache
        loader.cache.lock().unwrap().reserve(cache_size);
        loader
    }
}

impl Default for HttpModuleLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl ModuleLoader for HttpModuleLoader {
    fn load_module(&self, specifier: &str) -> ModuleResult<String> {
        // Check cache first
        {
            let cache = self.cache.lock().unwrap();
            if let Some(cached) = cache.get(specifier) {
                return Ok(cached.clone());
            }
        }

        // Fetch from network
        let mut response =
            self.client
                .get(specifier)
                .call()
                .map_err(|e| ModuleError::NotFound {
                    specifier: format!("{specifier}: {e}"),
                })?;

        let content =
            response
                .body_mut()
                .read_to_string()
                .map_err(|e| ModuleError::RuntimeError {
                    path: specifier.to_string(),
                    message: e.to_string(),
                })?;

        // Cache the result
        {
            let mut cache = self.cache.lock().unwrap();
            cache.insert(specifier.to_string(), content.clone());
        }

        Ok(content)
    }

    fn resolve_specifier(&self, specifier: &str, base: Option<&str>) -> ModuleResult<String> {
        if specifier.starts_with("http://") || specifier.starts_with("https://") {
            Ok(specifier.to_string())
        } else if let Some(base) = base {
            // Resolve relative to base URL
            if base.starts_with("http://") || base.starts_with("https://") {
                let base_url = url::Url::parse(base).map_err(|e| ModuleError::ResolutionError {
                    message: format!("Invalid base URL {base}: {e}"),
                })?;
                let resolved =
                    base_url
                        .join(specifier)
                        .map_err(|e| ModuleError::ResolutionError {
                            message: format!(
                                "Failed to resolve {specifier} relative to {base}: {e}"
                            ),
                        })?;
                Ok(resolved.to_string())
            } else {
                Err(ModuleError::ResolutionError {
                    message: format!("HTTP loader requires HTTP base URL, got: {base}"),
                })
            }
        } else {
            Err(ModuleError::ResolutionError {
                message: format!("HTTP loader requires full URL or base URL: {specifier}"),
            })
        }
    }

    fn module_exists(&self, specifier: &str) -> bool {
        // For HTTP, we could do a HEAD request, but for now assume it exists
        specifier.starts_with("http://") || specifier.starts_with("https://")
    }

    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["js", "ts", "mjs", "json"]
    }
}

/// Composite module loader that tries multiple loaders
pub struct CompositeModuleLoader {
    loaders: Vec<Box<dyn ModuleLoader>>,
}

impl CompositeModuleLoader {
    pub fn new() -> Self {
        Self {
            loaders: Vec::new(),
        }
    }

    pub fn add_loader(&mut self, loader: Box<dyn ModuleLoader>) {
        self.loaders.push(loader);
    }

    pub fn default_loaders() -> Self {
        let mut composite = Self::new();
        composite.add_loader(Box::new(FileSystemModuleLoader::new(".")));
        composite.add_loader(Box::new(HttpModuleLoader::new()));
        composite
    }
}

impl Default for CompositeModuleLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl ModuleLoader for CompositeModuleLoader {
    fn load_module(&self, specifier: &str) -> ModuleResult<String> {
        for loader in &self.loaders {
            if let Ok(content) = loader.load_module(specifier) {
                return Ok(content);
            }
        }
        Err(ModuleError::NotFound {
            specifier: specifier.to_string(),
        })
    }

    fn resolve_specifier(&self, specifier: &str, base: Option<&str>) -> ModuleResult<String> {
        for loader in &self.loaders {
            if let Ok(resolved) = loader.resolve_specifier(specifier, base) {
                return Ok(resolved);
            }
        }
        Err(ModuleError::ResolutionError {
            message: format!("Failed to resolve: {specifier}"),
        })
    }

    fn module_exists(&self, specifier: &str) -> bool {
        self.loaders
            .iter()
            .any(|loader| loader.module_exists(specifier))
    }

    fn supported_extensions(&self) -> Vec<&'static str> {
        let mut extensions = Vec::new();
        for loader in &self.loaders {
            extensions.extend(loader.supported_extensions());
        }
        extensions.sort();
        extensions.dedup();
        extensions
    }
}

/// Module dependency graph for circular dependency detection
#[derive(Debug)]
pub struct DependencyGraph {
    /// Adjacency list representation
    graph: HashMap<String, HashSet<String>>,
    /// Modules currently being resolved (for cycle detection)
    resolving: HashSet<String>,
}

impl DependencyGraph {
    pub fn new() -> Self {
        Self {
            graph: HashMap::new(),
            resolving: HashSet::new(),
        }
    }

    /// Add a dependency edge
    pub fn add_dependency(&mut self, from: &str, to: &str) {
        self.graph
            .entry(from.to_string())
            .or_default()
            .insert(to.to_string());
    }

    /// Check for circular dependencies using DFS
    pub fn check_circular(&mut self, module: &str) -> ModuleResult<()> {
        if self.resolving.contains(module) {
            // Found a cycle
            let cycle_path = self.find_cycle_path(module);
            return Err(ModuleError::CircularImport {
                cycle: cycle_path.join(" -> "),
            });
        }

        self.resolving.insert(module.to_string());

        // Clone dependencies to avoid borrowing issues
        if let Some(dependencies) = self.graph.get(module).cloned() {
            for dep in dependencies {
                self.check_circular(&dep)?;
            }
        }

        self.resolving.remove(module);
        Ok(())
    }

    /// Find the actual cycle path for better error reporting
    fn find_cycle_path(&self, start: &str) -> Vec<String> {
        let mut path = Vec::new();
        let mut visited = HashSet::new();
        self.dfs_cycle(start, start, &mut path, &mut visited);
        path
    }

    fn dfs_cycle(
        &self,
        current: &str,
        target: &str,
        path: &mut Vec<String>,
        visited: &mut HashSet<String>,
    ) -> bool {
        if visited.contains(current) {
            return false;
        }

        visited.insert(current.to_string());
        path.push(current.to_string());

        if let Some(dependencies) = self.graph.get(current) {
            for dep in dependencies {
                if dep == target {
                    path.push(dep.clone());
                    return true;
                }
                if self.dfs_cycle(dep, target, path, visited) {
                    return true;
                }
            }
        }

        path.pop();
        false
    }

    /// Get topological order for dependency resolution
    pub fn topological_sort(&self) -> ModuleResult<Vec<String>> {
        let mut in_degree: HashMap<String, usize> = HashMap::new();
        let mut queue = VecDeque::new();
        let mut result = Vec::new();

        // Initialize in-degrees
        for (node, dependencies) in &self.graph {
            in_degree.entry(node.clone()).or_insert(0);
            for dep in dependencies {
                *in_degree.entry(dep.clone()).or_insert(0) += 1;
            }
        }

        // Find nodes with no incoming edges
        for (node, &degree) in &in_degree {
            if degree == 0 {
                queue.push_back(node.clone());
            }
        }

        // Process nodes
        while let Some(node) = queue.pop_front() {
            result.push(node.clone());

            if let Some(dependencies) = self.graph.get(&node) {
                for dep in dependencies {
                    if let Some(degree) = in_degree.get_mut(dep) {
                        *degree -= 1;
                        if *degree == 0 {
                            queue.push_back(dep.clone());
                        }
                    }
                }
            }
        }

        // Check for cycles
        if result.len() != in_degree.len() {
            Err(ModuleError::CircularImport {
                cycle: "Circular dependency detected in module graph".to_string(),
            })
        } else {
            Ok(result)
        }
    }
}

impl Default for DependencyGraph {
    fn default() -> Self {
        Self::new()
    }
}

/// Complete module system implementation
pub struct ModuleSystem {
    /// Module loader for fetching source code
    loader: Box<dyn ModuleLoader>,
    /// Cache of loaded modules
    modules: HashMap<String, ModuleRecord>,
    /// Dependency graph for cycle detection
    dependency_graph: DependencyGraph,
    /// Module resolution cache
    resolution_cache: HashMap<(String, Option<String>), String>,
}

impl ModuleSystem {
    /// Create a new module system with the given loader
    pub fn new(loader: Box<dyn ModuleLoader>) -> Self {
        Self {
            loader,
            modules: HashMap::new(),
            dependency_graph: DependencyGraph::new(),
            resolution_cache: HashMap::new(),
        }
    }

    /// Load a module and all its dependencies
    pub fn load_module(&mut self, specifier: &str, base: Option<&str>) -> ModuleResult<String> {
        let resolved = self.resolve_specifier(specifier, base)?;

        // Check if already loaded
        if let Some(module) = self.modules.get(&resolved) {
            match &module.state {
                ModuleState::Evaluated => return Ok(module.id.clone()),
                ModuleState::Failed(error) => {
                    return Err(ModuleError::RuntimeError {
                        path: resolved,
                        message: error.clone(),
                    });
                }
                _ => {
                    return Err(ModuleError::AlreadyLoaded {
                        specifier: resolved,
                    });
                }
            }
        }

        self.load_module_recursive(&resolved, &mut HashSet::new())
    }

    /// Recursively load a module and its dependencies
    fn load_module_recursive(
        &mut self,
        specifier: &str,
        loading_stack: &mut HashSet<String>,
    ) -> ModuleResult<String> {
        // Check for circular dependency
        if loading_stack.contains(specifier) {
            return Err(ModuleError::CircularImport {
                cycle: format!("Circular import detected: {specifier}"),
            });
        }

        loading_stack.insert(specifier.to_string());

        // Create module record
        let module_id = format!("module_{}", self.modules.len());
        let mut module = ModuleRecord {
            id: module_id.clone(),
            specifier: specifier.to_string(),
            source: String::new(),
            state: ModuleState::Resolving,
            dependencies: Vec::new(),
            exports: Vec::new(),
            imports: Vec::new(),
            is_es_module: true,
            module_type: self.determine_module_type(specifier),
            error: None,
        };

        // Fetch source
        module.state = ModuleState::Fetching;
        module.source = self.loader.load_module(specifier).inspect_err(|e| {
            module.state = ModuleState::Failed(e.to_string());
        })?;

        // Parse module
        module.state = ModuleState::Parsing;
        self.parse_module(&mut module)?;

        // Load dependencies
        let dependencies = module.imports.clone();
        for import in dependencies {
            let dep_id = self.load_module_recursive(&import.specifier, loading_stack)?;
            module.dependencies.push(dep_id.clone());
            self.dependency_graph.add_dependency(&module.id, &dep_id);
        }

        module.state = ModuleState::Instantiating;

        // Store module
        self.modules.insert(specifier.to_string(), module);
        loading_stack.remove(specifier);

        Ok(module_id)
    }

    /// Parse a module to extract imports and exports
    fn parse_module(&self, module: &mut ModuleRecord) -> ModuleResult<()> {
        match module.module_type {
            ModuleType::Json => {
                // JSON modules have a single default export
                module.exports.push(ModuleExport {
                    // default export
                    name: None,
                    is_reexport: false,
                    source_module: None,
                    source_name: None,
                });
                module.is_es_module = true;
            }
            ModuleType::JavaScript | ModuleType::TypeScript => {
                // Parse JavaScript/TypeScript for imports and exports
                self.parse_js_ts_module(module)?;
            }
            _ => {
                return Err(ModuleError::ParseError {
                    path: module.specifier.clone(),
                    message: format!("Unsupported module type: {:?}", module.module_type),
                });
            }
        }

        Ok(())
    }

    /// Parse JavaScript/TypeScript using proper AST parser
    fn parse_js_ts_module(&self, module: &mut ModuleRecord) -> ModuleResult<()> {
        let source = &module.source;
        let source_type = self.determine_source_type(&module.specifier, &module.module_type);

        // Create allocator for the parser
        let allocator = Allocator::default();

        // Parse the source code into an AST
        let ParserReturn {
            program, errors, ..
        } = Parser::new(&allocator, source, source_type).parse();

        // Check for parse errors
        if !errors.is_empty() {
            let error_messages: Vec<String> = errors.iter().map(|e| format!("{e:?}")).collect();
            return Err(ModuleError::ParseError {
                path: module.specifier.clone(),
                message: format!("Parse errors: {}", error_messages.join(", ")),
            });
        }

        // Create an AST visitor to extract imports and exports
        let mut visitor = ModuleVisitor::new();
        visitor.analyze_program(&program);

        // Extract the results
        module.imports = visitor.imports;
        module.exports = visitor.exports;

        Ok(())
    }

    /// Determine the source type for the OXC parser
    fn determine_source_type(&self, specifier: &str, module_type: &ModuleType) -> SourceType {
        let path = Path::new(specifier);
        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("js");

        match module_type {
            ModuleType::TypeScript => {
                if extension == "tsx" {
                    SourceType::tsx()
                } else {
                    SourceType::ts()
                }
            }
            ModuleType::JavaScript => {
                if extension == "jsx" {
                    SourceType::jsx()
                } else {
                    SourceType::mjs()
                }
            }
            _ => SourceType::mjs(),
        }
        // Always treat as ES modules
        .with_module(true)
    }

    /// Resolve a module specifier to an absolute path/URL
    fn resolve_specifier(&mut self, specifier: &str, base: Option<&str>) -> ModuleResult<String> {
        let cache_key = (specifier.to_string(), base.map(|s| s.to_string()));

        if let Some(cached) = self.resolution_cache.get(&cache_key) {
            return Ok(cached.clone());
        }

        let resolved = self.loader.resolve_specifier(specifier, base)?;
        self.resolution_cache.insert(cache_key, resolved.clone());
        Ok(resolved)
    }

    /// Determine module type from specifier
    fn determine_module_type(&self, specifier: &str) -> ModuleType {
        if let Some(ext) = Path::new(specifier).extension() {
            if let Some(ext_str) = ext.to_str() {
                return ModuleType::from_extension(ext_str);
            }
        }
        // default
        ModuleType::JavaScript
    }

    /// Get a module by ID
    pub fn get_module(&self, id: &str) -> Option<&ModuleRecord> {
        self.modules.values().find(|m| m.id == id)
    }

    /// Get all loaded modules
    pub fn get_all_modules(&self) -> Vec<&ModuleRecord> {
        self.modules.values().collect()
    }

    /// Clear the module cache
    pub fn clear_cache(&mut self) {
        self.modules.clear();
        self.resolution_cache.clear();
        self.dependency_graph = DependencyGraph::new();
    }

    /// Get dependency order for loading
    pub fn get_dependency_order(&self) -> ModuleResult<Vec<String>> {
        self.dependency_graph.topological_sort()
    }
}

impl Default for ModuleSystem {
    fn default() -> Self {
        Self::new(Box::new(CompositeModuleLoader::default_loaders()))
    }
}

/// AST visitor for extracting imports and exports from JavaScript/TypeScript modules
struct ModuleVisitor {
    imports: Vec<ModuleImport>,
    exports: Vec<ModuleExport>,
}

impl ModuleVisitor {
    fn new() -> Self {
        Self {
            imports: Vec::new(),
            exports: Vec::new(),
        }
    }

    fn analyze_program(&mut self, program: &ast::Program) {
        for stmt in &program.body {
            match stmt {
                ast::Statement::ImportDeclaration(decl) => {
                    self.handle_import_declaration(decl);
                }
                ast::Statement::ExportAllDeclaration(decl) => {
                    self.handle_export_all_declaration(decl);
                }
                ast::Statement::ExportDefaultDeclaration(decl) => {
                    self.handle_export_default_declaration(decl);
                }
                ast::Statement::ExportNamedDeclaration(decl) => {
                    self.handle_export_named_declaration(decl);
                }
                _ => {}
            }
        }
    }

    fn handle_import_declaration(&mut self, decl: &ast::ImportDeclaration) {
        let specifier = decl.source.value.to_string();

        let mut import = ModuleImport {
            specifier,
            imports: None,
            default_import: None,
            namespace_import: None,
        };

        // Parse import specifiers
        if let Some(specifiers) = &decl.specifiers {
            let mut named_imports = Vec::new();

            for spec in specifiers {
                match spec {
                    ast::ImportDeclarationSpecifier::ImportSpecifier(spec) => {
                        // Named import: import { name } from 'module'
                        named_imports.push(spec.local.name.to_string());
                    }
                    ast::ImportDeclarationSpecifier::ImportDefaultSpecifier(spec) => {
                        // Default import: import name from 'module'
                        import.default_import = Some(spec.local.name.to_string());
                    }
                    ast::ImportDeclarationSpecifier::ImportNamespaceSpecifier(spec) => {
                        // Namespace import: import * as name from 'module'
                        import.namespace_import = Some(spec.local.name.to_string());
                    }
                }
            }

            if !named_imports.is_empty() {
                import.imports = Some(named_imports);
            }
        }

        self.imports.push(import);
    }

    fn handle_export_all_declaration(&mut self, decl: &ast::ExportAllDeclaration) {
        // export * from 'module'
        self.exports.push(ModuleExport {
            name: Some("*".to_string()),
            is_reexport: true,
            source_module: Some(decl.source.value.to_string()),
            source_name: None,
        });
    }

    fn handle_export_default_declaration(&mut self, _decl: &ast::ExportDefaultDeclaration) {
        // export default ...
        self.exports.push(ModuleExport {
            // None indicates default export
            name: None,
            is_reexport: false,
            source_module: None,
            source_name: None,
        });
    }

    fn handle_export_named_declaration(&mut self, decl: &ast::ExportNamedDeclaration) {
        if let Some(source) = &decl.source {
            // Re-export from another module: export { name } from 'module'
            if !decl.specifiers.is_empty() {
                for spec in &decl.specifiers {
                    let export_name = match &spec.exported {
                        ast::ModuleExportName::IdentifierName(name) => name.name.to_string(),
                        ast::ModuleExportName::IdentifierReference(name) => name.name.to_string(),
                        ast::ModuleExportName::StringLiteral(lit) => lit.value.to_string(),
                    };

                    let source_name = match &spec.local {
                        ast::ModuleExportName::IdentifierName(name) => name.name.to_string(),
                        ast::ModuleExportName::IdentifierReference(name) => name.name.to_string(),
                        ast::ModuleExportName::StringLiteral(lit) => lit.value.to_string(),
                    };

                    self.exports.push(ModuleExport {
                        name: Some(export_name),
                        is_reexport: true,
                        source_module: Some(source.value.to_string()),
                        source_name: Some(source_name),
                    });
                }
            }
        } else {
            // Direct export: export { name } or export const name = ...
            if !decl.specifiers.is_empty() {
                for spec in &decl.specifiers {
                    let export_name = match &spec.exported {
                        ast::ModuleExportName::IdentifierName(name) => name.name.to_string(),
                        ast::ModuleExportName::IdentifierReference(name) => name.name.to_string(),
                        ast::ModuleExportName::StringLiteral(lit) => lit.value.to_string(),
                    };

                    self.exports.push(ModuleExport {
                        name: Some(export_name),
                        is_reexport: false,
                        source_module: None,
                        source_name: None,
                    });
                }
            } else if let Some(_declaration) = &decl.declaration {
                // export const/let/var/function/class declarations
                // For now, we'll mark these as generic exports
                // TODO: extract the actual names
                self.exports.push(ModuleExport {
                    name: Some("declaration".to_string()),
                    is_reexport: false,
                    source_module: None,
                    source_name: None,
                });
            }
        }
    }
}
