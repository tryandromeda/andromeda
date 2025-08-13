use clap::{Parser, Subcommand};
use std::path::PathBuf;

// Import the wpt module from the library
use andromeda::{error::Result, wpt};

/// WPT (Web Platform Tests) test runner for Andromeda
#[derive(Debug, Parser)]
#[command(name = "wpt")]
#[command(
    about = "Run WPT conformance tests for Andromeda",
    version = env!("CARGO_PKG_VERSION"),
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
    
    /// Update test expectations and metrics (shorthand for 'run --save-results')
    #[arg(short = 'u', long = "update", conflicts_with = "command")]
    update: bool,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Run WPT conformance tests
    Run {
        /// Path to WPT directory (default: tests/wpt)
        #[arg(long)]
        wpt_path: Option<PathBuf>,

        /// Number of threads to use for parallel test execution
        #[arg(long, default_value = "4")]
        threads: usize,

        /// Only run tests matching this pattern
        #[arg(long)]
        filter: Option<String>,

        /// Skip tests matching this pattern
        #[arg(long)]
        skip: Option<String>,

        /// Test specific directories (e.g., fetch, dom, streams)
        #[arg(long)]
        suite: Option<String>,

        /// Save test results to files
        #[arg(long)]
        save_results: bool,

        /// Directory to save results (default: tests/results)
        #[arg(long)]
        output_dir: Option<PathBuf>,
    },

    /// Display current metrics and WPT statistics
    Metrics {
        /// Show detailed metrics for each suite
        #[arg(long)]
        detailed: bool,

        /// Show trend information
        #[arg(long)]
        trends: bool,
    },
}

fn main() {
    if let Err(error) = run_main() {
        eprintln!("Error: {}", error);
        std::process::exit(1);
    }
}

fn run_main() -> Result<()> {
    let cli = Cli::parse();
    
    // Handle the -u shorthand flag
    if cli.update {
        return wpt::run_wpt_command(
            None,      // wpt_path
            4,         // threads
            None,      // filter
            None,      // skip
            None,      // suite
            true,      // save_results
            None,      // output_dir
        );
    }
    
    match cli.command {
        Some(Command::Run {
            wpt_path,
            threads,
            filter,
            skip,
            suite,
            save_results,
            output_dir,
        }) => wpt::run_wpt_command(
            wpt_path,
            threads,
            filter,
            skip,
            suite,
            save_results,
            output_dir,
        ),
        Some(Command::Metrics { detailed, trends }) => wpt::display_metrics(detailed, trends),
        None => {
            // Default behavior: show available suites
            wpt::run_wpt_command(
                None,  // wpt_path
                4,     // threads
                None,  // filter
                None,  // skip
                None,  // suite (None means show available suites)
                false, // save_results
                None,  // output_dir
            )
        }
    }
}