// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use miette::{MietteHandlerOpts, Report};
use owo_colors::OwoColorize;
use std::sync::Once;

use miette as oxc_miette;

static INIT: Once = Once::new();

/// Configuration for error reporting
#[derive(Debug, Clone)]
pub struct ErrorReportingConfig {
    /// Enable clickable terminal links for error codes/URLs
    pub terminal_links: bool,
    /// Use Unicode characters in error output
    pub unicode: bool,
    /// Enable colored output
    pub color: bool,
    /// Number of context lines to show around errors
    pub context_lines: usize,
    /// Tab width for source code display
    pub tab_width: usize,
    /// Force graphical error display
    pub force_graphical: bool,
    /// Maximum width for error output (None for terminal width)
    pub width: Option<usize>,
}

impl Default for ErrorReportingConfig {
    fn default() -> Self {
        Self {
            terminal_links: true,
            unicode: true,
            color: true,
            context_lines: 5,
            tab_width: 4,
            force_graphical: true,
            width: Some(120),
        }
    }
}

/// Initialize miette error reporting with the given configuration.
///
/// This is safe to call multiple times - only the first call has effect.
/// Subsequent calls are no-ops.
///
/// # Example
///
/// ```
/// use andromeda_core::{ErrorReportingConfig, init_error_reporting};
///
/// init_error_reporting(ErrorReportingConfig {
///     context_lines: 3,
///     ..Default::default()
/// });
/// ```
pub fn init_error_reporting(config: ErrorReportingConfig) {
    INIT.call_once(|| {
        let _ = oxc_miette::set_hook(Box::new(move |_| {
            let mut opts = MietteHandlerOpts::new()
                .terminal_links(config.terminal_links)
                .unicode(config.unicode)
                .color(config.color)
                .context_lines(config.context_lines)
                .tab_width(config.tab_width)
                .force_graphical(config.force_graphical);

            if let Some(width) = config.width {
                opts = opts.width(width);
            }

            Box::new(opts.build())
        }));
    });
}

/// Initialize error reporting with default configuration.
///
/// This is a convenience function equivalent to:
/// ```
/// init_error_reporting(ErrorReportingConfig::default());
/// ```
pub fn init_error_reporting_default() {
    init_error_reporting(ErrorReportingConfig::default());
}

/// Print a miette-compatible error with standard Andromeda formatting.
///
/// The error is printed to stderr with a header and visual separator.
///
/// # Example
///
/// ```
/// use andromeda_core::print_error;
///
/// let error = RuntimeError::runtime_error("Something went wrong");
/// print_error(error);
/// ```
pub fn print_error<E>(error: E)
where
    E: std::error::Error + miette::Diagnostic + Send + Sync + 'static,
{
    eprintln!();
    eprintln!(
        "{} {}",
        "🚨".red().bold(),
        "Andromeda Error".bright_red().bold()
    );
    eprintln!("{}", "─".repeat(50).red());
    eprintln!("{:?}", Report::new(error));
}

/// Print a cloneable miette-compatible error by reference.
///
/// This variant accepts a reference and clones the error internally,
/// which is useful when you need to keep the original error.
pub fn print_error_ref<E>(error: &E)
where
    E: std::error::Error + miette::Diagnostic + Send + Sync + Clone + 'static,
{
    print_error(error.clone());
}

/// Format a miette-compatible error to a string.
///
/// This is useful when you need to capture the error output for logging
/// or other purposes rather than printing directly to stderr.
///
/// # Example
///
/// ```
/// use andromeda_core::format_error;
///
/// let error = RuntimeError::runtime_error("Something went wrong");
/// let formatted = format_error(error);
/// log::error!("{}", formatted);
/// ```
pub fn format_error<E>(error: E) -> String
where
    E: std::error::Error + miette::Diagnostic + Send + Sync + 'static,
{
    format!("{:?}", Report::new(error))
}

/// Format a cloneable miette-compatible error by reference.
pub fn format_error_ref<E>(error: &E) -> String
where
    E: std::error::Error + miette::Diagnostic + Send + Sync + Clone + 'static,
{
    format_error(error.clone())
}

/// Print multiple errors with enhanced formatting.
///
/// When there are multiple errors, each is numbered and separated
/// for clarity.
///
/// # Example
///
/// ```
/// use andromeda_core::print_errors;
///
/// let errors = vec![
///     RuntimeError::runtime_error("First error"),
///     RuntimeError::runtime_error("Second error"),
/// ];
/// print_errors(&errors);
/// ```
pub fn print_errors<E>(errors: &[E])
where
    E: std::error::Error + miette::Diagnostic + Send + Sync + Clone + 'static,
{
    if errors.is_empty() {
        return;
    }

    eprintln!();
    eprintln!(
        "{} {} ({} error{})",
        "🚨".red().bold(),
        "Andromeda Errors".bright_red().bold(),
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
        eprintln!("{:?}", Report::new(error.clone()));
    }
}
