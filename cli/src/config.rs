// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::error::{AndromedaError, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Andromeda configuration structure
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(default)]
pub struct AndromedaConfig {
    /// Runtime configuration
    pub runtime: RuntimeConfig,
    /// Default formatting configuration
    pub format: FormatConfig,
    /// Linting configuration
    pub lint: LintConfig,
    /// Project name
    pub name: Option<String>,
    /// Project version
    pub version: Option<String>,
    /// Project description
    pub description: Option<String>,
    /// Project author(s)
    pub author: Option<String>,
    /// License
    pub license: Option<String>,
}

/// Runtime execution configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(default)]
pub struct RuntimeConfig {
    /// Disable strict mode
    pub no_strict: bool,
    /// Enable verbose output
    pub verbose: bool,
    /// Disable garbage collection (for debugging)
    pub disable_gc: bool,
    /// Print internal debugging information
    pub print_internals: bool,
    /// Expose Nova internal APIs
    pub expose_internals: bool,
    /// List of files to include in runtime
    pub include: Vec<String>,
    /// List of files to exclude from runtime
    pub exclude: Vec<String>,
    /// Runtime timeout in milliseconds
    pub timeout: Option<u64>,
}

/// Code formatting configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct FormatConfig {
    /// Line width for formatting
    pub line_width: u32,
    /// Use tabs instead of spaces
    pub use_tabs: bool,
    /// Tab width
    pub tab_width: u32,
    /// Trailing commas
    pub trailing_comma: bool,
    /// Semicolons preference
    pub semicolons: bool,
    /// Single quotes preference
    pub single_quotes: bool,
}

/// Linting configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct LintConfig {
    /// Enable linting
    pub enabled: bool,
    /// Lint rules to enable
    pub rules: Vec<String>,
    /// Lint rules to disable
    pub disabled_rules: Vec<String>,
    /// Maximum number of warnings before error
    pub max_warnings: Option<u32>,
}

impl Default for FormatConfig {
    fn default() -> Self {
        Self {
            line_width: 80,
            use_tabs: false,
            tab_width: 2,
            trailing_comma: false,
            semicolons: true,
            single_quotes: false,
        }
    }
}

impl Default for LintConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            rules: Vec::new(),
            disabled_rules: Vec::new(),
            max_warnings: None,
        }
    }
}

/// Configuration file formats
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConfigFormat {
    Json,
    Toml,
    Yaml,
}

impl ConfigFormat {
    /// Get the file extension for this format
    pub fn extension(&self) -> &'static str {
        match self {
            ConfigFormat::Json => "json",
            ConfigFormat::Toml => "toml",
            ConfigFormat::Yaml => "yaml",
        }
    }

    /// Detect format from file extension
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "json" => Some(ConfigFormat::Json),
            "toml" => Some(ConfigFormat::Toml),
            "yaml" | "yml" => Some(ConfigFormat::Yaml),
            _ => None,
        }
    }
}

/// Configuration loader and saver
pub struct ConfigManager;

impl ConfigManager {
    /// Find a configuration file in the current directory or parent directories
    pub fn find_config_file(start_dir: Option<&Path>) -> Option<(PathBuf, ConfigFormat)> {
        let start = start_dir.unwrap_or_else(|| Path::new("."));
        let mut current = start;

        // List of config file names to look for, in order of preference
        let config_files = [
            ("andromeda.json", ConfigFormat::Json),
            ("andromeda.toml", ConfigFormat::Toml),
            ("andromeda.yaml", ConfigFormat::Yaml),
            ("andromeda.yml", ConfigFormat::Yaml),
        ];

        loop {
            for (filename, format) in &config_files {
                let config_path = current.join(filename);
                if config_path.exists() && config_path.is_file() {
                    return Some((config_path, *format));
                }
            }

            // Move up to parent directory
            if let Some(parent) = current.parent() {
                current = parent;
            } else {
                break;
            }
        }

        None
    }

    /// Load configuration from a file
    #[allow(clippy::result_large_err)]
    pub fn load_config(path: &Path) -> Result<AndromedaConfig> {
        let content = std::fs::read_to_string(path).map_err(|e| {
            AndromedaError::config_error(
                format!("Failed to read config file: {}", path.display()),
                Some(path.to_path_buf()),
                Some(e),
            )
        })?;

        let format = path
            .extension()
            .and_then(|ext| ext.to_str())
            .and_then(ConfigFormat::from_extension)
            .ok_or_else(|| {
                AndromedaError::config_error(
                    format!("Unsupported config file format: {}", path.display()),
                    Some(path.to_path_buf()),
                    None::<std::io::Error>,
                )
            })?;

        Self::parse_config(&content, format, path)
    }

    /// Load configuration from current directory or create default
    pub fn load_or_default(start_dir: Option<&Path>) -> AndromedaConfig {
        if let Some((config_path, _)) = Self::find_config_file(start_dir) {
            match Self::load_config(&config_path) {
                Ok(config) => {
                    // println!("📝 Using config file: {}", config_path.display());
                    config
                }
                Err(err) => {
                    eprintln!("⚠️  Config file error: {err}");
                    eprintln!("   Using default configuration");
                    AndromedaConfig::default()
                }
            }
        } else {
            AndromedaConfig::default()
        }
    }

    /// Parse configuration from string content
    #[allow(clippy::result_large_err)]
    fn parse_config(content: &str, format: ConfigFormat, path: &Path) -> Result<AndromedaConfig> {
        let config = match format {
            ConfigFormat::Json => serde_json::from_str(content).map_err(|e| {
                AndromedaError::config_error(
                    format!("Invalid JSON in config file: {e}"),
                    Some(path.to_path_buf()),
                    Some(e),
                )
            }),
            ConfigFormat::Toml => toml::from_str(content).map_err(|e| {
                AndromedaError::config_error(
                    format!("Invalid TOML in config file: {e}"),
                    Some(path.to_path_buf()),
                    Some(e),
                )
            }),
            ConfigFormat::Yaml => serde_yaml::from_str(content).map_err(|e| {
                AndromedaError::config_error(
                    format!("Invalid YAML in config file: {e}"),
                    Some(path.to_path_buf()),
                    Some(e),
                )
            }),
        }?;

        Ok(config)
    }

    /// Save configuration to a file
    #[allow(clippy::result_large_err)]
    pub fn save_config(config: &AndromedaConfig, path: &Path, format: ConfigFormat) -> Result<()> {
        let content = match format {
            ConfigFormat::Json => serde_json::to_string_pretty(config).map_err(|e| {
                AndromedaError::config_error(
                    format!("Failed to serialize config as JSON: {e}"),
                    Some(path.to_path_buf()),
                    Some(e),
                )
            }),
            ConfigFormat::Toml => toml::to_string_pretty(config).map_err(|e| {
                AndromedaError::config_error(
                    format!("Failed to serialize config as TOML: {e}"),
                    Some(path.to_path_buf()),
                    Some(e),
                )
            }),
            ConfigFormat::Yaml => serde_yaml::to_string(config).map_err(|e| {
                AndromedaError::config_error(
                    format!("Failed to serialize config as YAML: {e}"),
                    Some(path.to_path_buf()),
                    Some(e),
                )
            }),
        }?;

        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                AndromedaError::config_error(
                    format!("Failed to create config directory: {}", parent.display()),
                    Some(path.to_path_buf()),
                    Some(e),
                )
            })?;
        }

        std::fs::write(path, content).map_err(|e| {
            AndromedaError::config_error(
                format!("Failed to write config file: {}", path.display()),
                Some(path.to_path_buf()),
                Some(e),
            )
        })?;

        Ok(())
    }

    /// Create a new config file with default values
    #[allow(clippy::result_large_err)]
    pub fn create_default_config(path: &Path, format: ConfigFormat) -> Result<()> {
        let config = AndromedaConfig::default();
        Self::save_config(&config, path, format)?;
        println!("✅ Created default config file: {}", path.display());
        Ok(())
    }

    /// Validate configuration
    #[allow(clippy::result_large_err)]
    pub fn validate_config(config: &AndromedaConfig) -> Result<()> {
        // Validate runtime configuration
        if let Some(timeout) = config.runtime.timeout {
            if timeout == 0 {
                return Err(AndromedaError::config_error(
                    "Runtime timeout cannot be zero".to_string(),
                    None,
                    None::<std::io::Error>,
                ));
            }
        }

        // Validate format configuration
        if config.format.line_width < 20 || config.format.line_width > 500 {
            return Err(AndromedaError::config_error(
                "Format line width must be between 20 and 500".to_string(),
                None,
                None::<std::io::Error>,
            ));
        }

        if config.format.tab_width == 0 || config.format.tab_width > 16 {
            return Err(AndromedaError::config_error(
                "Tab width must be between 1 and 16".to_string(),
                None,
                None::<std::io::Error>,
            ));
        }

        // Validate lint configuration
        if let Some(max_warnings) = config.lint.max_warnings {
            if max_warnings == 0 {
                return Err(AndromedaError::config_error(
                    "Maximum warnings cannot be zero".to_string(),
                    None,
                    None::<std::io::Error>,
                ));
            }
        }

        Ok(())
    }
}
