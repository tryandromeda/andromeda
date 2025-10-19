// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::{
    collections::{HashMap, HashSet, VecDeque},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::SystemTime,
};

use oxc_allocator::Allocator;
use oxc_ast::ast;
use oxc_parser::{Parser, ParserReturn};
use oxc_span::SourceType;
use serde::{Deserialize, Serialize};

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
    /// SHA-256 hash of the source code for integrity verification
    pub source_hash: Option<String>,
    /// Cached compiled bytecode for faster loading
    pub compiled_bytecode: Option<Vec<u8>>,
    /// Last modification time for cache invalidation
    pub last_modified: Option<SystemTime>,
}

impl ModuleRecord {
    /// Calculate and store the SHA-256 hash of the source code
    pub fn calculate_source_hash(&mut self) {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        self.source.hash(&mut hasher);
        self.source_hash = Some(format!("{:x}", hasher.finish()));
    }

    /// Check if the module needs recompilation based on hash
    pub fn needs_recompilation(&self, new_source_hash: &str) -> bool {
        match &self.source_hash {
            Some(current_hash) => current_hash != new_source_hash,
            None => true,
        }
    }

    /// Check if cached bytecode is valid
    pub fn has_valid_bytecode(&self) -> bool {
        self.compiled_bytecode.is_some() && self.source_hash.is_some()
    }

    /// Create a new ModuleRecord with default caching fields
    pub fn new(id: String, specifier: String, source: String, module_type: ModuleType) -> Self {
        let mut record = Self {
            id,
            specifier,
            source,
            state: ModuleState::Resolving,
            dependencies: Vec::new(),
            exports: Vec::new(),
            imports: Vec::new(),
            is_es_module: true,
            module_type,
            error: None,
            source_hash: None,
            compiled_bytecode: None,
            last_modified: Some(SystemTime::now()),
        };
        record.calculate_source_hash();
        record
    }
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

/// File system based module loader with caching
pub struct FileSystemModuleLoader {
    pub base_path: PathBuf,
    pub extensions: Vec<&'static str>,
    cache: Arc<Mutex<HashMap<String, (String, SystemTime)>>>,
}

impl FileSystemModuleLoader {
    pub fn new(base_path: impl Into<PathBuf>) -> Self {
        Self {
            base_path: base_path.into(),
            extensions: vec!["ts", "js", "mjs", "json"],
            cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn new_with_cache_size(base_path: impl Into<PathBuf>, cache_size: usize) -> Self {
        Self {
            base_path: base_path.into(),
            extensions: vec!["ts", "js", "mjs", "json"],
            cache: Arc::new(Mutex::new(HashMap::with_capacity(cache_size))),
        }
    }

    /// Clear the module cache
    pub fn clear_cache(&self) {
        if let Ok(mut cache) = self.cache.lock() {
            cache.clear();
        }
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> (usize, usize) {
        if let Ok(cache) = self.cache.lock() {
            (cache.len(), cache.capacity())
        } else {
            (0, 0)
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

        // Check cache first
        if let Ok(cache) = self.cache.lock()
            && let Some((cached_content, cached_mtime)) = cache.get(specifier)
            && let Ok(metadata) = std::fs::metadata(&resolved_path)
            && let Ok(current_mtime) = metadata.modified()
            && current_mtime <= *cached_mtime
        {
            return Ok(cached_content.clone());
        }

        // Load file and update cache
        let content = std::fs::read_to_string(&resolved_path).map_err(|e| ModuleError::Io {
            message: format!("Failed to read {}: {}", resolved_path.display(), e),
        })?;

        // Update cache with new content and modification time
        if let Ok(metadata) = std::fs::metadata(&resolved_path)
            && let Ok(mtime) = metadata.modified()
            && let Ok(mut cache) = self.cache.lock()
        {
            cache.insert(specifier.to_string(), (content.clone(), mtime));
        }

        Ok(content)
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

#[cfg_attr(feature = "hotpath", hotpath::measure_all)]
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

    pub fn with_import_map(import_map: ImportMap, base_url: String) -> Self {
        let mut composite = Self::new();
        let fallback = Self::default_loaders();
        composite.add_loader(Box::new(ImportMapModuleLoader::new(
            import_map,
            base_url,
            Box::new(fallback),
        )));
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

/// Import map configuration for module resolution
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(default)]
pub struct ImportMap {
    /// Direct module specifier mappings
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub imports: HashMap<String, String>,
    /// Scope-specific mappings
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub scopes: HashMap<String, HashMap<String, String>>,
    /// Integrity metadata mappings
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub integrity: HashMap<String, String>,
}

impl ImportMap {
    /// Load import map from JSON file
    pub fn from_file<P: AsRef<Path>>(path: P) -> ModuleResult<Self> {
        let content = std::fs::read_to_string(path.as_ref()).map_err(|e| ModuleError::Io {
            message: format!(
                "Failed to read import map file {}: {}",
                path.as_ref().display(),
                e
            ),
        })?;

        let import_map: ImportMap =
            serde_json::from_str(&content).map_err(|e| ModuleError::ParseError {
                path: path.as_ref().to_string_lossy().to_string(),
                message: format!("Invalid import map JSON: {e}"),
            })?;

        Ok(import_map)
    }

    /// Resolve a bare specifier using the import map
    pub fn resolve_specifier(&self, specifier: &str, base_url: Option<&str>) -> Option<String> {
        // Check if it's a relative or absolute specifier (should not be mapped)
        if specifier.starts_with("./")
            || specifier.starts_with("../")
            || specifier.starts_with("/")
            || specifier.contains("://")
        {
            return None;
        }

        // Try scope-specific mappings first if we have a base URL
        if let Some(base) = base_url {
            for (scope_prefix, scope_map) in &self.scopes {
                if base.starts_with(scope_prefix) {
                    // Try exact match first
                    if let Some(resolved) = scope_map.get(specifier) {
                        return Some(resolved.clone());
                    }

                    // Try prefix match
                    for (prefix, target) in scope_map {
                        if specifier.starts_with(prefix) && prefix.ends_with('/') {
                            let suffix = &specifier[prefix.len()..];
                            return Some(format!("{target}{suffix}"));
                        }
                    }
                }
            }
        }

        // Try global imports
        // Exact match first
        if let Some(resolved) = self.imports.get(specifier) {
            return Some(resolved.clone());
        }

        // Prefix match for directories
        for (prefix, target) in &self.imports {
            if specifier.starts_with(prefix) && prefix.ends_with('/') {
                let suffix = &specifier[prefix.len()..];
                return Some(format!("{target}{suffix}"));
            }
        }

        None
    }

    /// Merge another import map into this one
    pub fn merge(&mut self, other: ImportMap) {
        // Merge global imports
        for (key, value) in other.imports {
            self.imports.insert(key, value);
        }

        // Merge scopes
        for (scope, scope_map) in other.scopes {
            let existing_scope = self.scopes.entry(scope).or_default();
            for (key, value) in scope_map {
                existing_scope.insert(key, value);
            }
        }

        // Merge integrity metadata
        for (url, metadata) in other.integrity {
            self.integrity.insert(url, metadata);
        }
    }
}

/// Import map-aware module loader
pub struct ImportMapModuleLoader {
    /// The import map for resolving bare specifiers
    pub import_map: ImportMap,
    /// Base URL for resolving relative URLs in the import map
    pub base_url: String,
    /// Fallback loader for actual module loading
    pub fallback_loader: Box<dyn ModuleLoader>,
}

impl ImportMapModuleLoader {
    pub fn new(
        import_map: ImportMap,
        base_url: String,
        fallback_loader: Box<dyn ModuleLoader>,
    ) -> Self {
        Self {
            import_map,
            base_url,
            fallback_loader,
        }
    }

    pub fn from_config_files<P: AsRef<Path>>(
        config_files: &[P],
        base_url: String,
        fallback_loader: Box<dyn ModuleLoader>,
    ) -> ModuleResult<Self> {
        let mut import_map = ImportMap::default();

        for config_file in config_files {
            let file_import_map = ImportMap::from_file(config_file)?;
            import_map.merge(file_import_map);
        }

        Ok(Self::new(import_map, base_url, fallback_loader))
    }
}

impl ModuleLoader for ImportMapModuleLoader {
    fn load_module(&self, specifier: &str) -> ModuleResult<String> {
        self.fallback_loader.load_module(specifier)
    }

    fn resolve_specifier(&self, specifier: &str, base: Option<&str>) -> ModuleResult<String> {
        // Try import map resolution first
        if let Some(mapped_specifier) = self.import_map.resolve_specifier(specifier, base) {
            // Use the mapped specifier
            return self
                .fallback_loader
                .resolve_specifier(&mapped_specifier, base);
        }

        // Fall back to standard resolution
        self.fallback_loader.resolve_specifier(specifier, base)
    }

    fn module_exists(&self, specifier: &str) -> bool {
        // Try import map resolution first
        if let Some(mapped_specifier) = self.import_map.resolve_specifier(specifier, None) {
            return self.fallback_loader.module_exists(&mapped_specifier);
        }

        self.fallback_loader.module_exists(specifier)
    }

    fn supported_extensions(&self) -> Vec<&'static str> {
        self.fallback_loader.supported_extensions()
    }
}

/// Module dependency graph for circular dependency detection
#[derive(Debug)]
pub struct DependencyGraph {
    /// Adjacency list representation
    graph: HashMap<String, HashSet<String>>,
    /// Modules currently being resolved (for cycle detection)
    resolving: HashSet<String>,
    /// Cache for strongly connected components
    scc_cache: Option<Vec<Vec<String>>>,
    /// Version counter for cache invalidation
    version: u64,
}

impl DependencyGraph {
    pub fn new() -> Self {
        Self {
            graph: HashMap::new(),
            resolving: HashSet::new(),
            scc_cache: None,
            version: 0,
        }
    }

    /// Add a dependency edge
    pub fn add_dependency(&mut self, from: &str, to: &str) {
        let changed = self
            .graph
            .entry(from.to_string())
            .or_default()
            .insert(to.to_string());

        // Invalidate cache if dependency was actually added
        if changed {
            self.invalidate_cache();
        }
    }

    /// Invalidate caches when graph structure changes
    fn invalidate_cache(&mut self) {
        self.scc_cache = None;
        self.version += 1;
    }

    /// Check for circular dependencies using Tarjan's algorithm - more efficient
    pub fn check_circular(&mut self, _module: &str) -> ModuleResult<()> {
        let sccs = self.strongly_connected_components();

        for scc in &sccs {
            if scc.len() > 1 {
                // Found a strongly connected component with more than one node
                let cycle_path = scc.join(" -> ");
                return Err(ModuleError::CircularImport { cycle: cycle_path });
            }
            // Check for self-loops in single-node components
            if scc.len() == 1 && self.has_self_loop(&scc[0]) {
                return Err(ModuleError::CircularImport {
                    cycle: format!("{} -> {}", scc[0], scc[0]),
                });
            }
        }

        Ok(())
    }

    /// Legacy DFS-based circular dependency check (kept for compatibility)
    pub fn check_circular_dfs(&mut self, module: &str) -> ModuleResult<()> {
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
                self.check_circular_dfs(&dep)?;
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

    /// Use Tarjan's algorithm for better cycle detection - O(V + E) complexity
    pub fn strongly_connected_components(&mut self) -> Vec<Vec<String>> {
        // Return cached result if available
        if let Some(ref cached_sccs) = self.scc_cache {
            return cached_sccs.clone();
        }

        let mut tarjan_state = TarjanState::new();
        let mut sccs = Vec::new();

        // Get all nodes in the graph
        let mut all_nodes = HashSet::new();
        for (node, deps) in &self.graph {
            all_nodes.insert(node.clone());
            for dep in deps {
                all_nodes.insert(dep.clone());
            }
        }

        // Run Tarjan's algorithm on each unvisited node
        for node in &all_nodes {
            if !tarjan_state.indices.contains_key(node) {
                self.tarjan_scc(node, &mut tarjan_state, &mut sccs);
            }
        }

        // Cache the result
        self.scc_cache = Some(sccs.clone());
        sccs
    }

    /// Tarjan's algorithm implementation
    fn tarjan_scc(&self, node: &str, state: &mut TarjanState, sccs: &mut Vec<Vec<String>>) {
        // Set the depth index for v to the smallest unused index
        let current_index = state.index;
        state.indices.insert(node.to_string(), current_index);
        state.lowlinks.insert(node.to_string(), current_index);
        state.index += 1;
        state.stack.push(node.to_string());
        state.on_stack.insert(node.to_string());

        // Consider successors of node
        if let Some(successors) = self.graph.get(node) {
            for successor in successors {
                if !state.indices.contains_key(successor) {
                    // Successor has not yet been visited; recurse on it
                    self.tarjan_scc(successor, state, sccs);
                    let successor_lowlink = *state.lowlinks.get(successor).unwrap();
                    let current_lowlink = *state.lowlinks.get(node).unwrap();
                    state
                        .lowlinks
                        .insert(node.to_string(), current_lowlink.min(successor_lowlink));
                } else if state.on_stack.contains(successor) {
                    // Successor is in stack and hence in the current SCC
                    let successor_index = *state.indices.get(successor).unwrap();
                    let current_lowlink = *state.lowlinks.get(node).unwrap();
                    state
                        .lowlinks
                        .insert(node.to_string(), current_lowlink.min(successor_index));
                }
            }
        }

        // If node is a root node, pop the stack and generate an SCC
        let node_index = *state.indices.get(node).unwrap();
        let node_lowlink = *state.lowlinks.get(node).unwrap();

        if node_lowlink == node_index {
            let mut component = Vec::new();
            loop {
                let w = state.stack.pop().unwrap();
                state.on_stack.remove(&w);
                component.push(w.clone());
                if w == node {
                    break;
                }
            }
            if component.len() > 1 || (component.len() == 1 && self.has_self_loop(&component[0])) {
                sccs.push(component);
            }
        }
    }

    /// Check if a node has a self-loop
    fn has_self_loop(&self, node: &str) -> bool {
        self.graph
            .get(node)
            .map(|deps| deps.contains(node))
            .unwrap_or(false)
    }

    /// Add incremental updates for better performance
    pub fn update_dependency(&mut self, from: &str, old_to: Option<&str>, new_to: &str) {
        // Remove old dependency if it exists
        if let Some(old_to) = old_to
            && let Some(deps) = self.graph.get_mut(from)
            && deps.remove(old_to)
        {
            self.invalidate_cache();
        }

        // Add new dependency
        let changed = self
            .graph
            .entry(from.to_string())
            .or_default()
            .insert(new_to.to_string());

        if changed {
            self.invalidate_cache();
        }
    }

    /// Remove a dependency edge
    pub fn remove_dependency(&mut self, from: &str, to: &str) -> bool {
        let removed = self
            .graph
            .get_mut(from)
            .map(|deps| deps.remove(to))
            .unwrap_or(false);

        if removed {
            self.invalidate_cache();
        }

        removed
    }

    /// Get all modules that depend on the given module
    pub fn get_dependents(&self, module: &str) -> Vec<String> {
        let mut dependents = Vec::new();
        for (node, deps) in &self.graph {
            if deps.contains(module) {
                dependents.push(node.clone());
            }
        }
        dependents
    }
}

impl Default for DependencyGraph {
    fn default() -> Self {
        Self::new()
    }
}

/// State for Tarjan's strongly connected components algorithm
#[derive(Debug)]
struct TarjanState {
    indices: HashMap<String, usize>,
    lowlinks: HashMap<String, usize>,
    on_stack: HashSet<String>,
    stack: Vec<String>,
    index: usize,
}

impl TarjanState {
    fn new() -> Self {
        Self {
            indices: HashMap::new(),
            lowlinks: HashMap::new(),
            on_stack: HashSet::new(),
            stack: Vec::new(),
            index: 0,
        }
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
    /// Cache for parsed ASTs to avoid re-parsing
    ast_cache: HashMap<String, (oxc_ast::ast::Program<'static>, u64)>,
    /// Statistics for performance monitoring
    stats: ModuleSystemStats,
}

/// Statistics for module system performance monitoring
#[derive(Debug, Default)]
pub struct ModuleSystemStats {
    pub modules_loaded: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub cycles_detected: u64,
    pub resolution_time_ms: u64,
}

impl ModuleSystemStats {
    pub fn cache_hit_ratio(&self) -> f64 {
        if self.cache_hits + self.cache_misses == 0 {
            0.0
        } else {
            self.cache_hits as f64 / (self.cache_hits + self.cache_misses) as f64
        }
    }
}

#[cfg_attr(feature = "hotpath", hotpath::measure_all)]
impl ModuleSystem {
    /// Create a new module system with the given loader
    pub fn new(loader: Box<dyn ModuleLoader>) -> Self {
        Self {
            loader,
            modules: HashMap::new(),
            dependency_graph: DependencyGraph::new(),
            resolution_cache: HashMap::new(),
            ast_cache: HashMap::new(),
            stats: ModuleSystemStats::default(),
        }
    }

    /// Create a new module system with initial capacity hints for better performance
    pub fn with_capacity(loader: Box<dyn ModuleLoader>, capacity: usize) -> Self {
        Self {
            loader,
            modules: HashMap::with_capacity(capacity),
            dependency_graph: DependencyGraph::new(),
            resolution_cache: HashMap::with_capacity(capacity * 2),
            ast_cache: HashMap::with_capacity(capacity),
            stats: ModuleSystemStats::default(),
        }
    }

    /// Get performance statistics
    pub fn get_stats(&self) -> &ModuleSystemStats {
        &self.stats
    }

    /// Clear all caches and reset statistics
    pub fn clear_caches(&mut self) {
        self.resolution_cache.clear();
        self.ast_cache.clear();
        self.stats = ModuleSystemStats::default();
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
        let mut module = ModuleRecord::new(
            module_id.clone(),
            specifier.to_string(),
            String::new(),
            self.determine_module_type(specifier),
        );

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
        if let Some(ext) = Path::new(specifier).extension()
            && let Some(ext_str) = ext.to_str()
        {
            return ModuleType::from_extension(ext_str);
        }
        // default
        ModuleType::JavaScript
    }

    /// Get a module by ID
    #[allow(clippy::manual_find)]
    pub fn get_module(&self, id: &str) -> Option<&ModuleRecord> {
        for module in self.modules.values() {
            if module.id == id {
                return Some(module);
            }
        }
        None
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

    #[allow(dead_code)]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            imports: Vec::with_capacity(capacity),
            exports: Vec::with_capacity(capacity),
        }
    }

    fn analyze_program(&mut self, program: &ast::Program) {
        // Pre-allocate capacity based on program size
        let estimated_capacity = (program.body.len() / 4).max(4);
        self.imports.reserve(estimated_capacity);
        self.exports.reserve(estimated_capacity);

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
        let specifier = decl.source.value.as_str().to_owned();

        let mut import = ModuleImport {
            specifier,
            imports: None,
            default_import: None,
            namespace_import: None,
        };

        // Parse import specifiers
        if let Some(specifiers) = &decl.specifiers {
            let mut named_imports = Vec::with_capacity(specifiers.len());

            for spec in specifiers {
                match spec {
                    ast::ImportDeclarationSpecifier::ImportSpecifier(spec) => {
                        // Named import: import { name } from 'module'
                        named_imports.push(spec.local.name.as_str().to_owned());
                    }
                    ast::ImportDeclarationSpecifier::ImportDefaultSpecifier(spec) => {
                        // Default import: import name from 'module'
                        import.default_import = Some(spec.local.name.as_str().to_owned());
                    }
                    ast::ImportDeclarationSpecifier::ImportNamespaceSpecifier(spec) => {
                        // Namespace import: import * as name from 'module'
                        import.namespace_import = Some(spec.local.name.as_str().to_owned());
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
            name: Some("*".to_owned()),
            is_reexport: true,
            source_module: Some(decl.source.value.as_str().to_owned()),
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
                // Pre-allocate exports capacity
                self.exports.reserve(decl.specifiers.len());

                for spec in &decl.specifiers {
                    let export_name = self.extract_export_name(&spec.exported);
                    let source_name = self.extract_export_name(&spec.local);

                    self.exports.push(ModuleExport {
                        name: Some(export_name),
                        is_reexport: true,
                        source_module: Some(source.value.as_str().to_owned()),
                        source_name: Some(source_name),
                    });
                }
            }
        } else {
            // Direct export: export { name } or export const name = ...
            if !decl.specifiers.is_empty() {
                // Pre-allocate exports capacity
                self.exports.reserve(decl.specifiers.len());

                for spec in &decl.specifiers {
                    let export_name = self.extract_export_name(&spec.exported);

                    self.exports.push(ModuleExport {
                        name: Some(export_name),
                        is_reexport: false,
                        source_module: None,
                        source_name: None,
                    });
                }
            } else if let Some(declaration) = &decl.declaration {
                // export const/let/var/function/class declarations
                // Extract the actual names from the declaration
                self.extract_declaration_names(declaration);
            }
        }
    }

    /// Helper method to extract export names without cloning when possible
    #[inline]
    fn extract_export_name(&self, name: &ast::ModuleExportName) -> String {
        match name {
            ast::ModuleExportName::IdentifierName(name) => name.name.as_str().to_owned(),
            ast::ModuleExportName::IdentifierReference(name) => name.name.as_str().to_owned(),
            ast::ModuleExportName::StringLiteral(lit) => lit.value.as_str().to_owned(),
        }
    }

    /// Extract export names from declaration AST nodes
    fn extract_declaration_names(&mut self, declaration: &ast::Declaration) {
        match declaration {
            ast::Declaration::VariableDeclaration(var_decl) => {
                // Handle: export const x = 1, y = 2;
                for declarator in &var_decl.declarations {
                    self.extract_binding_names(&declarator.id.kind);
                }
            }
            ast::Declaration::FunctionDeclaration(func) => {
                // Handle: export function foo() {}
                if let Some(id) = &func.id {
                    self.exports.push(ModuleExport {
                        name: Some(id.name.as_str().to_owned()),
                        is_reexport: false,
                        source_module: None,
                        source_name: None,
                    });
                }
            }
            ast::Declaration::ClassDeclaration(class) => {
                // Handle: export class Foo {}
                if let Some(id) = &class.id {
                    self.exports.push(ModuleExport {
                        name: Some(id.name.as_str().to_owned()),
                        is_reexport: false,
                        source_module: None,
                        source_name: None,
                    });
                }
            }
            ast::Declaration::TSTypeAliasDeclaration(type_alias) => {
                // Handle: export type Foo = string;
                self.exports.push(ModuleExport {
                    name: Some(type_alias.id.name.as_str().to_owned()),
                    is_reexport: false,
                    source_module: None,
                    source_name: None,
                });
            }
            ast::Declaration::TSInterfaceDeclaration(interface) => {
                // Handle: export interface Foo {}
                self.exports.push(ModuleExport {
                    name: Some(interface.id.name.as_str().to_owned()),
                    is_reexport: false,
                    source_module: None,
                    source_name: None,
                });
            }
            ast::Declaration::TSEnumDeclaration(enum_decl) => {
                // Handle: export enum Foo {}
                self.exports.push(ModuleExport {
                    name: Some(enum_decl.id.name.as_str().to_owned()),
                    is_reexport: false,
                    source_module: None,
                    source_name: None,
                });
            }
            ast::Declaration::TSModuleDeclaration(module_decl) => {
                // Handle: export namespace Foo {} or export module Foo {}
                if let ast::TSModuleDeclarationName::Identifier(id) = &module_decl.id {
                    self.exports.push(ModuleExport {
                        name: Some(id.name.as_str().to_owned()),
                        is_reexport: false,
                        source_module: None,
                        source_name: None,
                    });
                }
            }
            ast::Declaration::TSImportEqualsDeclaration(import_equals) => {
                // Handle: export import Foo = require('foo');
                self.exports.push(ModuleExport {
                    name: Some(import_equals.id.name.as_str().to_owned()),
                    is_reexport: false,
                    source_module: None,
                    source_name: None,
                });
            }
        }
    }

    /// Extract names from binding patterns (handles destructuring)
    fn extract_binding_names(&mut self, binding: &ast::BindingPatternKind) {
        match binding {
            ast::BindingPatternKind::BindingIdentifier(id) => {
                // Simple binding: const x = 1
                self.exports.push(ModuleExport {
                    name: Some(id.name.as_str().to_owned()),
                    is_reexport: false,
                    source_module: None,
                    source_name: None,
                });
            }
            ast::BindingPatternKind::ObjectPattern(obj_pattern) => {
                // Object destructuring: const { x, y } = obj
                for property in &obj_pattern.properties {
                    self.extract_binding_names(&property.value.kind);
                }
                // Handle rest: const { ...rest } = obj
                if let Some(rest) = &obj_pattern.rest {
                    self.extract_binding_names(&rest.argument.kind);
                }
            }
            ast::BindingPatternKind::ArrayPattern(arr_pattern) => {
                // Array destructuring: const [x, y] = arr
                for element in arr_pattern.elements.iter().flatten() {
                    self.extract_binding_names(&element.kind);
                }
                // Handle rest: const [...rest] = arr
                if let Some(rest) = &arr_pattern.rest {
                    self.extract_binding_names(&rest.argument.kind);
                }
            }
            ast::BindingPatternKind::AssignmentPattern(assignment) => {
                // Default values: const { x = 5 } = obj
                self.extract_binding_names(&assignment.left.kind);
            }
        }
    }
}
