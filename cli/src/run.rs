use andromeda_core::{HostData, Runtime, RuntimeConfig, RuntimeFile};
use andromeda_runtime::{
    recommended_builtins, recommended_eventloop_handler, recommended_extensions,
};

pub fn run(verbose: bool, no_strict: bool, files: Vec<RuntimeFile>) {
    let (macro_task_tx, macro_task_rx) = std::sync::mpsc::channel();
    let host_data = HostData::new(macro_task_tx);
    let runtime = Runtime::new(
        RuntimeConfig {
            no_strict,
            files,
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
        Err(error) => runtime_output
            .agent
            .run_in_realm(&runtime_output.realm_root, |agent, gc| {
                eprintln!(
                    "Uncaught exception: {}",
                    error.value().string_repr(agent, gc).as_str(agent)
                );
                std::process::exit(1);
            }),
    }
}
