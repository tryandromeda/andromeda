// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::FillStyle;
use crate::ext::canvas::renderer::{CompositeOperation, LineCap, LineJoin};

/// Text alignment values for Canvas2D
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum TextAlign {
    #[default]
    Start,
    End,
    Left,
    Right,
    Center,
}

/// Text baseline values for Canvas2D
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum TextBaseline {
    Top,
    Hanging,
    Middle,
    #[default]
    Alphabetic,
    Ideographic,
    Bottom,
}

/// Text direction values for Canvas2D
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Ltr,
    Rtl,
    #[default]
    Inherit,
}

/// fontKerning values for Canvas2D (auto/normal/none).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum FontKerning {
    #[default]
    Auto,
    Normal,
    None,
}

impl FontKerning {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "auto" => Some(Self::Auto),
            "normal" => Some(Self::Normal),
            "none" => Some(Self::None),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Normal => "normal",
            Self::None => "none",
        }
    }
}

/// fontStretch values for Canvas2D (the nine CSS keywords).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum FontStretch {
    UltraCondensed,
    ExtraCondensed,
    Condensed,
    SemiCondensed,
    #[default]
    Normal,
    SemiExpanded,
    Expanded,
    ExtraExpanded,
    UltraExpanded,
}

impl FontStretch {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "ultra-condensed" => Some(Self::UltraCondensed),
            "extra-condensed" => Some(Self::ExtraCondensed),
            "condensed" => Some(Self::Condensed),
            "semi-condensed" => Some(Self::SemiCondensed),
            "normal" => Some(Self::Normal),
            "semi-expanded" => Some(Self::SemiExpanded),
            "expanded" => Some(Self::Expanded),
            "extra-expanded" => Some(Self::ExtraExpanded),
            "ultra-expanded" => Some(Self::UltraExpanded),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::UltraCondensed => "ultra-condensed",
            Self::ExtraCondensed => "extra-condensed",
            Self::Condensed => "condensed",
            Self::SemiCondensed => "semi-condensed",
            Self::Normal => "normal",
            Self::SemiExpanded => "semi-expanded",
            Self::Expanded => "expanded",
            Self::ExtraExpanded => "extra-expanded",
            Self::UltraExpanded => "ultra-expanded",
        }
    }

    pub fn to_cosmic(self) -> cosmic_text::Stretch {
        match self {
            Self::UltraCondensed => cosmic_text::Stretch::UltraCondensed,
            Self::ExtraCondensed => cosmic_text::Stretch::ExtraCondensed,
            Self::Condensed => cosmic_text::Stretch::Condensed,
            Self::SemiCondensed => cosmic_text::Stretch::SemiCondensed,
            Self::Normal => cosmic_text::Stretch::Normal,
            Self::SemiExpanded => cosmic_text::Stretch::SemiExpanded,
            Self::Expanded => cosmic_text::Stretch::Expanded,
            Self::ExtraExpanded => cosmic_text::Stretch::ExtraExpanded,
            Self::UltraExpanded => cosmic_text::Stretch::UltraExpanded,
        }
    }
}

/// fontVariantCaps values for Canvas2D.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum FontVariantCaps {
    #[default]
    Normal,
    SmallCaps,
    AllSmallCaps,
    PetiteCaps,
    AllPetiteCaps,
    Unicase,
    TitlingCaps,
}

impl FontVariantCaps {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "normal" => Some(Self::Normal),
            "small-caps" => Some(Self::SmallCaps),
            "all-small-caps" => Some(Self::AllSmallCaps),
            "petite-caps" => Some(Self::PetiteCaps),
            "all-petite-caps" => Some(Self::AllPetiteCaps),
            "unicase" => Some(Self::Unicase),
            "titling-caps" => Some(Self::TitlingCaps),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::SmallCaps => "small-caps",
            Self::AllSmallCaps => "all-small-caps",
            Self::PetiteCaps => "petite-caps",
            Self::AllPetiteCaps => "all-petite-caps",
            Self::Unicase => "unicase",
            Self::TitlingCaps => "titling-caps",
        }
    }
}

/// textRendering values for Canvas2D.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum TextRendering {
    #[default]
    Auto,
    OptimizeSpeed,
    OptimizeLegibility,
    GeometricPrecision,
}

impl TextRendering {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "auto" => Some(Self::Auto),
            "optimizeSpeed" => Some(Self::OptimizeSpeed),
            "optimizeLegibility" => Some(Self::OptimizeLegibility),
            "geometricPrecision" => Some(Self::GeometricPrecision),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::OptimizeSpeed => "optimizeSpeed",
            Self::OptimizeLegibility => "optimizeLegibility",
            Self::GeometricPrecision => "geometricPrecision",
        }
    }
}

/// Returns true if `s` is a valid CSS `<length>` for the canvas spacing
/// properties.
pub fn is_valid_spacing_length(s: &str) -> bool {
    let s = s.trim();
    if s.is_empty() {
        return false;
    }
    let (num, unit) = if let Some(stripped) = s.strip_suffix("px") {
        (stripped, "px")
    } else if let Some(stripped) = s.strip_suffix("em") {
        (stripped, "em")
    } else {
        return false;
    };
    let _ = unit;
    num.trim()
        .parse::<f64>()
        .map(|n| n.is_finite())
        .unwrap_or(false)
}

/// Resolve a spacing length string to pixels. Returns 0.0 for invalid input.
/// `font_size_px` is used to resolve `em` units.
pub fn resolve_spacing_length(s: &str, font_size_px: f64) -> f64 {
    let s = s.trim();
    if let Some(num) = s.strip_suffix("px") {
        num.trim().parse::<f64>().unwrap_or(0.0)
    } else if let Some(num) = s.strip_suffix("em") {
        num.trim()
            .parse::<f64>()
            .map(|n| n * font_size_px)
            .unwrap_or(0.0)
    } else {
        0.0
    }
}

/// Canvas drawing state that can be saved and restored
#[derive(Clone)]
pub struct CanvasState {
    pub fill_style: FillStyle,
    pub stroke_style: FillStyle,
    pub line_width: f64,
    pub global_alpha: f32,
    pub transform: [f64; 6],
    pub line_dash: Vec<f64>,
    pub line_dash_offset: f64,
    pub line_cap: LineCap,
    pub line_join: LineJoin,
    pub miter_limit: f64,
    pub composite_operation: CompositeOperation,
    // Shadow properties
    pub shadow_blur: f64,
    pub shadow_color: FillStyle,
    pub shadow_offset_x: f64,
    pub shadow_offset_y: f64,
    // Text properties
    pub font: String,
    pub text_align: TextAlign,
    pub text_baseline: TextBaseline,
    pub direction: Direction,
    pub letter_spacing: String,
    pub word_spacing: String,
    pub font_kerning: FontKerning,
    pub font_stretch: FontStretch,
    pub font_variant_caps: FontVariantCaps,
    pub text_rendering: TextRendering,
    pub lang: String,
}

impl Default for CanvasState {
    fn default() -> Self {
        Self::new()
    }
}

impl CanvasState {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            fill_style: FillStyle::Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            },
            stroke_style: FillStyle::Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            },
            line_width: 1.0,
            global_alpha: 1.0,
            transform: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
            line_dash: Vec::new(),
            line_dash_offset: 0.0,
            line_cap: LineCap::default(),
            line_join: LineJoin::default(),
            miter_limit: 10.0,
            composite_operation: CompositeOperation::default(),
            shadow_blur: 0.0,
            shadow_color: FillStyle::Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.0,
            },
            shadow_offset_x: 0.0,
            shadow_offset_y: 0.0,
            // Text properties - Canvas2D defaults
            font: "10px sans-serif".to_string(),
            text_align: TextAlign::default(),
            text_baseline: TextBaseline::default(),
            direction: Direction::default(),
            letter_spacing: "0px".to_string(),
            word_spacing: "0px".to_string(),
            font_kerning: FontKerning::default(),
            font_stretch: FontStretch::default(),
            font_variant_caps: FontVariantCaps::default(),
            text_rendering: TextRendering::default(),
            lang: "inherit".to_string(),
        }
    }
}
