// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use oxc_diagnostics::OxcDiagnostic;
#[cfg(feature = "llm")]
use std::sync::atomic::{AtomicBool, Ordering};

use crate::{ErrorReporter, RuntimeError};

/// Global flag to enable AI-powered error suggestions
#[cfg(feature = "llm")]
static AI_SUGGESTIONS_ENABLED: AtomicBool = AtomicBool::new(false);

/// Enable AI-powered error suggestions globally.
#[cfg(feature = "llm")]
pub fn enable_ai_suggestions() {
    AI_SUGGESTIONS_ENABLED.store(true, Ordering::SeqCst);
}

/// Disable AI-powered error suggestions globally.
#[cfg(feature = "llm")]
pub fn disable_ai_suggestions() {
    AI_SUGGESTIONS_ENABLED.store(false, Ordering::SeqCst);
}

/// Check if AI suggestions are enabled.
#[cfg(feature = "llm")]
pub fn is_ai_suggestions_enabled() -> bool {
    AI_SUGGESTIONS_ENABLED.load(Ordering::SeqCst)
}

/// Initialize enhanced error reporting system
pub fn init_error_system() {
    ErrorReporter::init();
}

/// Exit the program with enhanced parse errors using oxc-miette with beautiful colors.
pub fn exit_with_parse_errors(errors: Vec<OxcDiagnostic>, source_path: &str, source: &str) -> ! {
    assert!(!errors.is_empty());

    let parse_error = RuntimeError::parse_error(errors.clone(), source_path, source);
    ErrorReporter::print_error(&parse_error);

    // If AI suggestions are enabled, try to get and display a suggestion
    #[cfg(feature = "llm")]
    if is_ai_suggestions_enabled() {
        print_ai_suggestion_for_parse_error(&errors, source_path, source);
    }

    std::process::exit(1);
}

/// Print AI suggestion for parse errors
#[cfg(feature = "llm")]
fn print_ai_suggestion_for_parse_error(errors: &[OxcDiagnostic], source_path: &str, source: &str) {
    use crate::llm_suggestions::{ErrorContext, get_error_suggestion, is_llm_initialized};
    use owo_colors::OwoColorize;

    if !is_llm_initialized() {
        return;
    }

    // Build error message from diagnostics
    let error_message = errors
        .iter()
        .map(|e| e.to_string())
        .collect::<Vec<_>>()
        .join("; ");

    eprintln!();
    eprintln!("{}", "Fetching AI suggestion...".dimmed());

    let context = ErrorContext::new(&error_message)
        .with_source_code(source)
        .with_file_path(source_path)
        .with_error_type("SyntaxError");

    match get_error_suggestion(&context) {
        Some(suggestion) => {
            // Clear the "Fetching" message by moving cursor up
            eprint!("\x1b[1A\x1b[2K"); // Move up one line and clear it

            eprintln!();
            eprintln!(
                "{} {} {}",
                "💡".bright_yellow(),
                "AI Suggestion".bright_yellow().bold(),
                format!("(via {})", suggestion.provider_name).dimmed()
            );
            eprintln!("{}", "─".repeat(50).yellow());
            eprintln!("{}", suggestion.suggestion);
            eprintln!();
        }
        None => {
            // Clear the "Fetching" message
            eprint!("\x1b[1A\x1b[2K");
        }
    }
}

/// Exit the program with a formatted Andromeda error
pub fn exit_with_error(error: RuntimeError) -> ! {
    ErrorReporter::print_error(&error);

    #[cfg(feature = "llm")]
    if is_ai_suggestions_enabled() {
        print_ai_suggestion_for_runtime_error(&error);
    }

    std::process::exit(1);
}

/// Print AI suggestion for runtime errors
#[cfg(feature = "llm")]
fn print_ai_suggestion_for_runtime_error(error: &RuntimeError) {
    use crate::llm_suggestions::{ErrorContext, get_error_suggestion, is_llm_initialized};
    use owo_colors::OwoColorize;

    if !is_llm_initialized() {
        return;
    }

    let error_message = error.to_string();

    eprintln!();
    eprintln!("{}", "Fetching AI suggestion...".dimmed());

    let mut context = ErrorContext::new(&error_message);

    // Try to extract error type from the message
    if error_message.contains("ReferenceError") {
        context = context.with_error_type("ReferenceError");
    } else if error_message.contains("TypeError") {
        context = context.with_error_type("TypeError");
    } else if error_message.contains("SyntaxError") {
        context = context.with_error_type("SyntaxError");
    } else if error_message.contains("RangeError") {
        context = context.with_error_type("RangeError");
    }

    match get_error_suggestion(&context) {
        Some(suggestion) => {
            // Clear the "Fetching" message by moving cursor up
            eprint!("\x1b[1A\x1b[2K"); // Move up one line and clear it

            eprintln!();
            eprintln!(
                "{} {} {}",
                "💡".bright_yellow(),
                "AI Suggestion".bright_yellow().bold(),
                format!("(via {})", suggestion.provider_name).dimmed()
            );
            eprintln!("{}", "─".repeat(50).yellow());
            eprintln!("{}", suggestion.suggestion);
            eprintln!();
        }
        None => {
            // Clear the "Fetching" message
            eprint!("\x1b[1A\x1b[2K");
        }
    }
}

/// Exit the program with multiple errors
pub fn exit_with_errors(errors: Vec<RuntimeError>) -> ! {
    ErrorReporter::print_errors(&errors);
    std::process::exit(1);
}

/// Create a detailed parse error report without exiting, with syntax highlighting and enhanced context
pub fn create_parse_error_report(
    errors: Vec<OxcDiagnostic>,
    source_path: &str,
    source: &str,
) -> String {
    let parse_error = RuntimeError::parse_error(errors, source_path, source);
    ErrorReporter::create_detailed_report(&parse_error)
}

/// Create a comprehensive error report with full miette integration
pub fn create_comprehensive_error_report(error: &RuntimeError) -> String {
    ErrorReporter::create_detailed_report(error)
}

/// Print an error with enhanced formatting and context
pub fn print_enhanced_error(error: &RuntimeError) {
    ErrorReporter::print_error(error);
}

/// Format an error as a string with full miette reporting
pub fn format_enhanced_error(error: &RuntimeError) -> String {
    ErrorReporter::format_error(error)
}
