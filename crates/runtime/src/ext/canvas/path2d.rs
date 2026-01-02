// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::ext::canvas::renderer::Point;

/// Represents a Path2D object with path data and operations
#[derive(Clone, Debug)]
pub struct Path2D {
    /// The subpaths that make up this path
    pub subpaths: Vec<Subpath>,
    /// Flag indicating if a new subpath is needed
    pub need_new_subpath: bool,
    /// Last control point for smooth curve operations
    last_control_point: Option<Point>,
    /// Last command type for smooth curve operations
    last_command_type: Option<char>,
}

/// A subpath consisting of a list of points and whether it's closed
#[derive(Clone, Debug)]
pub struct Subpath {
    /// Points in this subpath
    pub points: Vec<Point>,
    /// Whether this subpath is closed
    pub closed: bool,
}

/// Path command types for SVG path data
#[derive(Clone, Debug)]
pub enum PathCommand {
    MoveTo {
        x: f64,
        y: f64,
    },
    LineTo {
        x: f64,
        y: f64,
    },
    HorizontalLineTo {
        x: f64,
    },
    VerticalLineTo {
        y: f64,
    },
    QuadraticCurveTo {
        cpx: f64,
        cpy: f64,
        x: f64,
        y: f64,
    },
    SmoothQuadraticCurveTo {
        x: f64,
        y: f64,
    },
    BezierCurveTo {
        cp1x: f64,
        cp1y: f64,
        cp2x: f64,
        cp2y: f64,
        x: f64,
        y: f64,
    },
    SmoothBezierCurveTo {
        cp2x: f64,
        cp2y: f64,
        x: f64,
        y: f64,
    },
    EllipticalArc {
        rx: f64,
        ry: f64,
        x_axis_rotation: f64,
        large_arc_flag: bool,
        sweep_flag: bool,
        x: f64,
        y: f64,
    },
    Arc {
        x: f64,
        y: f64,
        radius: f64,
        start_angle: f64,
        end_angle: f64,
        anticlockwise: bool,
    },
    ArcTo {
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
        radius: f64,
    },
    Ellipse {
        x: f64,
        y: f64,
        radius_x: f64,
        radius_y: f64,
        rotation: f64,
        start_angle: f64,
        end_angle: f64,
        anticlockwise: bool,
    },
    Rect {
        x: f64,
        y: f64,
        width: f64,
        height: f64,
    },
    RoundRect {
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        radii: Vec<f64>,
    },
    ClosePath,
}

/// Parameters for ellipse operations
#[derive(Clone, Debug)]
struct EllipseParams {
    x: f64,
    y: f64,
    radius_x: f64,
    radius_y: f64,
    rotation: f64,
    start_angle: f64,
    end_angle: f64,
    anticlockwise: bool,
}

/// Fill rule for path operations
#[derive(Clone, Debug, PartialEq, Default)]
pub enum FillRule {
    #[default]
    NonZero,
    EvenOdd,
}

impl Path2D {
    /// Create a new empty Path2D
    pub fn new() -> Self {
        Self {
            subpaths: Vec::new(),
            need_new_subpath: true,
            last_control_point: None,
            last_command_type: None,
        }
    }

    /// Create a Path2D from another path
    pub fn from_path(other: &Path2D) -> Self {
        other.clone()
    }

    /// Create a Path2D from SVG path data
    pub fn from_svg_path_data(data: &str) -> Result<Self, String> {
        let mut path = Self::new();
        path.parse_svg_path_data(data)?;
        Ok(path)
    }

    /// Move to a point, creating a new subpath
    pub fn move_to(&mut self, x: f64, y: f64) {
        if x.is_infinite() || x.is_nan() || y.is_infinite() || y.is_nan() {
            return;
        }

        let subpath = Subpath {
            points: vec![Point { x, y }],
            closed: false,
        };
        self.subpaths.push(subpath);
        self.need_new_subpath = false;

        // Reset control point tracking on move
        self.last_control_point = None;
        self.last_command_type = Some('M');
    }

    /// Add a line to the current subpath
    pub fn line_to(&mut self, x: f64, y: f64) {
        if x.is_infinite() || x.is_nan() || y.is_infinite() || y.is_nan() {
            return;
        }

        self.ensure_subpath(x, y);
        if let Some(subpath) = self.subpaths.last_mut() {
            subpath.points.push(Point { x, y });
        }

        // Reset control point tracking for non-curve commands
        self.last_control_point = None;
        self.last_command_type = Some('L');
    }

    /// Add a quadratic curve to the current subpath
    pub fn quadratic_curve_to(&mut self, cpx: f64, cpy: f64, x: f64, y: f64) {
        if cpx.is_infinite()
            || cpx.is_nan()
            || cpy.is_infinite()
            || cpy.is_nan()
            || x.is_infinite()
            || x.is_nan()
            || y.is_infinite()
            || y.is_nan()
        {
            return;
        }

        self.ensure_subpath(x, y);
        if let Some(subpath) = self.subpaths.last_mut()
            && let Some(last_point) = subpath.points.last().cloned()
        {
            let control_point = Point { x: cpx, y: cpy };
            let tessellated = tessellate_quadratic_bezier(
                last_point,
                control_point.clone(),
                Point { x, y },
                16, // segments
            );
            subpath.points.extend(tessellated.into_iter().skip(1)); // Skip first point as it's already in the path

            // Update tracking state
            self.last_control_point = Some(control_point);
            self.last_command_type = Some('Q');
        }
    }

    /// Add a smooth quadratic curve to the current subpath
    pub fn smooth_quadratic_curve_to(&mut self, x: f64, y: f64) {
        if x.is_infinite() || x.is_nan() || y.is_infinite() || y.is_nan() {
            return;
        }

        self.ensure_subpath(x, y);
        if let Some(subpath) = self.subpaths.last_mut()
            && let Some(last_point) = subpath.points.last().cloned()
        {
            // Calculate reflected control point for smooth curve
            let control_point = if let Some(last_ctrl) = &self.last_control_point
                && matches!(
                    self.last_command_type,
                    Some('Q') | Some('q') | Some('T') | Some('t')
                ) {
                // Reflect the last control point about the current point
                Point {
                    x: 2.0 * last_point.x - last_ctrl.x,
                    y: 2.0 * last_point.y - last_ctrl.y,
                }
            } else {
                // If no previous quadratic curve, use current point as control point
                last_point.clone()
            };

            let tessellated = tessellate_quadratic_bezier(
                last_point.clone(),
                control_point.clone(),
                Point { x, y },
                16, // segments
            );
            subpath.points.extend(tessellated.into_iter().skip(1));

            // Update tracking state
            self.last_control_point = Some(control_point);
            self.last_command_type = Some('T');
        }
    }

    /// Add a cubic bezier curve to the current subpath
    pub fn bezier_curve_to(&mut self, cp1x: f64, cp1y: f64, cp2x: f64, cp2y: f64, x: f64, y: f64) {
        if cp1x.is_infinite()
            || cp1x.is_nan()
            || cp1y.is_infinite()
            || cp1y.is_nan()
            || cp2x.is_infinite()
            || cp2x.is_nan()
            || cp2y.is_infinite()
            || cp2y.is_nan()
            || x.is_infinite()
            || x.is_nan()
            || y.is_infinite()
            || y.is_nan()
        {
            return;
        }

        self.ensure_subpath(x, y);
        if let Some(subpath) = self.subpaths.last_mut()
            && let Some(last_point) = subpath.points.last().cloned()
        {
            let cp2_point = Point { x: cp2x, y: cp2y };
            let tessellated = tessellate_cubic_bezier(
                last_point,
                Point { x: cp1x, y: cp1y },
                cp2_point.clone(),
                Point { x, y },
                16, // segments
            );
            subpath.points.extend(tessellated.into_iter().skip(1)); // Skip first point as it's already in the path

            // Update tracking state
            self.last_control_point = Some(cp2_point);
            self.last_command_type = Some('C');
        }
    }

    /// Add a smooth cubic bezier curve to the current subpath
    pub fn smooth_bezier_curve_to(&mut self, cp2x: f64, cp2y: f64, x: f64, y: f64) {
        if cp2x.is_infinite()
            || cp2x.is_nan()
            || cp2y.is_infinite()
            || cp2y.is_nan()
            || x.is_infinite()
            || x.is_nan()
            || y.is_infinite()
            || y.is_nan()
        {
            return;
        }

        self.ensure_subpath(x, y);
        if let Some(subpath) = self.subpaths.last_mut()
            && let Some(last_point) = subpath.points.last().cloned()
        {
            // Calculate reflected first control point for smooth curve
            let cp1 = if let Some(last_ctrl) = &self.last_control_point
                && matches!(
                    self.last_command_type,
                    Some('C') | Some('c') | Some('S') | Some('s')
                ) {
                // Reflect the last control point about the current point
                Point {
                    x: 2.0 * last_point.x - last_ctrl.x,
                    y: 2.0 * last_point.y - last_ctrl.y,
                }
            } else {
                // If no previous cubic curve, use current point as first control point
                last_point.clone()
            };

            let cp2 = Point { x: cp2x, y: cp2y };

            let tessellated = tessellate_cubic_bezier(
                last_point,
                cp1,
                cp2.clone(),
                Point { x, y },
                16, // segments
            );
            subpath.points.extend(tessellated.into_iter().skip(1));

            // Update tracking state
            self.last_control_point = Some(cp2);
            self.last_command_type = Some('S');
        }
    }

    /// Add an elliptical arc to the current subpath using SVG arc parameters
    #[allow(clippy::too_many_arguments)]
    pub fn elliptical_arc_to(
        &mut self,
        rx: f64,
        ry: f64,
        x_axis_rotation: f64,
        large_arc_flag: bool,
        sweep_flag: bool,
        x: f64,
        y: f64,
    ) {
        if rx.is_infinite()
            || rx.is_nan()
            || rx < 0.0
            || ry.is_infinite()
            || ry.is_nan()
            || ry < 0.0
            || x_axis_rotation.is_infinite()
            || x_axis_rotation.is_nan()
            || x.is_infinite()
            || x.is_nan()
            || y.is_infinite()
            || y.is_nan()
        {
            return;
        }

        self.ensure_subpath(x, y);
        if let Some(subpath) = self.subpaths.last_mut()
            && let Some(current_point) = subpath.points.last().cloned()
        {
            let arc_points = tessellate_svg_elliptical_arc(
                current_point,
                Point { x, y },
                rx,
                ry,
                x_axis_rotation,
                large_arc_flag,
                sweep_flag,
                32,
            );
            subpath.points.extend(arc_points.into_iter().skip(1));
        }
    }

    /// Add an arc to the current subpath
    pub fn arc(
        &mut self,
        x: f64,
        y: f64,
        radius: f64,
        start_angle: f64,
        end_angle: f64,
        anticlockwise: bool,
    ) {
        if x.is_infinite()
            || x.is_nan()
            || y.is_infinite()
            || y.is_nan()
            || radius.is_infinite()
            || radius.is_nan()
            || radius < 0.0
            || start_angle.is_infinite()
            || start_angle.is_nan()
            || end_angle.is_infinite()
            || end_angle.is_nan()
        {
            return;
        }

        let tessellated = tessellate_arc(
            x,
            y,
            radius,
            radius,
            0.0,
            start_angle,
            end_angle,
            anticlockwise,
            32,
        );

        if !tessellated.is_empty() {
            // Connect to the arc's start point if we have an existing path
            if let Some(subpath) = self.subpaths.last_mut() {
                if !subpath.points.is_empty() {
                    subpath.points.push(tessellated[0].clone());
                }
                subpath.points.extend(tessellated.into_iter().skip(1));
            } else {
                // Create new subpath starting with the arc
                self.subpaths.push(Subpath {
                    points: tessellated,
                    closed: false,
                });
            }
            self.need_new_subpath = false;
        }
    }

    /// Add an ellipse to the current subpath
    #[allow(clippy::too_many_arguments)]
    pub fn ellipse(
        &mut self,
        x: f64,
        y: f64,
        radius_x: f64,
        radius_y: f64,
        rotation: f64,
        start_angle: f64,
        end_angle: f64,
        anticlockwise: bool,
    ) {
        if x.is_infinite()
            || x.is_nan()
            || y.is_infinite()
            || y.is_nan()
            || radius_x.is_infinite()
            || radius_x.is_nan()
            || radius_x < 0.0
            || radius_y.is_infinite()
            || radius_y.is_nan()
            || radius_y < 0.0
            || rotation.is_infinite()
            || rotation.is_nan()
            || start_angle.is_infinite()
            || start_angle.is_nan()
            || end_angle.is_infinite()
            || end_angle.is_nan()
        {
            return;
        }

        let tessellated = tessellate_arc(
            x,
            y,
            radius_x,
            radius_y,
            rotation,
            start_angle,
            end_angle,
            anticlockwise,
            32,
        );

        if !tessellated.is_empty() {
            // Connect to the ellipse's start point if we have an existing path
            if let Some(subpath) = self.subpaths.last_mut() {
                if !subpath.points.is_empty() {
                    subpath.points.push(tessellated[0].clone());
                }
                subpath.points.extend(tessellated.into_iter().skip(1));
            } else {
                // Create new subpath starting with the ellipse
                self.subpaths.push(Subpath {
                    points: tessellated,
                    closed: false,
                });
            }
            self.need_new_subpath = false;
        }
    }

    /// Add a rectangle to the path as a closed subpath
    pub fn rect(&mut self, x: f64, y: f64, w: f64, h: f64) {
        if x.is_infinite()
            || x.is_nan()
            || y.is_infinite()
            || y.is_nan()
            || w.is_infinite()
            || w.is_nan()
            || h.is_infinite()
            || h.is_nan()
        {
            return;
        }

        let subpath = Subpath {
            points: vec![
                Point { x, y },
                Point { x: x + w, y },
                Point { x: x + w, y: y + h },
                Point { x, y: y + h },
            ],
            closed: true,
        };
        self.subpaths.push(subpath);

        // Create a new subpath with the starting point for subsequent operations
        let new_subpath = Subpath {
            points: vec![Point { x, y }],
            closed: false,
        };
        self.subpaths.push(new_subpath);
        self.need_new_subpath = false;
    }

    /// Close the current subpath
    pub fn close_path(&mut self) {
        if let Some(subpath) = self.subpaths.last_mut()
            && !subpath.points.is_empty()
        {
            subpath.closed = true;
            // Create a new subpath starting with the same point as the previous subpath's first point
            if let Some(first_point) = subpath.points.first().cloned() {
                let new_subpath = Subpath {
                    points: vec![first_point],
                    closed: false,
                };
                self.subpaths.push(new_subpath);
            }
        }
    }

    /// Add another path to this path with optional transformation
    pub fn add_path(&mut self, other: &Path2D, transform: Option<[f64; 6]>) {
        for subpath in &other.subpaths {
            let mut new_subpath = Subpath {
                points: Vec::new(),
                closed: subpath.closed,
            };

            for point in &subpath.points {
                let transformed_point = if let Some(matrix) = transform {
                    transform_point(point, &matrix)
                } else {
                    point.clone()
                };
                new_subpath.points.push(transformed_point);
            }

            if !new_subpath.points.is_empty() {
                self.subpaths.push(new_subpath);
            }
        }
    }

    /// Add an arc to the current path using control points
    pub fn arc_to(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, radius: f64) {
        if x1.is_infinite()
            || x1.is_nan()
            || y1.is_infinite()
            || y1.is_nan()
            || x2.is_infinite()
            || x2.is_nan()
            || y2.is_infinite()
            || y2.is_nan()
            || radius.is_infinite()
            || radius.is_nan()
            || radius < 0.0
        {
            return;
        }

        self.ensure_subpath(x1, y1);

        if let Some(subpath) = self.subpaths.last_mut()
            && let Some(current_point) = subpath.points.last().cloned()
        {
            let arc_points = calculate_arc_to_points(
                current_point,
                Point { x: x1, y: y1 },
                Point { x: x2, y: y2 },
                radius,
            );
            subpath.points.extend(arc_points);
        }
    }

    /// Add a rounded rectangle to the path as a closed subpath
    pub fn round_rect(&mut self, x: f64, y: f64, w: f64, h: f64, radii: &[f64]) {
        if x.is_infinite()
            || x.is_nan()
            || y.is_infinite()
            || y.is_nan()
            || w.is_infinite()
            || w.is_nan()
            || h.is_infinite()
            || h.is_nan()
        {
            return;
        }

        let (top_left, top_right, bottom_right, bottom_left) = parse_border_radii(radii);

        let max_radius_x = w / 2.0;
        let max_radius_y = h / 2.0;

        let tl = top_left.min(max_radius_x).min(max_radius_y);
        let tr = top_right.min(max_radius_x).min(max_radius_y);
        let br = bottom_right.min(max_radius_x).min(max_radius_y);
        let bl = bottom_left.min(max_radius_x).min(max_radius_y);

        let mut points = Vec::new();

        let start_x = x + tl;
        let start_y = y;
        points.push(Point {
            x: start_x,
            y: start_y,
        });

        if tr > 0.0 {
            points.push(Point { x: x + w - tr, y });
            let arc_points = tessellate_arc(
                x + w - tr,
                y + tr,
                tr,
                tr,
                0.0,
                -std::f64::consts::PI / 2.0,
                0.0,
                false,
                8,
            );
            points.extend(arc_points.into_iter().skip(1));
        } else {
            points.push(Point { x: x + w, y });
        }

        // Right edge to bottom-right corner
        if br > 0.0 {
            points.push(Point {
                x: x + w,
                y: y + h - br,
            });
            // Bottom-right corner arc
            let arc_points = tessellate_arc(
                x + w - br,
                y + h - br,
                br,
                br,
                0.0,
                0.0,
                std::f64::consts::PI / 2.0,
                false,
                8,
            );
            points.extend(arc_points.into_iter().skip(1));
        } else {
            points.push(Point { x: x + w, y: y + h });
        }

        // Bottom edge to bottom-left corner
        if bl > 0.0 {
            points.push(Point {
                x: x + bl,
                y: y + h,
            });
            // Bottom-left corner arc
            let arc_points = tessellate_arc(
                x + bl,
                y + h - bl,
                bl,
                bl,
                0.0,
                std::f64::consts::PI / 2.0,
                std::f64::consts::PI,
                false,
                8,
            );
            points.extend(arc_points.into_iter().skip(1));
        } else {
            points.push(Point { x, y: y + h });
        }

        // Left edge to top-left corner
        if tl > 0.0 {
            points.push(Point { x, y: y + tl });
            // Top-left corner arc
            let arc_points = tessellate_arc(
                x + tl,
                y + tl,
                tl,
                tl,
                0.0,
                std::f64::consts::PI,
                3.0 * std::f64::consts::PI / 2.0,
                false,
                8,
            );
            points.extend(arc_points.into_iter().skip(1));
        } else {
            points.push(Point { x, y });
        }

        let subpath = Subpath {
            points,
            closed: true,
        };
        self.subpaths.push(subpath);

        // Create a new subpath with the starting point for subsequent operations
        let new_subpath = Subpath {
            points: vec![Point {
                x: start_x,
                y: start_y,
            }],
            closed: false,
        };
        self.subpaths.push(new_subpath);
        self.need_new_subpath = false;
    }

    /// Add a rounded rectangle to the path (Web API compatible method)
    pub fn round_rect_web_api(&mut self, x: f64, y: f64, width: f64, height: f64, radii: &[f64]) {
        // Handle negative width/height by flipping coordinates and radii
        let (actual_x, actual_y, actual_width, actual_height, flipped_radii) =
            normalize_rect_params(x, y, width, height, radii);

        self.round_rect(
            actual_x,
            actual_y,
            actual_width,
            actual_height,
            &flipped_radii,
        );
    }

    /// Check if a point is inside the path using the specified fill rule
    pub fn is_point_in_path(&self, x: f64, y: f64, fill_rule: FillRule) -> bool {
        if x.is_infinite() || x.is_nan() || y.is_infinite() || y.is_nan() {
            return false;
        }

        let point = Point { x, y };

        match fill_rule {
            FillRule::NonZero => self.is_point_in_path_nonzero(&point),
            FillRule::EvenOdd => self.is_point_in_path_evenodd(&point),
        }
    }

    /// Check if a point is within the stroke of the path
    pub fn is_point_in_stroke(&self, x: f64, y: f64, line_width: f64) -> bool {
        if x.is_infinite() || x.is_nan() || y.is_infinite() || y.is_nan() {
            return false;
        }

        let point = Point { x, y };
        let half_width = line_width / 2.0;

        for subpath in &self.subpaths {
            if subpath.points.len() < 2 {
                continue;
            }

            // Check each line segment in the subpath
            for i in 0..subpath.points.len() - 1 {
                let p1 = &subpath.points[i];
                let p2 = &subpath.points[i + 1];

                if point_to_line_distance(&point, p1, p2) <= half_width {
                    return true;
                }
            }

            // Check closing line if subpath is closed
            if subpath.closed && subpath.points.len() > 2 {
                let first = &subpath.points[0];
                let last = &subpath.points[subpath.points.len() - 1];
                if point_to_line_distance(&point, last, first) <= half_width {
                    return true;
                }
            }
        }

        false
    }

    /// Get all points from all subpaths (for rendering)
    pub fn get_all_points(&self) -> Vec<Point> {
        let mut all_points = Vec::new();
        for subpath in &self.subpaths {
            all_points.extend(subpath.points.iter().cloned());
        }
        all_points
    }

    /// Ensure there is a subpath for the given coordinates
    fn ensure_subpath(&mut self, x: f64, y: f64) {
        if self.need_new_subpath || self.subpaths.is_empty() {
            let subpath = Subpath {
                points: vec![Point { x, y }],
                closed: false,
            };
            self.subpaths.push(subpath);
            self.need_new_subpath = false;
        }
    }

    /// Check if point is inside using non-zero winding rule
    fn is_point_in_path_nonzero(&self, point: &Point) -> bool {
        let mut winding_number = 0;

        for subpath in &self.subpaths {
            if subpath.points.len() < 3 {
                continue; // Need at least 3 points for a fillable shape
            }

            // Check all edges of the subpath
            for i in 0..subpath.points.len() {
                let p1 = &subpath.points[i];
                let p2 = if i == subpath.points.len() - 1 {
                    if subpath.closed {
                        &subpath.points[0] // Close the path
                    } else {
                        continue; // Open path, skip the last edge
                    }
                } else {
                    &subpath.points[i + 1]
                };

                // Ray casting algorithm for winding number
                if p1.y <= point.y {
                    if p2.y > point.y {
                        // Upward crossing
                        if is_left(p1, p2, point) > 0.0 {
                            winding_number += 1;
                        }
                    }
                } else if p2.y <= point.y {
                    // Downward crossing
                    if is_left(p1, p2, point) < 0.0 {
                        winding_number -= 1;
                    }
                }
            }
        }

        winding_number != 0
    }

    /// Check if point is inside using even-odd rule
    fn is_point_in_path_evenodd(&self, point: &Point) -> bool {
        let mut crossing_count = 0;

        for subpath in &self.subpaths {
            if subpath.points.len() < 3 {
                continue; // Need at least 3 points for a fillable shape
            }

            // Check all edges of the subpath
            for i in 0..subpath.points.len() {
                let p1 = &subpath.points[i];
                let p2 = if i == subpath.points.len() - 1 {
                    if subpath.closed {
                        &subpath.points[0] // Close the path
                    } else {
                        continue; // Open path, skip the last edge
                    }
                } else {
                    &subpath.points[i + 1]
                };

                // Ray casting from point to the right
                if ((p1.y > point.y) != (p2.y > point.y))
                    && (point.x < (p2.x - p1.x) * (point.y - p1.y) / (p2.y - p1.y) + p1.x)
                {
                    crossing_count += 1;
                }
            }
        }

        crossing_count % 2 == 1
    }

    /// Parse SVG path data string
    fn parse_svg_path_data(&mut self, data: &str) -> Result<(), String> {
        let commands = parse_svg_path_commands(data)?;

        for command in commands {
            match command {
                PathCommand::MoveTo { x, y } => self.move_to(x, y),
                PathCommand::LineTo { x, y } => self.line_to(x, y),
                PathCommand::HorizontalLineTo { x } => {
                    if let Some(subpath) = self.subpaths.last()
                        && let Some(last_point) = subpath.points.last()
                    {
                        self.line_to(x, last_point.y);
                    }
                }
                PathCommand::VerticalLineTo { y } => {
                    if let Some(subpath) = self.subpaths.last()
                        && let Some(last_point) = subpath.points.last()
                    {
                        self.line_to(last_point.x, y);
                    }
                }
                PathCommand::QuadraticCurveTo { cpx, cpy, x, y } => {
                    self.quadratic_curve_to(cpx, cpy, x, y)
                }
                PathCommand::SmoothQuadraticCurveTo { x, y } => {
                    self.smooth_quadratic_curve_to(x, y)
                }
                PathCommand::BezierCurveTo {
                    cp1x,
                    cp1y,
                    cp2x,
                    cp2y,
                    x,
                    y,
                } => self.bezier_curve_to(cp1x, cp1y, cp2x, cp2y, x, y),
                PathCommand::SmoothBezierCurveTo { cp2x, cp2y, x, y } => {
                    self.smooth_bezier_curve_to(cp2x, cp2y, x, y)
                }
                PathCommand::EllipticalArc {
                    rx,
                    ry,
                    x_axis_rotation,
                    large_arc_flag,
                    sweep_flag,
                    x,
                    y,
                } => self.elliptical_arc_to(
                    rx,
                    ry,
                    x_axis_rotation,
                    large_arc_flag,
                    sweep_flag,
                    x,
                    y,
                ),
                PathCommand::Arc {
                    x,
                    y,
                    radius,
                    start_angle,
                    end_angle,
                    anticlockwise,
                } => self.arc(x, y, radius, start_angle, end_angle, anticlockwise),
                PathCommand::ArcTo {
                    x1,
                    y1,
                    x2,
                    y2,
                    radius,
                } => self.arc_to(x1, y1, x2, y2, radius),
                PathCommand::Ellipse {
                    x,
                    y,
                    radius_x,
                    radius_y,
                    rotation,
                    start_angle,
                    end_angle,
                    anticlockwise,
                } => self.ellipse(
                    x,
                    y,
                    radius_x,
                    radius_y,
                    rotation,
                    start_angle,
                    end_angle,
                    anticlockwise,
                ),
                PathCommand::Rect {
                    x,
                    y,
                    width,
                    height,
                } => self.rect(x, y, width, height),
                PathCommand::RoundRect {
                    x,
                    y,
                    width,
                    height,
                    radii,
                } => {
                    // Implement rounded rectangle tessellation
                    self.round_rect(x, y, width, height, &radii);
                }
                PathCommand::ClosePath => self.close_path(),
            }
        }

        Ok(())
    }
}

impl Default for Path2D {
    fn default() -> Self {
        Self::new()
    }
}

/// Transform a point using a 2D transformation matrix [a, b, c, d, e, f]
fn transform_point(point: &Point, matrix: &[f64; 6]) -> Point {
    let [a, b, c, d, e, f] = *matrix;
    Point {
        x: a * point.x + c * point.y + e,
        y: b * point.x + d * point.y + f,
    }
}

/// Calculate the distance from a point to a line segment
fn point_to_line_distance(point: &Point, line_start: &Point, line_end: &Point) -> f64 {
    let dx = line_end.x - line_start.x;
    let dy = line_end.y - line_start.y;

    if dx == 0.0 && dy == 0.0 {
        // Line segment is actually a point
        let dx_point = point.x - line_start.x;
        let dy_point = point.y - line_start.y;
        return (dx_point * dx_point + dy_point * dy_point).sqrt();
    }

    let length_squared = dx * dx + dy * dy;
    let t = ((point.x - line_start.x) * dx + (point.y - line_start.y) * dy) / length_squared;
    let t = t.clamp(0.0, 1.0); // Clamp to line segment

    let projection_x = line_start.x + t * dx;
    let projection_y = line_start.y + t * dy;

    let distance_x = point.x - projection_x;
    let distance_y = point.y - projection_y;

    (distance_x * distance_x + distance_y * distance_y).sqrt()
}

/// Test which side of a line a point is on (for winding number calculation)
fn is_left(p0: &Point, p1: &Point, p2: &Point) -> f64 {
    (p1.x - p0.x) * (p2.y - p0.y) - (p2.x - p0.x) * (p1.y - p0.y)
}

/// Tessellate a quadratic Bezier curve into line segments
fn tessellate_quadratic_bezier(
    start: Point,
    control: Point,
    end: Point,
    segments: usize,
) -> Vec<Point> {
    let mut points = Vec::with_capacity(segments + 1);

    for i in 0..=segments {
        let t = i as f64 / segments as f64;
        let t_inv = 1.0 - t;

        // Quadratic Bezier formula: B(t) = (1-t)²P₀ + 2(1-t)tP₁ + t²P₂
        let x = t_inv * t_inv * start.x + 2.0 * t_inv * t * control.x + t * t * end.x;
        let y = t_inv * t_inv * start.y + 2.0 * t_inv * t * control.y + t * t * end.y;

        points.push(Point { x, y });
    }

    points
}

/// Tessellate a cubic Bezier curve into line segments
fn tessellate_cubic_bezier(
    start: Point,
    control1: Point,
    control2: Point,
    end: Point,
    segments: usize,
) -> Vec<Point> {
    let mut points = Vec::with_capacity(segments + 1);

    for i in 0..=segments {
        let t = i as f64 / segments as f64;
        let t_inv = 1.0 - t;

        // Cubic Bezier formula: B(t) = (1-t)³P₀ + 3(1-t)²tP₁ + 3(1-t)t²P₂ + t³P₃
        let t_inv_sq = t_inv * t_inv;
        let t_sq = t * t;

        let x = t_inv_sq * t_inv * start.x
            + 3.0 * t_inv_sq * t * control1.x
            + 3.0 * t_inv * t_sq * control2.x
            + t_sq * t * end.x;

        let y = t_inv_sq * t_inv * start.y
            + 3.0 * t_inv_sq * t * control1.y
            + 3.0 * t_inv * t_sq * control2.y
            + t_sq * t * end.y;

        points.push(Point { x, y });
    }

    points
}

/// Tessellate an arc/ellipse into line segments
#[allow(clippy::too_many_arguments)]
fn tessellate_arc(
    center_x: f64,
    center_y: f64,
    radius_x: f64,
    radius_y: f64,
    rotation: f64,
    start_angle: f64,
    end_angle: f64,
    anticlockwise: bool,
    segments: usize,
) -> Vec<Point> {
    let params = EllipseParams {
        x: center_x,
        y: center_y,
        radius_x,
        radius_y,
        rotation,
        start_angle,
        end_angle,
        anticlockwise,
    };
    tessellate_arc_with_params(&params, segments)
}

/// Tessellate an arc/ellipse into line segments using parameters struct
fn tessellate_arc_with_params(params: &EllipseParams, segments: usize) -> Vec<Point> {
    let mut points = Vec::new();

    let current_angle = params.start_angle;
    let mut end_target = params.end_angle;

    // Handle anticlockwise direction
    if params.anticlockwise && end_target > current_angle {
        end_target -= 2.0 * std::f64::consts::PI;
    } else if !params.anticlockwise && end_target < current_angle {
        end_target += 2.0 * std::f64::consts::PI;
    }

    let angle_diff = end_target - current_angle;
    let step = angle_diff / segments as f64;

    let cos_rotation = params.rotation.cos();
    let sin_rotation = params.rotation.sin();

    for i in 0..=segments {
        let angle = current_angle + i as f64 * step;

        // Point on unrotated ellipse
        let ellipse_x = params.radius_x * angle.cos();
        let ellipse_y = params.radius_y * angle.sin();

        // Apply rotation
        let rotated_x = ellipse_x * cos_rotation - ellipse_y * sin_rotation;
        let rotated_y = ellipse_x * sin_rotation + ellipse_y * cos_rotation;

        // Translate to center
        points.push(Point {
            x: params.x + rotated_x,
            y: params.y + rotated_y,
        });
    }

    points
}

/// Enhanced SVG path command parser supporting all standard commands
fn parse_svg_path_commands(data: &str) -> Result<Vec<PathCommand>, String> {
    let mut commands = Vec::new();
    let mut chars = data.chars().peekable();
    let mut current_x = 0.0;
    let mut current_y = 0.0;
    let mut last_command = ' ';

    while let Some(&ch) = chars.peek() {
        if ch.is_whitespace() || ch == ',' {
            chars.next();
            continue;
        }

        let command_char = if ch.is_alphabetic() {
            chars.next().unwrap()
        } else {
            // Implicit command repetition
            last_command
        };

        match command_char {
            'M' | 'm' => {
                let (x, y) = parse_coordinate_pair(&mut chars)?;
                if command_char == 'm' && !(current_x == 0.0 && current_y == 0.0) {
                    current_x += x;
                    current_y += y;
                } else {
                    current_x = x;
                    current_y = y;
                }
                commands.push(PathCommand::MoveTo {
                    x: current_x,
                    y: current_y,
                });
                last_command = if command_char == 'M' { 'L' } else { 'l' }; // Subsequent coordinates are LineTo
            }
            'L' | 'l' => {
                let (x, y) = parse_coordinate_pair(&mut chars)?;
                if command_char == 'l' {
                    current_x += x;
                    current_y += y;
                } else {
                    current_x = x;
                    current_y = y;
                }
                commands.push(PathCommand::LineTo {
                    x: current_x,
                    y: current_y,
                });
                last_command = command_char;
            }
            'H' | 'h' => {
                let x = parse_single_number(&mut chars)?;
                if command_char == 'h' {
                    current_x += x;
                } else {
                    current_x = x;
                }
                commands.push(PathCommand::HorizontalLineTo { x: current_x });
                last_command = command_char;
            }
            'V' | 'v' => {
                let y = parse_single_number(&mut chars)?;
                if command_char == 'v' {
                    current_y += y;
                } else {
                    current_y = y;
                }
                commands.push(PathCommand::VerticalLineTo { y: current_y });
                last_command = command_char;
            }
            'Q' | 'q' => {
                let (cpx, cpy) = parse_coordinate_pair(&mut chars)?;
                let (x, y) = parse_coordinate_pair(&mut chars)?;

                let (abs_cpx, abs_cpy, abs_x, abs_y) = if command_char == 'q' {
                    (
                        current_x + cpx,
                        current_y + cpy,
                        current_x + x,
                        current_y + y,
                    )
                } else {
                    (cpx, cpy, x, y)
                };

                current_x = abs_x;
                current_y = abs_y;

                commands.push(PathCommand::QuadraticCurveTo {
                    cpx: abs_cpx,
                    cpy: abs_cpy,
                    x: current_x,
                    y: current_y,
                });
                last_command = command_char;
            }
            'T' | 't' => {
                let (x, y) = parse_coordinate_pair(&mut chars)?;

                if command_char == 't' {
                    current_x += x;
                    current_y += y;
                } else {
                    current_x = x;
                    current_y = y;
                }

                commands.push(PathCommand::SmoothQuadraticCurveTo {
                    x: current_x,
                    y: current_y,
                });
                last_command = command_char;
            }
            'C' | 'c' => {
                let (cp1x, cp1y) = parse_coordinate_pair(&mut chars)?;
                let (cp2x, cp2y) = parse_coordinate_pair(&mut chars)?;
                let (x, y) = parse_coordinate_pair(&mut chars)?;

                let (abs_cp1x, abs_cp1y, abs_cp2x, abs_cp2y, abs_x, abs_y) = if command_char == 'c'
                {
                    (
                        current_x + cp1x,
                        current_y + cp1y,
                        current_x + cp2x,
                        current_y + cp2y,
                        current_x + x,
                        current_y + y,
                    )
                } else {
                    (cp1x, cp1y, cp2x, cp2y, x, y)
                };

                current_x = abs_x;
                current_y = abs_y;

                commands.push(PathCommand::BezierCurveTo {
                    cp1x: abs_cp1x,
                    cp1y: abs_cp1y,
                    cp2x: abs_cp2x,
                    cp2y: abs_cp2y,
                    x: current_x,
                    y: current_y,
                });
                last_command = command_char;
            }
            'S' | 's' => {
                let (cp2x, cp2y) = parse_coordinate_pair(&mut chars)?;
                let (x, y) = parse_coordinate_pair(&mut chars)?;

                let (abs_cp2x, abs_cp2y, abs_x, abs_y) = if command_char == 's' {
                    (
                        current_x + cp2x,
                        current_y + cp2y,
                        current_x + x,
                        current_y + y,
                    )
                } else {
                    (cp2x, cp2y, x, y)
                };

                current_x = abs_x;
                current_y = abs_y;

                commands.push(PathCommand::SmoothBezierCurveTo {
                    cp2x: abs_cp2x,
                    cp2y: abs_cp2y,
                    x: current_x,
                    y: current_y,
                });
                last_command = command_char;
            }
            'A' | 'a' => {
                let rx = parse_single_number(&mut chars)?;
                skip_whitespace_and_commas(&mut chars);
                let ry = parse_single_number(&mut chars)?;
                skip_whitespace_and_commas(&mut chars);
                let x_axis_rotation = parse_single_number(&mut chars)?;
                skip_whitespace_and_commas(&mut chars);
                let large_arc_flag = parse_single_number(&mut chars)? != 0.0;
                skip_whitespace_and_commas(&mut chars);
                let sweep_flag = parse_single_number(&mut chars)? != 0.0;
                skip_whitespace_and_commas(&mut chars);
                let (x, y) = parse_coordinate_pair(&mut chars)?;

                if command_char == 'a' {
                    current_x += x;
                    current_y += y;
                } else {
                    current_x = x;
                    current_y = y;
                }

                commands.push(PathCommand::EllipticalArc {
                    rx,
                    ry,
                    x_axis_rotation: x_axis_rotation.to_radians(),
                    large_arc_flag,
                    sweep_flag,
                    x: current_x,
                    y: current_y,
                });
                last_command = command_char;
            }
            'Z' | 'z' => {
                commands.push(PathCommand::ClosePath);
                last_command = command_char;
            }
            _ => {
                return Err(format!("Unsupported SVG path command: {}", command_char));
            }
        }
    }

    Ok(commands)
}

/// Parse a single number from SVG path data
fn parse_single_number(chars: &mut std::iter::Peekable<std::str::Chars>) -> Result<f64, String> {
    skip_whitespace_and_commas(chars);
    parse_number(chars)
}

/// Parse a coordinate pair from SVG path data
fn parse_coordinate_pair(
    chars: &mut std::iter::Peekable<std::str::Chars>,
) -> Result<(f64, f64), String> {
    skip_whitespace(chars);
    let x = parse_number(chars)?;
    skip_whitespace_and_commas(chars);
    let y = parse_number(chars)?;
    Ok((x, y))
}

/// Skip whitespace characters
fn skip_whitespace(chars: &mut std::iter::Peekable<std::str::Chars>) {
    while let Some(&ch) = chars.peek() {
        if ch.is_whitespace() {
            chars.next();
        } else {
            break;
        }
    }
}

/// Skip whitespace and comma characters
fn skip_whitespace_and_commas(chars: &mut std::iter::Peekable<std::str::Chars>) {
    while let Some(&ch) = chars.peek() {
        if ch.is_whitespace() || ch == ',' {
            chars.next();
        } else {
            break;
        }
    }
}

/// Enhanced number parser supporting scientific notation and edge cases
fn parse_number(chars: &mut std::iter::Peekable<std::str::Chars>) -> Result<f64, String> {
    let mut number_str = String::new();
    let mut has_decimal = false;
    let mut has_digits = false;
    let mut has_exponent = false;

    // Handle optional negative sign
    if let Some(&'-') = chars.peek() {
        number_str.push(chars.next().unwrap());
    } else if let Some(&'+') = chars.peek() {
        chars.next(); // Skip positive sign
    }

    // Parse main number part
    while let Some(&ch) = chars.peek() {
        if ch.is_ascii_digit() {
            number_str.push(chars.next().unwrap());
            has_digits = true;
        } else if ch == '.' && !has_decimal && !has_exponent {
            number_str.push(chars.next().unwrap());
            has_decimal = true;
        } else if (ch == 'e' || ch == 'E') && !has_exponent && has_digits {
            number_str.push(chars.next().unwrap());
            has_exponent = true;

            // Handle optional sign after exponent
            if let Some(&next_ch) = chars.peek()
                && (next_ch == '+' || next_ch == '-')
            {
                number_str.push(chars.next().unwrap());
            }
        } else {
            break;
        }
    }

    if !has_digits {
        return Err("Invalid number in SVG path data".to_string());
    }

    number_str
        .parse()
        .map_err(|_| format!("Failed to parse number: {}", number_str))
}

/// Calculate points for arcTo operation
fn calculate_arc_to_points(p0: Point, p1: Point, p2: Point, radius: f64) -> Vec<Point> {
    if radius == 0.0 {
        // If radius is 0, just draw a line to p1
        return vec![p1];
    }

    // Vector from p0 to p1
    let v1 = Point {
        x: p1.x - p0.x,
        y: p1.y - p0.y,
    };

    // Vector from p1 to p2
    let v2 = Point {
        x: p2.x - p1.x,
        y: p2.y - p1.y,
    };

    // Normalize vectors
    let len1 = (v1.x * v1.x + v1.y * v1.y).sqrt();
    let len2 = (v2.x * v2.x + v2.y * v2.y).sqrt();

    if len1 == 0.0 || len2 == 0.0 {
        return vec![p1]; // Degenerate case
    }

    let unit1 = Point {
        x: v1.x / len1,
        y: v1.y / len1,
    };

    let unit2 = Point {
        x: v2.x / len2,
        y: v2.y / len2,
    };

    // Calculate angle between vectors
    let dot_product = unit1.x * unit2.x + unit1.y * unit2.y;
    let angle = dot_product.clamp(-1.0, 1.0).acos();

    // If lines are parallel, just draw line to p1
    if angle.abs() < 1e-10 || (angle - std::f64::consts::PI).abs() < 1e-10 {
        return vec![p1];
    }

    // Calculate distance from p1 to tangent points
    let distance = radius / (angle / 2.0).tan();

    // Calculate tangent points
    let t1 = Point {
        x: p1.x - unit1.x * distance,
        y: p1.y - unit1.y * distance,
    };

    let t2 = Point {
        x: p1.x + unit2.x * distance,
        y: p1.y + unit2.y * distance,
    };

    // Calculate center of the arc
    let normal1 = Point {
        x: -unit1.y,
        y: unit1.x,
    };

    let center = Point {
        x: t1.x + normal1.x * radius,
        y: t1.y + normal1.y * radius,
    };

    // Calculate start and end angles for the arc
    let start_angle = (t1.y - center.y).atan2(t1.x - center.x);
    let end_angle = (t2.y - center.y).atan2(t2.x - center.x);

    // Generate arc points
    let mut points = vec![t1]; // Start with line to first tangent point
    let arc_points = tessellate_arc(
        center.x,
        center.y,
        radius,
        radius,
        0.0,
        start_angle,
        end_angle,
        false,
        8,
    );
    points.extend(arc_points.into_iter().skip(1)); // Skip first point of arc

    points
}

/// Parse border radii according to CSS border-radius rules
fn parse_border_radii(radii: &[f64]) -> (f64, f64, f64, f64) {
    match radii.len() {
        0 => (0.0, 0.0, 0.0, 0.0),
        1 => {
            let r = radii[0];
            (r, r, r, r)
        }
        2 => {
            let r1 = radii[0];
            let r2 = radii[1];
            (r1, r2, r1, r2)
        }
        3 => {
            let r1 = radii[0];
            let r2 = radii[1];
            let r3 = radii[2];
            (r1, r2, r3, r2)
        }
        _ => {
            // 4 or more values
            let r1 = radii[0];
            let r2 = radii[1];
            let r3 = radii[2];
            let r4 = radii[3];
            (r1, r2, r3, r4)
        }
    }
}

/// Tessellate SVG elliptical arc into line segments
#[allow(clippy::too_many_arguments)]
fn tessellate_svg_elliptical_arc(
    start: Point,
    end: Point,
    rx: f64,
    ry: f64,
    x_axis_rotation: f64,
    large_arc_flag: bool,
    sweep_flag: bool,
    segments: usize,
) -> Vec<Point> {
    // Handle degenerate cases
    if rx == 0.0 || ry == 0.0 {
        return vec![start, end];
    }

    if start.x == end.x && start.y == end.y {
        return vec![start];
    }

    // Convert to center parameterization
    let (center, start_angle, delta_angle) = svg_arc_to_center_params(
        start,
        end,
        rx,
        ry,
        x_axis_rotation,
        large_arc_flag,
        sweep_flag,
    );

    let end_angle = start_angle + delta_angle;

    tessellate_arc(
        center.x,
        center.y,
        rx,
        ry,
        x_axis_rotation,
        start_angle,
        end_angle,
        delta_angle < 0.0,
        segments,
    )
}

/// Convert SVG arc parameters to center parameterization
#[allow(clippy::too_many_arguments)]
fn svg_arc_to_center_params(
    start: Point,
    end: Point,
    mut rx: f64,
    mut ry: f64,
    x_axis_rotation: f64,
    large_arc_flag: bool,
    sweep_flag: bool,
) -> (Point, f64, f64) {
    // Ensure positive radii
    rx = rx.abs();
    ry = ry.abs();

    let cos_phi = x_axis_rotation.cos();
    let sin_phi = x_axis_rotation.sin();

    // Step 1: Compute (x1', y1')
    let dx = (start.x - end.x) / 2.0;
    let dy = (start.y - end.y) / 2.0;

    let x1_prime = cos_phi * dx + sin_phi * dy;
    let y1_prime = -sin_phi * dx + cos_phi * dy;

    // Correct radii if necessary
    let lambda = (x1_prime * x1_prime) / (rx * rx) + (y1_prime * y1_prime) / (ry * ry);
    if lambda > 1.0 {
        rx *= lambda.sqrt();
        ry *= lambda.sqrt();
    }

    // Step 2: Compute (cx', cy')
    let sign = if large_arc_flag == sweep_flag {
        -1.0
    } else {
        1.0
    };

    let sq = ((rx * rx * ry * ry - rx * rx * y1_prime * y1_prime - ry * ry * x1_prime * x1_prime)
        / (rx * rx * y1_prime * y1_prime + ry * ry * x1_prime * x1_prime))
        .max(0.0);

    let coeff = sign * sq.sqrt();
    let cx_prime = coeff * rx * y1_prime / ry;
    let cy_prime = -coeff * ry * x1_prime / rx;

    // Step 3: Compute (cx, cy)
    let cx = cos_phi * cx_prime - sin_phi * cy_prime + (start.x + end.x) / 2.0;
    let cy = sin_phi * cx_prime + cos_phi * cy_prime + (start.y + end.y) / 2.0;

    // Step 4: Compute angles
    let ux = (x1_prime - cx_prime) / rx;
    let uy = (y1_prime - cy_prime) / ry;
    let vx = (-x1_prime - cx_prime) / rx;
    let vy = (-y1_prime - cy_prime) / ry;

    let start_angle = vector_angle(1.0, 0.0, ux, uy);
    let mut delta_angle = vector_angle(ux, uy, vx, vy);

    if !sweep_flag && delta_angle > 0.0 {
        delta_angle -= 2.0 * std::f64::consts::PI;
    } else if sweep_flag && delta_angle < 0.0 {
        delta_angle += 2.0 * std::f64::consts::PI;
    }

    (Point { x: cx, y: cy }, start_angle, delta_angle)
}

/// Calculate angle between two vectors
fn vector_angle(ux: f64, uy: f64, vx: f64, vy: f64) -> f64 {
    let dot = ux * vx + uy * vy;
    let det = ux * vy - uy * vx;
    det.atan2(dot)
}

/// Normalize rectangle parameters to handle negative width/height
fn normalize_rect_params(
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    radii: &[f64],
) -> (f64, f64, f64, f64, Vec<f64>) {
    let mut actual_x = x;
    let mut actual_y = y;
    let mut actual_width = width;
    let mut actual_height = height;
    let mut flipped_radii = radii.to_vec();

    // Handle negative width (flip horizontally)
    if width < 0.0 {
        actual_x = x + width;
        actual_width = -width;

        // Flip radii horizontally: [TL, TR, BR, BL] -> [TR, TL, BL, BR]
        if radii.len() >= 4 {
            flipped_radii = vec![radii[1], radii[0], radii[3], radii[2]];
        } else if radii.len() == 2 {
            flipped_radii = vec![radii[1], radii[0]];
        }
        // For 1 or 3 elements, no change needed
    }

    // Handle negative height (flip vertically)
    if height < 0.0 {
        actual_y = y + height;
        actual_height = -height;

        // Flip radii vertically: [TL, TR, BR, BL] -> [BL, BR, TR, TL]
        if flipped_radii.len() >= 4 {
            flipped_radii = vec![
                flipped_radii[3],
                flipped_radii[2],
                flipped_radii[1],
                flipped_radii[0],
            ];
        } else if flipped_radii.len() == 3 {
            flipped_radii = vec![flipped_radii[2], flipped_radii[1], flipped_radii[0]];
        }
        // For 1 or 2 elements, no change needed for vertical flip only
    }

    (
        actual_x,
        actual_y,
        actual_width,
        actual_height,
        flipped_radii,
    )
}
