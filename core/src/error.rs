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
        help("Check that the file exists and you have proper permissions")
    )]
    FsError {
        operation: String,
        path: String,
        error_message: String,
    },

    /// Parse errors from JavaScript/TypeScript source
    #[diagnostic(
        code(andromeda::parse::syntax_error),
        help("Check the syntax of your JavaScript/TypeScript code")
    )]
    ParseError {
        errors: Vec<OxcDiagnostic>,
        source_path: String,
        #[source_code]
        source_code: NamedSource<String>,
    },

    /// Runtime execution errors
    #[diagnostic(
        code(andromeda::runtime::execution_error),
        help("Check the runtime state and ensure all required resources are available")
    )]
    RuntimeError {
        message: String,
        #[label("error occurred here")]
        location: Option<SourceSpan>,
        #[source_code]
        source_code: Option<NamedSource<String>>,
    },

    /// Extension loading errors
    #[diagnostic(
        code(andromeda::extension::load_error),
        help("Ensure the extension is properly configured and dependencies are available")
    )]
    ExtensionError {
        extension_name: String,
        message: String,
    },

    /// Resource management errors
    #[diagnostic(
        code(andromeda::resource::invalid_rid),
        help("Ensure the resource ID is valid and the resource hasn't been closed")
    )]
    ResourceError {
        rid: u32,
        operation: String,
        message: String,
    },

    /// Task management errors
    #[diagnostic(
        code(andromeda::task::task_error),
        help("Check task state and ensure proper cleanup")
    )]
    TaskError { task_id: u32, message: String },
    /// Network/HTTP errors
    #[diagnostic(
        code(andromeda::network::request_error),
        help("Check network connectivity and request parameters")
    )]
    NetworkError {
        url: String,
        operation: String,
        error_message: String,
    },

    /// Encoding/Decoding errors
    #[diagnostic(
        code(andromeda::encoding::decode_error),
        help("Ensure the input data is properly formatted")
    )]
    EncodingError { format: String, message: String },

    /// Configuration errors
    #[diagnostic(
        code(andromeda::config::invalid_config),
        help("Check the configuration file format and required fields")
    )]
    ConfigError { field: String, message: String },
}

impl AndromedaError {
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
        Self::ParseError {
            errors,
            source_path: source_path.clone(),
            source_code: NamedSource::new(source_path, source_code),
        }
    }

    /// Create a new runtime error
    pub fn runtime_error(message: impl Into<String>) -> Self {
        Self::RuntimeError {
            message: message.into(),
            location: None,
            source_code: None,
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
        }
    }

    /// Create a new extension error
    pub fn extension_error(extension_name: impl Into<String>, message: impl Into<String>) -> Self {
        Self::ExtensionError {
            extension_name: extension_name.into(),
            message: message.into(),
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
        }
    }

    /// Create a new task error
    pub fn task_error(task_id: u32, message: impl Into<String>) -> Self {
        Self::TaskError {
            task_id,
            message: message.into(),
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
        }
    }

    /// Create a new encoding error
    pub fn encoding_error(format: impl Into<String>, message: impl Into<String>) -> Self {
        Self::EncodingError {
            format: format.into(),
            message: message.into(),
        }
    }

    /// Create a new configuration error
    pub fn config_error(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self::ConfigError {
            field: field.into(),
            message: message.into(),
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
            } => {
                write!(f, "Extension '{extension_name}' error: {message}")
            }
            AndromedaError::ResourceError {
                rid,
                operation,
                message,
            } => {
                write!(f, "Resource {rid} error during {operation}: {message}")
            }
            AndromedaError::TaskError { task_id, message } => {
                write!(f, "Task {task_id} error: {message}")
            }
            AndromedaError::NetworkError { url, operation, .. } => {
                write!(f, "Network error during {operation} for {url}")
            }
            AndromedaError::EncodingError { format, message } => {
                write!(f, "Encoding error ({format}): {message}")
            }
            AndromedaError::ConfigError { field, message } => {
                write!(f, "Configuration error in field '{field}': {message}")
            }
        }
    }
}

impl std::error::Error for AndromedaError {}

/// Result type alias for Andromeda operations
pub type AndromedaResult<T> = Result<T, AndromedaError>;

/// Enhanced error reporting utilities
pub struct ErrorReporter;

impl ErrorReporter {
    /// Print a formatted error with miette
    pub fn print_error(error: &AndromedaError) {
        eprintln!();
        eprintln!(
            "{} {}",
            "🚨".red().bold(),
            "Andromeda Runtime Error".bright_red().bold()
        );
        eprintln!("{}", "─".repeat(50).red());

        // Use Display instead of miette for now to avoid trait issues
        eprintln!("{error}");
    }

    /// Print multiple errors
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
            }
            eprintln!("{error}");
        }
    }

    /// Create a formatted error report as string
    pub fn format_error(error: &AndromedaError) -> String {
        error.to_string()
    }
}

/// Trait for converting various error types to AndromedaError
pub trait IntoAndromedaError<T> {
    fn into_andromeda_error(self) -> AndromedaResult<T>;
}

impl<T> IntoAndromedaError<T> for Result<T, std::io::Error> {
    fn into_andromeda_error(self) -> AndromedaResult<T> {
        self.map_err(|e| AndromedaError::fs_error(e, "unknown", "unknown"))
    }
}

/// Macro for quick error creation
#[macro_export]
macro_rules! andromeda_error {
    (fs: $op:expr, $path:expr, $source:expr) => {
        $crate::AndromedaError::fs_error($source, $op, $path)
    };
    (runtime: $msg:expr) => {
        $crate::AndromedaError::runtime_error($msg)
    };
    (extension: $name:expr, $msg:expr) => {
        $crate::AndromedaError::extension_error($name, $msg)
    };
    (resource: $rid:expr, $op:expr, $msg:expr) => {
        $crate::AndromedaError::resource_error($rid, $op, $msg)
    };
    (task: $id:expr, $msg:expr) => {
        $crate::AndromedaError::task_error($id, $msg)
    };
    (encoding: $format:expr, $msg:expr) => {
        $crate::AndromedaError::encoding_error($format, $msg)
    };
    (config: $field:expr, $msg:expr) => {
        $crate::AndromedaError::config_error($field, $msg)
    };
}
