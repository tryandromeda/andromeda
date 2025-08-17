use andromeda::{CliError, CliResult};
use clap::{Args, Parser as ClapParser};
use std::num::NonZeroUsize;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, ClapParser)]
#[command(name = "wpt")]
#[command(about = "Web Platform Tests runner for Andromeda", long_about = None)]
struct Cli {
    #[command(flatten)]
    run_args: RunArgs,
}

#[derive(Debug, Args)]
struct RunArgs {
    #[arg(short = 'j', long)]
    num_threads: Option<NonZeroUsize>,

    #[arg(short = 'u', long)]
    update: bool,

    #[arg(long)]
    update_expectations: Option<bool>,

    #[arg(long)]
    update_metrics: Option<bool>,

    #[arg(short = 'n', long)]
    noprogress: bool,

    suites: Vec<String>,

    #[arg(long)]
    filter: Option<String>,

    #[arg(long)]
    skip: Option<String>,

    #[arg(short = 'o', long)]
    output: Option<PathBuf>,

    #[arg(long, default_value = "wpt")]
    wpt_dir: PathBuf,

    #[arg(long, default_value = "30")]
    timeout: u64,

    #[arg(short = 'v', long)]
    verbose: bool,
}

fn main() -> CliResult<()> {
    let cli = Cli::parse();
    let args = cli.run_args;

    run_wpt_tests(args)
}

fn run_wpt_tests(args: RunArgs) -> CliResult<()> {
    let update_expectations = args.update_expectations.unwrap_or(args.update);
    let update_metrics = args.update_metrics.unwrap_or(args.update);

    let current_dir = std::env::current_dir().map_err(CliError::Io)?;
    let in_tests_dir = current_dir.ends_with("tests");

    let wpt_dir = if args.wpt_dir.is_absolute() {
        args.wpt_dir.clone()
    } else {
        current_dir.join(&args.wpt_dir)
    };

    let (mut cmd, using_cargo) = if in_tests_dir {
        let mut c = Command::new("cargo");
        c.arg("run");
        c.arg("--bin");
        c.arg("wpt_test_runner");

        if std::env::args().any(|arg| arg == "--release") {
            c.arg("--release");
        }
        (c, true)
    } else {
        let test_runner_release = current_dir.join("target/release/wpt_test_runner");
        let test_runner_debug = current_dir.join("target/debug/wpt_test_runner");

        if test_runner_release.exists() && std::env::args().any(|arg| arg == "--release") {
            let mut c = Command::new(test_runner_release);
            c.current_dir("tests");
            (c, false)
        } else if test_runner_debug.exists() {
            let mut c = Command::new(test_runner_debug);
            c.current_dir("tests");
            (c, false)
        } else {
            let mut c = Command::new("cargo");
            c.arg("run");
            c.arg("--bin");
            c.arg("wpt_test_runner");
            c.arg("--manifest-path");
            c.arg("tests/Cargo.toml");

            if std::env::args().any(|arg| arg == "--release") {
                c.arg("--release");
            }
            (c, true)
        }
    };

    if using_cargo {
        cmd.arg("--");
    }

    cmd.arg("run");

    let suites_to_run = if args.suites.is_empty() {
        get_non_skipped_suites(&wpt_dir)
    } else {
        args.suites.clone()
    };

    if suites_to_run.is_empty() {
        eprintln!("No suites to run (all suites are skipped or not found)");
        eprintln!("WPT directory checked: {wpt_dir:?}");
        eprintln!("Directory exists: {}", wpt_dir.exists());
        if wpt_dir.exists() {
            eprintln!("Directory is_dir: {}", wpt_dir.is_dir());
        }
        std::process::exit(1);
    }

    if suites_to_run.len() > 1 {
        println!(
            "Running {} suites: {}",
            suites_to_run.len(),
            suites_to_run.join(", ")
        );
        println!("═══════════════════════════════════════════════════");
    }

    for suite in &suites_to_run {
        cmd.arg(suite);
    }

    if let Some(filter) = args.filter {
        cmd.arg("--filter");
        cmd.arg(filter);
    }

    if let Some(skip) = args.skip {
        cmd.arg("--skip");
        cmd.arg(skip);
    }

    if let Some(num_threads) = args.num_threads {
        cmd.arg("--threads");
        cmd.arg(num_threads.to_string());
    }

    cmd.arg("--timeout");
    cmd.arg(args.timeout.to_string());

    cmd.arg("--wpt-dir");
    if in_tests_dir {
        cmd.arg(&args.wpt_dir);
    } else {
        cmd.arg("wpt");
    }

    if let Some(output) = args.output {
        cmd.arg("--output");
        cmd.arg(output);
    } else if update_expectations || update_metrics {
        cmd.arg("--output");
        cmd.arg("results");
    }

    if args.verbose {
        cmd.arg("--verbose");
    }

    let status = cmd
        .status()
        .map_err(|e| CliError::TestExecution(format!("Failed to run WPT test runner: {e}")))?;

    if !status.success() {
        return Err(CliError::TestExecution(format!(
            "WPT test runner failed with exit code: {}",
            status.code().unwrap_or(1)
        )));
    }

    if update_expectations || update_metrics {
        handle_updates(update_expectations, update_metrics);
    }

    Ok(())
}

#[allow(dead_code)]
fn show_available_suites(wpt_dir: &PathBuf) {
    println!("Available WPT suites:");
    println!("═══════════════════════");

    if let Ok(entries) = std::fs::read_dir(wpt_dir) {
        let mut suites: Vec<String> = entries
            .filter_map(|entry| {
                entry.ok().and_then(|e| {
                    if e.file_type().ok()?.is_dir() {
                        let name = e.file_name().to_string_lossy().to_string();
                        if !name.starts_with('.') && !name.starts_with("common") {
                            Some(name)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
            })
            .collect();

        suites.sort();

        let current_dir = std::env::current_dir().expect("Failed to get current directory");
        let skip_path = if current_dir.ends_with("tests") {
            PathBuf::from("skip.json")
        } else {
            PathBuf::from("tests/skip.json")
        };
        let skipped_suites: std::collections::HashSet<String> = if skip_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&skip_path) {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                    json["skip"]
                        .as_array()
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|v| v.as_str().map(String::from))
                                .collect()
                        })
                        .unwrap_or_default()
                } else {
                    Default::default()
                }
            } else {
                Default::default()
            }
        } else {
            Default::default()
        };

        for suite in &suites {
            if skipped_suites.contains(suite) {
                println!("  {suite} (skipped)");
            } else {
                println!("  {suite}");
            }
        }

        println!("\nUsage:");
        println!("  wpt <suite_name>      # Run a specific suite");
        println!("  wpt -u <suite_name>   # Run and update expectations");
        println!("  wpt -j 8 <suite_name> # Run with 8 threads");
        println!("\nExamples:");
        println!("  wpt console");
        println!("  wpt fetch dom");
        println!("  wpt -u console");
    } else {
        eprintln!("Error: Could not read WPT directory at {wpt_dir:?}");
        std::process::exit(1);
    }
}

fn get_non_skipped_suites(wpt_dir: &PathBuf) -> Vec<String> {
    let current_dir = std::env::current_dir().expect("Failed to get current directory");
    let skip_path = if current_dir.ends_with("tests") {
        PathBuf::from("skip.json")
    } else {
        PathBuf::from("tests/skip.json")
    };
    let skipped_suites: std::collections::HashSet<String> = if skip_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&skip_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                json["skip"]
                    .as_array()
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(String::from))
                            .collect()
                    })
                    .unwrap_or_default()
            } else {
                Default::default()
            }
        } else {
            Default::default()
        }
    } else {
        Default::default()
    };

    let mut non_skipped = Vec::new();
    if let Ok(entries) = std::fs::read_dir(wpt_dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            if entry.file_type().ok().is_some_and(|t| t.is_dir()) {
                if let Some(name) = entry.file_name().to_str() {
                    if !name.starts_with('.')
                        && !name.starts_with("common")
                        && !skipped_suites.contains(name)
                    {
                        non_skipped.push(name.to_string());
                    }
                }
            }
        }
    }

    non_skipped.sort();
    non_skipped
}

fn handle_updates(update_expectations: bool, update_metrics: bool) {
    if update_expectations {
        println!("✅ Expectations updated in expectation.json");
    }
    if update_metrics {
        println!("📊 Metrics updated in metrics.json");
    }
}
