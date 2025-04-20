// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
use andromeda_core::{HostData, Runtime, RuntimeConfig};
use andromeda_runtime::{
    recommended_builtins, recommended_eventloop_handler, recommended_extensions,
};
use clap::{Parser as ClapParser, Subcommand};
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
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
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
            let (macro_task_tx, macro_task_rx) = std::sync::mpsc::channel();
            let host_data = HostData::new(macro_task_tx);
            let runtime = Runtime::new(
                RuntimeConfig {
                    no_strict,
                    paths,
                    verbose,
                    extensions: recommended_extensions(),
                    builtins: recommended_builtins(),
                    eventloop_handler: recommended_eventloop_handler,
                    macro_task_rx,
                },
                host_data,
            );
            let mut runtime_output = runtime.run();

            match runtime_output.result {
                Ok(result) => {
                    if verbose {
                        println!("{:?}", result);
                    }
                }
                Err(error) => {
                    runtime_output
                        .agent
                        .run_in_realm(&runtime_output.realm_root, |agent, gc| {
                            eprintln!(
                                "Uncaught exception: {}",
                                error.value().string_repr(agent, gc).as_str(agent)
                            );
                            std::process::exit(1);
                        })
                }
            }
        }
    });

    rt.block_on(nova_thread)
        .expect("oh no! Something went wrong when running Andromeda.");

    Ok(())
}
