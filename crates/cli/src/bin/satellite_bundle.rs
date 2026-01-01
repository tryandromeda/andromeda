// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at https://mozilla.org/MPL/2.0/.

/// Andromeda Satellite - Bundle
///
/// A minimal executable focused solely on bundling and minifying JavaScript/TypeScript files.
/// Designed for container instances where only bundling capability is needed.
use andromeda::{CliError, CliResult};
use clap::Parser as ClapParser;
use std::path::PathBuf;

#[derive(Debug, ClapParser)]
#[command(name = "andromeda-bundle")]
#[command(about = "Andromeda Satellite - Bundle and minify JavaScript/TypeScript")]
#[command(version = env!("CARGO_PKG_VERSION"))]
struct Cli {
    /// The input file to bundle
    #[arg(required = true)]
    input: PathBuf,

    /// The output file to write the bundled code
    #[arg(required = true)]
    output: PathBuf,
}

fn main() -> CliResult<()> {
    andromeda::error::init_error_reporting();

    let cli = Cli::parse();

    andromeda::bundle::bundle(cli.input.to_str().unwrap(), cli.output.to_str().unwrap())
        .map_err(|e| CliError::TestExecution(format!("Bundle failed: {e}")))?;

    println!("✅ Successfully bundled and minified to {:?}", cli.output);

    Ok(())
}
