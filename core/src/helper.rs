// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use miette::NamedSource;
use owo_colors::OwoColorize;
use oxc_diagnostics::OxcDiagnostic;

/// Exit the program with enhanced parse errors using oxc-miette with beautiful colors.
pub fn exit_with_parse_errors(errors: Vec<OxcDiagnostic>, source_path: &str, source: &str) -> ! {
    assert!(!errors.is_empty());

    eprintln!();
    eprintln!(
        "{} Parse Error in {}: {}",
        "".red().bold(),
        source_path.bright_yellow(),
        "────────────────────────────────────────────────".red()
    );
    let source: &'static str = unsafe { std::mem::transmute(source) };

    let named_source = NamedSource::new(source_path, source);

    for (index, error) in errors.iter().enumerate() {
        if errors.len() > 1 {
            eprintln!(
                "\n{} Error {} of {}:",
                "".cyan(),
                (index + 1).to_string().bright_cyan(),
                errors.len().to_string().bright_cyan()
            );
        }

        let report = error.clone().with_source_code(named_source.clone());
        eprintln!("{}", report);
    }

    std::process::exit(1);
}

/// Create a detailed parse error report without exiting, with syntax highlighting
pub fn create_parse_error_report(
    errors: Vec<OxcDiagnostic>,
    source_path: &str,
    source: &str,
) -> String {
    let mut output = String::new();
    output.push_str(&format!(" Parse Error in {}:\n", source_path));
    output.push_str("────────────────────────────────────────────────\n");
    let source_owned = source.to_string();
    let source_path_owned = source_path.to_string();

    let named_source = NamedSource::new(source_path_owned, source_owned);

    for (index, error) in errors.iter().enumerate() {
        if errors.len() > 1 {
            output.push_str(&format!("\n Error {} of {}:\n", index + 1, errors.len()));
        }
        let report = error.clone().with_source_code(named_source.clone());
        output.push_str(&format!("{}\n", report));
    }

    output
}
