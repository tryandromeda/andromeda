// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at https://mozilla.org/MPL/2.0/.

/// Andromeda Satellite - Check
///
/// A minimal executable focused solely on type-checking TypeScript files.
/// Designed for container instances where only type-checking capability is needed.
use andromeda::{CliError, CliResult};
use clap::Parser as ClapParser;
use std::path::PathBuf;

#[derive(Debug, ClapParser)]
#[command(name = "andromeda-check")]
#[command(about = "Andromeda Satellite - Type-check TypeScript files")]
#[command(version = env!("CARGO_PKG_VERSION"))]
struct Cli {
    /// The file(s) or directory(ies) to type-check
    #[arg(required = false)]
    paths: Vec<PathBuf>,
}

fn main() -> CliResult<()> {
    andromeda::error::init_error_reporting();

    let cli = Cli::parse();

    let config = andromeda::config::ConfigManager::load_or_default(None);

    andromeda::check::check_files_with_config(&cli.paths, Some(config))
        .map_err(|e| CliError::TestExecution(format!("{e}")))?;

    Ok(())
}
