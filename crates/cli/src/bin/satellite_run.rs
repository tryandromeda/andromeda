// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at https://mozilla.org/MPL/2.0/.

#![allow(clippy::result_large_err)]

/// Andromeda Satellite - Run
///
/// A minimal executable focused solely on running JavaScript/TypeScript files.
/// Designed for container instances where only execution capability is needed.
use andromeda::{CliError, CliResult};
use andromeda_core::RuntimeFile;
use clap::Parser as ClapParser;

#[derive(Debug, ClapParser)]
#[command(name = "andromeda-run")]
#[command(about = "Andromeda Satellite - Execute JavaScript/TypeScript files")]
#[command(version = env!("CARGO_PKG_VERSION"))]
struct Cli {
    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Disable strict mode
    #[arg(short = 's', long)]
    no_strict: bool,

    /// The files to run
    #[arg(required = true)]
    paths: Vec<String>,
}

fn main() -> CliResult<()> {
    andromeda::error::init_error_reporting();
    andromeda_core::ErrorReporter::init();

    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .expect("failed to install default rustls CryptoProvider");

    let cli = Cli::parse();

    let runtime_files: Vec<RuntimeFile> = cli
        .paths
        .into_iter()
        .map(|path| RuntimeFile::Local { path })
        .collect();

    andromeda::run::run(cli.verbose, cli.no_strict, runtime_files)
        .map_err(|e| CliError::runtime_error_simple(format!("{e}")))?;

    Ok(())
}
