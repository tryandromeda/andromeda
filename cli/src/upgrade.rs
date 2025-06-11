// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use anyhow::{Context, Result, anyhow};
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

/// Run the upgrade process
pub fn run_upgrade(force: bool, target_version: Option<String>, dry_run: bool) -> Result<()> {
    println!("🚀 Andromeda Upgrade Tool");
    println!("Current version: {}", CURRENT_VERSION);
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
        println!("✅ You are already running the latest version!");
        return Ok(());
    }

    if release.tag_name == CURRENT_VERSION && !force {
        println!(
            "ℹ️  You are already on version {}. Use --force to reinstall.",
            CURRENT_VERSION
        );
        return Ok(());
    }

    let asset = release
        .assets
        .iter()
        .find(|asset| asset.name == platform.asset_name)
        .ok_or_else(|| {
            anyhow!(
                "No release asset found for platform: {}",
                platform.asset_name
            )
        })?;

    println!("Found asset: {} ({} bytes)", asset.name, asset.size);

    if dry_run {
        println!(
            "🔍 Dry run mode - would upgrade from {} to {}",
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
        std::io::stdout().flush()?;

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let input = input.trim().to_lowercase();

        if input != "y" && input != "yes" {
            println!("Upgrade cancelled.");
            return Ok(());
        }
    }

    println!("⬇️  Downloading {}...", asset.name);
    let new_binary = download_asset(&asset.browser_download_url)?;

    println!("🔄 Installing new version...");
    install_binary(&new_binary, &platform)?;

    println!("✅ Successfully upgraded to version {}!", release.tag_name);
    println!("Run 'andromeda --version' to verify the new version.");

    Ok(())
}

/// Detect the current platform and return appropriate asset information
fn detect_platform() -> Result<PlatformInfo> {
    let os = if cfg!(target_os = "linux") {
        "linux"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(target_os = "windows") {
        "windows"
    } else {
        return Err(anyhow!("Unsupported operating system"));
    };

    let arch = if cfg!(target_arch = "x86_64") {
        "amd64"
    } else if cfg!(target_arch = "aarch64") && cfg!(target_os = "macos") {
        "arm64"
    } else {
        return Err(anyhow!("Unsupported architecture"));
    };

    let asset_name = if os == "windows" {
        format!("andromeda-{}-{}.exe", os, arch)
    } else {
        format!("andromeda-{}-{}", os, arch)
    };

    Ok(PlatformInfo {
        os: os.to_string(),
        arch: arch.to_string(),
        asset_name,
    })
}

/// Get the latest release from GitHub
fn get_latest_release() -> Result<GitHubRelease> {
    let url = format!(
        "https://api.github.com/repos/{}/{}/releases/latest",
        REPO_OWNER, REPO_NAME
    );

    let response = ureq::get(&url)
        .set("User-Agent", &format!("andromeda-cli/{}", CURRENT_VERSION))
        .call();

    match response {
        Ok(response) => {
            let release: GitHubRelease = response
                .into_json()
                .context("Failed to parse release information")?;
            Ok(release)
        }
        Err(_) => {
            println!("ℹ️  No stable release found, checking for pre-releases...");
            get_most_recent_release()
        }
    }
}

/// Get the most recent release (including drafts and prereleases)
fn get_most_recent_release() -> Result<GitHubRelease> {
    let url = format!(
        "https://api.github.com/repos/{}/{}/releases",
        REPO_OWNER, REPO_NAME
    );

    let response = ureq::get(&url)
        .set("User-Agent", &format!("andromeda-cli/{}", CURRENT_VERSION))
        .call()
        .context("Failed to fetch releases")?;

    let releases: Vec<GitHubRelease> = response
        .into_json()
        .context("Failed to parse releases information")?;

    releases
        .into_iter()
        .find(|release| !release.assets.is_empty())
        .ok_or_else(|| anyhow!("No releases with assets found"))
}

/// Get a specific release by tag
fn get_release_by_tag(tag: &str) -> Result<GitHubRelease> {
    let url = format!(
        "https://api.github.com/repos/{}/{}/releases/tags/{}",
        REPO_OWNER, REPO_NAME, tag
    );

    let response = ureq::get(&url)
        .set("User-Agent", &format!("andromeda-cli/{}", CURRENT_VERSION))
        .call()
        .context("Failed to fetch release information")?;

    let release: GitHubRelease = response
        .into_json()
        .context("Failed to parse release information")?;

    Ok(release)
}

/// Download an asset from the given URL
fn download_asset(url: &str) -> Result<Vec<u8>> {
    let response = ureq::get(url)
        .set("User-Agent", &format!("andromeda-cli/{}", CURRENT_VERSION))
        .call()
        .context("Failed to download asset")?;

    let mut buffer = Vec::new();
    response
        .into_reader()
        .read_to_end(&mut buffer)
        .context("Failed to read downloaded asset")?;

    Ok(buffer)
}

/// Install the new binary, replacing the current one
fn install_binary(binary_data: &[u8], _platform: &PlatformInfo) -> Result<()> {
    let current_exe = env::current_exe().context("Failed to get current executable path")?;

    let backup_path = current_exe.with_extension("bak");
    if backup_path.exists() {
        fs::remove_file(&backup_path)?;
    }
    fs::copy(&current_exe, &backup_path).context("Failed to create backup of current binary")?;
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
) -> Result<()> {
    let temp_dir = env::temp_dir();
    let temp_binary = temp_dir.join("andromeda_new.exe");
    fs::write(&temp_binary, binary_data)
        .context("Failed to write new binary to temporary location")?;

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
    fs::write(&batch_path, batch_script).context("Failed to create upgrade script")?;

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        use std::process::Command;

        Command::new("cmd")
            .args(["/C", &batch_path.to_string_lossy()])
            .creation_flags(0x08000000) // CREATE_NO_WINDOW
            .spawn()
            .context("Failed to start upgrade process")?;

        println!("⚠️  The upgrade will complete after this process exits.");
    }

    #[cfg(not(windows))]
    {
        return Err(anyhow!(
            "Windows-specific function called on non-Windows platform"
        ));
    }

    Ok(())
}

fn install_binary_unix(binary_data: &[u8], current_exe: &Path, backup_path: &Path) -> Result<()> {
    fs::write(current_exe, binary_data).context("Failed to write new binary")?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(current_exe)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(current_exe, perms)?;
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
