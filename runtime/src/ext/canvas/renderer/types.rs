// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

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
