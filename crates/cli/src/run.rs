// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::config::{AndromedaConfig, ConfigManager};
use crate::error::{CliResult, read_file_with_context};
use andromeda_core::{
    ErrorReporter, HostData, ImportMap, Runtime, RuntimeConfig, RuntimeError, RuntimeFile,
};
use andromeda_runtime::{
    recommended_builtins, recommended_eventloop_handler, recommended_extensions,
};

/// Run a single Andromeda file
///
/// Note: While this function accepts a Vec<RuntimeFile> for internal flexibility,
/// the CLI interface only passes a single file.
#[allow(clippy::result_large_err)]
#[hotpath::measure]
pub fn run(verbose: bool, no_strict: bool, files: Vec<RuntimeFile>) -> CliResult<()> {
    run_with_config(verbose, no_strict, files, None)
}

#[allow(clippy::result_large_err)]
#[hotpath::measure]
pub fn run_with_config(
    verbose: bool,
    no_strict: bool,
    files: Vec<RuntimeFile>,
    config_override: Option<AndromedaConfig>,
) -> CliResult<()> {
    // Initialize LLM provider automatically when the llm feature is enabled
    #[cfg(feature = "llm")]
    init_llm_provider();

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
        return Err(crate::error::CliError::invalid_argument(
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
                return Err(crate::error::CliError::file_not_found(
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
            let enhanced_error = if let (Some(path), Some(content)) = (&file_path, &source_content)
            {
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

                RuntimeError::runtime_error_with_location(
                    error_message.clone(),
                    content.clone(),
                    path.clone(),
                    source_span,
                )
            } else {
                RuntimeError::runtime_error(error_message.clone())
            };

            // Print the enhanced error and AI suggestion if available
            #[cfg(feature = "llm")]
            {
                print_error_with_llm_suggestion(
                    &enhanced_error,
                    &error_message,
                    source_content.as_deref(),
                    file_path.as_deref(),
                );
            }

            #[cfg(not(feature = "llm"))]
            {
                ErrorReporter::print_error(&enhanced_error);
            }

            // Exit directly instead of returning another error to avoid double printing
            std::process::exit(1);
        }
    }
}

/// Initialize the LLM provider (called automatically when llm feature is enabled)
#[cfg(feature = "llm")]
fn init_llm_provider() {
    use andromeda_core::enable_ai_suggestions;
    use andromeda_core::llm_suggestions::{
        LlmSuggestionConfig, copilot::init_copilot_provider, is_llm_initialized,
    };

    if !is_llm_initialized() {
        // Try to initialize the Copilot provider
        let config = LlmSuggestionConfig::default();
        match init_copilot_provider(config) {
            Ok(()) => {
                // Enable AI suggestions globally so parse errors also get suggestions
                enable_ai_suggestions();
            }
            Err(_) => {
                // Silently fail - AI suggestions just won't be available
                // User likely doesn't have GITHUB_TOKEN set
            }
        }
    }
}

/// Print error with LLM suggestion
#[cfg(feature = "llm")]
fn print_error_with_llm_suggestion(
    error: &RuntimeError,
    error_message: &str,
    source_code: Option<&str>,
    file_path: Option<&str>,
) {
    use andromeda_core::llm_suggestions::{ErrorContext, get_error_suggestion, is_llm_initialized};
    use owo_colors::OwoColorize;

    // First, print the error using the standard reporter
    ErrorReporter::print_error(error);

    // Then try to get and print an LLM suggestion if initialized
    if is_llm_initialized() {
        eprintln!();
        eprintln!(
            "{} {}",
            "🤖".bright_cyan(),
            "Fetching AI suggestion...".dimmed()
        );

        let mut context = ErrorContext::new(error_message);

        if let Some(source) = source_code {
            context = context.with_source_code(source);
        }

        if let Some(path) = file_path {
            context = context.with_file_path(path);
        }

        // Try to extract error type from the message
        if error_message.contains("ReferenceError") {
            context = context.with_error_type("ReferenceError");
        } else if error_message.contains("TypeError") {
            context = context.with_error_type("TypeError");
        } else if error_message.contains("SyntaxError") {
            context = context.with_error_type("SyntaxError");
        } else if error_message.contains("RangeError") {
            context = context.with_error_type("RangeError");
        }

        match get_error_suggestion(&context) {
            Some(suggestion) => {
                // Clear the "Fetching" message by moving cursor up
                eprint!("\x1b[1A\x1b[2K"); // Move up one line and clear it

                eprintln!();
                eprintln!(
                    "{} {} {}",
                    "💡".bright_yellow(),
                    "AI Suggestion".bright_yellow().bold(),
                    format!("(via {})", suggestion.provider_name).dimmed()
                );
                eprintln!("{}", "─".repeat(50).yellow());
                eprintln!("{}", suggestion.suggestion);
                eprintln!();
            }
            None => {
                // Clear the "Fetching" message
                eprint!("\x1b[1A\x1b[2K");
            }
        }
    }
}

/// Apply include/exclude filters from configuration
#[allow(clippy::result_large_err)]
fn apply_file_filters(
    files: Vec<RuntimeFile>,
    config: &AndromedaConfig,
) -> CliResult<Vec<RuntimeFile>> {
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
) -> CliResult<Option<ImportMap>> {
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
            return Err(crate::error::CliError::file_not_found(
                config_path.clone(),
                std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Import map config file not found",
                ),
            ));
        }

        let file_import_map = ImportMap::from_file(&config_path).map_err(|e| {
            crate::error::CliError::config_error(
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
