// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::config::{ConfigManager, FormatConfig};
use crate::error::{AndromedaError, Result};
use dprint_core::configuration::{ConfigKeyValue, GlobalConfiguration, NewLineKind};
use dprint_plugin_json::{configuration as json_config, format_text as json_format};
use dprint_plugin_typescript::{
    FormatTextOptions as TsFormatTextOptions, configuration as ts_config, format_text as ts_format,
};
use indexmap::IndexMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, PartialEq)]
enum FileType {
    TypeScript,
    JavaScript,
    Json,
}

/// Creates a dprint configuration for TypeScript/JavaScript formatting
fn create_ts_dprint_config(format_config: &FormatConfig) -> IndexMap<String, ConfigKeyValue> {
    let mut config = IndexMap::new();

    config.insert(
        "lineWidth".to_string(),
        ConfigKeyValue::from_i32(format_config.line_width as i32),
    );
    config.insert(
        "indentWidth".to_string(),
        ConfigKeyValue::from_i32(format_config.tab_width as i32),
    );
    config.insert(
        "useTabs".to_string(),
        ConfigKeyValue::from_bool(format_config.use_tabs),
    );
    config.insert(
        "semiColons".to_string(),
        ConfigKeyValue::from_str(if format_config.semicolons {
            "always"
        } else {
            "prefer_none"
        }),
    );
    config.insert(
        "quoteStyle".to_string(),
        ConfigKeyValue::from_str(if format_config.single_quotes {
            "preferSingle"
        } else {
            "preferDouble"
        }),
    );
    config.insert(
        "trailingCommas".to_string(),
        ConfigKeyValue::from_str(if format_config.trailing_comma {
            "always"
        } else {
            "onlyMultiLine"
        }),
    );
    config.insert("newLineKind".to_string(), ConfigKeyValue::from_str("lf"));
    config.insert(
        "useBraces".to_string(),
        ConfigKeyValue::from_str("whenNotSingleLine"),
    );
    config.insert(
        "bracePosition".to_string(),
        ConfigKeyValue::from_str("sameLineUnlessHanging"),
    );
    config.insert(
        "singleBodyPosition".to_string(),
        ConfigKeyValue::from_str("sameLine"),
    );
    config.insert(
        "nextControlFlowPosition".to_string(),
        ConfigKeyValue::from_str("sameLine"),
    );
    config.insert(
        "operatorPosition".to_string(),
        ConfigKeyValue::from_str("sameLine"),
    );
    config.insert(
        "preferHanging".to_string(),
        ConfigKeyValue::from_bool(false),
    );

    config
}

/// Creates a dprint configuration for JSON formatting
fn create_json_dprint_config() -> IndexMap<String, ConfigKeyValue> {
    let mut config = IndexMap::new();

    config.insert("lineWidth".to_string(), ConfigKeyValue::from_i32(100));
    config.insert("indentWidth".to_string(), ConfigKeyValue::from_i32(2));
    config.insert("useTabs".to_string(), ConfigKeyValue::from_bool(false));
    config.insert("newLineKind".to_string(), ConfigKeyValue::from_str("lf"));

    config
}

/// Determines the file type based on the file extension
#[allow(clippy::result_large_err)]
fn get_file_info(path: &std::path::Path) -> Result<FileType> {
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .ok_or_else(|| {
            AndromedaError::format_error(
                format!("Could not determine file type for: {}", path.display()),
                None::<std::io::Error>,
            )
        })?;

    match extension {
        "ts" | "tsx" | "mts" | "cts" => Ok(FileType::TypeScript),
        "js" | "jsx" | "mjs" | "cjs" => Ok(FileType::JavaScript),
        "json" | "jsonc" => Ok(FileType::Json),
        _ => Err(AndromedaError::format_error(
            format!("Unsupported file type: .{extension}"),
            None::<std::io::Error>,
        )),
    }
}

/// Formats a JavaScript, TypeScript, or JSON file using dprint.
#[allow(clippy::result_large_err)]
pub fn format_file(path: &PathBuf) -> Result<()> {
    format_file_with_config(path, None)
}

/// Formats a JavaScript, TypeScript, or JSON file using dprint with custom config.
#[allow(clippy::result_large_err)]
pub fn format_file_with_config(
    path: &PathBuf,
    config_override: Option<FormatConfig>,
) -> Result<()> {
    let original_content =
        fs::read_to_string(path).map_err(|e| AndromedaError::file_read_error(path.clone(), e))?;

    let file_type = get_file_info(path)?;

    // Load configuration
    let config = config_override.unwrap_or_else(|| {
        let andromeda_config = ConfigManager::load_or_default(path.parent());
        andromeda_config.format
    });

    let global_config = GlobalConfiguration {
        line_width: Some(config.line_width),
        use_tabs: Some(config.use_tabs),
        indent_width: Some(config.tab_width as u8),
        new_line_kind: Some(NewLineKind::LineFeed),
    };

    let formatted_content = match file_type {
        FileType::TypeScript | FileType::JavaScript => {
            format_ts_js_file(path, &original_content, &global_config, &config)?
        }
        FileType::Json => format_json_file(path, &original_content, &global_config)?,
    };

    match formatted_content {
        Some(content) if content != original_content => {
            fs::write(path, &content)
                .map_err(|e| AndromedaError::file_read_error(path.clone(), e))?;
            println!("📄 Formatted {}", path.display());
        }
        _ => {
            println!("✨ File {} is already formatted.", path.display());
        }
    }

    Ok(())
}

/// Formats TypeScript/JavaScript files
#[allow(clippy::result_large_err)]
fn format_ts_js_file(
    path: &PathBuf,
    original_content: &str,
    global_config: &GlobalConfiguration,
    format_config: &FormatConfig,
) -> Result<Option<String>> {
    let config = create_ts_dprint_config(format_config);
    let resolved_config = ts_config::resolve_config(config, global_config);

    if !resolved_config.diagnostics.is_empty() {
        let error_msg = resolved_config
            .diagnostics
            .iter()
            .map(|d| format!("{d}"))
            .collect::<Vec<_>>()
            .join(", ");
        return Err(AndromedaError::format_error(
            format!("Failed to resolve dprint TS configuration: {error_msg}"),
            None::<std::io::Error>,
        ));
    }

    let config = resolved_config.config;
    let format_options = TsFormatTextOptions {
        path: path.as_ref(),
        extension: path.extension().and_then(|ext| ext.to_str()),
        text: original_content.to_string(),
        external_formatter: None,
        config: &config,
    };

    ts_format(format_options).map_err(|e| {
        AndromedaError::format_error(
            format!("Failed to format TS/JS file {}: {}", path.display(), e),
            None::<std::io::Error>,
        )
    })
}

/// Formats JSON files
#[allow(clippy::result_large_err)]
fn format_json_file(
    path: &PathBuf,
    original_content: &str,
    global_config: &GlobalConfiguration,
) -> Result<Option<String>> {
    let config = create_json_dprint_config();
    let resolved_config = json_config::resolve_config(config, global_config);

    if !resolved_config.diagnostics.is_empty() {
        let error_msg = resolved_config
            .diagnostics
            .iter()
            .map(|d| format!("{d}"))
            .collect::<Vec<_>>()
            .join(", ");
        return Err(AndromedaError::format_error(
            format!("Failed to resolve dprint JSON configuration: {error_msg}"),
            None::<std::io::Error>,
        ));
    }

    let config = resolved_config.config;

    json_format(path.as_ref(), original_content, &config).map_err(|e| {
        AndromedaError::format_error(
            format!("Failed to format JSON file {}: {}", path.display(), e),
            None::<std::io::Error>,
        )
    })
}
