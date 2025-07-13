// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use miette::NamedSource;
use oxc_diagnostics::OxcDiagnostic;

use crate::{AndromedaError, ErrorReporter};

/// Exit the program with enhanced parse errors using oxc-miette with beautiful colors.
pub fn exit_with_parse_errors(errors: Vec<OxcDiagnostic>, source_path: &str, source: &str) -> ! {
    assert!(!errors.is_empty());

    let parse_error = AndromedaError::parse_error(errors, source_path, source);
    ErrorReporter::print_error(&parse_error);
    std::process::exit(1);
}

/// Exit the program with a formatted Andromeda error
pub fn exit_with_error(error: AndromedaError) -> ! {
    ErrorReporter::print_error(&error);
    std::process::exit(1);
}

/// Exit the program with multiple errors
pub fn exit_with_errors(errors: Vec<AndromedaError>) -> ! {
    ErrorReporter::print_errors(&errors);
    std::process::exit(1);
}

/// Create a detailed parse error report without exiting, with syntax highlighting
pub fn create_parse_error_report(
    errors: Vec<OxcDiagnostic>,
    source_path: &str,
    source: &str,
) -> String {
    let mut output = String::new();
    output.push_str(&format!(" Parse Error in {source_path}:\n"));
    output.push_str("────────────────────────────────────────────────\n");
    let source_owned = source.to_string();
    let source_path_owned = source_path.to_string();

    let named_source = NamedSource::new(source_path_owned, source_owned);

    for (index, error) in errors.iter().enumerate() {
        if errors.len() > 1 {
            output.push_str(&format!("\n Error {} of {}:\n", index + 1, errors.len()));
        }
        let report = error.clone().with_source_code(named_source.clone());
        output.push_str(&format!("{report}\n"));
    }

    output
}
