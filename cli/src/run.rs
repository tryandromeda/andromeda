// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::config::{AndromedaConfig, ConfigManager};
use crate::error::{Result, read_file_with_context};
use notify::Watcher;
use std::time::{Duration};
use andromeda_core::{
    AndromedaError, ErrorReporter, HostData, ImportMap, Runtime, RuntimeConfig, RuntimeFile,
};
use andromeda_runtime::{
    recommended_builtins, recommended_eventloop_handler, recommended_extensions,
};

#[allow(clippy::result_large_err)]
#[cfg_attr(feature = "hotpath", hotpath::measure)]
pub fn run(verbose: bool, no_strict: bool, watch: bool, files: Vec<RuntimeFile>) -> Result<()> {
    // get the directories of all local files
    let directories: Vec<std::path::PathBuf> = files
        .iter()
        .filter_map(|file| {
            if let RuntimeFile::Local { path } = file {
                std::path::Path::new(path).parent().map(|p| p.to_path_buf())
            } else {
                None
            }
        })
        .collect();

    if watch {
        watch_and_run(verbose, no_strict, files, directories)
    } else {
        create_runtime_files(verbose, no_strict, files, None)
    }
}

fn watch_and_run(
    verbose: bool,
    no_strict: bool,
    files: Vec<RuntimeFile>,
    directories: Vec<std::path::PathBuf>,
) -> Result<()> {
    let (tx, rx) = std::sync::mpsc::channel();
    let mut watcher = notify::recommended_watcher(tx).unwrap();

    for dir in &directories {
        watcher
            .watch(dir, notify::RecursiveMode::Recursive).unwrap();
    }

    create_runtime_files(verbose, no_strict, files.clone(), None)?;

    let debounce = Duration::from_millis(300);
    loop {
        // Wait for the first event (blocking)
        match rx.recv() {
            Ok(_event) => {
                // Now, collect any additional events that arrive within the debounce window
                while rx.recv_timeout(debounce).is_ok() {
                    // Just drain events within debounce window
                }
                // After debounce window, trigger the action
                create_runtime_files(verbose, no_strict, files.clone(), None)?;
            }
            Err(_e) => {
                break;
            }
        }
    }
    Ok(())
}

#[allow(clippy::result_large_err)]
#[cfg_attr(feature = "hotpath", hotpath::measure)]
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

    // Build import map from config
    let start_dir = filtered_files.first().and_then(|file| {
        if let RuntimeFile::Local { path } = file {
            std::path::Path::new(path).parent()
        } else {
            None
        }
    });
    let import_map = build_import_map(&config, start_dir)?;

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
            import_map,
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

/// Build import map from configuration
#[allow(clippy::result_large_err)]
fn build_import_map(
    config: &AndromedaConfig,
    start_dir: Option<&std::path::Path>,
) -> Result<Option<ImportMap>> {
    if config.imports.is_empty() && config.scopes.is_empty() && config.import_map_files.is_empty() {
        return Ok(None);
    }

    let mut import_map = ImportMap {
        imports: config.imports.clone(),
        scopes: config.scopes.clone(),
        integrity: config.integrity.clone(),
    };

    // Load import maps from config files
    for config_file in &config.import_map_files {
        let config_path = if std::path::Path::new(config_file).is_absolute() {
            std::path::PathBuf::from(config_file)
        } else {
            // Make relative to the config directory or start directory
            let base_dir = start_dir.unwrap_or_else(|| std::path::Path::new("."));
            base_dir.join(config_file)
        };

        if !config_path.exists() {
            return Err(crate::error::AndromedaError::file_not_found(
                config_path.clone(),
                std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Import map config file not found",
                ),
            ));
        }

        let file_import_map = ImportMap::from_file(&config_path).map_err(|e| {
            crate::error::AndromedaError::config_error(
                format!(
                    "Failed to load import map from {}: {}",
                    config_path.display(),
                    e
                ),
                Some(config_path),
                None::<std::io::Error>,
            )
        })?;

        import_map.merge(file_import_map);
    }

    Ok(Some(import_map))
}
