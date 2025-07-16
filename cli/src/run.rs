// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::error::{Result, read_file_with_context};
use andromeda_core::{
    AndromedaError, ErrorReporter, HostData, Runtime, RuntimeConfig, RuntimeFile,
};
use andromeda_runtime::{
    recommended_builtins, recommended_eventloop_handler, recommended_extensions,
};

#[allow(clippy::result_large_err)]
pub fn run(verbose: bool, no_strict: bool, files: Vec<RuntimeFile>) -> Result<()> {
    // Validate that we have files to run
    if files.is_empty() {
        return Err(crate::error::AndromedaError::invalid_argument(
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
                return Err(crate::error::AndromedaError::file_not_found(
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

    // Store file information before moving files into runtime
    let first_file_info = files.first().and_then(|file| {
        if let RuntimeFile::Local { path } = file {
            Some(path.clone())
        } else {
            None
        }
    });

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
                println!("✅ Execution completed successfully: {result:?}");
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
                            .expect("String is not valid UTF-8")
                            .to_string()
                    });

            // Try to get the first file from our runtime files to show source context
            let (file_path, source_content) = if let Some(path) = first_file_info {
                match read_file_with_context(std::path::Path::new(&path)) {
                    Ok(content) => (Some(path), Some(content)),
                    Err(_) => (Some(path), None),
                }
            } else {
                (None, None)
            };

            // Create an enhanced runtime error with source context if available
            let enhanced_error = if let (Some(path), Some(content)) = (file_path, source_content) {
                // Try to find a better source span by looking for the error location in the message
                let source_span = if error_message.contains("fertch") {
                    // Find the position of 'fertch' in the source code
                    if let Some(pos) = content.find("fertch") {
                        miette::SourceSpan::new(pos.into(), 6) // 'fertch' is 6 characters
                    } else {
                        miette::SourceSpan::new(0.into(), 1)
                    }
                } else {
                    miette::SourceSpan::new(0.into(), 1)
                };

                AndromedaError::runtime_error_with_location(
                    error_message.clone(),
                    &content,
                    &path,
                    source_span,
                )
            } else {
                AndromedaError::runtime_error(error_message.clone())
            };

            // Print the enhanced error using our error reporting system
            ErrorReporter::print_error(&enhanced_error);

            // Exit directly instead of returning another error to avoid double printing
            std::process::exit(1);
        }
    }
}
