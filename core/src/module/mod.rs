// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! ES6 Module System Architecture for Andromeda
//!
//! This demonstrates the module system architecture we've designed.
//! Note: This is a simplified version that shows the structure
//! without using internal Nova VM APIs that aren't publicly available.

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

    #[error("Circular import detected: {path}")]
    CircularImport { path: String },

    #[error("Import not found: {import} in module {module}")]
    ImportNotFound { import: String, module: String },

    #[error("Ambiguous export: {export} in module {module}")]
    AmbiguousExport { export: String, module: String },
}

/// Result type for module operations
pub type ModuleResult<T> = Result<T, ModuleError>;

/// Trait for module loader implementations
pub trait ModuleLoader {
    /// Load a module from a specifier
    fn load_module(&mut self, specifier: &str) -> ModuleResult<String>;

    /// Resolve a module specifier relative to a base path
    fn resolve_specifier(&self, specifier: &str, base: Option<&str>) -> ModuleResult<String>;

    /// Check if a module exists
    fn module_exists(&self, specifier: &str) -> bool;
}

/// File system based module loader
pub struct FileSystemModuleLoader {
    pub base_path: std::path::PathBuf,
}

impl FileSystemModuleLoader {
    pub fn new(base_path: impl Into<std::path::PathBuf>) -> Self {
        Self {
            base_path: base_path.into(),
        }
    }
}

impl ModuleLoader for FileSystemModuleLoader {
    fn load_module(&mut self, specifier: &str) -> ModuleResult<String> {
        let path = self.base_path.join(specifier);
        std::fs::read_to_string(&path).map_err(|e| ModuleError::NotFound {
            specifier: format!("{}: {}", specifier, e),
        })
    }

    fn resolve_specifier(&self, specifier: &str, base: Option<&str>) -> ModuleResult<String> {
        // Simple resolution logic
        let resolved = if let Some(base) = base {
            format!("{}/{}", base, specifier)
        } else {
            specifier.to_string()
        };

        // Add extension if missing
        if !resolved.ends_with(".js") && !resolved.ends_with(".ts") && !resolved.ends_with(".mjs") {
            Ok(format!("{}.ts", resolved))
        } else {
            Ok(resolved)
        }
    }

    fn module_exists(&self, specifier: &str) -> bool {
        let path = self.base_path.join(specifier);
        path.exists()
    }
}

/// HTTP-based module loader
pub struct HttpModuleLoader {
    client: ureq::Agent,
}

impl HttpModuleLoader {
    pub fn new() -> Self {
        Self {
            client: ureq::Agent::new_with_defaults(),
        }
    }
}

impl ModuleLoader for HttpModuleLoader {
    fn load_module(&mut self, specifier: &str) -> ModuleResult<String> {
        let mut response =
            self.client
                .get(specifier)
                .call()
                .map_err(|e| ModuleError::NotFound {
                    specifier: format!("{}: {}", specifier, e),
                })?;

        response
            .body_mut()
            .read_to_string()
            .map_err(|e| ModuleError::RuntimeError {
                path: specifier.to_string(),
                message: e.to_string(),
            })
    }

    fn resolve_specifier(&self, specifier: &str, _base: Option<&str>) -> ModuleResult<String> {
        // For HTTP, we expect full URLs
        if specifier.starts_with("http://") || specifier.starts_with("https://") {
            Ok(specifier.to_string())
        } else {
            Err(ModuleError::ResolutionError {
                message: format!("HTTP loader requires full URL: {}", specifier),
            })
        }
    }

    fn module_exists(&self, _specifier: &str) -> bool {
        // For HTTP, we assume it exists - would need HEAD request to check
        true
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
}

impl ModuleLoader for CompositeModuleLoader {
    fn load_module(&mut self, specifier: &str) -> ModuleResult<String> {
        for loader in &mut self.loaders {
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
            message: format!("Failed to resolve: {}", specifier),
        })
    }

    fn module_exists(&self, specifier: &str) -> bool {
        self.loaders
            .iter()
            .any(|loader| loader.module_exists(specifier))
    }
}

/// Main module system coordinator
///
/// This demonstrates the architecture for ES6 module support.
/// In a complete implementation, this would integrate with Nova VM's
/// internal module system to provide:
/// - Static imports (`import { x } from 'module'`)
/// - Dynamic imports (`import('module')`)
/// - Named exports (`export { x }`)
/// - Default exports (`export default value`)
/// - Re-exports (`export { x } from 'module'`)
/// - Star exports (`export * from 'module'`)
/// - Module namespaces (`import * as ns from 'module'`)
/// - Import.meta support
pub struct ModuleSystem<L: ModuleLoader> {
    loader: L,
    module_cache: std::collections::HashMap<String, String>,
}

impl<L: ModuleLoader> ModuleSystem<L> {
    /// Create a new module system with the given loader
    pub fn new(loader: L) -> Self {
        Self {
            loader,
            module_cache: std::collections::HashMap::new(),
        }
    }

    /// Load and cache a module
    pub fn load_module(&mut self, specifier: &str, base: Option<&str>) -> ModuleResult<String> {
        let resolved_specifier = self.loader.resolve_specifier(specifier, base)?;

        // Check cache first
        if let Some(cached) = self.module_cache.get(&resolved_specifier) {
            return Ok(cached.clone());
        }

        // Load from loader
        let source = self.loader.load_module(&resolved_specifier)?;

        // Cache the result
        self.module_cache
            .insert(resolved_specifier.clone(), source.clone());

        Ok(source)
    }

    /// Get a default module system with file system and HTTP loaders
    pub fn default_system() -> ModuleSystem<CompositeModuleLoader> {
        let mut composite = CompositeModuleLoader::new();
        composite.add_loader(Box::new(FileSystemModuleLoader::new(".")));
        composite.add_loader(Box::new(HttpModuleLoader::new()));
        ModuleSystem::new(composite)
    }
}
