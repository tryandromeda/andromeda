#[derive(Clone, Debug)]
pub struct Dimensions {
    pub height: u32,
    pub width: u32,
}

#[derive(Debug)]
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
