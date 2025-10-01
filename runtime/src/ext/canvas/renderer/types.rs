// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::FillStyle;

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
#[derive(Clone, Debug, PartialEq, Default)]
pub enum LineCap {
    #[default]
    Butt,
    Round,
    Square,
}

/// Line join style for stroke operations
#[derive(Clone, Debug, PartialEq, Default)]
pub enum LineJoin {
    Bevel,
    Round,
    #[default]
    Miter,
}

pub struct RenderState {
    pub fill_style: FillStyle,
    pub global_alpha: f32,
    pub transform: [f64; 6],
    pub line_cap: LineCap,
    pub line_join: LineJoin,
    pub miter_limit: f64,
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
