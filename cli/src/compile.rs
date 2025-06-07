// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::error::{AndromedaError, Result, read_file_with_context};
use libsui::{Elf, Macho, PortableExecutable};
use std::{env::current_exe, fs::File, path::Path};

pub static ANDROMEDA_JS_CODE_SECTION: &str = "ANDROMEDABINCODE";

#[allow(clippy::result_large_err)]
pub fn compile(result_name: &Path, input_file: &Path) -> Result<()> {
    // Validate input file exists and is readable
    if !input_file.exists() {
        return Err(AndromedaError::file_not_found(
            input_file.to_path_buf(),
            std::io::Error::new(std::io::ErrorKind::NotFound, "Input file not found"),
        ));
    }

    // Validate we can read the input file
    let js_content = read_file_with_context(input_file)?;

    // Validate JS content is not empty
    if js_content.trim().is_empty() {
        return Err(AndromedaError::invalid_argument(
            "input_file".to_string(),
            "non-empty JavaScript/TypeScript file".to_string(),
            "empty file".to_string(),
        ));
    }

    // Get current executable
    let exe_path = current_exe().map_err(|e| {
        AndromedaError::config_error(
            "Failed to get current executable path".to_string(),
            None,
            Some(Box::new(e)),
        )
    })?;

    let exe = std::fs::read(&exe_path)
        .map_err(|e| AndromedaError::file_read_error(exe_path.clone(), e))?;

    let js = js_content.into_bytes();

    // Validate output directory exists or can be created
    if let Some(parent) = result_name.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent).map_err(|e| {
                AndromedaError::permission_denied(
                    format!("creating output directory {}", parent.display()),
                    Some(parent.to_path_buf()),
                    e,
                )
            })?;
        }
    }

    let mut out = File::create(result_name).map_err(|e| {
        AndromedaError::permission_denied(
            format!("creating output file {}", result_name.display()),
            Some(result_name.to_path_buf()),
            e,
        )
    })?;

    let os = std::env::consts::OS;

    match os {
        "macos" => {
            Macho::from(exe)
                .map_err(|e| {
                    AndromedaError::compile_error(
                        "Failed to parse macOS executable".to_string(),
                        input_file.to_path_buf(),
                        result_name.to_path_buf(),
                        Some(Box::new(e)),
                    )
                })?
                .write_section(ANDROMEDA_JS_CODE_SECTION, js)
                .map_err(|e| {
                    AndromedaError::compile_error(
                        "Failed to write JavaScript section to macOS executable".to_string(),
                        input_file.to_path_buf(),
                        result_name.to_path_buf(),
                        Some(Box::new(e)),
                    )
                })?
                .build_and_sign(&mut out)
                .map_err(|e| {
                    AndromedaError::compile_error(
                        "Failed to build and sign macOS executable".to_string(),
                        input_file.to_path_buf(),
                        result_name.to_path_buf(),
                        Some(Box::new(e)),
                    )
                })?;
        }
        "linux" => {
            Elf::new(&exe)
                .append(ANDROMEDA_JS_CODE_SECTION, &js, &mut out)
                .map_err(|e| {
                    AndromedaError::compile_error(
                        "Failed to append JavaScript section to Linux executable".to_string(),
                        input_file.to_path_buf(),
                        result_name.to_path_buf(),
                        Some(Box::new(e)),
                    )
                })?;
        }
        "windows" => {
            PortableExecutable::from(&exe)
                .map_err(|e| {
                    AndromedaError::compile_error(
                        "Failed to parse Windows executable".to_string(),
                        input_file.to_path_buf(),
                        result_name.to_path_buf(),
                        Some(Box::new(e)),
                    )
                })?
                .write_resource(ANDROMEDA_JS_CODE_SECTION, js)
                .map_err(|e| {
                    AndromedaError::compile_error(
                        "Failed to write JavaScript resource to Windows executable".to_string(),
                        input_file.to_path_buf(),
                        result_name.to_path_buf(),
                        Some(Box::new(e)),
                    )
                })?
                .build(&mut out)
                .map_err(|e| {
                    AndromedaError::compile_error(
                        "Failed to build Windows executable".to_string(),
                        input_file.to_path_buf(),
                        result_name.to_path_buf(),
                        Some(Box::new(e)),
                    )
                })?;
        }
        _ => {
            return Err(AndromedaError::unsupported_platform(os.to_string()));
        }
    }

    // Make the binary executable on Unix-like systems
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        use std::fs::{metadata, set_permissions};
        use std::os::unix::fs::PermissionsExt;

        if matches!(os, "macos" | "linux") {
            let mut perms = metadata(result_name).map_err(|e| {
                AndromedaError::permission_denied(
                    format!("reading permissions for {}", result_name.display()),
                    Some(result_name.to_path_buf()),
                    e,
                )
            })?;
            perms.set_mode(0o755); // rwxr-xr-x permissions
            set_permissions(result_name, perms).map_err(|e| {
                AndromedaError::permission_denied(
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
