// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at https://mozilla.org/MPL/2.0/.

use andromeda::{CliError, CliResult};
use clap::{Args, Parser as ClapParser};
use serde::Deserialize;
use std::env;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, ClapParser)]
#[command(name = "andromeda-installer")]
#[command(about = "Andromeda Installation Tool", long_about = None)]
struct Cli {
    #[command(flatten)]
    install_args: InstallArgs,
}

#[derive(Debug, Args)]
struct InstallArgs {
    /// Installation directory (default: %USERPROFILE%\.local\bin)
    #[arg(short = 'd', long)]
    install_dir: Option<PathBuf>,

    /// Specific version to install (default: latest)
    #[arg(short = 'v', long)]
    version: Option<String>,

    /// Force installation even if already installed
    #[arg(short = 'f', long)]
    force: bool,

    /// Show verbose output
    #[arg(long)]
    verbose: bool,

    /// Skip PATH configuration
    #[arg(long)]
    skip_path: bool,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct GitHubRelease {
    tag_name: String,
    name: String,
    assets: Vec<GitHubAsset>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
}

const REPO_OWNER: &str = "tryandromeda";
const REPO_NAME: &str = "andromeda";

fn main() -> CliResult<()> {
    let cli = Cli::parse();
    install_andromeda(cli.install_args)
}

fn install_andromeda(args: InstallArgs) -> CliResult<()> {
    print_header();

    // Determine installation directory
    let install_dir = args.install_dir.unwrap_or_else(|| {
        let user_profile = env::var("USERPROFILE")
            .unwrap_or_else(|_| env::var("HOME").unwrap_or_else(|_| ".".to_string()));
        PathBuf::from(user_profile).join(".local").join("bin")
    });

    if args.verbose {
        print_info(&format!(
            "Installation directory: {}",
            install_dir.display()
        ));
    }

    // Create installation directory
    if !install_dir.exists() {
        print_info("Creating installation directory...");
        fs::create_dir_all(&install_dir).map_err(CliError::Io)?;
    }

    // Check if already installed
    let binary_path = install_dir.join("andromeda.exe");
    if binary_path.exists() && !args.force {
        print_warning("Andromeda is already installed. Use --force to reinstall.");

        // Check current version
        if let Ok(output) = Command::new(&binary_path).arg("--version").output()
            && let Ok(version) = String::from_utf8(output.stdout)
        {
            print_info(&format!("Current version: {}", version.trim()));
        }

        return Ok(());
    }

    // Get version to install
    let version = if let Some(v) = args.version {
        v
    } else {
        print_info("Fetching latest release information...");
        get_latest_version(args.verbose)?
    };

    print_info(&format!("Installing Andromeda {version}..."));

    // Detect platform
    let asset_name = detect_platform()?;
    if args.verbose {
        print_info(&format!("Detected platform asset: {asset_name}"));
    }

    // Download binary
    let download_url = format!(
        "https://github.com/{REPO_OWNER}/{REPO_NAME}/releases/download/{version}/{asset_name}"
    );

    if args.verbose {
        print_info(&format!("Download URL: {download_url}"));
    }

    print_info("Downloading Andromeda binary...");
    let binary_data = download_file(&download_url, args.verbose)?;

    // Install binary
    print_info("Installing binary...");
    fs::write(&binary_path, binary_data).map_err(CliError::Io)?;

    print_success(&format!(
        "Andromeda installed to: {}",
        binary_path.display()
    ));

    if !args.skip_path {
        configure_path(&install_dir, args.verbose)?;
    }

    // Verify installation
    print_info("Verifying installation...");
    match Command::new(&binary_path).arg("--version").output() {
        Ok(output) => {
            if output.status.success() {
                let version_output = String::from_utf8_lossy(&output.stdout);
                print_success("Installation verified successfully!");
                print_info(&format!("Installed version: {}", version_output.trim()));
            } else {
                print_warning("Installation completed but verification failed.");
            }
        }
        Err(_) => {
            print_warning("Installation completed but verification failed.");
        }
    }

    println!();
    print_success("Andromeda has been installed successfully!");
    print_info("Try running: andromeda --help");

    if !args.skip_path {
        print_info(
            "You may need to restart your terminal or run 'refreshenv' for PATH changes to take effect.",
        );
    }

    Ok(())
}

fn get_latest_version(verbose: bool) -> CliResult<String> {
    let api_url = format!("https://api.github.com/repos/{REPO_OWNER}/{REPO_NAME}/releases/latest");

    if verbose {
        print_info(&format!("Fetching from: {api_url}"));
    }

    let mut response = ureq::get(&api_url)
        .header(
            "User-Agent",
            &format!("andromeda-installer/{}", env!("CARGO_PKG_VERSION")),
        )
        .call()
        .map_err(|e| CliError::TestExecution(format!("Failed to fetch release info: {e}")))?;

    let release: GitHubRelease = response
        .body_mut()
        .read_json()
        .map_err(|e| CliError::TestExecution(format!("Failed to parse release info: {e}")))?;

    Ok(release.tag_name)
}

fn detect_platform() -> CliResult<String> {
    let os = env::consts::OS;
    let arch = env::consts::ARCH;

    let asset_name = match (os, arch) {
        ("windows", "x86_64") => "andromeda-windows-amd64.exe",
        ("windows", "aarch64") => "andromeda-windows-arm64.exe",
        ("linux", "x86_64") => "andromeda-linux-amd64",
        ("linux", "aarch64") => "andromeda-linux-arm64",
        ("macos", "x86_64") => "andromeda-macos-amd64",
        ("macos", "aarch64") => "andromeda-macos-arm64",
        _ => {
            return Err(CliError::Config(format!(
                "Unsupported platform: {os} {arch}"
            )));
        }
    };

    Ok(asset_name.to_string())
}

fn download_file(url: &str, verbose: bool) -> CliResult<Vec<u8>> {
    if verbose {
        print_info(&format!("Downloading from: {url}"));
    }

    let mut response = ureq::get(url)
        .header(
            "User-Agent",
            &format!("andromeda-installer/{}", env!("CARGO_PKG_VERSION")),
        )
        .call()
        .map_err(|e| CliError::TestExecution(format!("Failed to download file: {e}")))?;

    let mut bytes = Vec::new();
    response
        .body_mut()
        .as_reader()
        .read_to_end(&mut bytes)
        .map_err(CliError::Io)?;

    if verbose {
        print_info(&format!("Downloaded {} bytes", bytes.len()));
    }

    Ok(bytes)
}

fn configure_path(install_dir: &Path, verbose: bool) -> CliResult<()> {
    let install_dir_str = install_dir.to_string_lossy();

    // Check if already in PATH
    if let Ok(current_path) = env::var("PATH")
        && current_path.contains(&*install_dir_str)
    {
        if verbose {
            print_info("Installation directory already in PATH");
        }
        return Ok(());
    }

    print_info("Configuring PATH environment variable...");

    let output = Command::new("powershell")
        .args([
            "-Command",
            &format!(
                "$oldPath = [Environment]::GetEnvironmentVariable('PATH', 'User'); \
                 if ($oldPath -notlike '*{install_dir_str}*') {{ \
                     $newPath = $oldPath + ';{install_dir_str}'; \
                     [Environment]::SetEnvironmentVariable('PATH', $newPath, 'User'); \
                     Write-Host 'PATH updated successfully' \
                 }} else {{ \
                     Write-Host 'PATH already contains installation directory' \
                 }}"
            ),
        ])
        .output();

    match output {
        Ok(result) => {
            if result.status.success() {
                print_success("PATH environment variable updated!");
                print_info("Changes will take effect in new terminal sessions.");
            } else {
                print_warning("Failed to update PATH automatically.");
                print_manual_path_instructions(install_dir);
            }
        }
        Err(_) => {
            print_warning("Failed to update PATH automatically.");
            print_manual_path_instructions(install_dir);
        }
    }

    Ok(())
}

fn print_manual_path_instructions(install_dir: &Path) {
    println!();
    print_warning("Please manually add the installation directory to your PATH:");
    print_info("1. Open System Properties > Advanced > Environment Variables");
    print_info("2. Under User Variables, find and edit 'PATH'");
    print_info(&format!("3. Add this directory: {}", install_dir.display()));
    print_info("4. Click OK and restart your terminal");
    println!();
    print_info("Alternatively, run this PowerShell command as Administrator:");
    println!(
        "   [Environment]::SetEnvironmentVariable('PATH', $env:PATH + ';{}', 'User')",
        install_dir.display()
    );
}

// Utility functions for colored output
fn print_header() {
    println!("🚀 Andromeda Installation Tool");
    println!("════════════════════════════════");
    println!();
}

fn print_info(message: &str) {
    println!("ℹ️  {message}");
}

fn print_success(message: &str) {
    println!("✅ {message}");
}

fn print_warning(message: &str) {
    println!("⚠️  {message}");
}

#[allow(dead_code)]
fn print_error(message: &str) {
    eprintln!("❌ {message}");
}
