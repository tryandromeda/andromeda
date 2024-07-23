// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
mod helper;
mod runtime;
mod theme;

use andromeda_core::initialize_recommended_extensions;
use anymap::AnyMap;
use clap::{Parser as ClapParser, Subcommand};
use cliclack::{input, intro, set_theme};
use helper::exit_with_parse_errors;
use nova_vm::ecmascript::{
    execution::{
        agent::{HostHooks, Job, Options},
        initialize_host_defined_realm, Agent, Realm,
    },
    scripts_and_modules::script::{parse_script, script_evaluation},
    types::{Object, Value},
};
use runtime::attach_builtins;
use std::{any::Any, cell::RefCell, collections::VecDeque, fmt::Debug};
use theme::DefaultTheme;

/// A JavaScript runtime
#[derive(Debug, ClapParser)] // requires `derive` feature
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

    /// Runs the REPL
    Repl {},
}

struct CliHostHooks {
    promise_job_queue: RefCell<VecDeque<Job>>,
    storage: RefCell<AnyMap>,
}

impl Default for CliHostHooks {
    fn default() -> Self {
        Self {
            promise_job_queue: RefCell::default(),
            storage: RefCell::new(AnyMap::new()),
        }
    }
}

// RefCell doesn't implement Debug
impl Debug for CliHostHooks {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CliHostHooks")
            //.field("promise_job_queue", &*self.promise_job_queue.borrow())
            .finish()
    }
}

impl CliHostHooks {
    fn pop_promise_job(&self) -> Option<Job> {
        self.promise_job_queue.borrow_mut().pop_front()
    }
}

impl HostHooks for CliHostHooks {
    fn enqueue_promise_job(&self, job: Job) {
        self.promise_job_queue.borrow_mut().push_back(job);
    }

    fn get_host_data(&self) -> &dyn Any {
        &self.storage
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();

    match args.command {
        Command::Run {
            verbose,
            no_strict,
            paths,
        } => {
            let allocator = Default::default();

            let host_hooks: &CliHostHooks = &*Box::leak(Box::default());
            let mut agent = Agent::new(
                Options {
                    disable_gc: false,
                    print_internals: verbose,
                },
                host_hooks,
            );
            {
                let create_global_object: Option<fn(&mut Realm) -> Object> = None;
                let create_global_this_value: Option<fn(&mut Realm) -> Object> = None;
                initialize_host_defined_realm(
                    &mut agent,
                    create_global_object,
                    create_global_this_value,
                    Some(initialize_recommended_extensions),
                );
            }
            let realm = agent.current_realm_id();

            assert!(!paths.is_empty());
            attach_builtins(&allocator, &mut agent, no_strict);

            let mut final_result = Ok(Value::Null);
            // Fetch the runtime mod.ts file using a macro and add it to the paths
            for path in paths {
                let file = std::fs::read_to_string(&path)?;
                let script = match parse_script(&allocator, file.into(), realm, !no_strict, None) {
                    Ok(script) => script,
                    Err((file, errors)) => exit_with_parse_errors(errors, &path, &file),
                };
                final_result = script_evaluation(&mut agent, script);
                if final_result.is_err() {
                    break;
                }
            }

            if final_result.is_ok() {
                while let Some(job) = host_hooks.pop_promise_job() {
                    if let Err(err) = job.run(&mut agent) {
                        final_result = Err(err);
                        break;
                    }
                }
            }

            match final_result {
                Ok(result) => {
                    if verbose {
                        println!("{:?}", result);
                    }
                }
                Err(error) => {
                    eprintln!(
                        "Uncaught exception: {}",
                        error.value().string_repr(&mut agent).as_str(&agent)
                    );
                    std::process::exit(1);
                }
            }
        }
        Command::Repl {} => {
            let allocator = Default::default();
            let host_hooks: &CliHostHooks = &*Box::leak(Box::default());
            let mut agent = Agent::new(
                Options {
                    disable_gc: false,
                    print_internals: true,
                },
                host_hooks,
            );
            {
                let create_global_object: Option<fn(&mut Realm) -> Object> = None;
                let create_global_this_value: Option<fn(&mut Realm) -> Object> = None;
                initialize_host_defined_realm(
                    &mut agent,
                    create_global_object,
                    create_global_this_value,
                    Some(initialize_recommended_extensions),
                );
            }
            let realm = agent.current_realm_id();
            attach_builtins(&allocator, &mut agent, false);

            set_theme(DefaultTheme);
            println!("\n\n");
            let mut placeholder = "Enter a line of Javascript".to_string();

            loop {
                intro("Nova Repl (type exit or ctrl+c to exit)")?;
                let input: String = input("").placeholder(&placeholder).interact()?;

                if input.matches("exit").count() == 1 {
                    std::process::exit(0);
                }
                placeholder = input.to_string();
                let script = match parse_script(&allocator, input.into(), realm, true, None) {
                    Ok(script) => script,
                    Err((file, errors)) => {
                        exit_with_parse_errors(errors, "<stdin>", &file);
                    }
                };
                let result = script_evaluation(&mut agent, script);
                match result {
                    Ok(result) => {
                        println!("{:?}\n", result);
                    }
                    Err(error) => {
                        eprintln!(
                            "Uncaught exception: {}",
                            error.value().string_repr(&mut agent).as_str(&agent)
                        );
                    }
                }
            }
        }
    }
    Ok(())
}
