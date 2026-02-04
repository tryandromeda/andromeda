// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

#![allow(clippy::result_large_err)]

use andromeda_core::RuntimeError;
use miette::{Diagnostic, NamedSource, SourceSpan};
use oxc_diagnostics::OxcDiagnostic;
use std::path::PathBuf;
use thiserror::Error;

/// Comprehensive error types for the Andromeda CLI.
///
/// These errors are designed for user-facing CLI interactions and provide
/// helpful messages and diagnostics for common CLI operations.
#[derive(Error, Diagnostic, Debug)]
#[allow(clippy::result_large_err)]
pub enum CliError {
    #[error("📁 File not found: {}", path.display())]
    #[diagnostic(
        code(andromeda::cli::file_not_found),
        help("💡 Make sure the file exists and you have permission to read it"),
        url("https://doc.rust-lang.org/std/fs/fn.read_to_string.html")
    )]
    FileNotFound {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("📄 File reading error: {}", path.display())]
    #[diagnostic(
        code(andromeda::cli::file_read_error),
        help("💡 Check file permissions and ensure the file is not corrupted")
    )]
    FileReadError {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("🚨 JavaScript/TypeScript parsing failed in {file_path}")]
    #[diagnostic(
        code(andromeda::cli::parse_error),
        help("💡 Syntax errors detected. Check for missing semicolons, brackets, or quotes."),
        url("https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Grammar_and_types")
    )]
    ParseError {
        diagnostics: Vec<OxcDiagnostic>,
        file_path: String,
        #[source_code]
        source_code: NamedSource<String>,
        error_spans: Vec<miette::SourceSpan>,
    },

    #[error("⚡ JavaScript runtime error: {message}")]
    #[diagnostic(
        code(andromeda::cli::runtime_error),
        help("💡 Check the stack trace for the exact error location and cause"),
        url("https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Errors")
    )]
    RuntimeError {
        message: String,
        file_path: Option<String>,
        line: Option<u32>,
        column: Option<u32>,
        #[source_code]
        source_code: Option<NamedSource<String>>,
        #[label("⚠️ Error occurred here")]
        error_span: Option<SourceSpan>,
    },

    #[error("Compilation failed: {reason}")]
    #[diagnostic(
        code(andromeda::cli::compile_error),
        help("Try checking if you have write permissions for the output directory")
    )]
    CompileError {
        reason: String,
        input_path: PathBuf,
        output_path: PathBuf,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("REPL error: {message}")]
    #[diagnostic(
        code(andromeda::cli::repl_error),
        help("Try typing 'help' for available commands")
    )]
    ReplError {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("Unsupported platform: {platform}")]
    #[diagnostic(
        code(andromeda::cli::unsupported_platform),
        help("Andromeda currently supports Windows, macOS, and Linux")
    )]
    UnsupportedPlatform { platform: String },

    #[error("Permission denied: {operation}")]
    #[diagnostic(
        code(andromeda::cli::permission_denied),
        help("Try running with appropriate permissions or check file ownership")
    )]
    PermissionDenied {
        operation: String,
        path: Option<PathBuf>,
        #[source]
        source: std::io::Error,
    },

    #[error("Invalid argument: {argument}")]
    #[diagnostic(
        code(andromeda::cli::invalid_argument),
        help("Use --help to see valid arguments and their formats")
    )]
    InvalidArgument {
        argument: String,
        expected: String,
        received: String,
    },

    #[error("Configuration error: {message}")]
    #[diagnostic(
        code(andromeda::cli::config_error),
        help("Check your configuration file format and values")
    )]
    ConfigError {
        message: String,
        config_path: Option<PathBuf>,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("Formatting error: {message}")]
    #[diagnostic(
        code(andromeda::cli::format_error),
        help(
            "Check that the file is valid JavaScript/TypeScript and that dprint is configured correctly"
        )
    )]
    FormatError {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("Task error: {message}")]
    #[diagnostic(
        code(andromeda::cli::task_error),
        help("Check your task configuration in andromeda.toml")
    )]
    TaskError {
        message: String,
        task_name: Option<String>,
    },
}

impl CliError {
    /// Create a file not found error with context
    pub fn file_not_found(path: PathBuf, source: std::io::Error) -> Self {
        Self::FileNotFound { path, source }
    }

    /// Create a file read error with context
    pub fn file_read_error(path: PathBuf, source: std::io::Error) -> Self {
        Self::FileReadError { path, source }
    }

    /// Create a parse error from OXC diagnostics with enhanced styling
    pub fn parse_error(
        diagnostics: Vec<OxcDiagnostic>,
        file_path: String,
        source_content: String,
    ) -> Self {
        let source_code = NamedSource::new(file_path.clone(), source_content);
        let error_spans: Vec<miette::SourceSpan> = diagnostics
            .iter()
            .filter_map(|diagnostic| {
                diagnostic.labels.as_ref().and_then(|labels| {
                    labels
                        .first()
                        .map(|label| miette::SourceSpan::new(label.offset().into(), label.len()))
                })
            })
            .collect();

        Self::ParseError {
            diagnostics,
            file_path,
            source_code,
            error_spans,
        }
    }

    /// Create a runtime error with optional source location
    pub fn runtime_error(
        message: String,
        file_path: Option<String>,
        line: Option<u32>,
        column: Option<u32>,
        source_content: Option<String>,
    ) -> Self {
        let (source_code, error_span) = if let (Some(path), Some(content), Some(line), Some(col)) =
            (&file_path, source_content, line, column)
        {
            let source = NamedSource::new(path.clone(), content);
            let span_start = (line.saturating_sub(1) as usize) * 50 + (col as usize);
            let span = SourceSpan::new(span_start.into(), 1);
            (Some(source), Some(span))
        } else {
            (None, None)
        };

        Self::RuntimeError {
            message,
            file_path,
            line,
            column,
            source_code,
            error_span,
        }
    }

    /// Create a simple runtime error with just a message
    pub fn runtime_error_simple(message: impl Into<String>) -> Self {
        Self::RuntimeError {
            message: message.into(),
            file_path: None,
            line: None,
            column: None,
            source_code: None,
            error_span: None,
        }
    }

    /// Create a compilation error
    pub fn compile_error(
        reason: String,
        input_path: PathBuf,
        output_path: PathBuf,
        source: Option<impl std::error::Error + Send + Sync + 'static>,
    ) -> Self {
        Self::CompileError {
            reason,
            input_path,
            output_path,
            source: source.map(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>),
        }
    }

    /// Create a REPL error
    pub fn repl_error(
        message: String,
        source: Option<impl std::error::Error + Send + Sync + 'static>,
    ) -> Self {
        Self::ReplError {
            message,
            source: source.map(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>),
        }
    }

    /// Create an unsupported platform error
    pub fn unsupported_platform(platform: String) -> Self {
        Self::UnsupportedPlatform { platform }
    }

    /// Create a permission denied error
    pub fn permission_denied(
        operation: String,
        path: Option<PathBuf>,
        source: std::io::Error,
    ) -> Self {
        Self::PermissionDenied {
            operation,
            path,
            source,
        }
    }

    /// Create an invalid argument error
    pub fn invalid_argument(argument: String, expected: String, received: String) -> Self {
        Self::InvalidArgument {
            argument,
            expected,
            received,
        }
    }

    /// Create a configuration error
    pub fn config_error(
        message: String,
        config_path: Option<PathBuf>,
        source: Option<impl std::error::Error + Send + Sync + 'static>,
    ) -> Self {
        Self::ConfigError {
            message,
            config_path,
            source: source.map(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>),
        }
    }

    /// Create a format error
    pub fn format_error(
        message: String,
        source: Option<impl std::error::Error + Send + Sync + 'static>,
    ) -> Self {
        Self::FormatError {
            message,
            source: source.map(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>),
        }
    }

    /// Create a task error
    pub fn task_error(message: String, task_name: Option<String>) -> Self {
        Self::TaskError { message, task_name }
    }
}

impl From<Box<RuntimeError>> for CliError {
    fn from(err: Box<RuntimeError>) -> Self {
        (*err).into()
    }
}

impl From<RuntimeError> for CliError {
    fn from(err: RuntimeError) -> Self {
        match err {
            RuntimeError::FsError {
                path,
                error_message,
                ..
            } => CliError::FileReadError {
                path: PathBuf::from(path),
                source: std::io::Error::other(error_message),
            },
            RuntimeError::ParseError {
                errors,
                source_path,
                ..
            } => CliError::ParseError {
                diagnostics: errors,
                file_path: source_path.clone(),
                source_code: NamedSource::new(source_path, String::new()),
                error_spans: vec![],
            },
            RuntimeError::RuntimeError { message, .. } => CliError::RuntimeError {
                message,
                file_path: None,
                line: None,
                column: None,
                source_code: None,
                error_span: None,
            },
            RuntimeError::ExtensionError {
                extension_name,
                message,
                ..
            } => CliError::RuntimeError {
                message: format!("Extension '{}' error: {}", extension_name, message),
                file_path: None,
                line: None,
                column: None,
                source_code: None,
                error_span: None,
            },
            RuntimeError::ResourceError {
                rid,
                operation,
                message,
                ..
            } => CliError::RuntimeError {
                message: format!("Resource {} error during {}: {}", rid, operation, message),
                file_path: None,
                line: None,
                column: None,
                source_code: None,
                error_span: None,
            },
            RuntimeError::TaskError {
                task_id, message, ..
            } => CliError::TaskError {
                message: format!("Task {} error: {}", task_id, message),
                task_name: Some(format!("task_{}", task_id)),
            },
            RuntimeError::NetworkError {
                url, error_message, ..
            } => CliError::RuntimeError {
                message: format!("Network error for {}: {}", url, error_message),
                file_path: None,
                line: None,
                column: None,
                source_code: None,
                error_span: None,
            },
            RuntimeError::EncodingError {
                format, message, ..
            } => CliError::RuntimeError {
                message: format!("Encoding error ({}): {}", format, message),
                file_path: None,
                line: None,
                column: None,
                source_code: None,
                error_span: None,
            },
            RuntimeError::ConfigError { field, message, .. } => CliError::ConfigError {
                message: format!("Field '{}': {}", field, message),
                config_path: None,
                source: None,
            },
            RuntimeError::TypeError {
                message,
                expected_type,
                actual_type,
                ..
            } => CliError::RuntimeError {
                message: format!(
                    "Type error: {} (expected {}, got {})",
                    message, expected_type, actual_type
                ),
                file_path: None,
                line: None,
                column: None,
                source_code: None,
                error_span: None,
            },
            RuntimeError::MemoryError {
                message, operation, ..
            } => CliError::RuntimeError {
                message: format!("Memory error during {}: {}", operation, message),
                file_path: None,
                line: None,
                column: None,
                source_code: None,
                error_span: None,
            },
            RuntimeError::ModuleNotFound { specifier, .. } => CliError::RuntimeError {
                message: format!("Module not found: {}", specifier),
                file_path: None,
                line: None,
                column: None,
                source_code: None,
                error_span: None,
            },
            RuntimeError::ModuleParseError { path, message, .. } => CliError::RuntimeError {
                message: format!("Parse error in module {}: {}", path, message),
                file_path: Some(path),
                line: None,
                column: None,
                source_code: None,
                error_span: None,
            },
            RuntimeError::ModuleResolutionError { message, .. } => CliError::RuntimeError {
                message: format!("Module resolution error: {}", message),
                file_path: None,
                line: None,
                column: None,
                source_code: None,
                error_span: None,
            },
            RuntimeError::ModuleRuntimeError { path, message, .. } => CliError::RuntimeError {
                message: format!("Runtime error in module {}: {}", path, message),
                file_path: Some(path),
                line: None,
                column: None,
                source_code: None,
                error_span: None,
            },
            RuntimeError::CircularImport { cycle, .. } => CliError::RuntimeError {
                message: format!("Circular import detected: {}", cycle),
                file_path: None,
                line: None,
                column: None,
                source_code: None,
                error_span: None,
            },
            RuntimeError::ImportNotFound { import, module, .. } => CliError::RuntimeError {
                message: format!("Import '{}' not found in module '{}'", import, module),
                file_path: Some(module),
                line: None,
                column: None,
                source_code: None,
                error_span: None,
            },
            RuntimeError::AmbiguousExport { export, module, .. } => CliError::RuntimeError {
                message: format!("Ambiguous export '{}' in module '{}'", export, module),
                file_path: Some(module),
                line: None,
                column: None,
                source_code: None,
                error_span: None,
            },
            RuntimeError::ModuleAlreadyLoaded { specifier, .. } => CliError::RuntimeError {
                message: format!("Module already loaded: {}", specifier),
                file_path: None,
                line: None,
                column: None,
                source_code: None,
                error_span: None,
            },
            RuntimeError::InvalidModuleSpecifier { specifier, .. } => CliError::RuntimeError {
                message: format!("Invalid module specifier: {}", specifier),
                file_path: None,
                line: None,
                column: None,
                source_code: None,
                error_span: None,
            },
            RuntimeError::ModuleIoError { message, path, .. } => CliError::RuntimeError {
                message: format!("Module I/O error: {}", message),
                file_path: path,
                line: None,
                column: None,
                source_code: None,
                error_span: None,
            },
            RuntimeError::LlmNotInitialized => CliError::RuntimeError {
                message: "LLM provider not initialized".to_string(),
                file_path: None,
                line: None,
                column: None,
                source_code: None,
                error_span: None,
            },
            RuntimeError::LlmProviderError { message, .. } => CliError::RuntimeError {
                message: format!("LLM provider error: {}", message),
                file_path: None,
                line: None,
                column: None,
                source_code: None,
                error_span: None,
            },
            RuntimeError::LlmTimeout { timeout_ms, .. } => CliError::RuntimeError {
                message: format!("LLM request timed out after {}ms", timeout_ms),
                file_path: None,
                line: None,
                column: None,
                source_code: None,
                error_span: None,
            },
            RuntimeError::LlmDisabled => CliError::RuntimeError {
                message: "LLM suggestions are disabled".to_string(),
                file_path: None,
                line: None,
                column: None,
                source_code: None,
                error_span: None,
            },
            RuntimeError::LlmAuthenticationError { message, .. } => CliError::RuntimeError {
                message: format!("LLM authentication error: {}", message),
                file_path: None,
                line: None,
                column: None,
                source_code: None,
                error_span: None,
            },
            RuntimeError::LlmNetworkError { message, .. } => CliError::RuntimeError {
                message: format!("LLM network error: {}", message),
                file_path: None,
                line: None,
                column: None,
                source_code: None,
                error_span: None,
            },
            RuntimeError::InternalError { message, .. } => CliError::RuntimeError {
                message: format!("Internal error: {}", message),
                file_path: None,
                line: None,
                column: None,
                source_code: None,
                error_span: None,
            },
        }
    }
}

/// Result type alias for CLI operations
pub type CliResult<T> = std::result::Result<T, CliError>;

/// Extension trait for converting `RuntimeResult` to `CliResult`.
///
/// This allows seamless conversion of core runtime errors to CLI errors.
///
/// # Example
///
/// ```
/// use andromeda::error::{CliResult, IntoCliResult};
/// use andromeda_core::RuntimeResult;
///
/// fn run_something() -> RuntimeResult<()> {
///     Ok(())
/// }
///
/// fn cli_command() -> CliResult<()> {
///     run_something().into_cli_result()
/// }
/// ```
#[allow(dead_code)]
pub trait IntoCliResult<T> {
    fn into_cli_result(self) -> CliResult<T>;
}

impl<T> IntoCliResult<T> for andromeda_core::RuntimeResult<T> {
    fn into_cli_result(self) -> CliResult<T> {
        self.map_err(CliError::from)
    }
}

/// Initialize error reporting with miette.
///
/// This delegates to the shared error reporting module in `andromeda_core`.
/// It's safe to call this multiple times - only the first call has effect.
pub fn init_error_reporting() {
    andromeda_core::init_error_reporting_default();
}

/// Print a CLI error to stderr with miette formatting.
pub fn print_error(error: CliError) {
    eprintln!("{:?}", miette::Report::new(error));
}

/// Handle file reading with proper error context.
///
/// This function reads a file and converts I/O errors to appropriate
/// `CliError` variants based on the error kind.
#[allow(clippy::result_large_err)]
pub fn read_file_with_context(path: &std::path::Path) -> CliResult<String> {
    std::fs::read_to_string(path).map_err(|e| match e.kind() {
        std::io::ErrorKind::NotFound => CliError::file_not_found(path.to_path_buf(), e),
        std::io::ErrorKind::PermissionDenied => CliError::permission_denied(
            format!("reading file {}", path.display()),
            Some(path.to_path_buf()),
            e,
        ),
        _ => CliError::file_read_error(path.to_path_buf(), e),
    })
}
