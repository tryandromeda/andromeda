// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use miette as oxc_miette;
use owo_colors::OwoColorize;
use oxc_diagnostics::OxcDiagnostic;
use oxc_miette::{Diagnostic, NamedSource, SourceSpan};
use std::fmt;

/// Comprehensive error type for Andromeda runtime operations
#[derive(Diagnostic, Debug, Clone)]
pub enum AndromedaError {
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
}

impl AndromedaError {
    /// Box this error for use with AndromedaResult
    pub fn boxed(self) -> Box<Self> {
        Box::new(self)
    }

    /// Create a boxed error directly
    pub fn into_result<T>(self) -> AndromedaResult<T> {
        Err(Box::new(self))
    }

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

    /// Create a new runtime error
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
}

impl fmt::Display for AndromedaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AndromedaError::FsError {
                operation, path, ..
            } => {
                write!(f, "File system error during {operation}: {path}")
            }
            AndromedaError::ParseError { source_path, .. } => {
                write!(f, "Parse error in {source_path}")
            }
            AndromedaError::RuntimeError { message, .. } => {
                write!(f, "Runtime error: {message}")
            }
            AndromedaError::ExtensionError {
                extension_name,
                message,
                ..
            } => {
                write!(f, "Extension '{extension_name}' error: {message}")
            }
            AndromedaError::ResourceError {
                rid,
                operation,
                message,
                ..
            } => {
                write!(f, "Resource {rid} error during {operation}: {message}")
            }
            AndromedaError::TaskError {
                task_id, message, ..
            } => {
                write!(f, "Task {task_id} error: {message}")
            }
            AndromedaError::NetworkError { url, operation, .. } => {
                write!(f, "Network error during {operation} for {url}")
            }
            AndromedaError::EncodingError {
                format, message, ..
            } => {
                write!(f, "Encoding error ({format}): {message}")
            }
            AndromedaError::ConfigError { field, message, .. } => {
                write!(f, "Configuration error in field '{field}': {message}")
            }
            AndromedaError::TypeError {
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
            AndromedaError::MemoryError {
                message, operation, ..
            } => {
                write!(f, "Memory error during {operation}: {message}")
            }
        }
    }
}

impl std::error::Error for AndromedaError {}

/// Result type alias for Andromeda operations with boxed errors to reduce stack size
pub type AndromedaResult<T> = Result<T, Box<AndromedaError>>;

/// Enhanced error reporting utilities with full miette integration
pub struct ErrorReporter;

impl ErrorReporter {
    /// Initialize miette with enhanced reporting capabilities
    pub fn init() {
        let _ = oxc_miette::set_hook(Box::new(|_| {
            Box::new(
                oxc_miette::MietteHandlerOpts::new()
                    .terminal_links(true)
                    .unicode(true)
                    .color(true)
                    .context_lines(5)
                    .tab_width(4)
                    .force_graphical(true)
                    .width(120)
                    .build(),
            )
        }));
    }

    /// Print a formatted error with full miette reporting
    pub fn print_error(error: &AndromedaError) {
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
    pub fn print_errors(errors: &[AndromedaError]) {
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
    pub fn format_error(error: &AndromedaError) -> String {
        format!("{:?}", oxc_miette::Report::new(error.clone()))
    }

    /// Create a comprehensive error report with suggestions and related information
    pub fn create_detailed_report(error: &AndromedaError) -> String {
        let mut report = String::new();

        // Header with emoji and color
        report.push_str(&format!("{} Andromeda Error Report\n", "🔍".bright_blue()));
        report.push_str(&format!("{}\n", "═".repeat(60).blue()));

        // Main error display
        report.push_str(&format!("{:?}\n", oxc_miette::Report::new(error.clone())));

        // Additional context based on error type
        match error {
            AndromedaError::ParseError { errors, .. } => {
                report.push_str(&format!("\n{} Parse Details:\n", "📝".bright_yellow()));
                report.push_str(&format!("Total errors found: {}\n", errors.len()));
                for (i, err) in errors.iter().enumerate() {
                    report.push_str(&format!("  {}. {}\n", i + 1, err));
                }
            }
            AndromedaError::RuntimeError {
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
            AndromedaError::NetworkError {
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
            AndromedaError::MemoryError {
                memory_stats: Some(stats),
                heap_info: Some(heap),
                ..
            } => {
                report.push_str(&format!("\n{} Memory Information:\n", "💾".bright_red()));
                report.push_str(&format!("Memory Stats: {stats}\n"));
                report.push_str(&format!("Heap Info: {heap}\n"));
            }
            _ => {}
        }

        report.push_str(&format!("\n{}\n", "═".repeat(60).blue()));
        report
    }
}

/// Trait for converting various error types to AndromedaError
pub trait IntoAndromedaError<T> {
    fn into_andromeda_error(self) -> AndromedaResult<T>;
}

impl<T> IntoAndromedaError<T> for Result<T, std::io::Error> {
    fn into_andromeda_error(self) -> AndromedaResult<T> {
        self.map_err(|e| Box::new(AndromedaError::fs_error(e, "unknown", "unknown")))
    }
}

/// Enhanced macros for quick error creation with location support
#[macro_export]
macro_rules! andromeda_error {
    // File system errors
    (fs: $op:expr, $path:expr, $source:expr) => {
        $crate::AndromedaError::fs_error($source, $op, $path)
    };
    (fs: $op:expr, $path:expr, $source:expr, at $code:expr, $src_path:expr, $span:expr) => {
        $crate::AndromedaError::fs_error_with_location($source, $op, $path, $code, $src_path, $span)
    };

    // Runtime errors
    (runtime: $msg:expr) => {
        $crate::AndromedaError::runtime_error($msg)
    };
    (runtime: $msg:expr, at $code:expr, $src_path:expr, $span:expr) => {
        $crate::AndromedaError::runtime_error_with_location($msg, $code, $src_path, $span)
    };
    (runtime: $msg:expr, with context $code:expr, $src_path:expr, $span:expr, stack $stack:expr, vars $vars:expr) => {
        $crate::AndromedaError::runtime_error_with_context(
            $msg, $code, $src_path, $span, $stack, $vars,
        )
    };

    // Extension errors
    (extension: $name:expr, $msg:expr) => {
        $crate::AndromedaError::extension_error($name, $msg)
    };
    (extension: $name:expr, $msg:expr, at $code:expr, $src_path:expr, $span:expr, missing $deps:expr) => {
        $crate::AndromedaError::extension_error_with_context(
            $name, $msg, $code, $src_path, $span, $deps,
        )
    };

    // Resource errors
    (resource: $rid:expr, $op:expr, $msg:expr) => {
        $crate::AndromedaError::resource_error($rid, $op, $msg)
    };
    (resource: $rid:expr, $op:expr, $msg:expr, state $state:expr) => {
        $crate::AndromedaError::resource_error_with_context($rid, $op, $msg, $state, None)
    };
    (resource: $rid:expr, $op:expr, $msg:expr, state $state:expr, at $code:expr, $src_path:expr, $span:expr) => {
        $crate::AndromedaError::resource_error_with_context(
            $rid,
            $op,
            $msg,
            $state,
            Some(($code, $src_path, $span)),
        )
    };

    // Task errors
    (task: $id:expr, $msg:expr) => {
        $crate::AndromedaError::task_error($id, $msg)
    };
    (task: $id:expr, $msg:expr, history $history:expr) => {
        $crate::AndromedaError::task_error_with_history($id, $msg, $history, None)
    };
    (task: $id:expr, $msg:expr, history $history:expr, at $code:expr, $src_path:expr, $span:expr) => {
        $crate::AndromedaError::task_error_with_history(
            $id,
            $msg,
            $history,
            Some(($code, $src_path, $span)),
        )
    };

    // Network errors
    (network: $source:expr, $url:expr, $op:expr) => {
        $crate::AndromedaError::network_error($source, $url, $op)
    };
    (network: $source:expr, $url:expr, $op:expr, status $status:expr, headers $headers:expr) => {
        $crate::AndromedaError::network_error_with_context(
            $source, $url, $op, $status, $headers, None,
        )
    };
    (network: $source:expr, $url:expr, $op:expr, status $status:expr, headers $headers:expr, at $code:expr, $src_path:expr, $span:expr) => {
        $crate::AndromedaError::network_error_with_context(
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
        $crate::AndromedaError::encoding_error($format, $msg)
    };
    (encoding: $format:expr, $msg:expr, expected $expected:expr, actual $actual:expr) => {
        $crate::AndromedaError::encoding_error_with_context($format, $msg, $expected, $actual, None)
    };
    (encoding: $format:expr, $msg:expr, expected $expected:expr, actual $actual:expr, at $code:expr, $src_path:expr, $span:expr) => {
        $crate::AndromedaError::encoding_error_with_context(
            $format,
            $msg,
            $expected,
            $actual,
            Some(($code, $src_path, $span)),
        )
    };

    // Configuration errors
    (config: $field:expr, $msg:expr) => {
        $crate::AndromedaError::config_error($field, $msg)
    };
    (config: $field:expr, $msg:expr, schema $schema:expr, fixes $fixes:expr) => {
        $crate::AndromedaError::config_error_with_suggestions($field, $msg, None, $schema, $fixes)
    };
    (config: $field:expr, $msg:expr, schema $schema:expr, fixes $fixes:expr, at $config:expr, $cfg_path:expr, $span:expr) => {
        $crate::AndromedaError::config_error_with_suggestions(
            $field,
            $msg,
            Some(($config, $cfg_path, $span)),
            $schema,
            $fixes,
        )
    };

    // Type errors
    (type_error: $msg:expr, expected $expected:expr, actual $actual:expr) => {
        $crate::AndromedaError::type_error($msg, $expected, $actual)
    };
    (type_error: $msg:expr, expected $expected:expr, actual $actual:expr, suggestions $suggestions:expr) => {
        $crate::AndromedaError::type_error_with_suggestions(
            $msg,
            $expected,
            $actual,
            None,
            vec![],
            $suggestions,
        )
    };
    (type_error: $msg:expr, expected $expected:expr, actual $actual:expr, suggestions $suggestions:expr, at $code:expr, $src_path:expr, $span:expr) => {
        $crate::AndromedaError::type_error_with_suggestions(
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
        $crate::AndromedaError::memory_error($msg, $op)
    };
    (memory: $msg:expr, $op:expr, stats $stats:expr, heap $heap:expr) => {
        $crate::AndromedaError::memory_error_with_stats($msg, $op, $stats, $heap, None)
    };
    (memory: $msg:expr, $op:expr, stats $stats:expr, heap $heap:expr, at $code:expr, $src_path:expr, $span:expr) => {
        $crate::AndromedaError::memory_error_with_stats(
            $msg,
            $op,
            $stats,
            $heap,
            Some(($code, $src_path, $span)),
        )
    };
}

/// Convenience macro for creating source spans
#[macro_export]
macro_rules! span {
    ($start:expr, $len:expr) => {
        oxc_miette::SourceSpan::new($start.into(), $len)
    };
    ($range:expr) => {
        oxc_miette::SourceSpan::new($range.start.into(), $range.end - $range.start)
    };
}

/// Convenience macro for creating named source with content
#[macro_export]
macro_rules! named_source {
    ($name:expr, $content:expr) => {
        oxc_miette::NamedSource::new($name, $content)
    };
}
