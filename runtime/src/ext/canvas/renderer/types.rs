// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::FillStyle;
use std::str::FromStr;

#[derive(Clone, Debug)]
pub struct Dimensions {
    pub height: u32,
    pub width: u32,
}

#[derive(Clone, Debug)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

pub type Path = Vec<Point>;

pub struct Rect {
    pub start: Point,
    pub end: Point,
}

#[allow(dead_code)]
pub type Color = [f32; 4];

/// Line cap style for stroke operations
#[derive(Copy, Clone, Debug, PartialEq, Default)]
pub enum LineCap {
    #[default]
    Butt,
    Round,
    Square,
}

/// Line join style for stroke operations
#[derive(Copy, Clone, Debug, PartialEq, Default)]
pub enum LineJoin {
    Bevel,
    Round,
    #[default]
    Miter,
}

/// Canvas composite operations (blend modes)
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum CompositeOperation {
    #[default]
    SourceOver = 0,
    SourceIn = 1,
    SourceOut = 2,
    SourceAtop = 3,
    DestinationOver = 4,
    DestinationIn = 5,
    DestinationOut = 6,
    DestinationAtop = 7,
    Lighter = 8,
    Copy = 9,
    Xor = 10,
    Multiply = 11,
    Screen = 12,
    Overlay = 13,
    Darken = 14,
    Lighten = 15,
    ColorDodge = 16,
    ColorBurn = 17,
    HardLight = 18,
    SoftLight = 19,
    Difference = 20,
    Exclusion = 21,
    Hue = 22,
    Saturation = 23,
    Color = 24,
    Luminosity = 25,
}

impl FromStr for CompositeOperation {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "source-over" => Ok(Self::SourceOver),
            "source-in" => Ok(Self::SourceIn),
            "source-out" => Ok(Self::SourceOut),
            "source-atop" => Ok(Self::SourceAtop),
            "destination-over" => Ok(Self::DestinationOver),
            "destination-in" => Ok(Self::DestinationIn),
            "destination-out" => Ok(Self::DestinationOut),
            "destination-atop" => Ok(Self::DestinationAtop),
            "lighter" => Ok(Self::Lighter),
            "copy" => Ok(Self::Copy),
            "xor" => Ok(Self::Xor),
            "multiply" => Ok(Self::Multiply),
            "screen" => Ok(Self::Screen),
            "overlay" => Ok(Self::Overlay),
            "darken" => Ok(Self::Darken),
            "lighten" => Ok(Self::Lighten),
            "color-dodge" => Ok(Self::ColorDodge),
            "color-burn" => Ok(Self::ColorBurn),
            "hard-light" => Ok(Self::HardLight),
            "soft-light" => Ok(Self::SoftLight),
            "difference" => Ok(Self::Difference),
            "exclusion" => Ok(Self::Exclusion),
            "hue" => Ok(Self::Hue),
            "saturation" => Ok(Self::Saturation),
            "color" => Ok(Self::Color),
            "luminosity" => Ok(Self::Luminosity),
            _ => Err(()),
        }
    }
}

impl CompositeOperation {
    /// Convert to string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::SourceOver => "source-over",
            Self::SourceIn => "source-in",
            Self::SourceOut => "source-out",
            Self::SourceAtop => "source-atop",
            Self::DestinationOver => "destination-over",
            Self::DestinationIn => "destination-in",
            Self::DestinationOut => "destination-out",
            Self::DestinationAtop => "destination-atop",
            Self::Lighter => "lighter",
            Self::Copy => "copy",
            Self::Xor => "xor",
            Self::Multiply => "multiply",
            Self::Screen => "screen",
            Self::Overlay => "overlay",
            Self::Darken => "darken",
            Self::Lighten => "lighten",
            Self::ColorDodge => "color-dodge",
            Self::ColorBurn => "color-burn",
            Self::HardLight => "hard-light",
            Self::SoftLight => "soft-light",
            Self::Difference => "difference",
            Self::Exclusion => "exclusion",
            Self::Hue => "hue",
            Self::Saturation => "saturation",
            Self::Color => "color",
            Self::Luminosity => "luminosity",
        }
    }
}

pub struct RenderState {
    pub fill_style: FillStyle,
    pub global_alpha: f32,
    pub transform: [f64; 6],
    pub line_cap: LineCap,
    pub line_join: LineJoin,
    pub miter_limit: f64,
    pub composite_operation: CompositeOperation,
    // Shadow properties
    pub shadow_blur: f64,
    pub shadow_color: FillStyle,
    pub shadow_offset_x: f64,
    pub shadow_offset_y: f64,
}

impl Default for RenderState {
    fn default() -> Self {
        Self {
            fill_style: FillStyle::default(),
            global_alpha: 1.0,
            transform: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
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
        }
    }
}

/// Helper function to multiply two 2D transformation matrices
/// Matrix format: [a, b, c, d, e, f] represents:
/// | a c e |
/// | b d f |
/// | 0 0 1 |
pub fn multiply_transform(m1: &[f64; 6], m2: &[f64; 6]) -> [f64; 6] {
    [
        m1[0] * m2[0] + m1[2] * m2[1],
        m1[1] * m2[0] + m1[3] * m2[1],
        m1[0] * m2[2] + m1[2] * m2[3],
        m1[1] * m2[2] + m1[3] * m2[3],
        m1[0] * m2[4] + m1[2] * m2[5] + m1[4],
        m1[1] * m2[4] + m1[3] * m2[5] + m1[5],
    ]
}

/// Convert 2D transformation matrix to 3x3 matrix for GPU (as 12 floats for alignment)
pub fn transform_to_mat3(transform: &[f64; 6]) -> [f32; 12] {
    [
        transform[0] as f32,
        transform[1] as f32,
        0.0,
        0.0, // Column 0
        transform[2] as f32,
        transform[3] as f32,
        0.0,
        0.0, // Column 1
        transform[4] as f32,
        transform[5] as f32,
        1.0,
        0.0, // Column 2
    ]
}

/// Apply a 2D transformation matrix to a point
/// Matrix format: [a, b, c, d, e, f] represents:
/// | a c e |
/// | b d f |
/// | 0 0 1 |
pub fn transform_point(point: &Point, transform: &[f64; 6]) -> Point {
    Point {
        x: transform[0] * point.x + transform[2] * point.y + transform[4],
        y: transform[1] * point.x + transform[3] * point.y + transform[5],
    }
}
