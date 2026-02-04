// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use miette::{self as oxc_miette, Diagnostic, NamedSource, SourceSpan};
use owo_colors::OwoColorize;
use oxc_diagnostics::OxcDiagnostic;
use std::fmt;

/// Comprehensive error type for Andromeda runtime operations.
#[derive(Diagnostic, Debug, Clone)]
pub enum RuntimeError {
    /// File system operation errors
    #[diagnostic(
        code(andromeda::fs::io_error),
        help(
            "🔍 Check that the file exists and you have proper permissions.\n💡 Try running with elevated permissions if needed.\n📂 Verify the path is correct and accessible."
        ),
        url("https://doc.rust-lang.org/std/fs/index.html")
    )]
    FsError {
        operation: String,
        path: String,
        error_message: String,
        #[label("📁 File operation failed here")]
        error_location: Option<SourceSpan>,
        #[source_code]
        source_code: Option<NamedSource<String>>,
    },

    /// Parse errors from JavaScript/TypeScript source
    #[diagnostic(
        code(andromeda::parse::syntax_error),
        help(
            "🔍 Check the syntax of your JavaScript/TypeScript code.\n💡 Look for missing semicolons, brackets, or quotes.\n📖 Refer to the JavaScript/TypeScript language specification."
        ),
        url("https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Grammar_and_types")
    )]
    ParseError {
        errors: Vec<OxcDiagnostic>,
        source_path: String,
        #[source_code]
        source_code: NamedSource<String>,
        #[label("❌ Parse error occurred here")]
        primary_error_span: Option<SourceSpan>,
        related_spans: Vec<SourceSpan>,
    },

    /// Runtime execution errors
    #[diagnostic(
        code(andromeda::runtime::execution_error),
        help(
            "🔍 Check the runtime state and ensure all required resources are available.\n💡 Verify that all variables are properly initialized.\n🐛 Review the call stack for the error source."
        ),
        url("https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Errors")
    )]
    RuntimeError {
        message: String,
        #[label("⚡ Runtime error occurred here")]
        location: Option<SourceSpan>,
        #[source_code]
        source_code: Option<NamedSource<String>>,
        /// Stack trace information for better debugging
        stack_trace: Option<String>,
        /// Variable context at the time of error
        variable_context: Vec<(String, String)>,
        related_locations: Vec<SourceSpan>,
    },

    /// Extension loading errors
    #[diagnostic(
        code(andromeda::extension::load_error),
        help(
            "🔍 Ensure the extension is properly configured and dependencies are available.\n💡 Check that the extension exports are correctly defined.\n📦 Verify all required dependencies are installed."
        ),
        url("https://docs.andromeda.dev/extensions")
    )]
    ExtensionError {
        extension_name: String,
        message: String,
        #[label("🧩 Extension error occurred here")]
        error_location: Option<SourceSpan>,
        #[source_code]
        extension_source: Option<NamedSource<String>>,
        /// Related extension dependencies that might be missing
        missing_dependencies: Vec<String>,
    },

    /// Resource management errors
    #[diagnostic(
        code(andromeda::resource::invalid_rid),
        help(
            "🔍 Ensure the resource ID is valid and the resource hasn't been closed.\n💡 Check for race conditions in resource cleanup.\n🔒 Verify resource lifecycle management."
        ),
        url("https://docs.andromeda.dev/resources")
    )]
    ResourceError {
        rid: u32,
        operation: String,
        message: String,
        #[label("🗂️ Resource operation failed here")]
        error_location: Option<SourceSpan>,
        #[source_code]
        source_code: Option<NamedSource<String>>,
        /// Current resource state for debugging
        resource_state: String,
    },

    /// Task management errors
    #[diagnostic(
        code(andromeda::task::task_error),
        help(
            "🔍 Check task state and ensure proper cleanup.\n💡 Verify async task handling and synchronization.\n⏱️ Check for deadlocks or infinite loops."
        ),
        url("https://docs.andromeda.dev/tasks")
    )]
    TaskError {
        task_id: u32,
        message: String,
        #[label("⚙️ Task error occurred here")]
        error_location: Option<SourceSpan>,
        #[source_code]
        source_code: Option<NamedSource<String>>,
        /// Task execution history for debugging
        execution_history: Vec<String>,
    },

    /// Network/HTTP errors
    #[diagnostic(
        code(andromeda::network::request_error),
        help(
            "🔍 Check network connectivity and request parameters.\n💡 Verify the URL format and endpoint availability.\n🌐 Check firewall and proxy settings."
        ),
        url("https://developer.mozilla.org/en-US/docs/Web/API/Fetch_API")
    )]
    NetworkError {
        url: String,
        operation: String,
        error_message: String,
        #[label("🌐 Network error occurred here")]
        error_location: Option<SourceSpan>,
        #[source_code]
        source_code: Option<NamedSource<String>>,
        /// HTTP status code if available
        status_code: Option<u16>,
        /// Request headers for debugging
        request_headers: Vec<(String, String)>,
    },

    /// Encoding/Decoding errors
    #[diagnostic(
        code(andromeda::encoding::decode_error),
        help(
            "🔍 Ensure the input data is properly formatted.\n💡 Check the encoding format and character set.\n📄 Verify data integrity and completeness."
        ),
        url("https://developer.mozilla.org/en-US/docs/Web/API/TextEncoder")
    )]
    EncodingError {
        format: String,
        message: String,
        #[label("🔤 Encoding error occurred here")]
        error_location: Option<SourceSpan>,
        #[source_code]
        source_code: Option<NamedSource<String>>,
        /// Expected vs actual encoding information
        expected_encoding: Option<String>,
        actual_encoding: Option<String>,
    },

    /// Configuration errors
    #[diagnostic(
        code(andromeda::config::invalid_config),
        help(
            "🔍 Check the configuration file format and required fields.\n💡 Verify JSON/TOML syntax and field types.\n📋 Ensure all required configuration options are present."
        ),
        url("https://docs.andromeda.dev/configuration")
    )]
    ConfigError {
        field: String,
        message: String,
        #[label("⚙️ Configuration error occurred here")]
        error_location: Option<SourceSpan>,
        #[source_code]
        config_source: Option<NamedSource<String>>,
        /// Expected configuration schema for reference
        expected_schema: Option<String>,
        /// Suggested fixes for common configuration errors
        suggested_fixes: Vec<String>,
    },

    /// Type-related errors with enhanced diagnostics
    #[diagnostic(
        code(andromeda::types::mismatch),
        help(
            "🔍 Check the types of your variables and function parameters.\n💡 Ensure type compatibility between operations.\n📝 Consider explicit type conversions if needed."
        ),
        url("https://developer.mozilla.org/en-US/docs/Web/JavaScript/Data_structures")
    )]
    TypeError {
        message: String,
        expected_type: String,
        actual_type: String,
        #[label("❌ Type error occurred here")]
        error_location: Option<SourceSpan>,
        #[source_code]
        source_code: Option<NamedSource<String>>,
        type_context: Vec<SourceSpan>,
        /// Suggested type conversions
        type_suggestions: Vec<String>,
    },

    /// Memory and performance related errors
    #[diagnostic(
        code(andromeda::memory::allocation_error),
        help(
            "🔍 Check memory usage and allocation patterns.\n💡 Consider reducing memory footprint or optimizing algorithms.\n📊 Monitor for memory leaks and excessive allocations."
        ),
        url("https://docs.andromeda.dev/performance")
    )]
    MemoryError {
        message: String,
        operation: String,
        #[label("💾 Memory error occurred here")]
        error_location: Option<SourceSpan>,
        #[source_code]
        source_code: Option<NamedSource<String>>,
        /// Memory usage statistics at time of error
        memory_stats: Option<String>,
        /// Heap information for debugging
        heap_info: Option<String>,
    },
    /// Module not found error
    #[diagnostic(
        code(andromeda::module::not_found),
        help(
            "🔍 Check that the module path is correct and the file exists.\n💡 Verify import specifiers match actual file names.\n📦 Ensure dependencies are installed."
        ),
        url("https://docs.andromeda.dev/modules")
    )]
    ModuleNotFound {
        specifier: String,
        #[label("📦 Module not found")]
        error_location: Option<SourceSpan>,
        #[source_code]
        source_code: Option<NamedSource<String>>,
        /// Suggested similar module names
        suggestions: Vec<String>,
    },

    /// Module parse error
    #[diagnostic(
        code(andromeda::module::parse_error),
        help(
            "🔍 Check the syntax of the module source code.\n💡 Look for missing semicolons, brackets, or quotes.\n📖 Ensure the file is valid JavaScript/TypeScript."
        ),
        url("https://docs.andromeda.dev/modules")
    )]
    ModuleParseError {
        path: String,
        message: String,
        #[label("❌ Module parse error")]
        error_location: Option<SourceSpan>,
        #[source_code]
        source_code: Option<NamedSource<String>>,
    },

    /// Module resolution error
    #[diagnostic(
        code(andromeda::module::resolution_error),
        help(
            "🔍 Check import paths and module specifiers.\n💡 Verify relative paths are correct.\n📂 Ensure the module resolution algorithm can find the file."
        ),
        url("https://docs.andromeda.dev/modules#resolution")
    )]
    ModuleResolutionError {
        message: String,
        specifier: Option<String>,
        referrer: Option<String>,
        #[label("🔍 Resolution failed here")]
        error_location: Option<SourceSpan>,
        #[source_code]
        source_code: Option<NamedSource<String>>,
    },

    /// Module runtime error
    #[diagnostic(
        code(andromeda::module::runtime_error),
        help(
            "🔍 Check the module's execution logic.\n💡 Verify all imports are correctly resolved.\n🐛 Review the module's dependencies."
        ),
        url("https://docs.andromeda.dev/modules#runtime")
    )]
    ModuleRuntimeError {
        path: String,
        message: String,
        #[label("⚡ Module runtime error")]
        error_location: Option<SourceSpan>,
        #[source_code]
        source_code: Option<NamedSource<String>>,
    },

    /// Circular import detected
    #[diagnostic(
        code(andromeda::module::circular_import),
        help(
            "🔍 Review the import chain to find the cycle.\n💡 Consider restructuring your modules to avoid circular dependencies.\n📦 Use dynamic imports or dependency injection to break cycles."
        ),
        url("https://docs.andromeda.dev/modules#circular-imports")
    )]
    CircularImport {
        cycle: String,
        #[label("🔄 Circular import detected")]
        error_location: Option<SourceSpan>,
        #[source_code]
        source_code: Option<NamedSource<String>>,
        /// The modules involved in the cycle
        involved_modules: Vec<String>,
    },

    /// Import not found in module
    #[diagnostic(
        code(andromeda::module::import_not_found),
        help(
            "🔍 Check that the export exists in the source module.\n💡 Verify the export name matches exactly (case-sensitive).\n📦 Ensure the module exports what you're trying to import."
        ),
        url("https://docs.andromeda.dev/modules#exports")
    )]
    ImportNotFound {
        import: String,
        module: String,
        #[label("❓ Import not found")]
        error_location: Option<SourceSpan>,
        #[source_code]
        source_code: Option<NamedSource<String>>,
        /// Available exports from the module
        available_exports: Vec<String>,
    },

    /// Ambiguous export in module
    #[diagnostic(
        code(andromeda::module::ambiguous_export),
        help(
            "🔍 The export name is defined multiple times.\n💡 Use explicit re-exports to resolve ambiguity.\n📦 Check for conflicting star exports."
        ),
        url("https://docs.andromeda.dev/modules#exports")
    )]
    AmbiguousExport {
        export: String,
        module: String,
        #[label("⚠️ Ambiguous export")]
        error_location: Option<SourceSpan>,
        #[source_code]
        source_code: Option<NamedSource<String>>,
        /// Sources of the conflicting exports
        conflict_sources: Vec<String>,
    },

    /// Module already loaded
    #[diagnostic(
        code(andromeda::module::already_loaded),
        help(
            "🔍 The module has already been loaded.\n💡 This may indicate a configuration issue.\n📦 Check for duplicate module registrations."
        ),
        url("https://docs.andromeda.dev/modules#caching")
    )]
    ModuleAlreadyLoaded {
        specifier: String,
        #[label("📦 Module already loaded")]
        error_location: Option<SourceSpan>,
        #[source_code]
        source_code: Option<NamedSource<String>>,
    },

    /// Invalid module specifier
    #[diagnostic(
        code(andromeda::module::invalid_specifier),
        help(
            "🔍 Check the module specifier format.\n💡 Use relative paths (./), absolute paths (/), or bare specifiers.\n📦 Ensure URLs are properly formatted."
        ),
        url("https://docs.andromeda.dev/modules#specifiers")
    )]
    InvalidModuleSpecifier {
        specifier: String,
        reason: Option<String>,
        #[label("❌ Invalid module specifier")]
        error_location: Option<SourceSpan>,
        #[source_code]
        source_code: Option<NamedSource<String>>,
    },

    /// Module I/O error
    #[diagnostic(
        code(andromeda::module::io_error),
        help(
            "🔍 Check file permissions and disk space.\n💡 Verify the file is not locked by another process.\n📂 Ensure the path is accessible."
        ),
        url("https://docs.andromeda.dev/modules#loading")
    )]
    ModuleIoError {
        message: String,
        path: Option<String>,
        #[label("📁 I/O error")]
        error_location: Option<SourceSpan>,
        #[source_code]
        source_code: Option<NamedSource<String>>,
    },

    /// LLM provider not initialized
    #[diagnostic(
        code(andromeda::llm::not_initialized),
        help(
            "🔍 Initialize the LLM provider before requesting suggestions.\n💡 Call init_llm_provider() or init_copilot_provider() first.\n🔧 Check that the llm feature is enabled."
        ),
        url("https://docs.andromeda.dev/llm-suggestions")
    )]
    LlmNotInitialized,

    /// LLM provider error
    #[diagnostic(
        code(andromeda::llm::provider_error),
        help(
            "🔍 Check the LLM provider configuration.\n💡 Verify API keys and credentials are valid.\n🌐 Ensure network connectivity to the LLM service."
        ),
        url("https://docs.andromeda.dev/llm-suggestions")
    )]
    LlmProviderError {
        message: String,
        provider_name: Option<String>,
        #[label("🤖 LLM provider error")]
        error_location: Option<SourceSpan>,
        #[source_code]
        source_code: Option<NamedSource<String>>,
    },

    /// LLM request timeout
    #[diagnostic(
        code(andromeda::llm::timeout),
        help(
            "🔍 The LLM request took too long.\n💡 Try increasing the timeout duration.\n🌐 Check network latency to the LLM service."
        ),
        url("https://docs.andromeda.dev/llm-suggestions#configuration")
    )]
    LlmTimeout {
        timeout_ms: u64,
        #[label("⏱️ Request timed out")]
        error_location: Option<SourceSpan>,
        #[source_code]
        source_code: Option<NamedSource<String>>,
    },

    /// LLM suggestions disabled
    #[diagnostic(
        code(andromeda::llm::disabled),
        help(
            "🔍 LLM suggestions are currently disabled.\n💡 Enable them in the configuration.\n🔧 Set enabled: true in LlmSuggestionConfig."
        ),
        url("https://docs.andromeda.dev/llm-suggestions#configuration")
    )]
    LlmDisabled,

    /// LLM authentication error
    #[diagnostic(
        code(andromeda::llm::auth_error),
        help(
            "🔍 Check your authentication credentials.\n💡 Verify API keys or tokens are valid and not expired.\n🔑 Ensure GITHUB_TOKEN is set for Copilot integration."
        ),
        url("https://docs.andromeda.dev/llm-suggestions#authentication")
    )]
    LlmAuthenticationError {
        message: String,
        #[label("🔑 Authentication failed")]
        error_location: Option<SourceSpan>,
        #[source_code]
        source_code: Option<NamedSource<String>>,
    },

    /// LLM network error
    #[diagnostic(
        code(andromeda::llm::network_error),
        help(
            "🔍 Check network connectivity.\n💡 Verify firewall and proxy settings.\n🌐 Ensure the LLM service endpoint is accessible."
        ),
        url("https://docs.andromeda.dev/llm-suggestions#troubleshooting")
    )]
    LlmNetworkError {
        message: String,
        #[label("🌐 Network error")]
        error_location: Option<SourceSpan>,
        #[source_code]
        source_code: Option<NamedSource<String>>,
    },

    /// Internal error (should not happen in normal operation)
    #[diagnostic(
        code(andromeda::internal::error),
        help(
            "🔍 This is an internal error that should not occur.\n💡 Please report this issue on GitHub.\n🐛 Include the error message and reproduction steps."
        ),
        url("https://github.com/aspect-build/andromeda/issues")
    )]
    InternalError {
        message: String,
        #[label("🐛 Internal error")]
        error_location: Option<SourceSpan>,
        #[source_code]
        source_code: Option<NamedSource<String>>,
    },
}

impl RuntimeError {
    /// Box this error for use with RuntimeResult
    #[must_use]
    pub fn boxed(self) -> Box<Self> {
        Box::new(self)
    }

    /// Create a boxed error directly
    pub fn into_result<T>(self) -> RuntimeResult<T> {
        Err(Box::new(self))
    }

    // -------------------- File System Errors --------------------

    /// Create a new file system error
    pub fn fs_error(
        source: std::io::Error,
        operation: impl Into<String>,
        path: impl Into<String>,
    ) -> Self {
        Self::FsError {
            error_message: source.to_string(),
            operation: operation.into(),
            path: path.into(),
            error_location: None,
            source_code: None,
        }
    }

    /// Create a new file system error with location context
    pub fn fs_error_with_location(
        source: std::io::Error,
        operation: impl Into<String>,
        path: impl Into<String>,
        source_code: impl Into<String>,
        source_path: impl Into<String>,
        location: SourceSpan,
    ) -> Self {
        Self::FsError {
            error_message: source.to_string(),
            operation: operation.into(),
            path: path.into(),
            error_location: Some(location),
            source_code: Some(NamedSource::new(source_path.into(), source_code.into())),
        }
    }

    // -------------------- Parse Errors --------------------

    /// Create a new parse error with enhanced diagnostics
    pub fn parse_error(
        errors: Vec<OxcDiagnostic>,
        source_path: impl Into<String>,
        source_code: impl Into<String>,
    ) -> Self {
        let source_path = source_path.into();
        let source_code = source_code.into();

        // Extract primary error span from the first diagnostic
        let primary_error_span = errors.first().and_then(|diagnostic| {
            diagnostic.labels.as_ref().and_then(|labels| {
                labels
                    .first()
                    .map(|label| SourceSpan::new(label.offset().into(), label.len()))
            })
        });

        // Extract additional spans for related errors
        let related_spans = errors
            .iter()
            .skip(1)
            .filter_map(|diagnostic| {
                diagnostic.labels.as_ref().and_then(|labels| {
                    labels
                        .first()
                        .map(|label| SourceSpan::new(label.offset().into(), label.len()))
                })
            })
            .collect();

        Self::ParseError {
            errors,
            source_path: source_path.clone(),
            source_code: NamedSource::new(source_path, source_code),
            primary_error_span,
            related_spans,
        }
    }

    // -------------------- Runtime Errors --------------------

    /// Create a new runtime error
    #[allow(clippy::self_named_constructors)]
    pub fn runtime_error(message: impl Into<String>) -> Self {
        Self::RuntimeError {
            message: message.into(),
            location: None,
            source_code: None,
            stack_trace: None,
            variable_context: Vec::new(),
            related_locations: Vec::new(),
        }
    }

    /// Create a new runtime error with source location
    pub fn runtime_error_with_location(
        message: impl Into<String>,
        source_code: impl Into<String>,
        source_path: impl Into<String>,
        location: SourceSpan,
    ) -> Self {
        Self::RuntimeError {
            message: message.into(),
            location: Some(location),
            source_code: Some(NamedSource::new(source_path.into(), source_code.into())),
            stack_trace: None,
            variable_context: Vec::new(),
            related_locations: Vec::new(),
        }
    }

    /// Create a new runtime error with full context
    pub fn runtime_error_with_context(
        message: impl Into<String>,
        source_code: impl Into<String>,
        source_path: impl Into<String>,
        location: SourceSpan,
        stack_trace: impl Into<String>,
        variable_context: Vec<(String, String)>,
    ) -> Self {
        Self::RuntimeError {
            message: message.into(),
            location: Some(location),
            source_code: Some(NamedSource::new(source_path.into(), source_code.into())),
            stack_trace: Some(stack_trace.into()),
            variable_context,
            related_locations: Vec::new(),
        }
    }

    // -------------------- Extension Errors --------------------

    /// Create a new extension error
    pub fn extension_error(extension_name: impl Into<String>, message: impl Into<String>) -> Self {
        Self::ExtensionError {
            extension_name: extension_name.into(),
            message: message.into(),
            error_location: None,
            extension_source: None,
            missing_dependencies: Vec::new(),
        }
    }

    /// Create a new extension error with context
    pub fn extension_error_with_context(
        extension_name: impl Into<String>,
        message: impl Into<String>,
        source_code: impl Into<String>,
        source_path: impl Into<String>,
        location: SourceSpan,
        missing_dependencies: Vec<String>,
    ) -> Self {
        Self::ExtensionError {
            extension_name: extension_name.into(),
            message: message.into(),
            error_location: Some(location),
            extension_source: Some(NamedSource::new(source_path.into(), source_code.into())),
            missing_dependencies,
        }
    }

    // -------------------- Resource Errors --------------------

    /// Create a new resource error
    pub fn resource_error(
        rid: u32,
        operation: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self::ResourceError {
            rid,
            operation: operation.into(),
            message: message.into(),
            error_location: None,
            source_code: None,
            resource_state: "unknown".to_string(),
        }
    }

    /// Create a new resource error with context
    pub fn resource_error_with_context(
        rid: u32,
        operation: impl Into<String>,
        message: impl Into<String>,
        resource_state: impl Into<String>,
        location: Option<(impl Into<String>, impl Into<String>, SourceSpan)>,
    ) -> Self {
        let (source_code, error_location) = if let Some((code, path, span)) = location {
            (
                Some(NamedSource::new(path.into(), code.into())),
                Some(span),
            )
        } else {
            (None, None)
        };

        Self::ResourceError {
            rid,
            operation: operation.into(),
            message: message.into(),
            error_location,
            source_code,
            resource_state: resource_state.into(),
        }
    }

    // -------------------- Task Errors --------------------

    /// Create a new task error
    pub fn task_error(task_id: u32, message: impl Into<String>) -> Self {
        Self::TaskError {
            task_id,
            message: message.into(),
            error_location: None,
            source_code: None,
            execution_history: Vec::new(),
        }
    }

    /// Create a new task error with execution history
    pub fn task_error_with_history(
        task_id: u32,
        message: impl Into<String>,
        execution_history: Vec<String>,
        location: Option<(impl Into<String>, impl Into<String>, SourceSpan)>,
    ) -> Self {
        let (source_code, error_location) = if let Some((code, path, span)) = location {
            (
                Some(NamedSource::new(path.into(), code.into())),
                Some(span),
            )
        } else {
            (None, None)
        };

        Self::TaskError {
            task_id,
            message: message.into(),
            error_location,
            source_code,
            execution_history,
        }
    }

    // -------------------- Network Errors --------------------

    /// Create a new network error
    pub fn network_error(
        source: Box<dyn std::error::Error + Send + Sync>,
        url: impl Into<String>,
        operation: impl Into<String>,
    ) -> Self {
        Self::NetworkError {
            error_message: source.to_string(),
            url: url.into(),
            operation: operation.into(),
            error_location: None,
            source_code: None,
            status_code: None,
            request_headers: Vec::new(),
        }
    }

    /// Create a new network error from a string message
    pub fn network_error_from_string(
        error_message: impl Into<String>,
        url: impl Into<String>,
        operation: impl Into<String>,
    ) -> Self {
        Self::NetworkError {
            error_message: error_message.into(),
            url: url.into(),
            operation: operation.into(),
            error_location: None,
            source_code: None,
            status_code: None,
            request_headers: Vec::new(),
        }
    }

    /// Create a new network error with full context
    pub fn network_error_with_context(
        source: Box<dyn std::error::Error + Send + Sync>,
        url: impl Into<String>,
        operation: impl Into<String>,
        status_code: Option<u16>,
        request_headers: Vec<(String, String)>,
        location: Option<(impl Into<String>, impl Into<String>, SourceSpan)>,
    ) -> Self {
        let (source_code, error_location) = if let Some((code, path, span)) = location {
            (
                Some(NamedSource::new(path.into(), code.into())),
                Some(span),
            )
        } else {
            (None, None)
        };

        Self::NetworkError {
            error_message: source.to_string(),
            url: url.into(),
            operation: operation.into(),
            error_location,
            source_code,
            status_code,
            request_headers,
        }
    }

    // -------------------- Encoding Errors --------------------

    /// Create a new encoding error
    pub fn encoding_error(format: impl Into<String>, message: impl Into<String>) -> Self {
        Self::EncodingError {
            format: format.into(),
            message: message.into(),
            error_location: None,
            source_code: None,
            expected_encoding: None,
            actual_encoding: None,
        }
    }

    /// Create a new encoding error with context
    pub fn encoding_error_with_context(
        format: impl Into<String>,
        message: impl Into<String>,
        expected_encoding: impl Into<String>,
        actual_encoding: impl Into<String>,
        location: Option<(impl Into<String>, impl Into<String>, SourceSpan)>,
    ) -> Self {
        let (source_code, error_location) = if let Some((code, path, span)) = location {
            (
                Some(NamedSource::new(path.into(), code.into())),
                Some(span),
            )
        } else {
            (None, None)
        };

        Self::EncodingError {
            format: format.into(),
            message: message.into(),
            error_location,
            source_code,
            expected_encoding: Some(expected_encoding.into()),
            actual_encoding: Some(actual_encoding.into()),
        }
    }

    // -------------------- Configuration Errors --------------------

    /// Create a new configuration error
    pub fn config_error(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self::ConfigError {
            field: field.into(),
            message: message.into(),
            error_location: None,
            config_source: None,
            expected_schema: None,
            suggested_fixes: Vec::new(),
        }
    }

    /// Create a new configuration error with suggestions
    pub fn config_error_with_suggestions(
        field: impl Into<String>,
        message: impl Into<String>,
        location: Option<(impl Into<String>, impl Into<String>, SourceSpan)>,
        expected_schema: Option<String>,
        suggested_fixes: Vec<String>,
    ) -> Self {
        let (config_source, error_location) = if let Some((code, path, span)) = location {
            (
                Some(NamedSource::new(path.into(), code.into())),
                Some(span),
            )
        } else {
            (None, None)
        };

        Self::ConfigError {
            field: field.into(),
            message: message.into(),
            error_location,
            config_source,
            expected_schema,
            suggested_fixes,
        }
    }

    // -------------------- Type Errors --------------------

    /// Create a new type error
    pub fn type_error(
        message: impl Into<String>,
        expected_type: impl Into<String>,
        actual_type: impl Into<String>,
    ) -> Self {
        Self::TypeError {
            message: message.into(),
            expected_type: expected_type.into(),
            actual_type: actual_type.into(),
            error_location: None,
            source_code: None,
            type_context: Vec::new(),
            type_suggestions: Vec::new(),
        }
    }

    /// Create a new type error with suggestions
    pub fn type_error_with_suggestions(
        message: impl Into<String>,
        expected_type: impl Into<String>,
        actual_type: impl Into<String>,
        location: Option<(impl Into<String>, impl Into<String>, SourceSpan)>,
        type_context: Vec<SourceSpan>,
        type_suggestions: Vec<String>,
    ) -> Self {
        let (source_code, error_location) = if let Some((code, path, span)) = location {
            (
                Some(NamedSource::new(path.into(), code.into())),
                Some(span),
            )
        } else {
            (None, None)
        };

        Self::TypeError {
            message: message.into(),
            expected_type: expected_type.into(),
            actual_type: actual_type.into(),
            error_location,
            source_code,
            type_context,
            type_suggestions,
        }
    }

    // -------------------- Memory Errors --------------------

    /// Create a new memory error
    pub fn memory_error(message: impl Into<String>, operation: impl Into<String>) -> Self {
        Self::MemoryError {
            message: message.into(),
            operation: operation.into(),
            error_location: None,
            source_code: None,
            memory_stats: None,
            heap_info: None,
        }
    }

    /// Create a new memory error with stats
    pub fn memory_error_with_stats(
        message: impl Into<String>,
        operation: impl Into<String>,
        memory_stats: impl Into<String>,
        heap_info: impl Into<String>,
        location: Option<(impl Into<String>, impl Into<String>, SourceSpan)>,
    ) -> Self {
        let (source_code, error_location) = if let Some((code, path, span)) = location {
            (
                Some(NamedSource::new(path.into(), code.into())),
                Some(span),
            )
        } else {
            (None, None)
        };

        Self::MemoryError {
            message: message.into(),
            operation: operation.into(),
            error_location,
            source_code,
            memory_stats: Some(memory_stats.into()),
            heap_info: Some(heap_info.into()),
        }
    }

    // -------------------- Module Errors --------------------

    /// Create a module not found error
    pub fn module_not_found(specifier: impl Into<String>) -> Self {
        Self::ModuleNotFound {
            specifier: specifier.into(),
            error_location: None,
            source_code: None,
            suggestions: Vec::new(),
        }
    }

    /// Create a module not found error with suggestions
    pub fn module_not_found_with_suggestions(
        specifier: impl Into<String>,
        suggestions: Vec<String>,
    ) -> Self {
        Self::ModuleNotFound {
            specifier: specifier.into(),
            error_location: None,
            source_code: None,
            suggestions,
        }
    }

    /// Create a module parse error
    pub fn module_parse_error(path: impl Into<String>, message: impl Into<String>) -> Self {
        Self::ModuleParseError {
            path: path.into(),
            message: message.into(),
            error_location: None,
            source_code: None,
        }
    }

    /// Create a module resolution error
    pub fn module_resolution_error(message: impl Into<String>) -> Self {
        Self::ModuleResolutionError {
            message: message.into(),
            specifier: None,
            referrer: None,
            error_location: None,
            source_code: None,
        }
    }

    /// Create a module resolution error with context
    pub fn module_resolution_error_with_context(
        message: impl Into<String>,
        specifier: impl Into<String>,
        referrer: Option<String>,
    ) -> Self {
        Self::ModuleResolutionError {
            message: message.into(),
            specifier: Some(specifier.into()),
            referrer,
            error_location: None,
            source_code: None,
        }
    }

    /// Create a module runtime error
    pub fn module_runtime_error(path: impl Into<String>, message: impl Into<String>) -> Self {
        Self::ModuleRuntimeError {
            path: path.into(),
            message: message.into(),
            error_location: None,
            source_code: None,
        }
    }

    /// Create a circular import error
    pub fn circular_import(cycle: impl Into<String>) -> Self {
        Self::CircularImport {
            cycle: cycle.into(),
            error_location: None,
            source_code: None,
            involved_modules: Vec::new(),
        }
    }

    /// Create a circular import error with involved modules
    pub fn circular_import_with_modules(cycle: impl Into<String>, modules: Vec<String>) -> Self {
        Self::CircularImport {
            cycle: cycle.into(),
            error_location: None,
            source_code: None,
            involved_modules: modules,
        }
    }

    /// Create an import not found error
    pub fn import_not_found(import: impl Into<String>, module: impl Into<String>) -> Self {
        Self::ImportNotFound {
            import: import.into(),
            module: module.into(),
            error_location: None,
            source_code: None,
            available_exports: Vec::new(),
        }
    }

    /// Create an import not found error with available exports
    pub fn import_not_found_with_exports(
        import: impl Into<String>,
        module: impl Into<String>,
        available_exports: Vec<String>,
    ) -> Self {
        Self::ImportNotFound {
            import: import.into(),
            module: module.into(),
            error_location: None,
            source_code: None,
            available_exports,
        }
    }

    /// Create an ambiguous export error
    pub fn ambiguous_export(export: impl Into<String>, module: impl Into<String>) -> Self {
        Self::AmbiguousExport {
            export: export.into(),
            module: module.into(),
            error_location: None,
            source_code: None,
            conflict_sources: Vec::new(),
        }
    }

    /// Create a module already loaded error
    pub fn module_already_loaded(specifier: impl Into<String>) -> Self {
        Self::ModuleAlreadyLoaded {
            specifier: specifier.into(),
            error_location: None,
            source_code: None,
        }
    }

    /// Create an invalid module specifier error
    pub fn invalid_module_specifier(specifier: impl Into<String>) -> Self {
        Self::InvalidModuleSpecifier {
            specifier: specifier.into(),
            reason: None,
            error_location: None,
            source_code: None,
        }
    }

    /// Create an invalid module specifier error with reason
    pub fn invalid_module_specifier_with_reason(
        specifier: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self::InvalidModuleSpecifier {
            specifier: specifier.into(),
            reason: Some(reason.into()),
            error_location: None,
            source_code: None,
        }
    }

    /// Create a module I/O error
    pub fn module_io_error(message: impl Into<String>) -> Self {
        Self::ModuleIoError {
            message: message.into(),
            path: None,
            error_location: None,
            source_code: None,
        }
    }

    /// Create a module I/O error with path
    pub fn module_io_error_with_path(message: impl Into<String>, path: impl Into<String>) -> Self {
        Self::ModuleIoError {
            message: message.into(),
            path: Some(path.into()),
            error_location: None,
            source_code: None,
        }
    }

    // -------------------- LLM Errors --------------------

    /// Create an LLM not initialized error
    pub fn llm_not_initialized() -> Self {
        Self::LlmNotInitialized
    }

    /// Create an LLM provider error
    pub fn llm_provider_error(message: impl Into<String>) -> Self {
        Self::LlmProviderError {
            message: message.into(),
            provider_name: None,
            error_location: None,
            source_code: None,
        }
    }

    /// Create an LLM provider error with provider name
    pub fn llm_provider_error_with_name(
        message: impl Into<String>,
        provider_name: impl Into<String>,
    ) -> Self {
        Self::LlmProviderError {
            message: message.into(),
            provider_name: Some(provider_name.into()),
            error_location: None,
            source_code: None,
        }
    }

    /// Create an LLM timeout error
    pub fn llm_timeout(timeout_ms: u64) -> Self {
        Self::LlmTimeout {
            timeout_ms,
            error_location: None,
            source_code: None,
        }
    }

    /// Create an LLM disabled error
    pub fn llm_disabled() -> Self {
        Self::LlmDisabled
    }

    /// Create an LLM authentication error
    pub fn llm_authentication_error(message: impl Into<String>) -> Self {
        Self::LlmAuthenticationError {
            message: message.into(),
            error_location: None,
            source_code: None,
        }
    }

    /// Create an LLM network error
    pub fn llm_network_error(message: impl Into<String>) -> Self {
        Self::LlmNetworkError {
            message: message.into(),
            error_location: None,
            source_code: None,
        }
    }

    // -------------------- Internal Errors --------------------

    /// Create an internal error
    pub fn internal_error(message: impl Into<String>) -> Self {
        Self::InternalError {
            message: message.into(),
            error_location: None,
            source_code: None,
        }
    }
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RuntimeError::FsError {
                operation, path, ..
            } => {
                write!(f, "File system error during {operation}: {path}")
            }
            RuntimeError::ParseError { source_path, .. } => {
                write!(f, "Parse error in {source_path}")
            }
            RuntimeError::RuntimeError { message, .. } => {
                write!(f, "Runtime error: {message}")
            }
            RuntimeError::ExtensionError {
                extension_name,
                message,
                ..
            } => {
                write!(f, "Extension '{extension_name}' error: {message}")
            }
            RuntimeError::ResourceError {
                rid,
                operation,
                message,
                ..
            } => {
                write!(f, "Resource {rid} error during {operation}: {message}")
            }
            RuntimeError::TaskError {
                task_id, message, ..
            } => {
                write!(f, "Task {task_id} error: {message}")
            }
            RuntimeError::NetworkError { url, operation, .. } => {
                write!(f, "Network error during {operation} for {url}")
            }
            RuntimeError::EncodingError {
                format, message, ..
            } => {
                write!(f, "Encoding error ({format}): {message}")
            }
            RuntimeError::ConfigError { field, message, .. } => {
                write!(f, "Configuration error in field '{field}': {message}")
            }
            RuntimeError::TypeError {
                message,
                expected_type,
                actual_type,
                ..
            } => {
                write!(
                    f,
                    "Type error: {message} (expected {expected_type}, got {actual_type})"
                )
            }
            RuntimeError::MemoryError {
                message, operation, ..
            } => {
                write!(f, "Memory error during {operation}: {message}")
            }
            RuntimeError::ModuleNotFound { specifier, .. } => {
                write!(f, "Module not found: {specifier}")
            }
            RuntimeError::ModuleParseError { path, message, .. } => {
                write!(f, "Parse error in module {path}: {message}")
            }
            RuntimeError::ModuleResolutionError { message, .. } => {
                write!(f, "Resolution error: {message}")
            }
            RuntimeError::ModuleRuntimeError { path, message, .. } => {
                write!(f, "Runtime error in module {path}: {message}")
            }
            RuntimeError::CircularImport { cycle, .. } => {
                write!(f, "Circular import detected: {cycle}")
            }
            RuntimeError::ImportNotFound { import, module, .. } => {
                write!(f, "Import not found: '{import}' in module '{module}'")
            }
            RuntimeError::AmbiguousExport { export, module, .. } => {
                write!(f, "Ambiguous export: '{export}' in module '{module}'")
            }
            RuntimeError::ModuleAlreadyLoaded { specifier, .. } => {
                write!(f, "Module already loaded: {specifier}")
            }
            RuntimeError::InvalidModuleSpecifier { specifier, .. } => {
                write!(f, "Invalid module specifier: {specifier}")
            }
            RuntimeError::ModuleIoError { message, .. } => {
                write!(f, "IO error: {message}")
            }
            RuntimeError::LlmNotInitialized => {
                write!(f, "LLM provider not initialized")
            }
            RuntimeError::LlmProviderError { message, .. } => {
                write!(f, "LLM provider error: {message}")
            }
            RuntimeError::LlmTimeout { timeout_ms, .. } => {
                write!(f, "LLM suggestion request timed out after {timeout_ms}ms")
            }
            RuntimeError::LlmDisabled => {
                write!(f, "LLM suggestions are disabled")
            }
            RuntimeError::LlmAuthenticationError { message, .. } => {
                write!(f, "LLM authentication error: {message}")
            }
            RuntimeError::LlmNetworkError { message, .. } => {
                write!(f, "LLM network error: {message}")
            }
            RuntimeError::InternalError { message, .. } => {
                write!(f, "Internal error: {message}")
            }
        }
    }
}

impl std::error::Error for RuntimeError {}

/// Result type alias for Andromeda operations with boxed errors to reduce stack size
pub type RuntimeResult<T> = Result<T, Box<RuntimeError>>;

/// Result type for module operations (convenience alias)
pub type ModuleResult<T> = Result<T, RuntimeError>;


/// Error reporting utilities with full miette integration
///
/// This struct provides methods for printing and formatting RuntimeErrors
/// with rich visual output using miette.
pub struct ErrorReporter;

impl ErrorReporter {
    /// Initialize miette with enhanced reporting capabilities.
    ///
    /// This delegates to the shared error reporting module.
    pub fn init() {
        crate::error_reporting::init_error_reporting_default();
    }

    /// Print a formatted error with full miette reporting
    pub fn print_error(error: &RuntimeError) {
        eprintln!();
        eprintln!(
            "{} {}",
            "🚨".red().bold(),
            "Andromeda Runtime Error".bright_red().bold()
        );
        eprintln!("{}", "─".repeat(50).red());
        eprintln!("{:?}", oxc_miette::Report::new(error.clone()));
    }

    /// Print multiple errors with enhanced formatting
    pub fn print_errors(errors: &[RuntimeError]) {
        eprintln!();
        eprintln!(
            "{} {} ({} error{})",
            "🚨".red().bold(),
            "Andromeda Runtime Errors".bright_red().bold(),
            errors.len(),
            if errors.len() == 1 { "" } else { "s" }
        );
        eprintln!("{}", "─".repeat(50).red());

        for (i, error) in errors.iter().enumerate() {
            if errors.len() > 1 {
                eprintln!();
                eprintln!(
                    "{} Error {} of {}:",
                    "📍".cyan(),
                    (i + 1).to_string().bright_cyan(),
                    errors.len().to_string().bright_cyan()
                );
                eprintln!("{}", "─".repeat(30).cyan());
            }
            eprintln!("{:?}", oxc_miette::Report::new(error.clone()));
        }
    }

    /// Create a formatted error report as string with full context
    #[must_use]
    pub fn format_error(error: &RuntimeError) -> String {
        format!("{:?}", oxc_miette::Report::new(error.clone()))
    }

    /// Create a comprehensive error report with suggestions and related information
    #[must_use]
    pub fn create_detailed_report(error: &RuntimeError) -> String {
        let mut report = String::new();

        // Header with emoji and color
        report.push_str(&format!("{} Andromeda Error Report\n", "🔍".bright_blue()));
        report.push_str(&format!("{}\n", "═".repeat(60).blue()));

        // Main error display
        report.push_str(&format!("{:?}\n", oxc_miette::Report::new(error.clone())));

        // Additional context based on error type
        match error {
            RuntimeError::ParseError { errors, .. } => {
                report.push_str(&format!("\n{} Parse Details:\n", "📝".bright_yellow()));
                report.push_str(&format!("Total errors found: {}\n", errors.len()));
                for (i, err) in errors.iter().enumerate() {
                    report.push_str(&format!("  {}. {}\n", i + 1, err));
                }
            }
            RuntimeError::RuntimeError {
                stack_trace: Some(stack),
                variable_context,
                ..
            } => {
                if !stack.is_empty() {
                    report.push_str(&format!("\n{} Stack Trace:\n", "📚".bright_magenta()));
                    report.push_str(stack);
                    report.push('\n');
                }
                if !variable_context.is_empty() {
                    report.push_str(&format!("\n{} Variable Context:\n", "🔍".bright_cyan()));
                    for (name, value) in variable_context {
                        report.push_str(&format!(
                            "  {} = {}\n",
                            name.bright_white(),
                            value.dimmed()
                        ));
                    }
                }
            }
            RuntimeError::NetworkError {
                status_code: Some(code),
                request_headers,
                ..
            } => {
                report.push_str(&format!("\n{} Network Details:\n", "🌐".bright_green()));
                report.push_str(&format!("Status Code: {code}\n"));
                if !request_headers.is_empty() {
                    report.push_str("Request Headers:\n");
                    for (key, value) in request_headers {
                        report.push_str(&format!("  {}: {}\n", key.bright_white(), value.dimmed()));
                    }
                }
            }
            RuntimeError::MemoryError {
                memory_stats: Some(stats),
                heap_info: Some(heap),
                ..
            } => {
                report.push_str(&format!("\n{} Memory Information:\n", "💾".bright_red()));
                report.push_str(&format!("Memory Stats: {stats}\n"));
                report.push_str(&format!("Heap Info: {heap}\n"));
            }
            RuntimeError::CircularImport {
                involved_modules, ..
            } if !involved_modules.is_empty() => {
                report.push_str(&format!("\n{} Involved Modules:\n", "🔄".bright_yellow()));
                for module in involved_modules {
                    report.push_str(&format!("  • {}\n", module));
                }
            }
            RuntimeError::ImportNotFound {
                available_exports, ..
            } if !available_exports.is_empty() => {
                report.push_str(&format!("\n{} Available Exports:\n", "📦".bright_green()));
                for export in available_exports {
                    report.push_str(&format!("  • {}\n", export));
                }
            }
            RuntimeError::ModuleNotFound { suggestions, .. } if !suggestions.is_empty() => {
                report.push_str(&format!("\n{} Did you mean:\n", "💡".bright_yellow()));
                for suggestion in suggestions {
                    report.push_str(&format!("  • {}\n", suggestion));
                }
            }
            _ => {}
        }

        report.push_str(&format!("\n{}\n", "═".repeat(60).blue()));
        report
    }
}

/// Trait for converting various error types to RuntimeError
pub trait IntoRuntimeError<T> {
    fn into_runtime_error(self) -> RuntimeResult<T>;
}

impl<T> IntoRuntimeError<T> for Result<T, std::io::Error> {
    fn into_runtime_error(self) -> RuntimeResult<T> {
        self.map_err(|e| Box::new(RuntimeError::fs_error(e, "unknown", "unknown")))
    }
}

/// Enhanced macros for quick error creation with location support
#[macro_export]
macro_rules! runtime_error {
    // File system errors
    (fs: $op:expr, $path:expr, $source:expr) => {
        $crate::RuntimeError::fs_error($source, $op, $path)
    };
    (fs: $op:expr, $path:expr, $source:expr, at $code:expr, $src_path:expr, $span:expr) => {
        $crate::RuntimeError::fs_error_with_location($source, $op, $path, $code, $src_path, $span)
    };

    // Runtime errors
    (runtime: $msg:expr) => {
        $crate::RuntimeError::runtime_error($msg)
    };
    (runtime: $msg:expr, at $code:expr, $src_path:expr, $span:expr) => {
        $crate::RuntimeError::runtime_error_with_location($msg, $code, $src_path, $span)
    };
    (runtime: $msg:expr, with context $code:expr, $src_path:expr, $span:expr, stack $stack:expr, vars $vars:expr) => {
        $crate::RuntimeError::runtime_error_with_context($msg, $code, $src_path, $span, $stack, $vars)
    };

    // Extension errors
    (extension: $name:expr, $msg:expr) => {
        $crate::RuntimeError::extension_error($name, $msg)
    };
    (extension: $name:expr, $msg:expr, at $code:expr, $src_path:expr, $span:expr, missing $deps:expr) => {
        $crate::RuntimeError::extension_error_with_context($name, $msg, $code, $src_path, $span, $deps)
    };

    // Resource errors
    (resource: $rid:expr, $op:expr, $msg:expr) => {
        $crate::RuntimeError::resource_error($rid, $op, $msg)
    };
    (resource: $rid:expr, $op:expr, $msg:expr, state $state:expr) => {
        $crate::RuntimeError::resource_error_with_context($rid, $op, $msg, $state, None::<(&str, &str, miette::SourceSpan)>)
    };
    (resource: $rid:expr, $op:expr, $msg:expr, state $state:expr, at $code:expr, $src_path:expr, $span:expr) => {
        $crate::RuntimeError::resource_error_with_context($rid, $op, $msg, $state, Some(($code, $src_path, $span)))
    };

    // Task errors
    (task: $id:expr, $msg:expr) => {
        $crate::RuntimeError::task_error($id, $msg)
    };
    (task: $id:expr, $msg:expr, history $history:expr) => {
        $crate::RuntimeError::task_error_with_history($id, $msg, $history, None::<(&str, &str, miette::SourceSpan)>)
    };
    (task: $id:expr, $msg:expr, history $history:expr, at $code:expr, $src_path:expr, $span:expr) => {
        $crate::RuntimeError::task_error_with_history($id, $msg, $history, Some(($code, $src_path, $span)))
    };

    // Network errors
    (network: $source:expr, $url:expr, $op:expr) => {
        $crate::RuntimeError::network_error($source, $url, $op)
    };
    (network: $source:expr, $url:expr, $op:expr, status $status:expr, headers $headers:expr) => {
        $crate::RuntimeError::network_error_with_context($source, $url, $op, $status, $headers, None::<(&str, &str, miette::SourceSpan)>)
    };
    (network: $source:expr, $url:expr, $op:expr, status $status:expr, headers $headers:expr, at $code:expr, $src_path:expr, $span:expr) => {
        $crate::RuntimeError::network_error_with_context($source, $url, $op, $status, $headers, Some(($code, $src_path, $span)))
    };

    // Encoding errors
    (encoding: $format:expr, $msg:expr) => {
        $crate::RuntimeError::encoding_error($format, $msg)
    };
    (encoding: $format:expr, $msg:expr, expected $expected:expr, actual $actual:expr) => {
        $crate::RuntimeError::encoding_error_with_context($format, $msg, $expected, $actual, None::<(&str, &str, miette::SourceSpan)>)
    };
    (encoding: $format:expr, $msg:expr, expected $expected:expr, actual $actual:expr, at $code:expr, $src_path:expr, $span:expr) => {
        $crate::RuntimeError::encoding_error_with_context($format, $msg, $expected, $actual, Some(($code, $src_path, $span)))
    };

    // Configuration errors
    (config: $field:expr, $msg:expr) => {
        $crate::RuntimeError::config_error($field, $msg)
    };
    (config: $field:expr, $msg:expr, schema $schema:expr, fixes $fixes:expr) => {
        $crate::RuntimeError::config_error_with_suggestions($field, $msg, None::<(&str, &str, miette::SourceSpan)>, $schema, $fixes)
    };
    (config: $field:expr, $msg:expr, schema $schema:expr, fixes $fixes:expr, at $config:expr, $cfg_path:expr, $span:expr) => {
        $crate::RuntimeError::config_error_with_suggestions($field, $msg, Some(($config, $cfg_path, $span)), $schema, $fixes)
    };

    // Type errors
    (type_error: $msg:expr, expected $expected:expr, actual $actual:expr) => {
        $crate::RuntimeError::type_error($msg, $expected, $actual)
    };
    (type_error: $msg:expr, expected $expected:expr, actual $actual:expr, suggestions $suggestions:expr) => {
        $crate::RuntimeError::type_error_with_suggestions($msg, $expected, $actual, None::<(&str, &str, miette::SourceSpan)>, vec![], $suggestions)
    };
    (type_error: $msg:expr, expected $expected:expr, actual $actual:expr, suggestions $suggestions:expr, at $code:expr, $src_path:expr, $span:expr) => {
        $crate::RuntimeError::type_error_with_suggestions($msg, $expected, $actual, Some(($code, $src_path, $span)), vec![], $suggestions)
    };

    // Memory errors
    (memory: $msg:expr, $op:expr) => {
        $crate::RuntimeError::memory_error($msg, $op)
    };
    (memory: $msg:expr, $op:expr, stats $stats:expr, heap $heap:expr) => {
        $crate::RuntimeError::memory_error_with_stats($msg, $op, $stats, $heap, None::<(&str, &str, miette::SourceSpan)>)
    };
    (memory: $msg:expr, $op:expr, stats $stats:expr, heap $heap:expr, at $code:expr, $src_path:expr, $span:expr) => {
        $crate::RuntimeError::memory_error_with_stats($msg, $op, $stats, $heap, Some(($code, $src_path, $span)))
    };

    // Module errors
    (module_not_found: $specifier:expr) => {
        $crate::RuntimeError::module_not_found($specifier)
    };
    (module_not_found: $specifier:expr, suggestions $suggestions:expr) => {
        $crate::RuntimeError::module_not_found_with_suggestions($specifier, $suggestions)
    };
    (module_parse: $path:expr, $msg:expr) => {
        $crate::RuntimeError::module_parse_error($path, $msg)
    };
    (module_resolution: $msg:expr) => {
        $crate::RuntimeError::module_resolution_error($msg)
    };
    (module_resolution: $msg:expr, specifier $spec:expr) => {
        $crate::RuntimeError::module_resolution_error_with_context($msg, $spec, None)
    };
    (module_resolution: $msg:expr, specifier $spec:expr, referrer $ref:expr) => {
        $crate::RuntimeError::module_resolution_error_with_context($msg, $spec, Some($ref.to_string()))
    };
    (module_runtime: $path:expr, $msg:expr) => {
        $crate::RuntimeError::module_runtime_error($path, $msg)
    };
    (circular_import: $cycle:expr) => {
        $crate::RuntimeError::circular_import($cycle)
    };
    (circular_import: $cycle:expr, modules $modules:expr) => {
        $crate::RuntimeError::circular_import_with_modules($cycle, $modules)
    };
    (import_not_found: $import:expr, in $module:expr) => {
        $crate::RuntimeError::import_not_found($import, $module)
    };
    (import_not_found: $import:expr, in $module:expr, available $exports:expr) => {
        $crate::RuntimeError::import_not_found_with_exports($import, $module, $exports)
    };
    (ambiguous_export: $export:expr, in $module:expr) => {
        $crate::RuntimeError::ambiguous_export($export, $module)
    };
    (module_already_loaded: $specifier:expr) => {
        $crate::RuntimeError::module_already_loaded($specifier)
    };
    (invalid_specifier: $specifier:expr) => {
        $crate::RuntimeError::invalid_module_specifier($specifier)
    };
    (invalid_specifier: $specifier:expr, reason $reason:expr) => {
        $crate::RuntimeError::invalid_module_specifier_with_reason($specifier, $reason)
    };
    (module_io: $msg:expr) => {
        $crate::RuntimeError::module_io_error($msg)
    };
    (module_io: $msg:expr, path $path:expr) => {
        $crate::RuntimeError::module_io_error_with_path($msg, $path)
    };

    // LLM errors
    (llm_not_initialized) => {
        $crate::RuntimeError::llm_not_initialized()
    };
    (llm_provider: $msg:expr) => {
        $crate::RuntimeError::llm_provider_error($msg)
    };
    (llm_provider: $msg:expr, name $name:expr) => {
        $crate::RuntimeError::llm_provider_error_with_name($msg, $name)
    };
    (llm_timeout: $ms:expr) => {
        $crate::RuntimeError::llm_timeout($ms)
    };
    (llm_disabled) => {
        $crate::RuntimeError::llm_disabled()
    };
    (llm_auth: $msg:expr) => {
        $crate::RuntimeError::llm_authentication_error($msg)
    };
    (llm_network: $msg:expr) => {
        $crate::RuntimeError::llm_network_error($msg)
    };

    // Internal errors
    (internal: $msg:expr) => {
        $crate::RuntimeError::internal_error($msg)
    };
}

/// Convenience macro for creating source spans
#[macro_export]
macro_rules! span {
    ($start:expr, $len:expr) => {
        miette::SourceSpan::new($start.into(), $len)
    };
    ($range:expr) => {
        miette::SourceSpan::new($range.start.into(), $range.end - $range.start)
    };
}

/// Convenience macro for creating named source with content
#[macro_export]
macro_rules! named_source {
    ($name:expr, $content:expr) => {
        miette::NamedSource::new($name, $content)
    };
}
