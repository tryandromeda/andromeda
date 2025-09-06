// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::config::{FormatConfig, LintConfig};
use crate::error::{AndromedaError, Result};
use glob::Pattern;
use std::fs;
use std::path::{Path, PathBuf};

/// Recursively finds all JavaScript, TypeScript, and JSON files in the given directories
#[allow(clippy::result_large_err)]
pub fn find_formattable_files(paths: &[PathBuf]) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    // If no paths provided, use current directory
    let search_paths = if paths.is_empty() {
        vec![PathBuf::from(".")]
    } else {
        paths.to_vec()
    };

    for path in search_paths {
        if path.is_file() {
            // If it's a file, check if it's a supported type and add it
            if is_formattable_file(&path) {
                files.push(path);
            }
        } else if path.is_dir() {
            // If it's a directory, recursively find all supported files
            find_files_in_directory(&path, &mut files)?;
        } else {
            return Err(AndromedaError::format_error(
                format!("Path does not exist: {}", path.display()),
                None::<std::io::Error>,
            ));
        }
    }

    // Sort files for consistent output
    files.sort();
    Ok(files)
}

/// Recursively searches a directory for formattable files
#[allow(clippy::result_large_err)]
fn find_files_in_directory(dir: &PathBuf, files: &mut Vec<PathBuf>) -> Result<()> {
    let entries = fs::read_dir(dir).map_err(|e| {
        AndromedaError::format_error(
            format!("Failed to read directory {}: {}", dir.display(), e),
            Some(e),
        )
    })?;

    for entry in entries {
        let entry = entry.map_err(|e| {
            AndromedaError::format_error(
                format!("Failed to read directory entry in {}: {}", dir.display(), e),
                Some(e),
            )
        })?;

        let path = entry.path();

        if path.is_dir() {
            // Skip common directories that shouldn't be formatted
            if let Some(dir_name) = path.file_name().and_then(|n| n.to_str())
                && should_skip_directory(dir_name)
            {
                continue;
            }
            // Recursively search subdirectories
            find_files_in_directory(&path, files)?;
        } else if path.is_file() && is_formattable_file(&path) {
            files.push(path);
        }
    }

    Ok(())
}

/// Checks if a file has a supported extension for formatting
fn is_formattable_file(path: &std::path::Path) -> bool {
    if let Some(extension) = path.extension().and_then(|ext| ext.to_str()) {
        matches!(
            extension,
            "ts" | "tsx" | "mts" | "cts" | "js" | "jsx" | "mjs" | "cjs" | "json" | "jsonc"
        )
    } else {
        false
    }
}

/// Checks if a directory should be skipped during recursive search
fn should_skip_directory(dir_name: &str) -> bool {
    matches!(
        dir_name,
        "node_modules"
            | ".git"
            | ".svn"
            | ".hg"
            | "target"
            | "dist"
            | "build"
            | ".next"
            | ".nuxt"
            | "coverage"
            | ".nyc_output"
            | ".cache"
            | ".vscode"
            | ".idea"
            | ".DS_Store"
    )
}

/// Apply include/exclude filters to a list of files using glob patterns
#[allow(clippy::result_large_err)]
pub fn apply_include_exclude_filters(
    files: Vec<PathBuf>,
    include_patterns: &[String],
    exclude_patterns: &[String],
) -> Result<Vec<PathBuf>> {
    let mut filtered_files = Vec::new();

    for file in files {
        if should_include_file(&file, include_patterns, exclude_patterns)? {
            filtered_files.push(file);
        }
    }

    Ok(filtered_files)
}

/// Check if a file should be included based on include/exclude patterns
#[allow(clippy::result_large_err)]
fn should_include_file(
    file_path: &Path,
    include_patterns: &[String],
    exclude_patterns: &[String],
) -> Result<bool> {
    let path_str = file_path.to_string_lossy();

    let is_included = if include_patterns.is_empty() {
        true
    } else {
        include_patterns.iter().any(|pattern_str| {
            if let Ok(pattern) = Pattern::new(pattern_str) {
                pattern.matches(&path_str)
            } else {
                false
            }
        })
    };

    if !is_included {
        return Ok(false);
    }

    let mut is_excluded = false;

    for pattern_str in exclude_patterns {
        // Handle negated patterns (un-exclude) starting with "!"
        if let Some(negated_pattern) = pattern_str.strip_prefix('!') {
            let pattern = Pattern::new(negated_pattern).map_err(|e| {
                AndromedaError::format_error(
                    format!("Invalid glob pattern '{negated_pattern}': {e}"),
                    None::<std::io::Error>,
                )
            })?;

            // If this is a negated pattern and it matches, we should include the file
            // (effectively un-excluding it)
            if pattern.matches(&path_str) {
                is_excluded = false;
                break; // Negated patterns take precedence
            }
        } else {
            let pattern = Pattern::new(pattern_str).map_err(|e| {
                AndromedaError::format_error(
                    format!("Invalid glob pattern '{pattern_str}': {e}"),
                    None::<std::io::Error>,
                )
            })?;

            if pattern.matches(&path_str) {
                is_excluded = true;
                // Don't break here - keep checking for negated patterns
            }
        }
    }

    Ok(!is_excluded)
}

/// Enhanced version of find_formattable_files that accepts include/exclude patterns
#[allow(clippy::result_large_err)]
pub fn find_formattable_files_with_filters(
    paths: &[PathBuf],
    include_patterns: &[String],
    exclude_patterns: &[String],
) -> Result<Vec<PathBuf>> {
    let all_files = find_formattable_files(paths)?;

    apply_include_exclude_filters(all_files, include_patterns, exclude_patterns)
}

/// Find formattable files using FormatConfig include/exclude patterns
#[allow(clippy::result_large_err)]
pub fn find_formattable_files_for_format(
    paths: &[PathBuf],
    format_config: &FormatConfig,
) -> Result<Vec<PathBuf>> {
    find_formattable_files_with_filters(paths, &format_config.include, &format_config.exclude)
}

/// Find formattable files using LintConfig include/exclude patterns
#[allow(clippy::result_large_err)]
pub fn find_formattable_files_for_lint(
    paths: &[PathBuf],
    lint_config: &LintConfig,
) -> Result<Vec<PathBuf>> {
    find_formattable_files_with_filters(paths, &lint_config.include, &lint_config.exclude)
}
