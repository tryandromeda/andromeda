// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at https://mozilla.org/MPL/2.0/.

/// Andromeda Satellite - Lint
///
/// A minimal executable focused solely on linting JavaScript/TypeScript files.
/// Designed for container instances where only linting capability is needed.
use andromeda::{CliError, CliResult};
use clap::Parser as ClapParser;
use std::path::PathBuf;

#[derive(Debug, ClapParser)]
#[command(name = "andromeda-lint")]
#[command(about = "Andromeda Satellite - Lint JavaScript/TypeScript files")]
#[command(version = env!("CARGO_PKG_VERSION"))]
struct Cli {
    /// The file(s) or directory(ies) to lint
    #[arg(required = false)]
    paths: Vec<PathBuf>,
}

fn main() -> CliResult<()> {
    andromeda::error::init_error_reporting();

    let cli = Cli::parse();

    let config = andromeda::config::ConfigManager::load_or_default(None);

    let files_to_lint =
        andromeda::helper::find_formattable_files_for_lint(&cli.paths, &config.lint)
            .map_err(|e| CliError::TestExecution(format!("{e}")))?;

    if files_to_lint.is_empty() {
        println!("No lintable files found.");
        return Ok(());
    }

    println!("Found {} file(s) to lint:", files_to_lint.len());
    let mut had_issues = false;

    for path in &files_to_lint {
        if let Err(e) = andromeda::lint::lint_file_with_config(path, Some(config.clone())) {
            andromeda::error::print_error(e);
            had_issues = true;
        }
    }

    if had_issues {
        return Err(CliError::TestExecution(
            "Linting completed with errors".to_string(),
        ));
    }

    Ok(())
}
