// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use andromeda_core::RuntimeFile;
use clap::{Parser as ClapParser, Subcommand};
use libsui::find_section;
use std::path::PathBuf;

mod compile;
use compile::{ANDROMEDA_JS_CODE_SECTION, compile};
mod repl;
use repl::run_repl;
mod run;
mod styles;
use run::run;
mod error;
use error::{AndromedaError, Result, init_error_reporting, print_error};

/// A JavaScript runtime
#[derive(Debug, ClapParser)]
#[command(name = "andromeda")]
#[command(
    about = "The coolest JavaScript Runtime",
    long_about = "JS/TS Runtime in rust powered by Nova with no transpilation needed.",
    version = env!("CARGO_PKG_VERSION"),
    author = "Andromeda Team",
)]
#[clap(styles = styles::ANDROMEDA_STYLING)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Runs a file or files
    Run {
        #[arg(short, long)]
        verbose: bool,

        #[arg(short, long)]
        no_strict: bool,

        /// The files to run
        #[arg(required = true)]
        paths: Vec<String>,
    },

    /// Compiles a js file into a single-file executable
    Compile {
        // The path of the file to compile the executable for
        #[arg(required = true)]
        path: PathBuf,

        // The output binary location
        #[arg(required = true)]
        out: PathBuf,
    },

    /// Start an interactive REPL (Read-Eval-Print Loop)
    Repl {
        /// Expose Nova internal APIs for debugging
        #[arg(long)]
        expose_internals: bool,

        /// Print internal debugging information
        #[arg(long)]
        print_internals: bool,

        /// Disable garbage collection
        #[arg(long)]
        disable_gc: bool,
    },
}

fn main() {
    // Initialize beautiful error reporting
    init_error_reporting(); // Run the main logic and handle errors
    if let Err(error) = run_main() {
        print_error(error);
        std::process::exit(1);
    }
}

#[allow(clippy::result_large_err)]
fn run_main() -> Result<()> {
    // Check if this is currently a single-file executable
    if let Ok(Some(js)) = find_section(ANDROMEDA_JS_CODE_SECTION) {
        // TODO: Store verbose and strict settings in a config section of the resultant binary
        return run(
            false,
            false,
            vec![RuntimeFile::Embedded {
                path: String::from("internal"),
                content: js,
            }],
        );
    }

    let args = Cli::parse();

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()
        .map_err(|e| {
            AndromedaError::config_error(
                "Failed to initialize async runtime".to_string(),
                None,
                Some(Box::new(e)),
            )
        })?;

    // Run Nova in a secondary blocking thread so tokio tasks can still run
    let nova_thread = rt.spawn_blocking(move || -> Result<()> {
        match args.command {
            Command::Run {
                verbose,
                no_strict,
                paths,
            } => {
                let runtime_files: Vec<RuntimeFile> = paths
                    .into_iter()
                    .map(|path| RuntimeFile::Local { path })
                    .collect();

                run(verbose, no_strict, runtime_files)
            }
            Command::Compile { path, out } => {
                compile(out.as_path(), path.as_path()).map_err(|e| {
                    AndromedaError::compile_error(
                        format!("Compilation failed: {}", e),
                        path.clone(),
                        out.clone(),
                        Some(e),
                    )
                })?;

                println!("✅ Successfully created the output binary at {:?}", out);
                Ok(())
            }
            Command::Repl {
                expose_internals,
                print_internals,
                disable_gc,
            } => run_repl(expose_internals, print_internals, disable_gc)
                .map_err(|e| AndromedaError::repl_error(format!("REPL failed: {}", e), Some(e))),
        }
    });

    match rt.block_on(nova_thread) {
        Ok(result) => result,
        Err(e) => Err(AndromedaError::config_error(
            "Runtime execution failed".to_string(),
            None,
            Some(Box::new(e)),
        )),
    }
}
