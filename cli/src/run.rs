// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::error::{AndromedaError, Result, extract_runtime_error_info, read_file_with_context};
use andromeda_core::{HostData, Runtime, RuntimeConfig, RuntimeFile};
use andromeda_runtime::{
    recommended_builtins, recommended_eventloop_handler, recommended_extensions,
};

#[allow(clippy::result_large_err)]
pub fn run(verbose: bool, no_strict: bool, files: Vec<RuntimeFile>) -> Result<()> {
    // Validate that we have files to run
    if files.is_empty() {
        return Err(AndromedaError::invalid_argument(
            "files".to_string(),
            "at least one file path".to_string(),
            "empty list".to_string(),
        ));
    }

    // Pre-validate all local files exist before starting the runtime
    for file in &files {
        if let RuntimeFile::Local { path } = file {
            let file_path = std::path::Path::new(path);
            if !file_path.exists() {
                return Err(AndromedaError::file_not_found(
                    file_path.to_path_buf(),
                    std::io::Error::new(std::io::ErrorKind::NotFound, "File not found"),
                ));
            }

            // Try to read the file to validate permissions
            read_file_with_context(file_path)?;
        }
    }

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
                println!("✅ Execution completed successfully: {:?}", result);
            }
            Ok(())
        }
        Err(error) => {
            // Extract detailed error information from Nova
            let error_message =
                runtime_output
                    .agent
                    .run_in_realm(&runtime_output.realm_root, |agent, gc| {
                        error
                            .value()
                            .string_repr(agent, gc)
                            .as_str(agent)
                            .to_string()
                    });

            // Try to extract file path from the error context if available
            let file_path =
                runtime_output
                    .agent
                    .run_in_realm(&runtime_output.realm_root, |_agent, _gc| {
                        // This is a simplified approach - Nova might provide better error context in the future
                        None::<String>
                    });

            // Extract line and column information from the error message
            let (clean_message, line, column) =
                extract_runtime_error_info(&error_message, file_path.clone());

            // Try to get source content if we have a file path
            let source_content = if let Some(ref path) = file_path {
                read_file_with_context(std::path::Path::new(path)).ok()
            } else {
                None
            };

            Err(AndromedaError::runtime_error(
                clean_message,
                file_path,
                line,
                column,
                source_content,
            ))
        }
    }
}
