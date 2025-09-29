// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::FillStyle;

/// Canvas drawing state that can be saved and restored
#[derive(Clone)]
pub struct CanvasState {
    pub fill_style: FillStyle,
    pub stroke_style: FillStyle,
    pub line_width: f64,
    pub global_alpha: f32,
    // TODO: Add transformation matrix, line dash, text properties, etc.
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
        }
    }
}
