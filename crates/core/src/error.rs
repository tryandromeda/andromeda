// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

#![allow(unused_assignments)]

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
            "Check that the file exists, the path is correct, and you have permission to access it."
        )
    )]
    FsError {
        operation: String,
        path: String,
        error_message: String,
        #[label("file operation failed here")]
        error_location: Option<SourceSpan>,
        #[source_code]
        source_code: Option<NamedSource<String>>,
    },

    /// Parse errors from JavaScript/TypeScript source
    #[diagnostic(
        code(andromeda::parse::syntax_error),
        help("Check for missing semicolons, brackets, or quotes.")
    )]
    ParseError {
        errors: Vec<OxcDiagnostic>,
        source_path: String,
        #[source_code]
        source_code: NamedSource<String>,
        #[label("parse error occurred here")]
        primary_error_span: Option<SourceSpan>,
        related_spans: Vec<SourceSpan>,
    },

    /// Runtime execution errors
    #[diagnostic(
        code(andromeda::runtime::execution_error),
        help(
            "Verify that all variables are initialized and review the call stack for the error source."
        )
    )]
    RuntimeError {
        message: String,
        #[label("runtime error occurred here")]
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
            "Check that the extension is configured correctly and all required dependencies are installed."
        )
    )]
    ExtensionError {
        extension_name: String,
        message: String,
        #[label("extension error occurred here")]
        error_location: Option<SourceSpan>,
        #[source_code]
        extension_source: Option<NamedSource<String>>,
        /// Related extension dependencies that might be missing
        missing_dependencies: Vec<String>,
    },

    /// Resource management errors
    #[diagnostic(
        code(andromeda::resource::invalid_rid),
        help("Ensure the resource ID is valid and the resource has not been closed.")
    )]
    ResourceError {
        rid: u32,
        operation: String,
        message: String,
        #[label("resource operation failed here")]
        error_location: Option<SourceSpan>,
        #[source_code]
        source_code: Option<NamedSource<String>>,
        /// Current resource state for debugging
        resource_state: String,
    },

    /// Task management errors
    #[diagnostic(
        code(andromeda::task::task_error),
        help("Check task state and async synchronization for deadlocks or improper cleanup.")
    )]
    TaskError {
        task_id: u32,
        message: String,
        #[label("task error occurred here")]
        error_location: Option<SourceSpan>,
        #[source_code]
        source_code: Option<NamedSource<String>>,
        /// Task execution history for debugging
        execution_history: Vec<String>,
    },

    /// Network/HTTP errors
    #[diagnostic(
        code(andromeda::network::request_error),
        help("Check the URL format, network connectivity, and any proxy or firewall settings.")
    )]
    NetworkError {
        url: String,
        operation: String,
        error_message: String,
        #[label("network error occurred here")]
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
        help("Verify the input data is well-formed and matches the expected encoding.")
    )]
    EncodingError {
        format: String,
        message: String,
        #[label("encoding error occurred here")]
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
        help("Verify the configuration file syntax and that all required fields are present.")
    )]
    ConfigError {
        field: String,
        message: String,
        #[label("configuration error occurred here")]
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
            "Check the types of variables and function parameters; consider an explicit conversion."
        )
    )]
    TypeError {
        message: String,
        expected_type: String,
        actual_type: String,
        #[label("type error occurred here")]
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
        help("Reduce memory footprint or investigate possible leaks and excessive allocations.")
    )]
    MemoryError {
        message: String,
        operation: String,
        #[label("memory error occurred here")]
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
        help("Check that the module path is correct and the file exists.")
    )]
    ModuleNotFound {
        specifier: String,
        #[label("module not found")]
        error_location: Option<SourceSpan>,
        #[source_code]
        source_code: Option<NamedSource<String>>,
        /// Suggested similar module names
        suggestions: Vec<String>,
    },

    /// Module parse error
    #[diagnostic(
        code(andromeda::module::parse_error),
        help("Check the syntax of the module source code.")
    )]
    ModuleParseError {
        path: String,
        message: String,
        #[label("module parse error")]
        error_location: Option<SourceSpan>,
        #[source_code]
        source_code: Option<NamedSource<String>>,
    },

    /// Module resolution error
    #[diagnostic(
        code(andromeda::module::resolution_error),
        help("Check import paths and verify the resolver can find the file.")
    )]
    ModuleResolutionError {
        message: String,
        specifier: Option<String>,
        referrer: Option<String>,
        #[label("resolution failed here")]
        error_location: Option<SourceSpan>,
        #[source_code]
        source_code: Option<NamedSource<String>>,
    },

    /// Module runtime error
    #[diagnostic(
        code(andromeda::module::runtime_error),
        help("Review the module's execution logic and verify all imports resolve correctly.")
    )]
    ModuleRuntimeError {
        path: String,
        message: String,
        #[label("module runtime error")]
        error_location: Option<SourceSpan>,
        #[source_code]
        source_code: Option<NamedSource<String>>,
    },

    /// Circular import detected
    #[diagnostic(
        code(andromeda::module::circular_import),
        help("Restructure modules to break the cycle, or use dynamic imports.")
    )]
    CircularImport {
        cycle: String,
        #[label("circular import detected")]
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
            "Verify the export name matches exactly (case-sensitive) and is exported by the module."
        )
    )]
    ImportNotFound {
        import: String,
        module: String,
        #[label("import not found")]
        error_location: Option<SourceSpan>,
        #[source_code]
        source_code: Option<NamedSource<String>>,
        /// Available exports from the module
        available_exports: Vec<String>,
    },

    /// Ambiguous export in module
    #[diagnostic(
        code(andromeda::module::ambiguous_export),
        help("Use explicit re-exports to disambiguate, and check for conflicting star exports.")
    )]
    AmbiguousExport {
        export: String,
        module: String,
        #[label("ambiguous export")]
        error_location: Option<SourceSpan>,
        #[source_code]
        source_code: Option<NamedSource<String>>,
        /// Sources of the conflicting exports
        conflict_sources: Vec<String>,
    },

    /// Module already loaded
    #[diagnostic(
        code(andromeda::module::already_loaded),
        help("Check for duplicate module registrations.")
    )]
    ModuleAlreadyLoaded {
        specifier: String,
        #[label("module already loaded")]
        error_location: Option<SourceSpan>,
        #[source_code]
        source_code: Option<NamedSource<String>>,
    },

    /// Invalid module specifier
    #[diagnostic(
        code(andromeda::module::invalid_specifier),
        help("Use a relative path (./), absolute path (/), bare specifier, or a well-formed URL.")
    )]
    InvalidModuleSpecifier {
        specifier: String,
        reason: Option<String>,
        #[label("invalid module specifier")]
        error_location: Option<SourceSpan>,
        #[source_code]
        source_code: Option<NamedSource<String>>,
    },

    /// Module I/O error
    #[diagnostic(
        code(andromeda::module::io_error),
        help("Check file permissions, disk space, and that the path is accessible.")
    )]
    ModuleIoError {
        message: String,
        path: Option<String>,
        #[label("I/O error")]
        error_location: Option<SourceSpan>,
        #[source_code]
        source_code: Option<NamedSource<String>>,
    },

    /// LLM provider not initialized
    #[diagnostic(
        code(andromeda::llm::not_initialized),
        help("Call init_llm_provider() or init_copilot_provider() before requesting suggestions.")
    )]
    LlmNotInitialized,

    /// LLM provider error
    #[diagnostic(
        code(andromeda::llm::provider_error),
        help("Verify the LLM provider configuration, API keys, and network connectivity.")
    )]
    LlmProviderError {
        message: String,
        provider_name: Option<String>,
        #[label("LLM provider error")]
        error_location: Option<SourceSpan>,
        #[source_code]
        source_code: Option<NamedSource<String>>,
    },

    /// LLM request timeout
    #[diagnostic(
        code(andromeda::llm::timeout),
        help("Increase the timeout duration or check network latency to the LLM service.")
    )]
    LlmTimeout {
        timeout_ms: u64,
        #[label("request timed out")]
        error_location: Option<SourceSpan>,
        #[source_code]
        source_code: Option<NamedSource<String>>,
    },

    /// LLM suggestions disabled
    #[diagnostic(
        code(andromeda::llm::disabled),
        help("Set enabled: true in LlmSuggestionConfig to turn on LLM suggestions.")
    )]
    LlmDisabled,

    /// LLM authentication error
    #[diagnostic(
        code(andromeda::llm::auth_error),
        help(
            "Verify API keys or tokens are valid; ensure GITHUB_TOKEN is set for Copilot integration."
        )
    )]
    LlmAuthenticationError {
        message: String,
        #[label("authentication failed")]
        error_location: Option<SourceSpan>,
        #[source_code]
        source_code: Option<NamedSource<String>>,
    },

    /// LLM network error
    #[diagnostic(
        code(andromeda::llm::network_error),
        help("Check network connectivity, firewall and proxy settings, and the LLM endpoint URL.")
    )]
    LlmNetworkError {
        message: String,
        #[label("network error")]
        error_location: Option<SourceSpan>,
        #[source_code]
        source_code: Option<NamedSource<String>>,
    },

    /// Windowing / native window subsystem errors (winit-backed extension)
    #[diagnostic(
        code(andromeda::window::error),
        help(
            "Verify the window feature is enabled, the platform supports native windowing, and operations run on the main thread."
        )
    )]
    WindowError {
        operation: String,
        message: String,
        #[label("window operation failed here")]
        error_location: Option<SourceSpan>,
        #[source_code]
        source_code: Option<NamedSource<String>>,
    },

    /// Subprocess execution errors
    #[diagnostic(
        code(andromeda::command::execution_failed),
        help("Check that the command exists on PATH, is executable, and accepts the given arguments.")
    )]
    CommandError {
        program: String,
        operation: String,
        message: String,
        exit_code: Option<i32>,
        #[label("command failed here")]
        error_location: Option<SourceSpan>,
        #[source_code]
        source_code: Option<NamedSource<String>>,
    },

    /// Process / signal management errors
    #[diagnostic(
        code(andromeda::process::operation_failed),
        help("Verify the signal is supported on this platform and the target process exists.")
    )]
    ProcessError {
        operation: String,
        message: String,
        signal: Option<String>,
        platform: String,
        #[label("process operation failed here")]
        error_location: Option<SourceSpan>,
        #[source_code]
        source_code: Option<NamedSource<String>>,
    },

    /// TLS / SSL operation errors
    #[diagnostic(
        code(andromeda::tls::operation_failed),
        help("Verify the peer's certificate chain and that the configured CA bundle is correct.")
    )]
    TlsError {
        operation: String,
        message: String,
        peer: Option<String>,
        certificate_info: Option<String>,
        #[label("TLS operation failed here")]
        error_location: Option<SourceSpan>,
        #[source_code]
        source_code: Option<NamedSource<String>>,
    },

    /// Web Locks API errors
    #[diagnostic(
        code(andromeda::lock::operation_failed),
        help("Check whether the lock name is already held; review for nested or cyclic acquisitions.")
    )]
    LockError {
        lock_name: String,
        operation: String,
        message: String,
        is_deadlock: bool,
        #[label("lock operation failed here")]
        error_location: Option<SourceSpan>,
        #[source_code]
        source_code: Option<NamedSource<String>>,
    },

    /// Worker thread errors
    #[diagnostic(
        code(andromeda::worker::operation_failed),
        help("Verify the worker script path resolves and the host has resources to spawn a thread.")
    )]
    WorkerError {
        worker_id: Option<u32>,
        operation: String,
        message: String,
        #[label("worker operation failed here")]
        error_location: Option<SourceSpan>,
        #[source_code]
        source_code: Option<NamedSource<String>>,
    },

    /// Database / SQLite errors
    #[diagnostic(
        code(andromeda::database::operation_failed),
        help("Check the SQL statement, database file permissions, and that the database is initialized.")
    )]
    DatabaseError {
        operation: String,
        database_name: Option<String>,
        message: String,
        sql: Option<String>,
        #[label("database operation failed here")]
        error_location: Option<SourceSpan>,
        #[source_code]
        source_code: Option<NamedSource<String>>,
    },

    /// Web Crypto operation errors
    #[diagnostic(
        code(andromeda::crypto::operation_failed),
        help("Verify the algorithm name, key length, and that the requested usage matches the key's usages.")
    )]
    CryptoError {
        operation: String,
        algorithm: String,
        message: String,
        key_usage: Option<String>,
        #[label("crypto operation failed here")]
        error_location: Option<SourceSpan>,
        #[source_code]
        source_code: Option<NamedSource<String>>,
    },

    /// URL parse errors
    #[diagnostic(
        code(andromeda::url::parse_failed),
        help("Use a valid URL with a scheme (http://, https://, file://, etc.) or a relative path resolvable against the base.")
    )]
    UrlParseError {
        url: String,
        reason: String,
        #[label("URL parse failed here")]
        error_location: Option<SourceSpan>,
        #[source_code]
        source_code: Option<NamedSource<String>>,
    },

    /// Web Storage / Cache Storage errors
    #[diagnostic(
        code(andromeda::storage::operation_failed),
        help("Check storage quota, file permissions on the storage backend, and the operation arguments.")
    )]
    StorageError {
        store_type: String,
        operation: String,
        message: String,
        quota_exceeded: bool,
        #[label("storage operation failed here")]
        error_location: Option<SourceSpan>,
        #[source_code]
        source_code: Option<NamedSource<String>>,
    },

    /// FFI (foreign function interface) errors
    #[diagnostic(
        code(andromeda::ffi::operation_failed),
        help("Verify the library path, symbol name, and argument types match the foreign declaration.")
    )]
    FfiCallError {
        operation: String,
        library: Option<String>,
        message: String,
        #[label("FFI operation failed here")]
        error_location: Option<SourceSpan>,
        #[source_code]
        source_code: Option<NamedSource<String>>,
    },

    /// HTTP module load errors
    #[diagnostic(
        code(andromeda::module::http_load_failed),
        help("Check the URL is reachable, the server returns 2xx, and the module syntax is valid.")
    )]
    HttpModuleLoadError {
        url: String,
        operation: String,
        status_code: Option<u16>,
        message: String,
        #[label("HTTP module load failed here")]
        error_location: Option<SourceSpan>,
        #[source_code]
        source_code: Option<NamedSource<String>>,
    },

    /// Import map configuration errors
    #[diagnostic(
        code(andromeda::module::import_map_invalid),
        help("Verify the import map is valid JSON and each entry maps a bare specifier to a resolvable URL or path.")
    )]
    ImportMapError {
        field: String,
        value: Option<String>,
        message: String,
        #[label("import map error here")]
        error_location: Option<SourceSpan>,
        #[source_code]
        source_code: Option<NamedSource<String>>,
    },

    /// Internal error (should not happen in normal operation)
    #[diagnostic(
        code(andromeda::internal::error),
        help(
            "This is an internal error. Please report it on GitHub with the error message and reproduction steps."
        ),
        url("https://github.com/aspect-build/andromeda/issues")
    )]
    InternalError {
        message: String,
        #[label("internal error")]
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
            (Some(NamedSource::new(path.into(), code.into())), Some(span))
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
            (Some(NamedSource::new(path.into(), code.into())), Some(span))
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
            (Some(NamedSource::new(path.into(), code.into())), Some(span))
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
            (Some(NamedSource::new(path.into(), code.into())), Some(span))
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
            (Some(NamedSource::new(path.into(), code.into())), Some(span))
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
            (Some(NamedSource::new(path.into(), code.into())), Some(span))
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
            (Some(NamedSource::new(path.into(), code.into())), Some(span))
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

    /// Create a new window error
    pub fn window_error(operation: impl Into<String>, message: impl Into<String>) -> Self {
        Self::WindowError {
            operation: operation.into(),
            message: message.into(),
            error_location: None,
            source_code: None,
        }
    }

    // -------------------- Command / Process Errors --------------------

    /// Create a new subprocess execution error
    pub fn command_error(
        program: impl Into<String>,
        operation: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self::CommandError {
            program: program.into(),
            operation: operation.into(),
            message: message.into(),
            exit_code: None,
            error_location: None,
            source_code: None,
        }
    }

    /// Create a new subprocess execution error with exit code
    pub fn command_error_with_exit_code(
        program: impl Into<String>,
        operation: impl Into<String>,
        message: impl Into<String>,
        exit_code: i32,
    ) -> Self {
        Self::CommandError {
            program: program.into(),
            operation: operation.into(),
            message: message.into(),
            exit_code: Some(exit_code),
            error_location: None,
            source_code: None,
        }
    }

    /// Create a new process / signal management error
    pub fn process_error(operation: impl Into<String>, message: impl Into<String>) -> Self {
        Self::ProcessError {
            operation: operation.into(),
            message: message.into(),
            signal: None,
            platform: std::env::consts::OS.to_string(),
            error_location: None,
            source_code: None,
        }
    }

    /// Create a new process error with signal information
    pub fn process_error_with_signal(
        operation: impl Into<String>,
        message: impl Into<String>,
        signal: impl Into<String>,
    ) -> Self {
        Self::ProcessError {
            operation: operation.into(),
            message: message.into(),
            signal: Some(signal.into()),
            platform: std::env::consts::OS.to_string(),
            error_location: None,
            source_code: None,
        }
    }

    // -------------------- TLS Errors --------------------

    /// Create a new TLS error
    pub fn tls_error(operation: impl Into<String>, message: impl Into<String>) -> Self {
        Self::TlsError {
            operation: operation.into(),
            message: message.into(),
            peer: None,
            certificate_info: None,
            error_location: None,
            source_code: None,
        }
    }

    /// Create a new TLS error with peer context
    pub fn tls_error_with_peer(
        operation: impl Into<String>,
        message: impl Into<String>,
        peer: impl Into<String>,
    ) -> Self {
        Self::TlsError {
            operation: operation.into(),
            message: message.into(),
            peer: Some(peer.into()),
            certificate_info: None,
            error_location: None,
            source_code: None,
        }
    }

    // -------------------- Lock Errors --------------------

    /// Create a new Web Locks API error
    pub fn lock_error(
        lock_name: impl Into<String>,
        operation: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self::LockError {
            lock_name: lock_name.into(),
            operation: operation.into(),
            message: message.into(),
            is_deadlock: false,
            error_location: None,
            source_code: None,
        }
    }

    /// Create a new Web Locks API error flagged as a deadlock
    pub fn lock_deadlock_error(
        lock_name: impl Into<String>,
        operation: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self::LockError {
            lock_name: lock_name.into(),
            operation: operation.into(),
            message: message.into(),
            is_deadlock: true,
            error_location: None,
            source_code: None,
        }
    }

    // -------------------- Worker Errors --------------------

    /// Create a new worker thread error
    pub fn worker_error(
        worker_id: Option<u32>,
        operation: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self::WorkerError {
            worker_id,
            operation: operation.into(),
            message: message.into(),
            error_location: None,
            source_code: None,
        }
    }

    // -------------------- Database Errors --------------------

    /// Create a new database / SQLite error
    pub fn database_error(operation: impl Into<String>, message: impl Into<String>) -> Self {
        Self::DatabaseError {
            operation: operation.into(),
            database_name: None,
            message: message.into(),
            sql: None,
            error_location: None,
            source_code: None,
        }
    }

    /// Create a database error with full SQL/database context
    pub fn database_error_with_context(
        operation: impl Into<String>,
        message: impl Into<String>,
        database_name: Option<String>,
        sql: Option<String>,
    ) -> Self {
        Self::DatabaseError {
            operation: operation.into(),
            database_name,
            message: message.into(),
            sql,
            error_location: None,
            source_code: None,
        }
    }

    // -------------------- Crypto Errors --------------------

    /// Create a new Web Crypto error
    pub fn crypto_error(
        operation: impl Into<String>,
        algorithm: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self::CryptoError {
            operation: operation.into(),
            algorithm: algorithm.into(),
            message: message.into(),
            key_usage: None,
            error_location: None,
            source_code: None,
        }
    }

    /// Create a new Web Crypto error with key usage context
    pub fn crypto_error_with_key_usage(
        operation: impl Into<String>,
        algorithm: impl Into<String>,
        message: impl Into<String>,
        key_usage: impl Into<String>,
    ) -> Self {
        Self::CryptoError {
            operation: operation.into(),
            algorithm: algorithm.into(),
            message: message.into(),
            key_usage: Some(key_usage.into()),
            error_location: None,
            source_code: None,
        }
    }

    // -------------------- URL Parse Errors --------------------

    /// Create a new URL parse error
    pub fn url_parse_error(url: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::UrlParseError {
            url: url.into(),
            reason: reason.into(),
            error_location: None,
            source_code: None,
        }
    }

    // -------------------- Storage Errors --------------------

    /// Create a new Web Storage / Cache Storage error
    pub fn storage_error(
        store_type: impl Into<String>,
        operation: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self::StorageError {
            store_type: store_type.into(),
            operation: operation.into(),
            message: message.into(),
            quota_exceeded: false,
            error_location: None,
            source_code: None,
        }
    }

    /// Create a new storage quota-exceeded error
    pub fn storage_quota_exceeded(
        store_type: impl Into<String>,
        operation: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self::StorageError {
            store_type: store_type.into(),
            operation: operation.into(),
            message: message.into(),
            quota_exceeded: true,
            error_location: None,
            source_code: None,
        }
    }

    // -------------------- FFI Errors --------------------

    /// Create a new FFI error
    pub fn ffi_call_error(operation: impl Into<String>, message: impl Into<String>) -> Self {
        Self::FfiCallError {
            operation: operation.into(),
            library: None,
            message: message.into(),
            error_location: None,
            source_code: None,
        }
    }

    /// Create a new FFI error with library context
    pub fn ffi_call_error_with_library(
        operation: impl Into<String>,
        library: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self::FfiCallError {
            operation: operation.into(),
            library: Some(library.into()),
            message: message.into(),
            error_location: None,
            source_code: None,
        }
    }

    // -------------------- HTTP Module Load Errors --------------------

    /// Create a new HTTP module load error
    pub fn http_module_load_error(
        url: impl Into<String>,
        operation: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self::HttpModuleLoadError {
            url: url.into(),
            operation: operation.into(),
            status_code: None,
            message: message.into(),
            error_location: None,
            source_code: None,
        }
    }

    /// Create a new HTTP module load error with status code
    pub fn http_module_load_error_with_status(
        url: impl Into<String>,
        operation: impl Into<String>,
        status_code: u16,
        message: impl Into<String>,
    ) -> Self {
        Self::HttpModuleLoadError {
            url: url.into(),
            operation: operation.into(),
            status_code: Some(status_code),
            message: message.into(),
            error_location: None,
            source_code: None,
        }
    }

    // -------------------- Import Map Errors --------------------

    /// Create a new import map error
    pub fn import_map_error(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self::ImportMapError {
            field: field.into(),
            value: None,
            message: message.into(),
            error_location: None,
            source_code: None,
        }
    }

    /// Create a new import map error with the offending value
    pub fn import_map_error_with_value(
        field: impl Into<String>,
        value: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self::ImportMapError {
            field: field.into(),
            value: Some(value.into()),
            message: message.into(),
            error_location: None,
            source_code: None,
        }
    }

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
            RuntimeError::WindowError {
                operation, message, ..
            } => {
                write!(f, "Window error during {operation}: {message}")
            }
            RuntimeError::CommandError {
                program,
                operation,
                message,
                exit_code,
                ..
            } => match exit_code {
                Some(code) => write!(
                    f,
                    "Command '{program}' error during {operation}: {message} (exit code {code})"
                ),
                None => write!(f, "Command '{program}' error during {operation}: {message}"),
            },
            RuntimeError::ProcessError {
                operation,
                message,
                signal,
                platform,
                ..
            } => match signal {
                Some(sig) => write!(
                    f,
                    "Process error during {operation} (signal {sig}, platform {platform}): {message}"
                ),
                None => write!(
                    f,
                    "Process error during {operation} (platform {platform}): {message}"
                ),
            },
            RuntimeError::TlsError {
                operation,
                message,
                peer,
                ..
            } => match peer {
                Some(p) => write!(f, "TLS error during {operation} with peer {p}: {message}"),
                None => write!(f, "TLS error during {operation}: {message}"),
            },
            RuntimeError::LockError {
                lock_name,
                operation,
                message,
                is_deadlock,
                ..
            } => {
                if *is_deadlock {
                    write!(
                        f,
                        "Deadlock detected on lock '{lock_name}' during {operation}: {message}"
                    )
                } else {
                    write!(f, "Lock '{lock_name}' error during {operation}: {message}")
                }
            }
            RuntimeError::WorkerError {
                worker_id,
                operation,
                message,
                ..
            } => match worker_id {
                Some(id) => write!(f, "Worker {id} error during {operation}: {message}"),
                None => write!(f, "Worker error during {operation}: {message}"),
            },
            RuntimeError::DatabaseError {
                operation,
                database_name,
                message,
                ..
            } => match database_name {
                Some(db) => write!(
                    f,
                    "Database '{db}' error during {operation}: {message}"
                ),
                None => write!(f, "Database error during {operation}: {message}"),
            },
            RuntimeError::CryptoError {
                operation,
                algorithm,
                message,
                ..
            } => write!(
                f,
                "Crypto error during {operation} ({algorithm}): {message}"
            ),
            RuntimeError::UrlParseError { url, reason, .. } => {
                write!(f, "Failed to parse URL '{url}': {reason}")
            }
            RuntimeError::StorageError {
                store_type,
                operation,
                message,
                quota_exceeded,
                ..
            } => {
                if *quota_exceeded {
                    write!(
                        f,
                        "Storage '{store_type}' quota exceeded during {operation}: {message}"
                    )
                } else {
                    write!(
                        f,
                        "Storage '{store_type}' error during {operation}: {message}"
                    )
                }
            }
            RuntimeError::FfiCallError {
                operation,
                library,
                message,
                ..
            } => match library {
                Some(lib) => write!(f, "FFI error during {operation} ({lib}): {message}"),
                None => write!(f, "FFI error during {operation}: {message}"),
            },
            RuntimeError::HttpModuleLoadError {
                url,
                operation,
                status_code,
                message,
                ..
            } => match status_code {
                Some(code) => write!(
                    f,
                    "HTTP module load failed during {operation} for {url} (status {code}): {message}"
                ),
                None => write!(
                    f,
                    "HTTP module load failed during {operation} for {url}: {message}"
                ),
            },
            RuntimeError::ImportMapError {
                field,
                value,
                message,
                ..
            } => match value {
                Some(v) => write!(f, "Import map error in field '{field}' (value '{v}'): {message}"),
                None => write!(f, "Import map error in field '{field}': {message}"),
            },
            RuntimeError::InternalError { message, .. } => {
                write!(f, "Internal error: {message}")
            }
        }
    }
}

impl std::error::Error for RuntimeError {}

/// Result type alias for Andromeda operations with boxed errors to reduce stack size
pub type RuntimeResult<T> = Result<T, Box<RuntimeError>>;

/// Result type for module operations (uses boxed error to reduce stack size)
pub type ModuleResult<T> = Result<T, Box<RuntimeError>>;

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
        eprintln!("{:?}", oxc_miette::Report::new(error.clone()));
    }

    /// Print multiple errors with enhanced formatting
    pub fn print_errors(errors: &[RuntimeError]) {
        for error in errors {
            eprintln!("{:?}", oxc_miette::Report::new(error.clone()));
        }
        if errors.len() > 1 {
            eprintln!();
            let plural = if errors.len() == 1 { "" } else { "s" };
            eprintln!("Found {} error{}.", errors.len(), plural);
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

        report.push_str(&format!("{:?}\n", oxc_miette::Report::new(error.clone())));

        match error {
            RuntimeError::ParseError { errors, .. } => {
                report.push_str("\nParse details:\n");
                report.push_str(&format!("  Total errors: {}\n", errors.len()));
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
                    report.push_str("\nStack trace:\n");
                    report.push_str(stack);
                    report.push('\n');
                }
                if !variable_context.is_empty() {
                    report.push_str("\nVariable context:\n");
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
                report.push_str("\nNetwork details:\n");
                report.push_str(&format!("  Status code: {code}\n"));
                if !request_headers.is_empty() {
                    report.push_str("  Request headers:\n");
                    for (key, value) in request_headers {
                        report.push_str(&format!(
                            "    {}: {}\n",
                            key.bright_white(),
                            value.dimmed()
                        ));
                    }
                }
            }
            RuntimeError::MemoryError {
                memory_stats: Some(stats),
                heap_info: Some(heap),
                ..
            } => {
                report.push_str("\nMemory information:\n");
                report.push_str(&format!("  Memory stats: {stats}\n"));
                report.push_str(&format!("  Heap info: {heap}\n"));
            }
            RuntimeError::CircularImport {
                involved_modules, ..
            } if !involved_modules.is_empty() => {
                report.push_str("\nInvolved modules:\n");
                for module in involved_modules {
                    report.push_str(&format!("  - {}\n", module));
                }
            }
            RuntimeError::ImportNotFound {
                available_exports, ..
            } if !available_exports.is_empty() => {
                report.push_str("\nAvailable exports:\n");
                for export in available_exports {
                    report.push_str(&format!("  - {}\n", export));
                }
            }
            RuntimeError::ModuleNotFound { suggestions, .. } if !suggestions.is_empty() => {
                report.push_str("\nDid you mean:\n");
                for suggestion in suggestions {
                    report.push_str(&format!("  - {}\n", suggestion));
                }
            }
            _ => {}
        }

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
        $crate::RuntimeError::runtime_error_with_context(
            $msg, $code, $src_path, $span, $stack, $vars,
        )
    };

    // Extension errors
    (extension: $name:expr, $msg:expr) => {
        $crate::RuntimeError::extension_error($name, $msg)
    };
    (extension: $name:expr, $msg:expr, at $code:expr, $src_path:expr, $span:expr, missing $deps:expr) => {
        $crate::RuntimeError::extension_error_with_context(
            $name, $msg, $code, $src_path, $span, $deps,
        )
    };

    // Resource errors
    (resource: $rid:expr, $op:expr, $msg:expr) => {
        $crate::RuntimeError::resource_error($rid, $op, $msg)
    };
    (resource: $rid:expr, $op:expr, $msg:expr, state $state:expr) => {
        $crate::RuntimeError::resource_error_with_context(
            $rid,
            $op,
            $msg,
            $state,
            None::<(&str, &str, miette::SourceSpan)>,
        )
    };
    (resource: $rid:expr, $op:expr, $msg:expr, state $state:expr, at $code:expr, $src_path:expr, $span:expr) => {
        $crate::RuntimeError::resource_error_with_context(
            $rid,
            $op,
            $msg,
            $state,
            Some(($code, $src_path, $span)),
        )
    };

    // Task errors
    (task: $id:expr, $msg:expr) => {
        $crate::RuntimeError::task_error($id, $msg)
    };
    (task: $id:expr, $msg:expr, history $history:expr) => {
        $crate::RuntimeError::task_error_with_history(
            $id,
            $msg,
            $history,
            None::<(&str, &str, miette::SourceSpan)>,
        )
    };
    (task: $id:expr, $msg:expr, history $history:expr, at $code:expr, $src_path:expr, $span:expr) => {
        $crate::RuntimeError::task_error_with_history(
            $id,
            $msg,
            $history,
            Some(($code, $src_path, $span)),
        )
    };

    // Network errors
    (network: $source:expr, $url:expr, $op:expr) => {
        $crate::RuntimeError::network_error($source, $url, $op)
    };
    (network: $source:expr, $url:expr, $op:expr, status $status:expr, headers $headers:expr) => {
        $crate::RuntimeError::network_error_with_context(
            $source,
            $url,
            $op,
            $status,
            $headers,
            None::<(&str, &str, miette::SourceSpan)>,
        )
    };
    (network: $source:expr, $url:expr, $op:expr, status $status:expr, headers $headers:expr, at $code:expr, $src_path:expr, $span:expr) => {
        $crate::RuntimeError::network_error_with_context(
            $source,
            $url,
            $op,
            $status,
            $headers,
            Some(($code, $src_path, $span)),
        )
    };

    // Encoding errors
    (encoding: $format:expr, $msg:expr) => {
        $crate::RuntimeError::encoding_error($format, $msg)
    };
    (encoding: $format:expr, $msg:expr, expected $expected:expr, actual $actual:expr) => {
        $crate::RuntimeError::encoding_error_with_context(
            $format,
            $msg,
            $expected,
            $actual,
            None::<(&str, &str, miette::SourceSpan)>,
        )
    };
    (encoding: $format:expr, $msg:expr, expected $expected:expr, actual $actual:expr, at $code:expr, $src_path:expr, $span:expr) => {
        $crate::RuntimeError::encoding_error_with_context(
            $format,
            $msg,
            $expected,
            $actual,
            Some(($code, $src_path, $span)),
        )
    };

    // Configuration errors
    (config: $field:expr, $msg:expr) => {
        $crate::RuntimeError::config_error($field, $msg)
    };
    (config: $field:expr, $msg:expr, schema $schema:expr, fixes $fixes:expr) => {
        $crate::RuntimeError::config_error_with_suggestions(
            $field,
            $msg,
            None::<(&str, &str, miette::SourceSpan)>,
            $schema,
            $fixes,
        )
    };
    (config: $field:expr, $msg:expr, schema $schema:expr, fixes $fixes:expr, at $config:expr, $cfg_path:expr, $span:expr) => {
        $crate::RuntimeError::config_error_with_suggestions(
            $field,
            $msg,
            Some(($config, $cfg_path, $span)),
            $schema,
            $fixes,
        )
    };

    // Type errors
    (type_error: $msg:expr, expected $expected:expr, actual $actual:expr) => {
        $crate::RuntimeError::type_error($msg, $expected, $actual)
    };
    (type_error: $msg:expr, expected $expected:expr, actual $actual:expr, suggestions $suggestions:expr) => {
        $crate::RuntimeError::type_error_with_suggestions(
            $msg,
            $expected,
            $actual,
            None::<(&str, &str, miette::SourceSpan)>,
            vec![],
            $suggestions,
        )
    };
    (type_error: $msg:expr, expected $expected:expr, actual $actual:expr, suggestions $suggestions:expr, at $code:expr, $src_path:expr, $span:expr) => {
        $crate::RuntimeError::type_error_with_suggestions(
            $msg,
            $expected,
            $actual,
            Some(($code, $src_path, $span)),
            vec![],
            $suggestions,
        )
    };

    // Memory errors
    (memory: $msg:expr, $op:expr) => {
        $crate::RuntimeError::memory_error($msg, $op)
    };
    (memory: $msg:expr, $op:expr, stats $stats:expr, heap $heap:expr) => {
        $crate::RuntimeError::memory_error_with_stats(
            $msg,
            $op,
            $stats,
            $heap,
            None::<(&str, &str, miette::SourceSpan)>,
        )
    };
    (memory: $msg:expr, $op:expr, stats $stats:expr, heap $heap:expr, at $code:expr, $src_path:expr, $span:expr) => {
        $crate::RuntimeError::memory_error_with_stats(
            $msg,
            $op,
            $stats,
            $heap,
            Some(($code, $src_path, $span)),
        )
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
        $crate::RuntimeError::module_resolution_error_with_context(
            $msg,
            $spec,
            Some($ref.to_string()),
        )
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

    // Window errors
    (window: $op:expr, $msg:expr) => {
        $crate::RuntimeError::window_error($op, $msg)
    };

    // Command / subprocess errors
    (command: $program:expr, $op:expr, $msg:expr) => {
        $crate::RuntimeError::command_error($program, $op, $msg)
    };
    (command: $program:expr, $op:expr, $msg:expr, exit $code:expr) => {
        $crate::RuntimeError::command_error_with_exit_code($program, $op, $msg, $code)
    };

    // Process errors
    (process: $op:expr, $msg:expr) => {
        $crate::RuntimeError::process_error($op, $msg)
    };
    (process: $op:expr, $msg:expr, signal $sig:expr) => {
        $crate::RuntimeError::process_error_with_signal($op, $msg, $sig)
    };

    // TLS errors
    (tls: $op:expr, $msg:expr) => {
        $crate::RuntimeError::tls_error($op, $msg)
    };
    (tls: $op:expr, $msg:expr, peer $peer:expr) => {
        $crate::RuntimeError::tls_error_with_peer($op, $msg, $peer)
    };

    // Lock errors
    (lock: $name:expr, $op:expr, $msg:expr) => {
        $crate::RuntimeError::lock_error($name, $op, $msg)
    };
    (deadlock: $name:expr, $op:expr, $msg:expr) => {
        $crate::RuntimeError::lock_deadlock_error($name, $op, $msg)
    };

    // Worker errors
    (worker: $id:expr, $op:expr, $msg:expr) => {
        $crate::RuntimeError::worker_error($id, $op, $msg)
    };

    // Database errors
    (database: $op:expr, $msg:expr) => {
        $crate::RuntimeError::database_error($op, $msg)
    };
    (database: $op:expr, $msg:expr, db $db:expr, sql $sql:expr) => {
        $crate::RuntimeError::database_error_with_context($op, $msg, $db, $sql)
    };

    // Crypto errors
    (crypto: $op:expr, $alg:expr, $msg:expr) => {
        $crate::RuntimeError::crypto_error($op, $alg, $msg)
    };
    (crypto: $op:expr, $alg:expr, $msg:expr, usage $usage:expr) => {
        $crate::RuntimeError::crypto_error_with_key_usage($op, $alg, $msg, $usage)
    };

    // URL parse errors
    (url_parse: $url:expr, $reason:expr) => {
        $crate::RuntimeError::url_parse_error($url, $reason)
    };

    // Storage errors
    (storage: $store_type:expr, $op:expr, $msg:expr) => {
        $crate::RuntimeError::storage_error($store_type, $op, $msg)
    };
    (storage_quota: $store_type:expr, $op:expr, $msg:expr) => {
        $crate::RuntimeError::storage_quota_exceeded($store_type, $op, $msg)
    };

    // FFI errors
    (ffi: $op:expr, $msg:expr) => {
        $crate::RuntimeError::ffi_call_error($op, $msg)
    };
    (ffi: $op:expr, $msg:expr, library $lib:expr) => {
        $crate::RuntimeError::ffi_call_error_with_library($op, $lib, $msg)
    };

    // HTTP module load errors
    (http_module: $url:expr, $op:expr, $msg:expr) => {
        $crate::RuntimeError::http_module_load_error($url, $op, $msg)
    };
    (http_module: $url:expr, $op:expr, $msg:expr, status $status:expr) => {
        $crate::RuntimeError::http_module_load_error_with_status($url, $op, $status, $msg)
    };

    // Import map errors
    (import_map: $field:expr, $msg:expr) => {
        $crate::RuntimeError::import_map_error($field, $msg)
    };
    (import_map: $field:expr, $msg:expr, value $value:expr) => {
        $crate::RuntimeError::import_map_error_with_value($field, $value, $msg)
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

#[cfg(test)]
mod new_variant_tests {
    use super::*;
    use miette::Diagnostic;

    #[test]
    fn command_error_carries_program_and_operation() {
        let err = RuntimeError::command_error("foo", "spawn", "no such file");
        let s = err.to_string();
        assert!(s.contains("foo"), "display: {s}");
        assert!(s.contains("spawn"), "display: {s}");
        assert_eq!(
            err.code().map(|c| c.to_string()),
            Some("andromeda::command::execution_failed".to_string())
        );
    }

    #[test]
    fn command_error_with_exit_code_shows_code() {
        let err =
            RuntimeError::command_error_with_exit_code("foo", "wait", "non-zero exit", 137);
        let s = err.to_string();
        assert!(s.contains("137"), "display: {s}");
    }

    #[test]
    fn process_error_signal_renders_signal() {
        let err = RuntimeError::process_error_with_signal(
            "register_signal_handler",
            "Unsupported signal",
            "SIGUSR3",
        );
        let s = err.to_string();
        assert!(s.contains("SIGUSR3"), "display: {s}");
        assert_eq!(
            err.code().map(|c| c.to_string()),
            Some("andromeda::process::operation_failed".to_string())
        );
    }

    #[test]
    fn tls_error_basic_code() {
        let err = RuntimeError::tls_error("handshake", "peer cert expired");
        assert_eq!(
            err.code().map(|c| c.to_string()),
            Some("andromeda::tls::operation_failed".to_string())
        );
    }

    #[test]
    fn lock_deadlock_distinguishes_in_display() {
        let normal = RuntimeError::lock_error("my-lock", "request", "already held").to_string();
        let dead = RuntimeError::lock_deadlock_error("my-lock", "request", "cycle").to_string();
        assert!(dead.to_lowercase().contains("deadlock"), "display: {dead}");
        assert!(!normal.to_lowercase().contains("deadlock"), "display: {normal}");
    }

    #[test]
    fn worker_error_optional_id_renders() {
        let with_id = RuntimeError::worker_error(Some(7), "spawn", "thread limit").to_string();
        let no_id = RuntimeError::worker_error(None, "spawn", "thread limit").to_string();
        assert!(with_id.contains("7"), "display: {with_id}");
        assert!(!no_id.contains("Worker 7"), "display: {no_id}");
    }

    #[test]
    fn database_error_code() {
        let err = RuntimeError::database_error("query", "syntax error near SELECT");
        assert_eq!(
            err.code().map(|c| c.to_string()),
            Some("andromeda::database::operation_failed".to_string())
        );
    }

    #[test]
    fn crypto_error_includes_algorithm() {
        let err = RuntimeError::crypto_error("encrypt", "AES-GCM", "invalid key length");
        let s = err.to_string();
        assert!(s.contains("AES-GCM"), "display: {s}");
    }

    #[test]
    fn url_parse_error_displays_url_and_reason() {
        let err = RuntimeError::url_parse_error("not://a url", "invalid scheme");
        let s = err.to_string();
        assert!(s.contains("not://a url"), "display: {s}");
        assert!(s.contains("invalid scheme"), "display: {s}");
    }

    #[test]
    fn storage_quota_exceeded_flagged() {
        let err = RuntimeError::storage_quota_exceeded("localStorage", "set", "over limit");
        let s = err.to_string();
        assert!(s.to_lowercase().contains("quota"), "display: {s}");
    }

    #[test]
    fn ffi_call_error_with_library_renders() {
        let err = RuntimeError::ffi_call_error_with_library("symbol_lookup", "libfoo.so", "missing");
        let s = err.to_string();
        assert!(s.contains("libfoo.so"), "display: {s}");
    }

    #[test]
    fn http_module_load_error_with_status() {
        let err = RuntimeError::http_module_load_error_with_status(
            "https://x/m.js",
            "fetch",
            404,
            "Not Found",
        );
        let s = err.to_string();
        assert!(s.contains("404"), "display: {s}");
        assert!(s.contains("https://x/m.js"), "display: {s}");
    }

    #[test]
    fn import_map_error_with_value_renders_value() {
        let err = RuntimeError::import_map_error_with_value(
            "imports",
            "../bad",
            "value must be resolvable",
        );
        let s = err.to_string();
        assert!(s.contains("../bad"), "display: {s}");
        assert!(s.contains("imports"), "display: {s}");
    }

    #[test]
    fn cli_conversion_covers_all_new_variants() {
        // Exhaustive-match safety net: building each variant should not panic
        // when consumed by code that does match analysis.
        let _ = RuntimeError::command_error("a", "b", "c");
        let _ = RuntimeError::process_error("a", "b");
        let _ = RuntimeError::tls_error("a", "b");
        let _ = RuntimeError::lock_error("a", "b", "c");
        let _ = RuntimeError::worker_error(None, "a", "b");
        let _ = RuntimeError::database_error("a", "b");
        let _ = RuntimeError::crypto_error("a", "b", "c");
        let _ = RuntimeError::url_parse_error("a", "b");
        let _ = RuntimeError::storage_error("a", "b", "c");
        let _ = RuntimeError::ffi_call_error("a", "b");
        let _ = RuntimeError::http_module_load_error("a", "b", "c");
        let _ = RuntimeError::import_map_error("a", "b");
    }
}
