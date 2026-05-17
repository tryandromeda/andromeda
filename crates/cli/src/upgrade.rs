// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

#![allow(clippy::result_large_err)]

use crate::error::{CliError, CliResult};
use serde::Deserialize;
use std::env;
use std::fs;
use std::io::Write;
use std::path::Path;

const REPO_OWNER: &str = "tryandromeda";
const REPO_NAME: &str = "andromeda";
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    #[allow(dead_code)]
    name: String,
    #[allow(dead_code)]
    draft: bool,
    #[allow(dead_code)]
    prerelease: bool,
    assets: Vec<GitHubAsset>,
}

#[derive(Debug, Deserialize)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
    size: u64,
}

#[derive(Debug)]
struct PlatformInfo {
    #[allow(dead_code)]
    os: String,
    #[allow(dead_code)]
    arch: String,
    asset_name: String,
}

fn upgrade_err(
    operation: &str,
    message: impl Into<String>,
    source: Option<impl std::error::Error + Send + Sync + 'static>,
) -> CliError {
    CliError::upgrade_error(operation, message, source)
}

/// Run the upgrade process
pub fn run_upgrade(force: bool, target_version: Option<String>, dry_run: bool) -> CliResult<()> {
    println!("Andromeda Upgrade Tool");
    println!("Current version: {CURRENT_VERSION}");
    println!();

    let platform = detect_platform()?;
    println!("Detected platform: {}", platform.asset_name);

    let release = if let Some(version) = target_version {
        get_release_by_tag(&version)?
    } else {
        get_latest_release()?
    };

    println!("Latest available version: {}", release.tag_name);

    if !force && release.tag_name == CURRENT_VERSION {
        println!("You are already running the latest version!");
        return Ok(());
    }

    if release.tag_name == CURRENT_VERSION && !force {
        println!("You are already on version {CURRENT_VERSION}. Use --force to reinstall.");
        return Ok(());
    }

    let asset = release
        .assets
        .iter()
        .find(|asset| asset.name == platform.asset_name)
        .ok_or_else(|| {
            upgrade_err(
                "select_asset",
                format!(
                    "No release asset found for platform: {}",
                    platform.asset_name
                ),
                None::<std::io::Error>,
            )
        })?;

    println!("Found asset: {} ({} bytes)", asset.name, asset.size);

    if dry_run {
        println!(
            "Dry run mode - would upgrade from {} to {}",
            CURRENT_VERSION, release.tag_name
        );
        println!("Would download: {}", asset.browser_download_url);
        return Ok(());
    }

    if !force {
        print!(
            "Do you want to upgrade from {} to {}? [y/N]: ",
            CURRENT_VERSION, release.tag_name
        );
        std::io::stdout()
            .flush()
            .map_err(|e| upgrade_err("prompt_flush", e.to_string(), Some(e)))?;

        let mut input = String::new();
        std::io::stdin()
            .read_line(&mut input)
            .map_err(|e| upgrade_err("prompt_read", e.to_string(), Some(e)))?;
        let input = input.trim().to_lowercase();

        if input != "y" && input != "yes" {
            println!("Upgrade cancelled.");
            return Ok(());
        }
    }

    println!("Downloading {}...", asset.name);
    let new_binary = download_asset(&asset.browser_download_url)?;

    println!("Installing new version...");
    install_binary(&new_binary, &platform)?;

    println!("Successfully upgraded to version {}!", release.tag_name);
    println!("Run 'andromeda --version' to verify the new version.");

    Ok(())
}

/// Detect the current platform and return appropriate asset information
fn detect_platform() -> CliResult<PlatformInfo> {
    let os = if cfg!(target_os = "linux") {
        "linux"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(target_os = "windows") {
        "windows"
    } else {
        return Err(CliError::unsupported_platform(
            std::env::consts::OS.to_string(),
        ));
    };

    let arch = if cfg!(target_arch = "x86_64") {
        "amd64"
    } else if cfg!(target_arch = "aarch64") {
        "arm64"
    } else {
        return Err(CliError::unsupported_platform(format!(
            "unsupported architecture: {}",
            std::env::consts::ARCH
        )));
    };

    let asset_name = if os == "windows" {
        format!("andromeda-{os}-{arch}.exe")
    } else {
        format!("andromeda-{os}-{arch}")
    };

    Ok(PlatformInfo {
        os: os.to_string(),
        arch: arch.to_string(),
        asset_name,
    })
}

/// Get the latest release from GitHub
fn get_latest_release() -> CliResult<GitHubRelease> {
    let url = format!("https://api.github.com/repos/{REPO_OWNER}/{REPO_NAME}/releases/latest");

    let response = ureq::get(&url)
        .header("User-Agent", &format!("andromeda-cli/{CURRENT_VERSION}"))
        .call();

    match response {
        Ok(mut response) => {
            let release: GitHubRelease = response.body_mut().read_json().map_err(|e| {
                upgrade_err(
                    "parse_release",
                    format!("Failed to parse release information: {e}"),
                    None::<std::io::Error>,
                )
            })?;
            Ok(release)
        }
        Err(_) => {
            println!("No stable release found, checking for pre-releases...");
            get_most_recent_release()
        }
    }
}

/// Get the most recent release (including drafts and prereleases)
fn get_most_recent_release() -> CliResult<GitHubRelease> {
    let url = format!("https://api.github.com/repos/{REPO_OWNER}/{REPO_NAME}/releases");

    let mut response = ureq::get(&url)
        .header("User-Agent", &format!("andromeda-cli/{CURRENT_VERSION}"))
        .call()
        .map_err(|e| {
            upgrade_err(
                "fetch_releases",
                format!("Failed to fetch releases: {e}"),
                None::<std::io::Error>,
            )
        })?;

    let releases: Vec<GitHubRelease> = response.body_mut().read_json().map_err(|e| {
        upgrade_err(
            "parse_releases",
            format!("Failed to parse releases information: {e}"),
            None::<std::io::Error>,
        )
    })?;

    releases
        .into_iter()
        .find(|release| !release.assets.is_empty())
        .ok_or_else(|| {
            upgrade_err(
                "select_release",
                "No releases with assets found",
                None::<std::io::Error>,
            )
        })
}

/// Get a specific release by tag
fn get_release_by_tag(tag: &str) -> CliResult<GitHubRelease> {
    let url = format!("https://api.github.com/repos/{REPO_OWNER}/{REPO_NAME}/releases/tags/{tag}");

    let mut response = ureq::get(&url)
        .header("User-Agent", &format!("andromeda-cli/{CURRENT_VERSION}"))
        .call()
        .map_err(|e| {
            upgrade_err(
                "fetch_release",
                format!("Failed to fetch release information: {e}"),
                None::<std::io::Error>,
            )
        })?;

    let release: GitHubRelease = response.body_mut().read_json().map_err(|e| {
        upgrade_err(
            "parse_release",
            format!("Failed to parse release information: {e}"),
            None::<std::io::Error>,
        )
    })?;

    Ok(release)
}

/// Download an asset from the given URL
fn download_asset(url: &str) -> CliResult<Vec<u8>> {
    let mut response = ureq::get(url)
        .header("User-Agent", &format!("andromeda-cli/{CURRENT_VERSION}"))
        .call()
        .map_err(|e| {
            upgrade_err(
                "download",
                format!("Failed to download asset: {e}"),
                None::<std::io::Error>,
            )
        })?;

    let mut buffer = Vec::new();
    response
        .body_mut()
        .as_reader()
        .read_to_end(&mut buffer)
        .map_err(|e| upgrade_err("download_read", e.to_string(), Some(e)))?;

    Ok(buffer)
}

/// Install the new binary, replacing the current one
fn install_binary(binary_data: &[u8], _platform: &PlatformInfo) -> CliResult<()> {
    let current_exe = env::current_exe()
        .map_err(|e| upgrade_err("locate_exe", e.to_string(), Some(e)))?;

    let backup_path = current_exe.with_extension("bak");
    if backup_path.exists() {
        fs::remove_file(&backup_path)
            .map_err(|e| upgrade_err("remove_old_backup", e.to_string(), Some(e)))?;
    }
    fs::copy(&current_exe, &backup_path)
        .map_err(|e| upgrade_err("create_backup", e.to_string(), Some(e)))?;
    if cfg!(windows) {
        install_binary_windows(binary_data, &current_exe, &backup_path)
    } else {
        install_binary_unix(binary_data, &current_exe, &backup_path)
    }
}

fn install_binary_windows(
    binary_data: &[u8],
    current_exe: &Path,
    backup_path: &Path,
) -> CliResult<()> {
    let temp_dir = env::temp_dir();
    let temp_binary = temp_dir.join("andromeda_new.exe");
    fs::write(&temp_binary, binary_data)
        .map_err(|e| upgrade_err("write_temp_binary", e.to_string(), Some(e)))?;

    let batch_script = format!(
        r#"@echo off
timeout /t 2 /nobreak >nul
move "{}" "{}"
if errorlevel 1 (
    echo Failed to update binary, restoring backup...
    move "{}" "{}"
    exit /b 1
)
del "{}"
echo Upgrade completed successfully!
"#,
        temp_binary.display(),
        current_exe.display(),
        backup_path.display(),
        current_exe.display(),
        backup_path.display()
    );

    let batch_path = temp_dir.join("andromeda_upgrade.bat");
    fs::write(&batch_path, batch_script)
        .map_err(|e| upgrade_err("create_upgrade_script", e.to_string(), Some(e)))?;

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        use std::process::Command;

        Command::new("cmd")
            .args(["/C", &batch_path.to_string_lossy()])
            .creation_flags(0x08000000) // CREATE_NO_WINDOW
            .spawn()
            .map_err(|e| upgrade_err("start_upgrade_process", e.to_string(), Some(e)))?;

        println!("The upgrade will complete after this process exits.");
        Ok(())
    }

    #[cfg(not(windows))]
    {
        let _ = (binary_data, current_exe, backup_path);
        Err(CliError::unsupported_platform(format!(
            "Windows-specific function called on {}",
            std::env::consts::OS
        )))
    }
}

fn install_binary_unix(
    binary_data: &[u8],
    current_exe: &Path,
    backup_path: &Path,
) -> CliResult<()> {
    fs::write(current_exe, binary_data)
        .map_err(|e| upgrade_err("write_binary", e.to_string(), Some(e)))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(current_exe)
            .map_err(|e| upgrade_err("read_perms", e.to_string(), Some(e)))?
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(current_exe, perms)
            .map_err(|e| upgrade_err("set_perms", e.to_string(), Some(e)))?;
    }

    fs::remove_file(backup_path).ok();

    Ok(())
}

/// Read extension trait for std::io::Read
trait ReadExt: std::io::Read {
    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> std::io::Result<usize> {
        std::io::Read::read_to_end(self, buf)
    }
}

impl<R: std::io::Read> ReadExt for R {}
