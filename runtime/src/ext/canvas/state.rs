// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::FillStyle;
use crate::ext::canvas::renderer::{CompositeOperation, LineCap, LineJoin};

/// Text alignment values for Canvas2D
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextAlign {
    Start,
    End,
    Left,
    Right,
    Center,
}

impl Default for TextAlign {
    fn default() -> Self {
        Self::Start
    }
}

/// Text baseline values for Canvas2D
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextBaseline {
    Top,
    Hanging,
    Middle,
    Alphabetic,
    Ideographic,
    Bottom,
}

impl Default for TextBaseline {
    fn default() -> Self {
        Self::Alphabetic
    }
}

/// Text direction values for Canvas2D
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Ltr,
    Rtl,
    Inherit,
}

impl Default for Direction {
    fn default() -> Self {
        Self::Inherit
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
        }
    }
}
