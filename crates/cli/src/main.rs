// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use andromeda_core::RuntimeFile;
use clap::{CommandFactory, Parser as ClapParser, Subcommand};
use clap_complete::{Shell, generate};
use console::Style;
use libsui::find_section;
use std::io;
use std::path::PathBuf;

mod bundle;
use bundle::bundle;
mod compile;
use compile::{ANDROMEDA_CONFIG_SECTION, ANDROMEDA_JS_CODE_SECTION, EmbeddedConfig, compile};
mod repl;
use repl::run_repl_with_config;
mod run;
mod styles;
use run::run;
mod error;
use error::{CliResult, init_error_reporting, print_error};
mod format;
use format::{FormatResult, format_file};
mod helper;
use helper::{find_formattable_files_for_format, find_formattable_files_for_lint};
mod lint;
use lint::lint_file_with_config;
mod config;
mod lsp;
mod task;
mod upgrade;
use config::{AndromedaConfig, ConfigFormat, ConfigManager};
use lsp::run_lsp_server;
use task::run_task;

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
    /// Runs a single file
    Run {
        #[arg(short, long)]
        verbose: bool,

        #[arg(short, long)]
        no_strict: bool,

        /// Run an HTTP-serving script in parallel across N OS threads.
        /// When set, `Andromeda.serve(handler)` inside the script will
        /// spawn (N - 1) additional Worker instances and all instances
        /// bind the same port via SO_REUSEPORT. If passed without a
        /// value, defaults to the number of available CPUs.
        #[arg(long, value_name = "N", num_args = 0..=1, default_missing_value = "0")]
        parallel: Option<usize>,

        /// The file to run
        #[arg(required = true)]
        path: String,

        /// Additional arguments (ignored by CLI, passed to Andromeda runtime)
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },

    /// Compiles a js file into a single-file executable
    Compile {
        // The path of the file to compile the executable for
        #[arg(required = true)]
        path: PathBuf,

        // The output binary location
        #[arg(required = true)]
        out: PathBuf,

        /// Enable verbose output in the compiled binary
        #[arg(short, long)]
        verbose: bool,

        /// Disable strict mode in the compiled binary
        #[arg(short = 's', long)]
        no_strict: bool,
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
    /// Bundle and minify a JavaScript/TypeScript file
    Bundle {
        /// The input file to bundle
        #[arg(required = true)]
        input: PathBuf,

        /// The output file to write the bundled code
        #[arg(required = true)]
        output: PathBuf,
    },

    /// Lint JavaScript/TypeScript files
    #[command(visible_alias = "check")]
    Lint {
        /// The file(s) or directory(ies) to lint
        #[arg(required = false)]
        paths: Vec<PathBuf>,
    },

    /// Start Language Server Protocol (LSP) server
    Lsp,

    /// Run tasks defined in configuration
    Task {
        /// The task name to run. If not provided, lists all available tasks
        task_name: Option<String>,
    },

    /// Configuration file management
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
}

#[derive(Debug, Subcommand)]
enum ConfigAction {
    /// Initialize a new config file
    Init {
        /// Config file format
        #[arg(value_enum, default_value = "json")]
        format: ConfigFileFormat,

        /// Output path for config file
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Force overwrite existing config file
        #[arg(short, long)]
        force: bool,
    },

    /// Show current configuration
    Show {
        /// Show configuration from specific file
        #[arg(short, long)]
        file: Option<PathBuf>,
    },

    /// Validate configuration file
    Validate {
        /// Config file to validate
        #[arg(short, long)]
        file: Option<PathBuf>,
    },
}

#[derive(Debug, Clone, clap::ValueEnum)]
enum ConfigFileFormat {
    Json,
    Toml,
    Yaml,
}

#[hotpath::main(percentiles = [50, 95, 99], limit = 10)]
fn main() {
    // Initialize beautiful error reporting from CLI
    init_error_reporting();
    // Also initialize the enhanced error reporting from core
    andromeda_core::ErrorReporter::init();

    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .expect("failed to install default rustls CryptoProvider");

    if let Err(error) = run_main() {
        print_error(error);
        std::process::exit(1);
    }
}

#[allow(clippy::result_large_err)]
#[hotpath::measure]
fn run_main() -> CliResult<()> {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_time()
        .enable_io()
        .build()
        .map_err(|e| {
            error::CliError::config_error(
                "Failed to initialize async runtime".to_string(),
                None,
                Some(Box::new(e)),
            )
        })?;
    let _tokio_guard = rt.enter();

    // Check if this is currently a single-file executable
    if let Ok(Some(js)) = find_section(ANDROMEDA_JS_CODE_SECTION) {
        // Try to load embedded config, fall back to defaults if not found
        let (verbose, no_strict) = match find_section(ANDROMEDA_CONFIG_SECTION) {
            Ok(Some(config_bytes)) => {
                match serde_json::from_slice::<EmbeddedConfig>(config_bytes) {
                    Ok(config) => (config.verbose, config.no_strict),
                    Err(_) => {
                        // If config is corrupted or in old format, use defaults
                        (false, false)
                    }
                }
            }
            _ => {
                // No config section found (old binary format), use defaults
                (false, false)
            }
        };

        return run(
            verbose,
            no_strict,
            vec![RuntimeFile::Embedded {
                path: String::from("internal"),
                content: js,
            }],
        );
    }

    use std::env;
    let mut raw_args: Vec<String> = env::args().collect();

    if !raw_args.is_empty() {
        raw_args.remove(0);
    }

    // If no arguments, alias to `repl`
    let args = if raw_args.is_empty() {
        Cli {
            command: Command::Repl {
                expose_internals: false,
                print_internals: false,
                disable_gc: false,
            },
        }
    } else if !raw_args.is_empty() && raw_args[0].ends_with(".ts") {
        let path = raw_args[0].clone();
        let args = raw_args[1..].to_vec();
        Cli {
            command: Command::Run {
                verbose: false,
                no_strict: false,
                parallel: None,
                path,
                args,
            },
        }
    } else {
        Cli::parse()
    };

    let nova_result: CliResult<()> = (move || -> CliResult<()> {
        match args.command {
            Command::Run {
                verbose,
                no_strict,
                parallel,
                path,
                args: _,
            } => {
                if let Some(n) = parallel {
                    let resolved = if n == 0 {
                        std::thread::available_parallelism()
                            .map(|p| p.get())
                            .unwrap_or(1)
                    } else {
                        n
                    };
                    // Surface to JS via env. `Andromeda.serve(...)` reads
                    // this as a default for the `parallel` option.
                    // SAFETY: only set before the runtime starts; no
                    // other thread reads env at this point.
                    unsafe {
                        std::env::set_var("ANDROMEDA_JOBS", resolved.to_string());
                    }
                }
                let runtime_file = RuntimeFile::Local { path };
                run::run(verbose, no_strict, vec![runtime_file])
            }
            Command::Compile {
                path,
                out,
                verbose,
                no_strict,
            } => {
                compile(out.as_path(), path.as_path(), verbose, no_strict).map_err(|e| {
                    error::CliError::compile_error(
                        format!("Compilation failed: {e}"),
                        path.clone(),
                        out.clone(),
                        Some(e),
                    )
                })?;
                let mut config_info = Vec::new();
                if verbose {
                    config_info.push("verbose mode enabled");
                }
                if no_strict {
                    config_info.push("strict mode disabled");
                }
                let config_str = if !config_info.is_empty() {
                    format!(" ({})", config_info.join(", "))
                } else {
                    String::new()
                };
                println!("Created output binary at {out:?}{config_str}");
                Ok(())
            }
            Command::Repl {
                expose_internals,
                print_internals,
                disable_gc,
            } => {
                // Load configuration
                let config = ConfigManager::load_or_default(None);
                run_repl_with_config(expose_internals, print_internals, disable_gc, Some(config))
                    .map_err(|e| error::CliError::repl_error(format!("REPL failed: {e}"), Some(e)))
            }
            Command::Fmt { paths } => {
                let config = ConfigManager::load_or_default(None);

                let files_to_format = find_formattable_files_for_format(&paths, &config.format)?;
                if files_to_format.is_empty() {
                    let warning = Style::new().yellow().apply_to("Warning");
                    eprintln!("{warning} No matching files found.");
                    return Ok(());
                }

                let mut already_formatted_count = 0;
                let mut formatted_count = 0;

                for path in &files_to_format {
                    match format_file(path)? {
                        FormatResult::Changed => {
                            let label = Style::new().green().apply_to("Format");
                            eprintln!("{label} {}", path.display());
                            formatted_count += 1;
                        }
                        FormatResult::AlreadyFormatted => already_formatted_count += 1,
                    }
                }

                let total = formatted_count + already_formatted_count;
                if formatted_count > 0 {
                    let plural = if formatted_count == 1 { "" } else { "s" };
                    eprintln!();
                    eprintln!(
                        "Formatted {formatted_count} file{plural} ({already_formatted_count} unchanged of {total})."
                    );
                } else {
                    let plural = if total == 1 { "" } else { "s" };
                    eprintln!("Checked {total} file{plural}.");
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
            } => upgrade::run_upgrade(force, version, dry_run),
            Command::Bundle { input, output } => {
                bundle(input.to_str().unwrap(), output.to_str().unwrap())?;
                println!("Bundled and minified to {output:?}");
                Ok(())
            }
            Command::Lint { paths } => {
                let config = ConfigManager::load_or_default(None);

                let files_to_lint = find_formattable_files_for_lint(&paths, &config.lint)?;
                if files_to_lint.is_empty() {
                    let warning = console::Style::new().yellow().apply_to("Warning");
                    eprintln!("{warning} No matching files found.");
                    return Ok(());
                }

                let mut total_issues = 0usize;
                let mut had_read_errors = false;
                for path in &files_to_lint {
                    let label = console::Style::new().green().apply_to("Lint");
                    eprintln!("{label} {}", path.display());
                    match lint_file_with_config(path, Some(config.clone())) {
                        Ok(count) => total_issues += count,
                        Err(e) => {
                            print_error(e);
                            had_read_errors = true;
                        }
                    }
                }

                if total_issues > 0 {
                    eprintln!();
                    let plural = if total_issues == 1 { "" } else { "s" };
                    eprintln!("Found {total_issues} issue{plural}.");
                }

                if total_issues > 0 || had_read_errors {
                    std::process::exit(1);
                }
                Ok(())
            }
            Command::Lsp => {
                run_lsp_server().map_err(|e| {
                    error::CliError::runtime_error(
                        format!("LSP server failed: {e}"),
                        None,
                        None,
                        None,
                        None,
                    )
                })?;
                Ok(())
            }
            Command::Task { task_name } => run_task(task_name).map_err(|e| *e),
            Command::Config { action } => handle_config_command(action),
        }
    })();
    drop(_tokio_guard);
    nova_result
}

fn generate_completions(shell: Option<Shell>) {
    let mut cmd = Cli::command();
    let bin_name = "andromeda";

    match shell {
        Some(shell) => {
            let mut out = io::stdout();
            generate(shell, &mut cmd, bin_name, &mut out as &mut dyn io::Write);
        }
        None => {
            if let Some(detected_shell) = detect_shell() {
                let mut out = io::stdout();
                generate(
                    detected_shell,
                    &mut cmd,
                    bin_name,
                    &mut out as &mut dyn io::Write,
                );
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

impl From<ConfigFileFormat> for ConfigFormat {
    fn from(format: ConfigFileFormat) -> Self {
        match format {
            ConfigFileFormat::Json => ConfigFormat::Json,
            ConfigFileFormat::Toml => ConfigFormat::Toml,
            ConfigFileFormat::Yaml => ConfigFormat::Yaml,
        }
    }
}

#[allow(clippy::result_large_err)]
fn handle_config_command(action: ConfigAction) -> CliResult<()> {
    match action {
        ConfigAction::Init {
            format,
            output,
            force,
        } => {
            let config_format = ConfigFormat::from(format);
            let config_path = output.unwrap_or_else(|| {
                PathBuf::from(format!("andromeda.{}", config_format.extension()))
            });

            // Check if file exists and force is not set
            if config_path.exists() && !force {
                return Err(error::CliError::config_error(
                    format!(
                        "Config file already exists: {}. Use --force to overwrite.",
                        config_path.display()
                    ),
                    Some(config_path),
                    None::<std::io::Error>,
                ));
            }

            ConfigManager::create_default_config(&config_path, config_format).map_err(|e| {
                error::CliError::config_error(
                    format!("Failed to create config file: {e}"),
                    Some(config_path.clone()),
                    None::<std::io::Error>,
                )
            })?;

            println!("Created config file: {}", config_path.display());
            Ok(())
        }
        ConfigAction::Show { file } => {
            let config = if let Some(path) = file {
                ConfigManager::load_config(&path).map_err(|e| {
                    error::CliError::config_error(
                        format!("Failed to load config: {e}"),
                        Some(path),
                        None::<std::io::Error>,
                    )
                })?
            } else {
                ConfigManager::load_or_default(None)
            };

            println!("Current Andromeda Configuration:");
            println!("{}", serde_json::to_string_pretty(&config).unwrap());
            Ok(())
        }
        ConfigAction::Validate { file } => {
            let config = if let Some(path) = file {
                ConfigManager::load_config(&path).map_err(|e| {
                    error::CliError::config_error(
                        format!("Failed to load config: {e}"),
                        Some(path),
                        None::<std::io::Error>,
                    )
                })?
            } else if let Some((config_path, _)) = ConfigManager::find_config_file(None) {
                ConfigManager::load_config(&config_path).map_err(|e| {
                    error::CliError::config_error(
                        format!("Failed to load config: {e}"),
                        Some(config_path),
                        None::<std::io::Error>,
                    )
                })?
            } else {
                println!("No config file found. Using default configuration.");
                AndromedaConfig::default()
            };

            ConfigManager::validate_config(&config).map_err(|e| {
                error::CliError::config_error(
                    format!("Config validation failed: {e}"),
                    None,
                    None::<std::io::Error>,
                )
            })?;

            println!("Configuration is valid.");
            Ok(())
        }
    }
}
