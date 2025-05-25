// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
use clap::{Parser as ClapParser, Subcommand};
use libsui::find_section;
use std::path::PathBuf;

mod compile;
use compile::{ANDROMEDA_JS_CODE_SECTION, compile};
mod run;
use run::run;

/// A JavaScript runtime
#[derive(Debug, ClapParser)]
#[command(name = "andromeda")]
#[command(
    about = "The coolest JavaScript Runtime",
    long_about = "JS/TS Runtime in rust powered by Nova with no transpilation BS"
)]
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
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Check if this is currently a single-file executable
    if let Ok(Some(js)) = find_section(ANDROMEDA_JS_CODE_SECTION) {
        run(false, false, (Vec::new(), vec![js]));
        return Ok(());
    }

    let args = Cli::parse();

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()
        .unwrap();

    // Run Nova in a secondary blocking thread so tokio tasks can still run
    let nova_thread = rt.spawn_blocking(move || match args.command {
        Command::Run {
            verbose,
            no_strict,
            paths,
        } => {
            run(verbose, no_strict, (paths, Vec::new()));
        }
        Command::Compile { path, out } => match compile(out.as_path(), path.as_path()) {
            Ok(_) => println!("Successfully created the output binary at {:?}", out),
            Err(e) => eprintln!("Failed to output binary: {}", e),
        },
    });

    rt.block_on(nova_thread)
        .expect("oh no! Something went wrong when running Andromeda.");

    Ok(())
}
