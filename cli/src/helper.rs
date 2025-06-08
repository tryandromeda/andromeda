// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::error::{AndromedaError, Result};
use std::fs;
use std::path::PathBuf;

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
            if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
                if should_skip_directory(dir_name) {
                    continue;
                }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_formattable_file() {
        assert!(is_formattable_file(&PathBuf::from("test.ts")));
        assert!(is_formattable_file(&PathBuf::from("test.tsx")));
        assert!(is_formattable_file(&PathBuf::from("test.js")));
        assert!(is_formattable_file(&PathBuf::from("test.jsx")));
        assert!(is_formattable_file(&PathBuf::from("test.json")));
        assert!(is_formattable_file(&PathBuf::from("test.jsonc")));
        assert!(!is_formattable_file(&PathBuf::from("test.txt")));
        assert!(!is_formattable_file(&PathBuf::from("test.md")));
        assert!(!is_formattable_file(&PathBuf::from("test.rs")));
    }

    #[test]
    fn test_should_skip_directory() {
        assert!(should_skip_directory("node_modules"));
        assert!(should_skip_directory(".git"));
        assert!(should_skip_directory("target"));
        assert!(should_skip_directory("dist"));
        assert!(!should_skip_directory("src"));
        assert!(!should_skip_directory("lib"));
        assert!(!should_skip_directory("components"));
    }
}
