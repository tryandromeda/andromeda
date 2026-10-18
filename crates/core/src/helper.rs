// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use oxc_diagnostics::OxcDiagnostic;

use crate::{ErrorReporter, RuntimeError};

/// Initialize enhanced error reporting system
pub fn init_error_system() {
    ErrorReporter::init();
}

/// Exit the program with enhanced parse errors using oxc-miette with beautiful colors.
pub fn exit_with_parse_errors(errors: Vec<OxcDiagnostic>, source_path: &str, source: &str) -> ! {
    assert!(!errors.is_empty());

    let parse_error = RuntimeError::parse_error(errors, source_path, source);
    ErrorReporter::print_error(&parse_error);
    std::process::exit(1);
}

/// Exit the program with a formatted Andromeda error
pub fn exit_with_error(error: RuntimeError) -> ! {
    ErrorReporter::print_error(&error);
    std::process::exit(1);
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
