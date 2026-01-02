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
use error::{Result, init_error_reporting, print_error};
mod format;
use format::{FormatResult, format_file};
mod helper;
use helper::{find_formattable_files_for_format, find_formattable_files_for_lint};
mod lint;
use lint::lint_file_with_config;
mod check;
use check::check_files_with_config;
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
    Lint {
        /// The file(s) or directory(ies) to lint
        #[arg(required = false)]
        paths: Vec<PathBuf>,
    },

    /// Type-check TypeScript files
    Check {
        /// The file(s) or directory(ies) to type-check
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
fn run_main() -> Result<()> {
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
                path,
                args,
            },
        }
    } else {
        Cli::parse()
    };

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .enable_io()
        .build()
        .map_err(|e| {
            error::AndromedaError::config_error(
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
                path,
                args: _,
            } => {
                let runtime_file = RuntimeFile::Local { path };
                run(verbose, no_strict, vec![runtime_file])
            }
            Command::Compile {
                path,
                out,
                verbose,
                no_strict,
            } => {
                compile(out.as_path(), path.as_path(), verbose, no_strict).map_err(|e| {
                    error::AndromedaError::compile_error(
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
                println!("✅ Successfully created the output binary at {out:?}{config_str}");
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
                    .map_err(|e| {
                        error::AndromedaError::repl_error(format!("REPL failed: {e}"), Some(e))
                    })
            }
            Command::Fmt { paths } => {
                // Load configuration
                let config = ConfigManager::load_or_default(None);

                let files_to_format = find_formattable_files_for_format(&paths, &config.format)?;
                if files_to_format.is_empty() {
                    let warning = Style::new().yellow().bold().apply_to("⚠️");
                    let msg = Style::new()
                        .yellow()
                        .apply_to("No formattable files found.");
                    println!("{warning} {msg}");
                    return Ok(());
                }

                let count = Style::new().cyan().apply_to(files_to_format.len());
                println!("Found {count} file(s) to format");
                println!("{}", Style::new().dim().apply_to("─".repeat(40)));

                let mut already_formatted_count = 0;
                let mut formatted_count = 0;

                for path in &files_to_format {
                    match format_file(path)? {
                        FormatResult::Changed => formatted_count += 1,
                        FormatResult::AlreadyFormatted => already_formatted_count += 1,
                    }
                }

                println!();
                let success = Style::new().green().bold().apply_to("✅");
                let complete_msg = Style::new().green().bold().apply_to("Formatting complete");
                println!("{success} {complete_msg}:");

                if formatted_count > 0 {
                    let formatted_icon = Style::new().green().apply_to("📄");
                    let formatted_num = Style::new().green().bold().apply_to(formatted_count);
                    let formatted_text = if formatted_count == 1 {
                        "file"
                    } else {
                        "files"
                    };
                    println!("   {formatted_icon} {formatted_num} {formatted_text} formatted");
                }

                if already_formatted_count > 0 {
                    let already_icon = Style::new().cyan().apply_to("✨");
                    let already_num = Style::new().cyan().bold().apply_to(already_formatted_count);
                    let already_text = if already_formatted_count == 1 {
                        "file"
                    } else {
                        "files"
                    };
                    let already_msg = Style::new().cyan().apply_to("already formatted");
                    println!("   {already_icon} {already_num} {already_text} {already_msg}");
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
                error::AndromedaError::runtime_error(
                    format!("Upgrade failed: {e}"),
                    None,
                    None,
                    None,
                    None,
                )
            }),
            Command::Bundle { input, output } => {
                bundle(input.to_str().unwrap(), output.to_str().unwrap()).map_err(|e| {
                    error::AndromedaError::runtime_error(
                        format!("Bundle failed: {e}"),
                        None,
                        None,
                        None,
                        None,
                    )
                })?;
                println!("✅ Successfully bundled and minified to {output:?}");
                Ok(())
            }
            Command::Lint { paths } => {
                // Load configuration
                let config = ConfigManager::load_or_default(None);

                let files_to_lint = find_formattable_files_for_lint(&paths, &config.lint)?;
                if files_to_lint.is_empty() {
                    println!("No lintable files found.");
                    return Ok(());
                }
                println!("Found {} file(s) to lint:", files_to_lint.len());
                let mut had_issues = false;
                for path in &files_to_lint {
                    if let Err(e) = lint_file_with_config(path, Some(config.clone())) {
                        print_error(e);
                        had_issues = true;
                    }
                }
                if had_issues {
                    Err(error::AndromedaError::runtime_error(
                        "Linting completed with errors".to_string(),
                        None,
                        None,
                        None,
                        None,
                    ))
                } else {
                    Ok(())
                }
            }
            Command::Check { paths } => {
                // Load configuration
                let config = ConfigManager::load_or_default(None);

                check_files_with_config(&paths, Some(config))
            }
            Command::Lsp => {
                run_lsp_server().map_err(|e| {
                    error::AndromedaError::runtime_error(
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
    });
    match rt.block_on(nova_thread) {
        Ok(result) => result,
        Err(e) => Err(error::AndromedaError::config_error(
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
fn handle_config_command(action: ConfigAction) -> Result<()> {
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
                return Err(error::AndromedaError::config_error(
                    format!(
                        "Config file already exists: {}. Use --force to overwrite.",
                        config_path.display()
                    ),
                    Some(config_path),
                    None::<std::io::Error>,
                ));
            }

            ConfigManager::create_default_config(&config_path, config_format).map_err(|e| {
                error::AndromedaError::config_error(
                    format!("Failed to create config file: {e}"),
                    Some(config_path.clone()),
                    None::<std::io::Error>,
                )
            })?;

            println!("✅ Created config file: {}", config_path.display());
            Ok(())
        }
        ConfigAction::Show { file } => {
            let config = if let Some(path) = file {
                ConfigManager::load_config(&path).map_err(|e| {
                    error::AndromedaError::config_error(
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
                    error::AndromedaError::config_error(
                        format!("Failed to load config: {e}"),
                        Some(path),
                        None::<std::io::Error>,
                    )
                })?
            } else if let Some((config_path, _)) = ConfigManager::find_config_file(None) {
                ConfigManager::load_config(&config_path).map_err(|e| {
                    error::AndromedaError::config_error(
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
                error::AndromedaError::config_error(
                    format!("Config validation failed: {e}"),
                    None,
                    None::<std::io::Error>,
                )
            })?;

            println!("✅ Configuration is valid!");
            Ok(())
        }
    }
}
