// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use miette::{Diagnostic, MietteHandlerOpts, NamedSource, SourceSpan};
use oxc_diagnostics::OxcDiagnostic;
use std::path::PathBuf;
use thiserror::Error;

/// Comprehensive error types for the Andromeda CLI
#[derive(Error, Diagnostic, Debug)]
#[allow(clippy::result_large_err)]
pub enum AndromedaError {
    #[error("📁 File not found: {}", path.display())]
    #[diagnostic(
        code(andromeda::file_not_found),
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
        code(andromeda::file_read_error),
        help("💡 Check file permissions and ensure the file is not corrupted")
    )]
    FileReadError {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("🚨 JavaScript/TypeScript parsing failed in {file_path}")]
    #[diagnostic(
        code(andromeda::parse_error),
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
        code(andromeda::runtime_error),
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
        code(andromeda::compile_error),
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
        code(andromeda::repl_error),
        help("Try typing 'help' for available commands")
    )]
    ReplError {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("Unsupported platform: {platform}")]
    #[diagnostic(
        code(andromeda::unsupported_platform),
        help("Andromeda currently supports Windows, macOS, and Linux")
    )]
    UnsupportedPlatform { platform: String },

    #[error("Permission denied: {operation}")]
    #[diagnostic(
        code(andromeda::permission_denied),
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
        code(andromeda::invalid_argument),
        help("Use --help to see valid arguments and their formats")
    )]
    InvalidArgument {
        argument: String,
        expected: String,
        received: String,
    },

    #[error("Configuration error: {message}")]
    #[diagnostic(
        code(andromeda::config_error),
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
        code(andromeda::format_error),
        help(
            "Check that the file is valid JavaScript/TypeScript and that dprint is configured correctly"
        )
    )]
    FormatError {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
}

impl AndromedaError {
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
}

pub type Result<T> = std::result::Result<T, AndromedaError>;

pub fn init_error_reporting() {
    if miette::set_hook(Box::new(|_| {
        Box::new(
            MietteHandlerOpts::new()
                .terminal_links(true)
                .unicode(true)
                .color(true)
                .context_lines(5)
                .tab_width(4)
                .force_graphical(true)
                .build(),
        )
    }))
    .is_err()
    {}
}

pub fn print_error(error: AndromedaError) {
    eprintln!("{:?}", miette::Report::new(error));
}

/// Handle file reading with proper error context
#[allow(clippy::result_large_err)]
pub fn read_file_with_context(path: &std::path::Path) -> Result<String> {
    std::fs::read_to_string(path).map_err(|e| match e.kind() {
        std::io::ErrorKind::NotFound => AndromedaError::file_not_found(path.to_path_buf(), e),
        std::io::ErrorKind::PermissionDenied => AndromedaError::permission_denied(
            format!("reading file {}", path.display()),
            Some(path.to_path_buf()),
            e,
        ),
        _ => AndromedaError::file_read_error(path.to_path_buf(), e),
    })
}

/// Extract meaningful error information from Nova VM errors
pub fn extract_runtime_error_info(
    error_string: &str,
    _file_path: Option<String>,
) -> (String, Option<u32>, Option<u32>) {
    for part in error_string.split_whitespace() {
        if let Some(colon_pos) = part.rfind(':') {
            if let Some(second_colon) = part[..colon_pos].rfind(':') {
                let line_str = &part[second_colon + 1..colon_pos];
                let col_str = &part[colon_pos + 1..];

                if let (Ok(line), Ok(col)) = (line_str.parse::<u32>(), col_str.parse::<u32>()) {
                    return (error_string.to_string(), Some(line), Some(col));
                }
            }
        }
    }

    (error_string.to_string(), None, None)
}
