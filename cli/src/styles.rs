use clap::builder::{
    Styles,
    styling::{AnsiColor, Effects},
};

pub const ANDROMEDA_STYLING: Styles = Styles::styled()
    .header(AnsiColor::BrightMagenta.on_default().effects(Effects::BOLD))
    .usage(AnsiColor::BrightMagenta.on_default().effects(Effects::BOLD))
    .literal(AnsiColor::Yellow.on_default().effects(Effects::BOLD))
    .placeholder(AnsiColor::BrightYellow.on_default())
    .error(AnsiColor::BrightRed.on_default().effects(Effects::BOLD))
    .valid(AnsiColor::Yellow.on_default().effects(Effects::BOLD))
    .invalid(AnsiColor::BrightYellow.on_default().effects(Effects::BOLD));
