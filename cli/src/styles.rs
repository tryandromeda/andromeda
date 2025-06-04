use clap::builder::{
    Styles,
    styling::{AnsiColor, Effects},
};
use cliclack::{Theme, ThemeState};
use console::Style;

pub const ANDROMEDA_STYLING: Styles = Styles::styled()
    .header(AnsiColor::BrightMagenta.on_default().effects(Effects::BOLD))
    .usage(AnsiColor::BrightMagenta.on_default().effects(Effects::BOLD))
    .literal(AnsiColor::Yellow.on_default().effects(Effects::BOLD))
    .placeholder(AnsiColor::BrightYellow.on_default())
    .error(AnsiColor::BrightRed.on_default().effects(Effects::BOLD))
    .valid(AnsiColor::Yellow.on_default().effects(Effects::BOLD))
    .invalid(AnsiColor::BrightYellow.on_default().effects(Effects::BOLD));

pub struct DefaultTheme;

impl Theme for DefaultTheme {
    fn bar_color(&self, _: &ThemeState) -> Style {
        Style::new().dim().bold()
    }

    fn state_symbol_color(&self, _: &ThemeState) -> Style {
        Style::new().cyan()
    }

    fn input_style(&self, _: &ThemeState) -> Style {
        Style::new().yellow()
    }

    fn format_intro(&self, title: &str) -> String {
        let color = self.bar_color(&ThemeState::Submit);
        format!(
            "{start_bar}  {title} {exit_instructions}\n{bar}\n",
            start_bar = color.apply_to("⚙"),
            bar = color.apply_to("|"),
            title = Style::new().bold().apply_to(title),
            exit_instructions = color.apply_to("(type exit or Ctrl+C to exit)"),
        )
    }
}
