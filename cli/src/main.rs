// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
use andromeda_core::{Runtime, RuntimeConfig};
use clap::{Parser as ClapParser, Subcommand};
/// A JavaScript runtime
#[derive(Debug, ClapParser)]
#[command(name = "andromeda")]
#[command(
    about = "The coolest JavaScript Runtime",
    long_about = "The only javascript runtime that actually runs typescript"
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
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()
        .unwrap();

    // Run Nova in a secondary blocking thread so tokio tasks can still run
    let nova_thread = rt.spawn_blocking(|| match args.command {
        Command::Run {
            verbose,
            no_strict,
            paths,
        } => {
            let mut runtime = Runtime::new(RuntimeConfig {
                no_strict,
                db_path: ":memory:".to_string(),
                paths,
                verbose,
            });
            let runtime_result = runtime.run();

            match runtime_result {
                Ok(result) => {
                    if verbose {
                        println!("{:?}", result);
                    }
                }
                Err(error) => {
                    eprintln!(
                        "Uncaught exception: {}",
                        error
                            .value()
                            .string_repr(&mut runtime.agent)
                            .as_str(&runtime.agent)
                    );
                    std::process::exit(1);
                }
            }
        }
    });

    rt.block_on(nova_thread)
        .expect("oh no! Something went wrong when running Andromeda.");

    Ok(())
}
