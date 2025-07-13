// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use andromeda_core::RuntimeFile;
use clap::{CommandFactory, Parser as ClapParser, Subcommand};
use clap_complete::{Shell, generate};
use libsui::find_section;
use std::io;
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
mod format;
use format::format_file;
mod helper;
use helper::find_formattable_files;
mod upgrade;

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
    /// Formats the specified files or directories
    Fmt {
        /// The file(s) or directory(ies) to format
        #[arg(required = false)]
        paths: Vec<PathBuf>,
    },
    /// Generate shell completion scripts
    Completions {
        /// The shell to generate completions for
        #[arg(value_enum)]
        shell: Option<Shell>,
    },
    /// Upgrade Andromeda to the latest version
    Upgrade {
        /// Force upgrade even if already on latest version
        #[arg(short, long)]
        force: bool,

        /// Upgrade to a specific version instead of latest
        #[arg(short, long)]
        version: Option<String>,

        /// Show what would be upgraded without actually upgrading
        #[arg(long)]
        dry_run: bool,
    },
}

fn main() {
    // Initialize beautiful error reporting
    init_error_reporting();
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
                        format!("Compilation failed: {e}"),
                        path.clone(),
                        out.clone(),
                        Some(e),
                    )
                })?;

                println!("✅ Successfully created the output binary at {out:?}");
                Ok(())
            }
            Command::Repl {
                expose_internals,
                print_internals,
                disable_gc,
            } => run_repl(expose_internals, print_internals, disable_gc)
                .map_err(|e| AndromedaError::repl_error(format!("REPL failed: {e}"), Some(e))),
            Command::Fmt { paths } => {
                let files_to_format = find_formattable_files(&paths)?;

                if files_to_format.is_empty() {
                    println!("No formattable files found.");
                    return Ok(());
                }

                println!("Found {} file(s) to format:", files_to_format.len());
                for path in &files_to_format {
                    format_file(path)?;
                }
                Ok(())
            }
            Command::Completions { shell } => {
                generate_completions(shell);
                Ok(())
            }
            Command::Upgrade {
                force,
                version,
                dry_run,
            } => upgrade::run_upgrade(force, version, dry_run).map_err(|e| {
                AndromedaError::runtime_error(
                    format!("Upgrade failed: {e}"),
                    None,
                    None,
                    None,
                    None,
                )
            }),
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

fn generate_completions(shell: Option<Shell>) {
    let mut cmd = Cli::command();
    let bin_name = "andromeda";

    match shell {
        Some(shell) => {
            generate(shell, &mut cmd, bin_name, &mut io::stdout());
        }
        None => {
            // If no shell is specified, try to detect it from environment
            // This mimics Deno's behavior
            if let Some(detected_shell) = detect_shell() {
                generate(detected_shell, &mut cmd, bin_name, &mut io::stdout());
            } else {
                eprintln!(
                    "Error: Could not detect shell. Please specify one of: bash, zsh, fish, powershell"
                );
                std::process::exit(1);
            }
        }
    }
}

fn detect_shell() -> Option<Shell> {
    // Try to detect shell from environment variables
    if let Ok(shell) = std::env::var("SHELL") {
        if shell.contains("bash") {
            return Some(Shell::Bash);
        } else if shell.contains("zsh") {
            return Some(Shell::Zsh);
        } else if shell.contains("fish") {
            return Some(Shell::Fish);
        }
    }

    // On Windows, check for PowerShell
    if cfg!(windows) && std::env::var("PSModulePath").is_ok() {
        return Some(Shell::PowerShell);
    }

    None
}
