// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at https://mozilla.org/MPL/2.0/.

#![allow(clippy::result_large_err)]

/// Andromeda Satellite - Compile
///
/// A minimal executable focused solely on compiling JavaScript/TypeScript files into executables.
/// Designed for container instances where only compilation capability is needed.
use andromeda::{CliError, CliResult};
use clap::Parser as ClapParser;
use std::path::PathBuf;

#[derive(Debug, ClapParser)]
#[command(name = "andromeda-compile")]
#[command(about = "Andromeda Satellite - Compile JavaScript/TypeScript to executable")]
#[command(version = env!("CARGO_PKG_VERSION"))]
struct Cli {
    /// The path of the file to compile
    #[arg(required = true)]
    path: PathBuf,

    /// The output binary location
    #[arg(required = true)]
    out: PathBuf,

    /// Enable verbose output in the compiled binary
    #[arg(short, long)]
    verbose: bool,

    /// Disable strict mode in the compiled binary
    #[arg(short = 's', long)]
    no_strict: bool,
}

fn main() -> CliResult<()> {
    andromeda::error::init_error_reporting();

    let cli = Cli::parse();

    andromeda::compile::compile(
        cli.out.as_path(),
        cli.path.as_path(),
        cli.verbose,
        cli.no_strict,
    )
    .map_err(|e| CliError::runtime_error_simple(format!("Compilation failed: {e}")))?;

    let mut config_info = Vec::new();
    if cli.verbose {
        config_info.push("verbose mode enabled");
    }
    if cli.no_strict {
        config_info.push("strict mode disabled");
    }
    let config_str = if !config_info.is_empty() {
        format!(" ({})", config_info.join(", "))
    } else {
        String::new()
    };

    println!(
        "✅ Successfully created the output binary at {:?}{}",
        cli.out, config_str
    );

    Ok(())
}
