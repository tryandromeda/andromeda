// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::error::{CliError, CliResult, read_file_with_context};
use libsui::{Elf, Macho, PortableExecutable};
use serde::{Deserialize, Serialize};
use std::{env::current_exe, fs::File, path::Path};

pub static ANDROMEDA_JS_CODE_SECTION: &str = "ANDROMEDABINCODE";
pub static ANDROMEDA_CONFIG_SECTION: &str = "ANDROMEDACONFIG";

/// Configuration embedded in compiled binaries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddedConfig {
    pub verbose: bool,
    pub no_strict: bool,
}

#[allow(clippy::result_large_err)]
#[hotpath::measure]
pub fn compile(
    result_name: &Path,
    input_file: &Path,
    verbose: bool,
    no_strict: bool,
) -> CliResult<()> {
    // Validate input file exists and is readable
    if !input_file.exists() {
        return Err(CliError::file_not_found(
            input_file.to_path_buf(),
            std::io::Error::new(std::io::ErrorKind::NotFound, "Input file not found"),
        ));
    }

    // Validate we can read the input file
    let js_content = read_file_with_context(input_file)?;

    // Validate JS content is not empty
    if js_content.trim().is_empty() {
        return Err(CliError::invalid_argument(
            "input_file".to_string(),
            "non-empty JavaScript/TypeScript file".to_string(),
            "empty file".to_string(),
        ));
    }

    // Get current executable
    let exe_path = current_exe().map_err(|e| {
        CliError::config_error(
            "Failed to get current executable path".to_string(),
            None,
            Some(Box::new(e)),
        )
    })?;

    let exe =
        std::fs::read(&exe_path).map_err(|e| CliError::file_read_error(exe_path.clone(), e))?;

    let js = js_content.into_bytes();

    // Create embedded config
    let config = EmbeddedConfig { verbose, no_strict };
    let config_json = serde_json::to_vec(&config).map_err(|e| {
        CliError::config_error(
            "Failed to serialize embedded config".to_string(),
            None,
            Some(Box::new(e)),
        )
    })?;

    // Validate output directory exists or can be created
    if let Some(parent) = result_name.parent()
        && !parent.exists()
    {
        std::fs::create_dir_all(parent).map_err(|e| {
            CliError::permission_denied(
                format!("creating output directory {}", parent.display()),
                Some(parent.to_path_buf()),
                e,
            )
        })?;
    }

    let mut out = File::create(result_name).map_err(|e| {
        CliError::permission_denied(
            format!("creating output file {}", result_name.display()),
            Some(result_name.to_path_buf()),
            e,
        )
    })?;

    let os = std::env::consts::OS;

    match os {
        "macos" => {
            // First pass: write JS code section
            Macho::from(exe)
                .map_err(|e| {
                    CliError::compile_error(
                        "Failed to parse macOS executable".to_string(),
                        input_file.to_path_buf(),
                        result_name.to_path_buf(),
                        Some(Box::new(e)),
                    )
                })?
                .write_section(ANDROMEDA_JS_CODE_SECTION, js)
                .map_err(|e| {
                    CliError::compile_error(
                        "Failed to write JavaScript section to macOS executable".to_string(),
                        input_file.to_path_buf(),
                        result_name.to_path_buf(),
                        Some(Box::new(e)),
                    )
                })?
                .build_and_sign(&mut out)
                .map_err(|e| {
                    CliError::compile_error(
                        "Failed to build and sign macOS executable (first pass)".to_string(),
                        input_file.to_path_buf(),
                        result_name.to_path_buf(),
                        Some(Box::new(e)),
                    )
                })?;

            // Second pass: re-read and add config section
            let exe_with_js = std::fs::read(result_name)
                .map_err(|e| CliError::file_read_error(result_name.to_path_buf(), e))?;
            let mut out = File::create(result_name).map_err(|e| {
                CliError::permission_denied(
                    format!("creating output file {}", result_name.display()),
                    Some(result_name.to_path_buf()),
                    e,
                )
            })?;
            Macho::from(exe_with_js)
                .map_err(|e| {
                    CliError::compile_error(
                        "Failed to parse macOS executable (second pass)".to_string(),
                        input_file.to_path_buf(),
                        result_name.to_path_buf(),
                        Some(Box::new(e)),
                    )
                })?
                .write_section(ANDROMEDA_CONFIG_SECTION, config_json)
                .map_err(|e| {
                    CliError::compile_error(
                        "Failed to write config section to macOS executable".to_string(),
                        input_file.to_path_buf(),
                        result_name.to_path_buf(),
                        Some(Box::new(e)),
                    )
                })?
                .build_and_sign(&mut out)
                .map_err(|e| {
                    CliError::compile_error(
                        "Failed to build and sign macOS executable (second pass)".to_string(),
                        input_file.to_path_buf(),
                        result_name.to_path_buf(),
                        Some(Box::new(e)),
                    )
                })?;
        }
        "linux" => {
            let elf = Elf::new(&exe);
            elf.append(ANDROMEDA_JS_CODE_SECTION, &js, &mut out)
                .map_err(|e| {
                    CliError::compile_error(
                        "Failed to append JavaScript section to Linux executable".to_string(),
                        input_file.to_path_buf(),
                        result_name.to_path_buf(),
                        Some(Box::new(e)),
                    )
                })?;
            // Note: libsui's Elf doesn't support multiple appends in sequence
            // We need to re-read the file and append the config section
            let exe_with_js = std::fs::read(result_name)
                .map_err(|e| CliError::file_read_error(result_name.to_path_buf(), e))?;
            let mut out = File::create(result_name).map_err(|e| {
                CliError::permission_denied(
                    format!("creating output file {}", result_name.display()),
                    Some(result_name.to_path_buf()),
                    e,
                )
            })?;
            Elf::new(&exe_with_js)
                .append(ANDROMEDA_CONFIG_SECTION, &config_json, &mut out)
                .map_err(|e| {
                    CliError::compile_error(
                        "Failed to append config section to Linux executable".to_string(),
                        input_file.to_path_buf(),
                        result_name.to_path_buf(),
                        Some(Box::new(e)),
                    )
                })?;
        }
        "windows" => {
            PortableExecutable::from(&exe)
                .map_err(|e| {
                    CliError::compile_error(
                        "Failed to parse Windows executable".to_string(),
                        input_file.to_path_buf(),
                        result_name.to_path_buf(),
                        Some(Box::new(e)),
                    )
                })?
                .write_resource(ANDROMEDA_JS_CODE_SECTION, js)
                .map_err(|e| {
                    CliError::compile_error(
                        "Failed to write JavaScript resource to Windows executable".to_string(),
                        input_file.to_path_buf(),
                        result_name.to_path_buf(),
                        Some(Box::new(e)),
                    )
                })?
                .write_resource(ANDROMEDA_CONFIG_SECTION, config_json)
                .map_err(|e| {
                    CliError::compile_error(
                        "Failed to write config resource to Windows executable".to_string(),
                        input_file.to_path_buf(),
                        result_name.to_path_buf(),
                        Some(Box::new(e)),
                    )
                })?
                .build(&mut out)
                .map_err(|e| {
                    CliError::compile_error(
                        "Failed to build Windows executable".to_string(),
                        input_file.to_path_buf(),
                        result_name.to_path_buf(),
                        Some(Box::new(e)),
                    )
                })?;
        }
        _ => {
            return Err(CliError::unsupported_platform(os.to_string()));
        }
    }

    // Make the binary executable on Unix-like systems
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        use std::fs::{metadata, set_permissions};
        use std::os::unix::fs::PermissionsExt;

        if matches!(os, "macos" | "linux") {
            let metadata = metadata(result_name).map_err(|e| {
                CliError::permission_denied(
                    format!("reading permissions for {}", result_name.display()),
                    Some(result_name.to_path_buf()),
                    e,
                )
            })?;
            let mut perms = metadata.permissions();
            perms.set_mode(0o755); // rwxr-xr-x permissions
            set_permissions(result_name, perms).map_err(|e| {
                CliError::permission_denied(
                    format!(
                        "setting executable permissions for {}",
                        result_name.display()
                    ),
                    Some(result_name.to_path_buf()),
                    e,
                )
            })?;
        }
    }

    Ok(())
}
