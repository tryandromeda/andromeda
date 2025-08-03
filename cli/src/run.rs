// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::config::{AndromedaConfig, ConfigManager};
use crate::error::{Result, read_file_with_context};
use andromeda_core::{
    AndromedaError, ErrorReporter, HostData, Runtime, RuntimeConfig, RuntimeFile,
};
use andromeda_runtime::{
    recommended_builtins, recommended_eventloop_handler, recommended_extensions,
};

#[allow(clippy::result_large_err)]
pub fn run(verbose: bool, no_strict: bool, files: Vec<RuntimeFile>) -> Result<()> {
    create_runtime_files(verbose, no_strict, files, None)
}

#[allow(clippy::result_large_err)]
pub fn create_runtime_files(
    verbose: bool,
    no_strict: bool,
    files: Vec<RuntimeFile>,
    config_override: Option<AndromedaConfig>,
) -> Result<()> {
    // Load configuration
    let config = config_override.unwrap_or_else(|| {
        // Try to load config from the directory of the first file, or current directory
        let start_dir = files.first().and_then(|file| {
            if let RuntimeFile::Local { path } = file {
                std::path::Path::new(path).parent()
            } else {
                None
            }
        });
        ConfigManager::load_or_default(start_dir)
    });

    // Apply CLI overrides to config
    let effective_verbose = verbose || config.runtime.verbose;
    let effective_no_strict = no_strict || config.runtime.no_strict;

    // Validate that we have files to run
    if files.is_empty() {
        return Err(crate::error::AndromedaError::invalid_argument(
            "files".to_string(),
            "at least one file path".to_string(),
            "empty list".to_string(),
        ));
    }

    // Apply include/exclude filters from config
    let filtered_files = apply_file_filters(files, &config)?;

    // Pre-validate all local files exist before starting the runtime
    for file in &filtered_files {
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
    let first_file_info = filtered_files.first().and_then(|file| {
        if let RuntimeFile::Local { path } = file {
            Some(path.clone())
        } else {
            None
        }
    });

    let runtime = Runtime::new(
        RuntimeConfig {
            no_strict: effective_no_strict,
            files: filtered_files,
            verbose: effective_verbose,
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
            if effective_verbose {
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
                // Try to find a keyword from the error message in the source code for better highlighting
                let keyword = error_message
                    .split_whitespace()
                    .find(|word| content.contains(*word));
                let source_span = if let Some(word) = keyword {
                    if let Some(pos) = content.find(word) {
                        miette::SourceSpan::new(pos.into(), word.len())
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

/// Apply include/exclude filters from configuration
#[allow(clippy::result_large_err)]
fn apply_file_filters(
    files: Vec<RuntimeFile>,
    config: &AndromedaConfig,
) -> Result<Vec<RuntimeFile>> {
    if config.runtime.include.is_empty() && config.runtime.exclude.is_empty() {
        return Ok(files);
    }

    let mut filtered_files = Vec::new();

    for file in files {
        let file_path = match &file {
            RuntimeFile::Local { path } => path,
            RuntimeFile::Embedded { path, .. } => path,
        };

        // Check exclude patterns first
        let mut excluded = false;
        for exclude_pattern in &config.runtime.exclude {
            if file_path.contains(exclude_pattern) {
                excluded = true;
                break;
            }
        }

        if excluded {
            continue;
        }

        // If include patterns are specified, file must match at least one
        if !config.runtime.include.is_empty() {
            let mut included = false;
            for include_pattern in &config.runtime.include {
                if file_path.contains(include_pattern) {
                    included = true;
                    break;
                }
            }
            if !included {
                continue;
            }
        }

        filtered_files.push(file);
    }

    Ok(filtered_files)
}
