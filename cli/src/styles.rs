// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use clap::builder::{
    Styles,
    styling::{AnsiColor, Effects},
};
use console::Style;

pub const ANDROMEDA_STYLING: Styles = Styles::styled()
    .header(AnsiColor::BrightCyan.on_default().effects(Effects::BOLD))
    .usage(AnsiColor::BrightCyan.on_default().effects(Effects::BOLD))
    .literal(AnsiColor::Yellow.on_default().effects(Effects::BOLD))
    .placeholder(AnsiColor::BrightBlue.on_default())
    .error(AnsiColor::BrightRed.on_default().effects(Effects::BOLD))
    .valid(AnsiColor::Green.on_default().effects(Effects::BOLD))
    .invalid(AnsiColor::BrightRed.on_default().effects(Effects::BOLD));

pub fn format_js_value(value: &str) -> (String, String) {
    let trimmed = value.trim();

    if trimmed == "undefined" {
        (
            Style::new().dim().apply_to(trimmed).to_string(),
            "undefined".to_string(),
        )
    } else if trimmed == "null" {
        (
            Style::new().red().apply_to(trimmed).to_string(),
            "null".to_string(),
        )
    } else if trimmed.starts_with('"') && trimmed.ends_with('"') {
        (
            Style::new().green().apply_to(trimmed).to_string(),
            "string".to_string(),
        )
    } else if trimmed.parse::<f64>().is_ok() {
        (
            Style::new().blue().apply_to(trimmed).to_string(),
            "number".to_string(),
        )
    } else if trimmed == "true" || trimmed == "false" {
        (
            Style::new().yellow().apply_to(trimmed).to_string(),
            "boolean".to_string(),
        )
    } else if trimmed.starts_with('[') && trimmed.ends_with(']') {
        (
            Style::new().magenta().apply_to(trimmed).to_string(),
            "array".to_string(),
        )
    } else if trimmed.starts_with('{') && trimmed.ends_with('}') {
        (
            Style::new().cyan().apply_to(trimmed).to_string(),
            "object".to_string(),
        )
    } else if trimmed.contains("function") || trimmed.starts_with("function") {
        (
            Style::new().bright().blue().apply_to(trimmed).to_string(),
            "function".to_string(),
        )
    } else {
        (
            Style::new().white().apply_to(trimmed).to_string(),
            "unknown".to_string(),
        )
    }
}
