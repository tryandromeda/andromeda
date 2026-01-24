// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at https://mozilla.org/MPL/2.0/.

#![allow(clippy::result_large_err)]

/// Andromeda Satellite - Format
///
/// A minimal executable focused solely on formatting JavaScript/TypeScript files.
/// Designed for container instances where only formatting capability is needed.
use andromeda::{CliError, CliResult};
use clap::Parser as ClapParser;
use console::Style;
use std::path::PathBuf;

#[derive(Debug, ClapParser)]
#[command(name = "andromeda-fmt")]
#[command(about = "Andromeda Satellite - Format JavaScript/TypeScript files")]
#[command(version = env!("CARGO_PKG_VERSION"))]
struct Cli {
    /// The file(s) or directory(ies) to format
    #[arg(required = false)]
    paths: Vec<PathBuf>,
}

fn main() -> CliResult<()> {
    andromeda::error::init_error_reporting();

    let cli = Cli::parse();

    let config = andromeda::config::ConfigManager::load_or_default(None);

    let files_to_format =
        andromeda::helper::find_formattable_files_for_format(&cli.paths, &config.format)
            .map_err(|e| CliError::runtime_error_simple(format!("{e}")))?;

    if files_to_format.is_empty() {
        let warning = Style::new().yellow().bold().apply_to("⚠️");
        let msg = Style::new()
            .yellow()
            .apply_to("No formattable files found.");
        println!("{warning} {msg}");
        return Ok(());
    }

    let count = Style::new().cyan().apply_to(files_to_format.len());
    println!("Found {count} file(s) to format");
    println!("{}", Style::new().dim().apply_to("─".repeat(40)));

    let mut already_formatted_count = 0;
    let mut formatted_count = 0;

    for path in &files_to_format {
        match andromeda::format::format_file(path)
            .map_err(|e| CliError::runtime_error_simple(format!("{e}")))?
        {
            andromeda::format::FormatResult::Changed => formatted_count += 1,
            andromeda::format::FormatResult::AlreadyFormatted => already_formatted_count += 1,
        }
    }

    println!();
    let success = Style::new().green().bold().apply_to("✅");
    let complete_msg = Style::new().green().bold().apply_to("Formatting complete");
    println!("{success} {complete_msg}:");

    if formatted_count > 0 {
        let formatted_icon = Style::new().green().apply_to("📄");
        let formatted_num = Style::new().green().bold().apply_to(formatted_count);
        let formatted_text = if formatted_count == 1 {
            "file"
        } else {
            "files"
        };
        println!("   {formatted_icon} {formatted_num} {formatted_text} formatted");
    }

    if already_formatted_count > 0 {
        let already_icon = Style::new().cyan().apply_to("✨");
        let already_num = Style::new().cyan().bold().apply_to(already_formatted_count);
        let already_text = if already_formatted_count == 1 {
            "file"
        } else {
            "files"
        };
        let already_msg = Style::new().cyan().apply_to("already formatted");
        println!("   {already_icon} {already_num} {already_text} {already_msg}");
    }

    Ok(())
}
