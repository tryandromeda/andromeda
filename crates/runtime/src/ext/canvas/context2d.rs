// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use super::CanvasResources;
use super::FillStyle;
use super::Rid;
use super::renderer::{Point, Rect};
use crate::RuntimeMacroTask;
use crate::ext::canvas::renderer::{CompositeOperation, LineCap, LineJoin, RenderState};
use andromeda_core::HostData;
use nova_vm::ecmascript::Number;
use nova_vm::{
    ecmascript::{Agent, ArgumentsList, JsResult, Value},
    engine::{Bindable, GcScope},
};

/// Yield `(start, end)` index pairs over `current_path` so each subpath
/// can be rendered independently.
///
/// `subpath_starts` holds the `current_path` index at which each `moveTo`
/// began a new subpath. The first subpath is implicit when the path opens
/// without an explicit `moveTo` (e.g. `beginPath(); arc(...); fill();`).
fn subpath_ranges(current_path: &[Point], subpath_starts: &[usize]) -> Vec<(usize, usize)> {
    if current_path.is_empty() {
        return Vec::new();
    }

    let mut starts: Vec<usize> = Vec::with_capacity(subpath_starts.len() + 1);
    if subpath_starts.first().copied() != Some(0) {
        starts.push(0);
    }
    for &s in subpath_starts {
        if s <= current_path.len() && starts.last().copied() != Some(s) {
            starts.push(s);
        }
    }

    let mut ranges = Vec::with_capacity(starts.len());
    for i in 0..starts.len() {
        let start = starts[i];
        let end = starts.get(i + 1).copied().unwrap_or(current_path.len());
        if end > start {
            ranges.push((start, end));
        }
    }
    ranges
}

/// A command to be executed on the canvas
#[derive(Clone)]
#[allow(dead_code)]
pub enum CanvasCommand<'gc> {
    /// Draw an image onto the canvas
    DrawImage {
        image_rid: u32,
        sx: f64,
        sy: f64,
        s_width: f64,
        s_height: f64,
        dx: f64,
        dy: f64,
        d_width: f64,
        d_height: f64,
    },
    Arc {
        x: Number<'gc>,
        y: Number<'gc>,
        radius: Number<'gc>,
        start_angle: Number<'gc>,
        end_angle: Number<'gc>,
        counter_clockwise: bool,
    },
    ArcTo {
        x1: Number<'gc>,
        y1: Number<'gc>,
        x2: Number<'gc>,
        y2: Number<'gc>,
        radius: Number<'gc>,
    },
    BeginPath,
    BezierCurveTo {
        cp1x: Number<'gc>,
        cp1y: Number<'gc>,
        cp2x: Number<'gc>,
        cp2y: Number<'gc>,
        x: Number<'gc>,
        y: Number<'gc>,
    },
    ClearRect {
        x: Number<'gc>,
        y: Number<'gc>,
        width: Number<'gc>,
        height: Number<'gc>,
    },
    ClosePath,
    CreateConicGradient {
        start_angle: Number<'gc>,
        x: Number<'gc>,
        y: Number<'gc>,
    },
    CreateLinearGradient {
        x0: Number<'gc>,
        y0: Number<'gc>,
        x1: Number<'gc>,
        y1: Number<'gc>,
    },
    CreateRadialGradient {
        x0: Number<'gc>,
        y0: Number<'gc>,
        r0: Number<'gc>,
        x1: Number<'gc>,
        y1: Number<'gc>,
        r1: Number<'gc>,
    },
    Ellipse {
        x: Number<'gc>,
        y: Number<'gc>,
        radius_x: Number<'gc>,
        radius_y: Number<'gc>,
        rotation: Number<'gc>,
        start_angle: Number<'gc>,
        end_angle: Number<'gc>,
        counter_clockwise: bool,
    },
    Fill,
    FillRect {
        x: Number<'gc>,
        y: Number<'gc>,
        width: Number<'gc>,
        height: Number<'gc>,
    },
    LineTo {
        x: Number<'gc>,
        y: Number<'gc>,
    },
    MoveTo {
        x: Number<'gc>,
        y: Number<'gc>,
    },
    QuadraticCurveTo {
        cpx: Number<'gc>,
        cpy: Number<'gc>,
        x: Number<'gc>,
        y: Number<'gc>,
    },
    Rect {
        x: Number<'gc>,
        y: Number<'gc>,
        width: Number<'gc>,
        height: Number<'gc>,
    },
    Reset,
    ResetTransform,
    Restore,
    Rotate {
        angle: Number<'gc>,
    },
    RoundRect {
        x: Number<'gc>,
        y: Number<'gc>,
        width: Number<'gc>,
        height: Number<'gc>,
        radius: Number<'gc>,
    },
    Save,
    Scale {
        x: Number<'gc>,
        y: Number<'gc>,
    },
    SetLineDash {
        segments: Vec<Number<'gc>>,
    },
    SetTransform {
        a: Number<'gc>,
        b: Number<'gc>,
        c: Number<'gc>,
        d: Number<'gc>,
        e: Number<'gc>,
        f: Number<'gc>,
    },
    Stroke,
    StrokeRect {
        x: Number<'gc>,
        y: Number<'gc>,
        width: Number<'gc>,
        height: Number<'gc>,
    },
    Transform {
        a: Number<'gc>,
        b: Number<'gc>,
        c: Number<'gc>,
        d: Number<'gc>,
        e: Number<'gc>,
        f: Number<'gc>,
    },
    Translate {
        x: Number<'gc>,
        y: Number<'gc>,
    },
    Clip {
        path: Vec<Point>,
    },
    SetStrokeStyle(crate::ext::canvas::FillStyle),
}

// Internal op to create an arc on a canvas by Rid
pub fn internal_canvas_arc<'gc>(
    agent: &mut Agent,
    _this: Value,
    args: ArgumentsList,
    mut gc: GcScope<'gc, '_>,
) -> JsResult<'gc, Value<'gc>> {
    let rid_val = args.get(0).to_int32(agent, gc.reborrow()).unbind().unwrap() as u32;
    let rid = Rid::from_index(rid_val);
    let x = args
        .get(1)
        .to_number(agent, gc.reborrow())
        .unbind()
        .unwrap();
    let y = args
        .get(2)
        .to_number(agent, gc.reborrow())
        .unbind()
        .unwrap();
    let radius = args
        .get(3)
        .to_number(agent, gc.reborrow())
        .unbind()
        .unwrap();
    let start_angle = args
        .get(4)
        .to_number(agent, gc.reborrow())
        .unbind()
        .unwrap();
    let end_angle = args
        .get(5)
        .to_number(agent, gc.reborrow())
        .unbind()
        .unwrap();
    let counter_clockwise = args.get(6).is_true();

    let host_data = agent
        .get_host_data()
        .downcast_ref::<HostData<RuntimeMacroTask>>()
        .unwrap();
    let mut storage = host_data.storage.borrow_mut();
    let res: &mut CanvasResources = storage.get_mut().unwrap();

    // Convert Nova VM Numbers to f64 for renderer
    let x_f64 = x.into_f64(agent);
    let y_f64 = y.into_f64(agent);
    let radius_f64 = radius.into_f64(agent);
    let start_angle_f64 = start_angle.into_f64(agent);
    let end_angle_f64 = end_angle.into_f64(agent);

    // Tessellate arc and add to current path
    let mut data = res.canvases.get_mut(rid).unwrap();
    tessellate_arc_to_path(
        &mut data.current_path,
        x_f64,
        y_f64,
        radius_f64,
        start_angle_f64,
        end_angle_f64,
        counter_clockwise,
    );

    data.commands.push(CanvasCommand::Arc {
        x,
        y,
        radius,
        start_angle,
        end_angle,
        counter_clockwise,
    });

    Ok(Value::Undefined)
}

// Internal op to create an arc on a canvas by Rid, with additional parameters
pub fn internal_canvas_arc_to<'gc>(
    agent: &mut Agent,
    _this: Value,
    args: ArgumentsList,
    mut gc: GcScope<'gc, '_>,
) -> JsResult<'gc, Value<'gc>> {
    let rid_val = args.get(0).to_int32(agent, gc.reborrow()).unbind().unwrap() as u32;
    let rid = Rid::from_index(rid_val);
    let x1 = args
        .get(1)
        .to_number(agent, gc.reborrow())
        .unbind()
        .unwrap();
    let y1 = args
        .get(2)
        .to_number(agent, gc.reborrow())
        .unbind()
        .unwrap();
    let x2 = args
        .get(3)
        .to_number(agent, gc.reborrow())
        .unbind()
        .unwrap();
    let y2 = args
        .get(4)
        .to_number(agent, gc.reborrow())
        .unbind()
        .unwrap();
    let radius = args
        .get(5)
        .to_number(agent, gc.reborrow())
        .unbind()
        .unwrap();

    let host_data = agent
        .get_host_data()
        .downcast_ref::<HostData<RuntimeMacroTask>>()
        .unwrap();
    let mut storage = host_data.storage.borrow_mut();
    let res: &mut CanvasResources = storage.get_mut().unwrap();
    let mut data = res.canvases.get_mut(rid).unwrap();

    data.commands.push(CanvasCommand::ArcTo {
        x1,
        y1,
        x2,
        y2,
        radius,
    });

    Ok(Value::Undefined)
}

/// Internal op to begin a path on a canvas by Rid
pub fn internal_canvas_begin_path<'gc>(
    agent: &mut Agent,
    _this: Value,
    args: ArgumentsList,
    mut gc: GcScope<'gc, '_>,
) -> JsResult<'gc, Value<'gc>> {
    let rid_val = args.get(0).to_int32(agent, gc.reborrow()).unbind().unwrap() as u32;
    let rid = Rid::from_index(rid_val);
    let host_data = agent
        .get_host_data()
        .downcast_ref::<HostData<RuntimeMacroTask>>()
        .unwrap();
    let mut storage = host_data.storage.borrow_mut();
    let res: &mut CanvasResources = storage.get_mut().unwrap();
    let mut data = res.canvases.get_mut(rid).unwrap();
    // Reset the in-flight path state. Without this, `data.current_path`
    // accumulates across every shape ever drawn on the canvas and the next
    // fill/stroke renders every point in the history as one giant polygon.
    data.current_path.clear();
    data.subpath_starts.clear();
    data.path_started = false;
    data.commands.push(CanvasCommand::BeginPath);
    Ok(Value::Undefined)
}

/// Internal op to create a bezier curve on a canvas by Rid
pub fn internal_canvas_bezier_curve_to<'gc>(
    agent: &mut Agent,
    _this: Value,
    args: ArgumentsList,
    mut gc: GcScope<'gc, '_>,
) -> JsResult<'gc, Value<'gc>> {
    let rid_val = args.get(0).to_int32(agent, gc.reborrow()).unbind().unwrap() as u32;
    let rid = Rid::from_index(rid_val);
    let cp1x = args
        .get(1)
        .to_number(agent, gc.reborrow())
        .unbind()
        .unwrap();
    let cp1y = args
        .get(2)
        .to_number(agent, gc.reborrow())
        .unbind()
        .unwrap();
    let cp2x = args
        .get(3)
        .to_number(agent, gc.reborrow())
        .unbind()
        .unwrap();
    let cp2y = args
        .get(4)
        .to_number(agent, gc.reborrow())
        .unbind()
        .unwrap();
    let x = args
        .get(5)
        .to_number(agent, gc.reborrow())
        .unbind()
        .unwrap();
    let y = args
        .get(6)
        .to_number(agent, gc.reborrow())
        .unbind()
        .unwrap();
    let host_data = agent
        .get_host_data()
        .downcast_ref::<HostData<RuntimeMacroTask>>()
        .unwrap();
    let mut storage = host_data.storage.borrow_mut();
    let res: &mut CanvasResources = storage.get_mut().unwrap();
    let mut data = res.canvases.get_mut(rid).unwrap();

    // Convert Nova VM Numbers to f64 for renderer
    let cp1x_f64 = cp1x.into_f64(agent);
    let cp1y_f64 = cp1y.into_f64(agent);
    let cp2x_f64 = cp2x.into_f64(agent);
    let cp2y_f64 = cp2y.into_f64(agent);
    let x_f64 = x.into_f64(agent);
    let y_f64 = y.into_f64(agent);

    // Get the last point as starting point for Bezier curve
    if let Some(last_point) = data.current_path.last().cloned() {
        let start = last_point;
        let cp1 = crate::ext::canvas::renderer::Point {
            x: cp1x_f64,
            y: cp1y_f64,
        };
        let cp2 = crate::ext::canvas::renderer::Point {
            x: cp2x_f64,
            y: cp2y_f64,
        };
        let end = crate::ext::canvas::renderer::Point { x: x_f64, y: y_f64 };

        tessellate_bezier_curve_to_path(&mut data.current_path, start, cp1, cp2, end);
    } else {
        // If no previous point, just add the end point
        data.current_path
            .push(crate::ext::canvas::renderer::Point { x: x_f64, y: y_f64 });
    }

    data.commands.push(CanvasCommand::BezierCurveTo {
        cp1x,
        cp1y,
        cp2x,
        cp2y,
        x,
        y,
    });
    Ok(Value::Undefined)
}

/// Internal op to clear a rectangle on a canvas by Rid
pub fn internal_canvas_clear_rect<'gc>(
    agent: &mut Agent,
    _this: Value,
    args: ArgumentsList,
    mut gc: GcScope<'gc, '_>,
) -> JsResult<'gc, Value<'gc>> {
    let rid_val = args.get(0).to_int32(agent, gc.reborrow()).unbind().unwrap() as u32;
    let rid = Rid::from_index(rid_val);
    let x = args
        .get(1)
        .to_number(agent, gc.reborrow())
        .unbind()
        .unwrap();
    let y = args
        .get(2)
        .to_number(agent, gc.reborrow())
        .unbind()
        .unwrap();
    let width = args
        .get(3)
        .to_number(agent, gc.reborrow())
        .unbind()
        .unwrap();
    let height = args
        .get(4)
        .to_number(agent, gc.reborrow())
        .unbind()
        .unwrap();
    let host_data = agent
        .get_host_data()
        .downcast_ref::<HostData<RuntimeMacroTask>>()
        .unwrap();
    let mut storage = host_data.storage.borrow_mut();
    let res: &mut CanvasResources = storage.get_mut().unwrap();

    // Try to use GPU renderer if available
    if let Some(mut renderer) = res.renderers.get_mut(rid) {
        // Convert Nova VM Numbers to f64
        let x_f64 = x.into_f64(agent);
        let y_f64 = y.into_f64(agent);
        let width_f64 = width.into_f64(agent);
        let height_f64 = height.into_f64(agent);

        // Create a rect for clearing (transparent/background color)
        let clear_rect = crate::ext::canvas::renderer::Rect {
            start: crate::ext::canvas::renderer::Point { x: x_f64, y: y_f64 },
            end: crate::ext::canvas::renderer::Point {
                x: x_f64 + width_f64,
                y: y_f64 + height_f64,
            },
        };

        // TODO: Implement proper clear operation in renderer
        // For now, render a transparent rectangle
        renderer.render_rect(
            clear_rect,
            &RenderState {
                fill_style: FillStyle::Color {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: 0.0,
                },
                global_alpha: 1.0,
                transform: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
                line_cap: LineCap::default(),
                line_join: LineJoin::default(),
                miter_limit: 10.0,
                shadow_blur: 0.0,
                shadow_color: FillStyle::Color {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: 0.0,
                },
                shadow_offset_x: 0.0,
                shadow_offset_y: 0.0,
                composite_operation: CompositeOperation::default(),
                clip_path: None,
            },
        );
    } else {
        // Fallback to command buffering if no renderer
        let mut data = res.canvases.get_mut(rid).unwrap();
        data.commands.push(CanvasCommand::ClearRect {
            x,
            y,
            width,
            height,
        });
    }

    Ok(Value::Undefined)
}

/// Internal op to close a path on a canvas by Rid
pub fn internal_canvas_close_path<'gc>(
    agent: &mut Agent,
    _this: Value,
    args: ArgumentsList,
    mut gc: GcScope<'gc, '_>,
) -> JsResult<'gc, Value<'gc>> {
    let rid_val = args.get(0).to_int32(agent, gc.reborrow()).unbind().unwrap() as u32;
    let rid = Rid::from_index(rid_val);
    let host_data = agent
        .get_host_data()
        .downcast_ref::<HostData<RuntimeMacroTask>>()
        .unwrap();
    let mut storage = host_data.storage.borrow_mut();
    let res: &mut CanvasResources = storage.get_mut().unwrap();
    let mut data = res.canvases.get_mut(rid).unwrap();

    // Close the CURRENT subpath by appending a copy of its first point.
    // The GPU render path reads data.current_path directly, so without this
    // the closing edge of a stroked polygon is never drawn.
    let subpath_start = data.subpath_starts.last().copied().unwrap_or(0);
    if subpath_start < data.current_path.len() {
        let first = data.current_path[subpath_start].clone();
        data.current_path.push(first);
    }

    data.commands.push(CanvasCommand::ClosePath);
    Ok(Value::Undefined)
}

/// Internal op to fill a rectangle on a canvas by Rid
pub fn internal_canvas_fill_rect<'gc>(
    agent: &mut Agent,
    _this: Value,
    args: ArgumentsList,
    mut gc: GcScope<'gc, '_>,
) -> JsResult<'gc, Value<'gc>> {
    let rid_val = args.get(0).to_int32(agent, gc.reborrow()).unbind().unwrap() as u32;
    let rid = Rid::from_index(rid_val);
    let x = args
        .get(1)
        .to_number(agent, gc.reborrow())
        .unbind()
        .unwrap();
    let y = args
        .get(2)
        .to_number(agent, gc.reborrow())
        .unbind()
        .unwrap();
    let width = args
        .get(3)
        .to_number(agent, gc.reborrow())
        .unbind()
        .unwrap();
    let height = args
        .get(4)
        .to_number(agent, gc.reborrow())
        .unbind()
        .unwrap();
    let host_data = agent
        .get_host_data()
        .downcast_ref::<HostData<RuntimeMacroTask>>()
        .unwrap();
    let mut storage = host_data.storage.borrow_mut();
    let res: &mut CanvasResources = storage.get_mut().unwrap();

    // Try to render directly with GPU if renderer exists
    let has_renderer = {
        // Check in a separate scope to avoid borrow conflicts
        res.renderers.get_mut(rid).is_some()
    };

    if has_renderer {
        // Convert Nova VM Number to f64 for GPU renderer
        // Use into_f64() method that Nova VM provides
        let x_val = x.into_f64(agent);
        let y_val = y.into_f64(agent);
        let width_val = width.into_f64(agent);
        let height_val = height.into_f64(agent);

        let rect = Rect {
            start: Point { x: x_val, y: y_val },
            end: Point {
                x: x_val + width_val,
                y: y_val + height_val,
            },
        };

        let data = res.canvases.get(rid).unwrap();

        res.renderers.get_mut(rid).unwrap().render_rect(
            rect,
            &RenderState {
                fill_style: data.fill_style,
                global_alpha: data.global_alpha,
                transform: data.transform,
                line_cap: data.line_cap,
                line_join: data.line_join,
                miter_limit: data.miter_limit,
                shadow_blur: data.shadow_blur,
                shadow_color: data.shadow_color,
                shadow_offset_x: data.shadow_offset_x,
                shadow_offset_y: data.shadow_offset_y,
                composite_operation: data.composite_operation,
                clip_path: None,
            },
        );
    } else {
        // Fallback to command storage if no renderer
        let mut data = res.canvases.get_mut(rid).unwrap();
        data.commands.push(CanvasCommand::FillRect {
            x,
            y,
            width,
            height,
        });
    }

    Ok(Value::Undefined)
}

/// Internal op to move to a point on a canvas by Rid
pub fn internal_canvas_move_to<'gc>(
    agent: &mut Agent,
    _this: Value,
    args: ArgumentsList,
    mut gc: GcScope<'gc, '_>,
) -> JsResult<'gc, Value<'gc>> {
    let rid_val = args.get(0).to_int32(agent, gc.reborrow()).unbind().unwrap() as u32;
    let rid = Rid::from_index(rid_val);
    let x = args
        .get(1)
        .to_number(agent, gc.reborrow())
        .unbind()
        .unwrap();
    let y = args
        .get(2)
        .to_number(agent, gc.reborrow())
        .unbind()
        .unwrap();

    let host_data = agent
        .get_host_data()
        .downcast_ref::<HostData<RuntimeMacroTask>>()
        .unwrap();
    let mut storage = host_data.storage.borrow_mut();
    let res: &mut CanvasResources = storage.get_mut().unwrap();
    let mut data = res.canvases.get_mut(rid).unwrap();

    // Convert Nova VM Numbers to f64 for renderer
    let x_f64 = x.into_f64(agent);
    let y_f64 = y.into_f64(agent);

    // Start a new subpath in the current path.
    //
    // Canvas 2D paths can contain multiple disconnected subpaths. Record
    // the index at which this subpath begins so `fill`/`stroke` can render
    // each subpath independently instead of drawing one giant polygon that
    // zig-zags between subpath endpoints.
    let start_idx = data.current_path.len();
    data.subpath_starts.push(start_idx);
    data.current_path
        .push(crate::ext::canvas::renderer::Point { x: x_f64, y: y_f64 });
    data.path_started = true;

    // Also store as command for fallback
    data.commands.push(CanvasCommand::MoveTo { x, y });

    Ok(Value::Undefined)
}

/// Internal op to draw a line to a point on a canvas by Rid
pub fn internal_canvas_line_to<'gc>(
    agent: &mut Agent,
    _this: Value,
    args: ArgumentsList,
    mut gc: GcScope<'gc, '_>,
) -> JsResult<'gc, Value<'gc>> {
    let rid_val = args.get(0).to_int32(agent, gc.reborrow()).unbind().unwrap() as u32;
    let rid = Rid::from_index(rid_val);
    let x = args
        .get(1)
        .to_number(agent, gc.reborrow())
        .unbind()
        .unwrap();
    let y = args
        .get(2)
        .to_number(agent, gc.reborrow())
        .unbind()
        .unwrap();

    let host_data = agent
        .get_host_data()
        .downcast_ref::<HostData<RuntimeMacroTask>>()
        .unwrap();
    let mut storage = host_data.storage.borrow_mut();
    let res: &mut CanvasResources = storage.get_mut().unwrap();
    let mut data = res.canvases.get_mut(rid).unwrap();

    // Convert Nova VM Numbers to f64 for renderer
    let x_f64 = x.into_f64(agent);
    let y_f64 = y.into_f64(agent);

    // Add point to current path if path is started
    if data.path_started {
        data.current_path
            .push(crate::ext::canvas::renderer::Point { x: x_f64, y: y_f64 });
    }

    // Also store as command for fallback
    data.commands.push(CanvasCommand::LineTo { x, y });

    Ok(Value::Undefined)
}

/// Internal op to fill the current path on a canvas by Rid
pub fn internal_canvas_fill<'gc>(
    agent: &mut Agent,
    _this: Value,
    args: ArgumentsList,
    mut gc: GcScope<'gc, '_>,
) -> JsResult<'gc, Value<'gc>> {
    let rid_val = args.get(0).to_int32(agent, gc.reborrow()).unbind().unwrap() as u32;
    let rid = Rid::from_index(rid_val);

    let host_data = agent
        .get_host_data()
        .downcast_ref::<HostData<RuntimeMacroTask>>()
        .unwrap();
    let mut storage = host_data.storage.borrow_mut();
    let res: &mut CanvasResources = storage.get_mut().unwrap();

    // Try to use GPU renderer if available and path has points
    if let Some(mut renderer) = res.renderers.get_mut(rid) {
        let data = res.canvases.get(rid).unwrap();

        let state = RenderState {
            fill_style: data.fill_style,
            global_alpha: data.global_alpha,
            transform: data.transform,
            line_cap: data.line_cap,
            line_join: data.line_join,
            miter_limit: data.miter_limit,
            shadow_blur: data.shadow_blur,
            shadow_color: data.shadow_color,
            shadow_offset_x: data.shadow_offset_x,
            shadow_offset_y: data.shadow_offset_y,
            composite_operation: data.composite_operation,
            clip_path: None,
        };

        // Render each subpath as its own polygon so compound paths (e.g.
        // two arcs separated by moveTo) don't collapse into one polygon
        // with a stray edge joining the subpaths.
        for (start, end) in subpath_ranges(&data.current_path, &data.subpath_starts) {
            if end - start >= 3 {
                let subpath = data.current_path[start..end].to_vec();
                renderer.render_polygon(subpath, &state);
            }
        }
    } else {
        // Fallback to command storage if no renderer
        let mut data = res.canvases.get_mut(rid).unwrap();
        data.commands.push(CanvasCommand::Fill);
    }

    Ok(Value::Undefined)
}

/// Internal op to stroke the current path on a canvas by Rid
pub fn internal_canvas_stroke<'gc>(
    agent: &mut Agent,
    _this: Value,
    args: ArgumentsList,
    mut gc: GcScope<'gc, '_>,
) -> JsResult<'gc, Value<'gc>> {
    let rid_val = args.get(0).to_int32(agent, gc.reborrow()).unbind().unwrap() as u32;
    let rid = Rid::from_index(rid_val);

    let host_data = agent
        .get_host_data()
        .downcast_ref::<HostData<RuntimeMacroTask>>()
        .unwrap();
    let mut storage = host_data.storage.borrow_mut();
    let res: &mut CanvasResources = storage.get_mut().unwrap();

    // Try to use GPU renderer if available and path has points
    if let Some(mut renderer) = res.renderers.get_mut(rid) {
        let data = res.canvases.get(rid).unwrap();

        let state = RenderState {
            fill_style: data.stroke_style,
            global_alpha: data.global_alpha,
            transform: data.transform,
            line_cap: data.line_cap,
            line_join: data.line_join,
            miter_limit: data.miter_limit,
            shadow_blur: data.shadow_blur,
            shadow_color: data.shadow_color,
            shadow_offset_x: data.shadow_offset_x,
            shadow_offset_y: data.shadow_offset_y,
            composite_operation: data.composite_operation,
            clip_path: None,
        };

        // Stroke each subpath independently so the renderer doesn't draw a
        // phantom segment joining the end of subpath A to the start of
        // subpath B. Each subpath's stroke is emitted as a triangle list
        // (6 verts per visible segment) so fan triangulation doesn't
        // collapse multi-segment strokes into a fan.
        for (start, end) in subpath_ranges(&data.current_path, &data.subpath_starts) {
            if end - start >= 2 {
                let subpath = &data.current_path[start..end];
                let stroke_triangles = generate_stroke_path(
                    subpath,
                    data.line_width,
                    &data.line_dash,
                    data.line_dash_offset,
                    data.line_cap,
                    data.line_join,
                    data.miter_limit,
                );
                if !stroke_triangles.is_empty() {
                    renderer.render_triangles(stroke_triangles, &state);
                }
            }
        }
    } else {
        // Fallback to command storage if no renderer
        let mut data = res.canvases.get_mut(rid).unwrap();
        data.commands.push(CanvasCommand::Stroke);
    }

    Ok(Value::Undefined)
}

/// Internal op to create a rectangle path on a canvas by Rid
pub fn internal_canvas_rect<'gc>(
    agent: &mut Agent,
    _this: Value,
    args: ArgumentsList,
    mut gc: GcScope<'gc, '_>,
) -> JsResult<'gc, Value<'gc>> {
    let rid_val = args.get(0).to_int32(agent, gc.reborrow()).unbind().unwrap() as u32;
    let rid = Rid::from_index(rid_val);
    let x = args
        .get(1)
        .to_number(agent, gc.reborrow())
        .unbind()
        .unwrap();
    let y = args
        .get(2)
        .to_number(agent, gc.reborrow())
        .unbind()
        .unwrap();
    let width = args
        .get(3)
        .to_number(agent, gc.reborrow())
        .unbind()
        .unwrap();
    let height = args
        .get(4)
        .to_number(agent, gc.reborrow())
        .unbind()
        .unwrap();

    let host_data = agent
        .get_host_data()
        .downcast_ref::<HostData<RuntimeMacroTask>>()
        .unwrap();
    let mut storage = host_data.storage.borrow_mut();
    let res: &mut CanvasResources = storage.get_mut().unwrap();
    let mut data = res.canvases.get_mut(rid).unwrap();

    // Convert Nova VM Numbers to f64 for renderer
    let x_f64 = x.into_f64(agent);
    let y_f64 = y.into_f64(agent);
    let width_f64 = width.into_f64(agent);
    let height_f64 = height.into_f64(agent);

    // Per the Canvas 2D spec, `rect` creates a new implicit subpath.
    // Record the subpath start so a preceding moveTo/lineTo block doesn't
    // get zig-zag-joined to these four corners under fan triangulation.
    let subpath_start = data.current_path.len();
    data.subpath_starts.push(subpath_start);

    // Add rectangle to current path as four corners, plus a fifth point
    // duplicating the first. Per the HTML Canvas 2D spec, `rect` emits a
    // CLOSED subpath — the duplicate closes it so `stroke()` walks all four
    // edges instead of stopping at the third segment.
    data.current_path
        .push(crate::ext::canvas::renderer::Point { x: x_f64, y: y_f64 });
    data.current_path.push(crate::ext::canvas::renderer::Point {
        x: x_f64 + width_f64,
        y: y_f64,
    });
    data.current_path.push(crate::ext::canvas::renderer::Point {
        x: x_f64 + width_f64,
        y: y_f64 + height_f64,
    });
    data.current_path.push(crate::ext::canvas::renderer::Point {
        x: x_f64,
        y: y_f64 + height_f64,
    });
    data.current_path
        .push(crate::ext::canvas::renderer::Point { x: x_f64, y: y_f64 });
    data.path_started = true;

    // Also store as command for fallback
    data.commands.push(CanvasCommand::Rect {
        x,
        y,
        width,
        height,
    });

    Ok(Value::Undefined)
}

/// Internal op to set the line width for stroking on a canvas by Rid
pub fn internal_canvas_set_line_width<'gc>(
    agent: &mut Agent,
    _this: Value,
    args: ArgumentsList,
    mut gc: GcScope<'gc, '_>,
) -> JsResult<'gc, Value<'gc>> {
    let rid_val = args.get(0).to_int32(agent, gc.reborrow()).unbind().unwrap() as u32;
    let rid = Rid::from_index(rid_val);
    let line_width = args
        .get(1)
        .to_number(agent, gc.reborrow())
        .unbind()
        .unwrap()
        .into_f64(agent);

    let host_data = agent
        .get_host_data()
        .downcast_ref::<HostData<RuntimeMacroTask>>()
        .unwrap();
    let mut storage = host_data.storage.borrow_mut();
    let res: &mut CanvasResources = storage.get_mut().unwrap();

    let mut data = res.canvases.get_mut(rid).unwrap();
    data.line_width = line_width;

    Ok(Value::Undefined)
}

/// Internal op to set the stroke style for a canvas by Rid
pub fn internal_canvas_set_stroke_style<'gc>(
    agent: &mut Agent,
    _this: Value,
    args: ArgumentsList,
    mut gc: GcScope<'gc, '_>,
) -> JsResult<'gc, Value<'gc>> {
    let rid_val = args.get(0).to_int32(agent, gc.reborrow()).unbind().unwrap() as u32;
    let rid = Rid::from_index(rid_val);
    let style = args.get(1);

    // If the style is a number it's a gradient or pattern rid; if it's a
    // string it's a CSS color. Resolve each path separately so that
    // `ctx.strokeStyle = gradient` actually propagates to the canvas's
    // `stroke_style` field (matches fillStyle's behavior).
    let is_number = style.is_number();
    let fill_rid = if is_number {
        style.to_uint32(agent, gc.reborrow()).unbind().ok()
    } else {
        None
    };
    let style_string = if style.is_string() {
        Some(style.to_string(agent, gc.reborrow()).unbind().unwrap())
    } else {
        None
    };

    let host_data = agent
        .get_host_data()
        .downcast_ref::<HostData<RuntimeMacroTask>>()
        .unwrap();
    let mut storage = host_data.storage.borrow_mut();
    let res: &mut CanvasResources = storage.get_mut().unwrap();

    let resolved: Option<FillStyle> = if let Some(rid_u32) = fill_rid {
        res.fill_styles.get(Rid::from_index(rid_u32))
    } else if let Some(style_str_obj) = &style_string {
        let style_str = style_str_obj
            .as_str(agent)
            .expect("String is not valid UTF-8");
        FillStyle::from_css_color(style_str).ok()
    } else {
        None
    };

    if let Some(parsed_style) = resolved {
        let mut data = res.canvases.get_mut(rid).unwrap();
        data.stroke_style = parsed_style.clone();
        data.commands
            .push(CanvasCommand::SetStrokeStyle(parsed_style));
    }
    Ok(Value::Undefined)
}

/// Internal op to create a quadratic curve on a canvas by Rid
pub fn internal_canvas_quadratic_curve_to<'gc>(
    agent: &mut Agent,
    _this: Value,
    args: ArgumentsList,
    mut gc: GcScope<'gc, '_>,
) -> JsResult<'gc, Value<'gc>> {
    let rid_val = args.get(0).to_int32(agent, gc.reborrow()).unbind().unwrap() as u32;
    let rid = Rid::from_index(rid_val);
    let cpx = args
        .get(1)
        .to_number(agent, gc.reborrow())
        .unbind()
        .unwrap();
    let cpy = args
        .get(2)
        .to_number(agent, gc.reborrow())
        .unbind()
        .unwrap();
    let x = args
        .get(3)
        .to_number(agent, gc.reborrow())
        .unbind()
        .unwrap();
    let y = args
        .get(4)
        .to_number(agent, gc.reborrow())
        .unbind()
        .unwrap();

    let host_data = agent
        .get_host_data()
        .downcast_ref::<HostData<RuntimeMacroTask>>()
        .unwrap();
    let mut storage = host_data.storage.borrow_mut();
    let res: &mut CanvasResources = storage.get_mut().unwrap();
    let mut data = res.canvases.get_mut(rid).unwrap();

    // Convert Nova VM Numbers to f64 for renderer
    let cpx_f64 = cpx.into_f64(agent);
    let cpy_f64 = cpy.into_f64(agent);
    let x_f64 = x.into_f64(agent);
    let y_f64 = y.into_f64(agent);

    // Get the last point as starting point for quadratic curve
    if let Some(last_point) = data.current_path.last().cloned() {
        let start = last_point;
        let control = crate::ext::canvas::renderer::Point {
            x: cpx_f64,
            y: cpy_f64,
        };
        let end = crate::ext::canvas::renderer::Point { x: x_f64, y: y_f64 };

        tessellate_quadratic_curve_to_path(&mut data.current_path, start, control, end);
    } else {
        // If no previous point, just add the end point
        data.current_path
            .push(crate::ext::canvas::renderer::Point { x: x_f64, y: y_f64 });
    }

    data.commands
        .push(CanvasCommand::QuadraticCurveTo { cpx, cpy, x, y });

    Ok(Value::Undefined)
}

/// Internal op to create an ellipse on a canvas by Rid
pub fn internal_canvas_ellipse<'gc>(
    agent: &mut Agent,
    _this: Value,
    args: ArgumentsList,
    mut gc: GcScope<'gc, '_>,
) -> JsResult<'gc, Value<'gc>> {
    let rid_val = args.get(0).to_int32(agent, gc.reborrow()).unbind().unwrap() as u32;
    let rid = Rid::from_index(rid_val);
    let x = args
        .get(1)
        .to_number(agent, gc.reborrow())
        .unbind()
        .unwrap();
    let y = args
        .get(2)
        .to_number(agent, gc.reborrow())
        .unbind()
        .unwrap();
    let radius_x = args
        .get(3)
        .to_number(agent, gc.reborrow())
        .unbind()
        .unwrap();
    let radius_y = args
        .get(4)
        .to_number(agent, gc.reborrow())
        .unbind()
        .unwrap();
    let rotation = args
        .get(5)
        .to_number(agent, gc.reborrow())
        .unbind()
        .unwrap();
    let start_angle = args
        .get(6)
        .to_number(agent, gc.reborrow())
        .unbind()
        .unwrap();
    let end_angle = args
        .get(7)
        .to_number(agent, gc.reborrow())
        .unbind()
        .unwrap();
    let counter_clockwise = {
        if args.get(8).is_number() || args.get(8).is_boolean() {
            let v = args.get(8).to_int32(agent, gc.reborrow()).unwrap_or(0);
            v != 0
        } else {
            false
        }
    };

    let host_data = agent
        .get_host_data()
        .downcast_ref::<HostData<RuntimeMacroTask>>()
        .unwrap();
    let mut storage = host_data.storage.borrow_mut();
    let res: &mut CanvasResources = storage.get_mut().unwrap();
    let mut data = res.canvases.get_mut(rid).unwrap();

    // Convert Nova VM Numbers to f64 for renderer
    let x_f64 = x.into_f64(agent);
    let y_f64 = y.into_f64(agent);
    let radius_x_f64 = radius_x.into_f64(agent);
    let radius_y_f64 = radius_y.into_f64(agent);
    let rotation_f64 = rotation.into_f64(agent);
    let start_angle_f64 = start_angle.into_f64(agent);
    let end_angle_f64 = end_angle.into_f64(agent);

    // Tessellate ellipse to current path
    tessellate_ellipse_to_path(
        &mut data.current_path,
        x_f64,
        y_f64,
        radius_x_f64,
        radius_y_f64,
        rotation_f64,
        start_angle_f64,
        end_angle_f64,
        counter_clockwise,
    );

    data.commands.push(CanvasCommand::Ellipse {
        x,
        y,
        radius_x,
        radius_y,
        rotation,
        start_angle,
        end_angle,
        counter_clockwise,
    });

    Ok(Value::Undefined)
}

/// Internal op to create a rounded rectangle on a canvas by Rid
pub fn internal_canvas_round_rect<'gc>(
    agent: &mut Agent,
    _this: Value,
    args: ArgumentsList,
    mut gc: GcScope<'gc, '_>,
) -> JsResult<'gc, Value<'gc>> {
    let rid_val = args.get(0).to_int32(agent, gc.reborrow()).unbind().unwrap() as u32;
    let rid = Rid::from_index(rid_val);
    let x = args
        .get(1)
        .to_number(agent, gc.reborrow())
        .unbind()
        .unwrap();
    let y = args
        .get(2)
        .to_number(agent, gc.reborrow())
        .unbind()
        .unwrap();
    let width = args
        .get(3)
        .to_number(agent, gc.reborrow())
        .unbind()
        .unwrap();
    let height = args
        .get(4)
        .to_number(agent, gc.reborrow())
        .unbind()
        .unwrap();
    // Read four per-corner radii (TL, TR, BR, BL) from args 5..=8.
    // Missing args default to 0, preserving a sharp corner, so callers
    // passing only the old 6-arg form still produce a plain rectangle.
    let tl_num = args
        .get(5)
        .to_number(agent, gc.reborrow())
        .unbind()
        .unwrap_or_else(|_| Number::from(0));
    let tr_num = args
        .get(6)
        .to_number(agent, gc.reborrow())
        .unbind()
        .unwrap_or_else(|_| Number::from(0));
    let br_num = args
        .get(7)
        .to_number(agent, gc.reborrow())
        .unbind()
        .unwrap_or_else(|_| Number::from(0));
    let bl_num = args
        .get(8)
        .to_number(agent, gc.reborrow())
        .unbind()
        .unwrap_or_else(|_| Number::from(0));
    let tl_f64 = tl_num.into_f64(agent);
    let tr_f64 = tr_num.into_f64(agent);
    let br_f64 = br_num.into_f64(agent);
    let bl_f64 = bl_num.into_f64(agent);

    // Pick the largest corner for the fallback replay Command (the
    // command-replay path doesn't yet carry per-corner radii; this is
    // a best-effort for the no-GPU fallback only).
    let radius_for_command = if tl_f64 >= tr_f64 && tl_f64 >= br_f64 && tl_f64 >= bl_f64 {
        tl_num
    } else if tr_f64 >= br_f64 && tr_f64 >= bl_f64 {
        tr_num
    } else if br_f64 >= bl_f64 {
        br_num
    } else {
        bl_num
    };

    let host_data = agent
        .get_host_data()
        .downcast_ref::<HostData<RuntimeMacroTask>>()
        .unwrap();
    let mut storage = host_data.storage.borrow_mut();
    let res: &mut CanvasResources = storage.get_mut().unwrap();
    let mut data = res.canvases.get_mut(rid).unwrap();

    let x_f64 = x.into_f64(agent);
    let y_f64 = y.into_f64(agent);
    let width_f64 = width.into_f64(agent);
    let height_f64 = height.into_f64(agent);

    tessellate_rounded_rect_to_path(
        &mut data.current_path,
        x_f64,
        y_f64,
        width_f64,
        height_f64,
        [tl_f64, tr_f64, br_f64, bl_f64],
    );

    data.commands.push(CanvasCommand::RoundRect {
        x,
        y,
        width,
        height,
        radius: radius_for_command,
    });

    Ok(Value::Undefined)
}

/// Internal op to save the current canvas state
pub fn internal_canvas_save<'gc>(
    agent: &mut Agent,
    _this: Value,
    args: ArgumentsList,
    mut gc: GcScope<'gc, '_>,
) -> JsResult<'gc, Value<'gc>> {
    let rid_val = args.get(0).to_int32(agent, gc.reborrow()).unbind().unwrap() as u32;
    let rid = Rid::from_index(rid_val);

    let host_data = agent
        .get_host_data()
        .downcast_ref::<HostData<RuntimeMacroTask>>()
        .unwrap();
    let mut storage = host_data.storage.borrow_mut();
    let res: &mut CanvasResources = storage.get_mut().unwrap();
    let mut data = res.canvases.get_mut(rid).unwrap();

    // Save current state to stack
    let current_state = crate::ext::canvas::state::CanvasState {
        fill_style: data.fill_style.clone(),
        stroke_style: data.stroke_style.clone(),
        line_width: data.line_width,
        global_alpha: data.global_alpha,
        transform: data.transform,
        line_dash: data.line_dash.clone(),
        line_dash_offset: data.line_dash_offset,
        line_cap: data.line_cap,
        line_join: data.line_join,
        miter_limit: data.miter_limit,
        shadow_blur: data.shadow_blur,
        shadow_color: data.shadow_color.clone(),
        shadow_offset_x: data.shadow_offset_x,
        shadow_offset_y: data.shadow_offset_y,
        composite_operation: data.composite_operation,
        font: data.font.clone(),
        text_align: data.text_align,
        text_baseline: data.text_baseline,
        direction: data.direction,
    };
    data.state_stack.push(current_state);

    // Add save command to command list
    data.commands.push(CanvasCommand::Save);

    Ok(Value::Undefined)
}

/// Internal op to restore the last saved canvas state
pub fn internal_canvas_restore<'gc>(
    agent: &mut Agent,
    _this: Value,
    args: ArgumentsList,
    mut gc: GcScope<'gc, '_>,
) -> JsResult<'gc, Value<'gc>> {
    let rid_val = args.get(0).to_int32(agent, gc.reborrow()).unbind().unwrap() as u32;
    let rid = Rid::from_index(rid_val);

    let host_data = agent
        .get_host_data()
        .downcast_ref::<HostData<RuntimeMacroTask>>()
        .unwrap();
    let mut storage = host_data.storage.borrow_mut();
    let res: &mut CanvasResources = storage.get_mut().unwrap();
    let mut data = res.canvases.get_mut(rid).unwrap();

    // Restore state from stack if available. The spec requires
    // restore() to revert EVERY field that save() captured — previously
    // this path only restored 8 of 19 fields, so shadow state, line
    // cap/join/miter, and all text attributes leaked across save/restore
    // boundaries. The most visible symptom: a fill drawn with shadows
    // inside a save()/restore() block still left a shadow trail behind
    // subsequent fills that expected no shadow.
    if let Some(saved_state) = data.state_stack.pop() {
        data.fill_style = saved_state.fill_style;
        data.stroke_style = saved_state.stroke_style;
        data.line_width = saved_state.line_width;
        data.global_alpha = saved_state.global_alpha;
        data.transform = saved_state.transform;
        data.line_dash = saved_state.line_dash;
        data.line_dash_offset = saved_state.line_dash_offset;
        data.line_cap = saved_state.line_cap;
        data.line_join = saved_state.line_join;
        data.miter_limit = saved_state.miter_limit;
        data.shadow_blur = saved_state.shadow_blur;
        data.shadow_color = saved_state.shadow_color;
        data.shadow_offset_x = saved_state.shadow_offset_x;
        data.shadow_offset_y = saved_state.shadow_offset_y;
        data.composite_operation = saved_state.composite_operation;
        data.font = saved_state.font;
        data.text_align = saved_state.text_align;
        data.text_baseline = saved_state.text_baseline;
        data.direction = saved_state.direction;
    }

    // Add restore command to command list
    data.commands.push(CanvasCommand::Restore);

    Ok(Value::Undefined)
}

/// Rendering context parameters grouped to avoid too many function arguments
#[allow(dead_code)]
pub struct RenderContext<'a> {
    pub fill_style: &'a crate::ext::canvas::FillStyle,
    pub stroke_style: &'a crate::ext::canvas::FillStyle,
    pub line_width: f64,
    pub global_alpha: f32,
    pub transform: [f64; 6],
    pub line_cap: LineCap,
    pub line_join: LineJoin,
    pub miter_limit: f64,
    pub composite_operation: CompositeOperation,
    pub shadow_blur: f64,
    pub shadow_color: &'a crate::ext::canvas::FillStyle,
    pub shadow_offset_x: f64,
    pub shadow_offset_y: f64,
}

/// Process all saved commands and render them using the renderer
#[allow(dead_code, clippy::needless_lifetimes)]
pub fn process_all_commands<'gc>(
    commands: &Vec<CanvasCommand<'gc>>,
    renderer: &mut crate::ext::canvas::renderer::Renderer,
    agent: &mut Agent,
    ctx: &RenderContext<'_>,
) {
    let mut current_path = Vec::new();

    for command in commands {
        match command {
            CanvasCommand::MoveTo { x, y } => {
                let x_f64 = x.into_f64(agent);
                let y_f64 = y.into_f64(agent);
                current_path.clear();
                current_path.push(crate::ext::canvas::renderer::Point { x: x_f64, y: y_f64 });
            }
            CanvasCommand::LineTo { x, y } => {
                let x_f64 = x.into_f64(agent);
                let y_f64 = y.into_f64(agent);
                current_path.push(crate::ext::canvas::renderer::Point { x: x_f64, y: y_f64 });
            }
            CanvasCommand::BezierCurveTo {
                cp1x,
                cp1y,
                cp2x,
                cp2y,
                x,
                y,
            } => {
                let cp1x_f64 = cp1x.into_f64(agent);
                let cp1y_f64 = cp1y.into_f64(agent);
                let cp2x_f64 = cp2x.into_f64(agent);
                let cp2y_f64 = cp2y.into_f64(agent);
                let x_f64 = x.into_f64(agent);
                let y_f64 = y.into_f64(agent);

                if let Some(last_point) = current_path.last().cloned() {
                    let start = last_point;
                    let cp1 = crate::ext::canvas::renderer::Point {
                        x: cp1x_f64,
                        y: cp1y_f64,
                    };
                    let cp2 = crate::ext::canvas::renderer::Point {
                        x: cp2x_f64,
                        y: cp2y_f64,
                    };
                    let end = crate::ext::canvas::renderer::Point { x: x_f64, y: y_f64 };

                    tessellate_bezier_curve_to_path(&mut current_path, start, cp1, cp2, end);
                }
            }
            CanvasCommand::QuadraticCurveTo { cpx, cpy, x, y } => {
                let cpx_f64 = cpx.into_f64(agent);
                let cpy_f64 = cpy.into_f64(agent);
                let x_f64 = x.into_f64(agent);
                let y_f64 = y.into_f64(agent);

                if let Some(last_point) = current_path.last().cloned() {
                    let start = last_point;
                    let control = crate::ext::canvas::renderer::Point {
                        x: cpx_f64,
                        y: cpy_f64,
                    };
                    let end = crate::ext::canvas::renderer::Point { x: x_f64, y: y_f64 };

                    tessellate_quadratic_curve_to_path(&mut current_path, start, control, end);
                }
            }
            CanvasCommand::Ellipse {
                x,
                y,
                radius_x,
                radius_y,
                rotation,
                start_angle,
                end_angle,
                counter_clockwise,
            } => {
                let x_f64 = x.into_f64(agent);
                let y_f64 = y.into_f64(agent);
                let radius_x_f64 = radius_x.into_f64(agent);
                let radius_y_f64 = radius_y.into_f64(agent);
                let rotation_f64 = rotation.into_f64(agent);
                let start_angle_f64 = start_angle.into_f64(agent);
                let end_angle_f64 = end_angle.into_f64(agent);

                tessellate_ellipse_to_path(
                    &mut current_path,
                    x_f64,
                    y_f64,
                    radius_x_f64,
                    radius_y_f64,
                    rotation_f64,
                    start_angle_f64,
                    end_angle_f64,
                    *counter_clockwise,
                );
            }
            CanvasCommand::RoundRect {
                x,
                y,
                width,
                height,
                radius,
            } => {
                let x_f64 = x.into_f64(agent);
                let y_f64 = y.into_f64(agent);
                let width_f64 = width.into_f64(agent);
                let height_f64 = height.into_f64(agent);
                let radius_f64 = radius.into_f64(agent);

                tessellate_rounded_rect_to_path(
                    &mut current_path,
                    x_f64,
                    y_f64,
                    width_f64,
                    height_f64,
                    [radius_f64, radius_f64, radius_f64, radius_f64],
                );
            }
            CanvasCommand::BeginPath => {
                current_path.clear();
            }
            CanvasCommand::ClosePath => {
                if !current_path.is_empty()
                    && let Some(first) = current_path.first()
                {
                    current_path.push(first.clone()); // Close the path by connecting to start
                }
            }
            CanvasCommand::Fill => {
                if !current_path.is_empty() {
                    renderer.render_polygon(
                        current_path.clone(),
                        &RenderState {
                            fill_style: ctx.fill_style.clone(),
                            global_alpha: ctx.global_alpha,
                            transform: ctx.transform,
                            line_cap: ctx.line_cap,
                            line_join: ctx.line_join,
                            miter_limit: ctx.miter_limit,
                            shadow_blur: ctx.shadow_blur,
                            shadow_color: ctx.shadow_color.clone(),
                            shadow_offset_x: ctx.shadow_offset_x,
                            shadow_offset_y: ctx.shadow_offset_y,
                            composite_operation: ctx.composite_operation,
                            clip_path: None,
                        },
                    );
                }
            }
            CanvasCommand::Stroke => {
                if !current_path.is_empty() {
                    renderer.render_polyline(
                        current_path.clone(),
                        &RenderState {
                            fill_style: ctx.stroke_style.clone(),
                            global_alpha: ctx.global_alpha,
                            transform: ctx.transform,
                            line_cap: ctx.line_cap,
                            line_join: ctx.line_join,
                            miter_limit: ctx.miter_limit,
                            shadow_blur: ctx.shadow_blur,
                            shadow_color: ctx.shadow_color.clone(),
                            shadow_offset_x: ctx.shadow_offset_x,
                            shadow_offset_y: ctx.shadow_offset_y,
                            composite_operation: ctx.composite_operation,
                            clip_path: None,
                        },
                        ctx.line_width,
                    );
                }
            }
            CanvasCommand::FillRect {
                x,
                y,
                width,
                height,
            } => {
                let x_f64 = x.into_f64(agent);
                let y_f64 = y.into_f64(agent);
                let width_f64 = width.into_f64(agent);
                let height_f64 = height.into_f64(agent);

                let rect = crate::ext::canvas::renderer::Rect {
                    start: crate::ext::canvas::renderer::Point { x: x_f64, y: y_f64 },
                    end: crate::ext::canvas::renderer::Point {
                        x: x_f64 + width_f64,
                        y: y_f64 + height_f64,
                    },
                };
                renderer.render_rect(
                    rect,
                    &RenderState {
                        fill_style: ctx.fill_style.clone(),
                        global_alpha: ctx.global_alpha,
                        transform: ctx.transform,
                        line_cap: ctx.line_cap,
                        line_join: ctx.line_join,
                        miter_limit: ctx.miter_limit,
                        shadow_blur: ctx.shadow_blur,
                        shadow_color: ctx.shadow_color.clone(),
                        shadow_offset_x: ctx.shadow_offset_x,
                        shadow_offset_y: ctx.shadow_offset_y,
                        composite_operation: ctx.composite_operation,
                        clip_path: None,
                    },
                );
            }
            CanvasCommand::StrokeRect {
                x,
                y,
                width,
                height,
            } => {
                let x_f64 = x.into_f64(agent);
                let y_f64 = y.into_f64(agent);
                let width_f64 = width.into_f64(agent);
                let height_f64 = height.into_f64(agent);

                // Create rectangle outline as polyline
                let rect_path = vec![
                    crate::ext::canvas::renderer::Point { x: x_f64, y: y_f64 },
                    crate::ext::canvas::renderer::Point {
                        x: x_f64 + width_f64,
                        y: y_f64,
                    },
                    crate::ext::canvas::renderer::Point {
                        x: x_f64 + width_f64,
                        y: y_f64 + height_f64,
                    },
                    crate::ext::canvas::renderer::Point {
                        x: x_f64,
                        y: y_f64 + height_f64,
                    },
                    crate::ext::canvas::renderer::Point { x: x_f64, y: y_f64 }, // Close the rectangle
                ];
                renderer.render_polyline(
                    rect_path,
                    &RenderState {
                        fill_style: ctx.stroke_style.clone(),
                        global_alpha: ctx.global_alpha,
                        transform: ctx.transform,
                        line_cap: ctx.line_cap,
                        line_join: ctx.line_join,
                        miter_limit: ctx.miter_limit,
                        shadow_blur: ctx.shadow_blur,
                        shadow_color: ctx.shadow_color.clone(),
                        shadow_offset_x: ctx.shadow_offset_x,
                        shadow_offset_y: ctx.shadow_offset_y,
                        composite_operation: ctx.composite_operation,
                        clip_path: None,
                    },
                    ctx.line_width,
                );
            }
            CanvasCommand::ClearRect {
                x,
                y,
                width,
                height,
            } => {
                let x_f64 = x.into_f64(agent);
                let y_f64 = y.into_f64(agent);
                let width_f64 = width.into_f64(agent);
                let height_f64 = height.into_f64(agent);

                // Clear with white color (background)
                let rect = crate::ext::canvas::renderer::Rect {
                    start: crate::ext::canvas::renderer::Point { x: x_f64, y: y_f64 },
                    end: crate::ext::canvas::renderer::Point {
                        x: x_f64 + width_f64,
                        y: y_f64 + height_f64,
                    },
                };
                renderer.render_rect(
                    rect,
                    &RenderState {
                        fill_style: FillStyle::Color {
                            r: 1.0,
                            g: 1.0,
                            b: 1.0,
                            a: 1.0,
                        },
                        global_alpha: 1.0,
                        transform: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
                        line_cap: LineCap::default(),
                        line_join: LineJoin::default(),
                        miter_limit: 10.0,
                        shadow_blur: 0.0,
                        shadow_color: FillStyle::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 0.0,
                        },
                        shadow_offset_x: 0.0,
                        shadow_offset_y: 0.0,
                        composite_operation: CompositeOperation::default(),
                        clip_path: None,
                    },
                ); // White background
            }
            // Handle other commands that don't directly affect rendering
            CanvasCommand::Arc { .. }
            | CanvasCommand::ArcTo { .. }
            | CanvasCommand::Rect { .. }
            | CanvasCommand::Save
            | CanvasCommand::Restore
            | CanvasCommand::Scale { .. }
            | CanvasCommand::Rotate { .. }
            | CanvasCommand::Translate { .. }
            | CanvasCommand::Transform { .. }
            | CanvasCommand::SetTransform { .. }
            | CanvasCommand::ResetTransform
            | CanvasCommand::Reset
            | CanvasCommand::SetLineDash { .. } => {
                // These commands don't directly affect path rendering
            }
            CanvasCommand::SetStrokeStyle(_style) => {
                // Update stroke style
                // Note: The actual style change is handled by the renderer
            }
            CanvasCommand::Clip { path } => {
                // Apply clipping to the renderer
                renderer.set_clip_path(Some(path.clone()));
            }
            CanvasCommand::CreateLinearGradient { .. }
            | CanvasCommand::CreateRadialGradient { .. }
            | CanvasCommand::CreateConicGradient { .. } => {
                // These commands would need more complex state management
                // For now, we'll skip them in this basic implementation
            }
            CanvasCommand::DrawImage {
                image_rid,
                sx,
                sy,
                s_width,
                s_height,
                dx,
                dy,
                d_width,
                d_height,
            } => {
                // Create render state for the image
                let render_state = RenderState {
                    fill_style: ctx.fill_style.clone(),
                    global_alpha: ctx.global_alpha,
                    transform: ctx.transform,
                    line_cap: ctx.line_cap,
                    line_join: ctx.line_join,
                    miter_limit: ctx.miter_limit,
                    shadow_blur: ctx.shadow_blur,
                    shadow_color: ctx.shadow_color.clone(),
                    shadow_offset_x: ctx.shadow_offset_x,
                    shadow_offset_y: ctx.shadow_offset_y,
                    composite_operation: ctx.composite_operation,
                    clip_path: None,
                };

                // Render the image
                renderer.render_image(
                    *image_rid,
                    *sx,
                    *sy,
                    *s_width,
                    *s_height,
                    *dx,
                    *dy,
                    *d_width,
                    *d_height,
                    &render_state,
                );
            }
        }
    }
}

/// Tessellate a quadratic Bezier curve into line segments and add to path
fn tessellate_quadratic_curve_to_path(
    path: &mut Vec<crate::ext::canvas::renderer::Point>,
    start: crate::ext::canvas::renderer::Point,
    control: crate::ext::canvas::renderer::Point,
    end: crate::ext::canvas::renderer::Point,
) {
    const SEGMENTS: usize = 16; // Number of segments for curve approximation

    for i in 1..=SEGMENTS {
        let t = i as f64 / SEGMENTS as f64;
        let one_minus_t = 1.0 - t;

        // Quadratic Bezier formula: (1-t)²P₀ + 2(1-t)tP₁ + t²P₂
        let x =
            one_minus_t * one_minus_t * start.x + 2.0 * one_minus_t * t * control.x + t * t * end.x;
        let y =
            one_minus_t * one_minus_t * start.y + 2.0 * one_minus_t * t * control.y + t * t * end.y;

        path.push(crate::ext::canvas::renderer::Point { x, y });
    }
}

/// Tessellate an ellipse into line segments and add to path
#[allow(clippy::too_many_arguments)]
fn tessellate_ellipse_to_path(
    path: &mut Vec<crate::ext::canvas::renderer::Point>,
    x: f64,
    y: f64,
    radius_x: f64,
    radius_y: f64,
    rotation: f64,
    start_angle: f64,
    end_angle: f64,
    counter_clockwise: bool,
) {
    const SEGMENTS: usize = 32; // Number of segments for ellipse approximation

    let current_angle = start_angle;
    let mut end_target = end_angle;

    // Handle counter-clockwise direction
    if counter_clockwise && end_target > current_angle {
        end_target -= 2.0 * std::f64::consts::PI;
    } else if !counter_clockwise && end_target < current_angle {
        end_target += 2.0 * std::f64::consts::PI;
    }

    let angle_diff = end_target - current_angle;
    let step = angle_diff / SEGMENTS as f64;

    let cos_rotation = rotation.cos();
    let sin_rotation = rotation.sin();

    for i in 0..=SEGMENTS {
        let angle = current_angle + i as f64 * step;

        // Point on unrotated ellipse
        let ellipse_x = radius_x * angle.cos();
        let ellipse_y = radius_y * angle.sin();

        // Apply rotation
        let rotated_x = ellipse_x * cos_rotation - ellipse_y * sin_rotation;
        let rotated_y = ellipse_x * sin_rotation + ellipse_y * cos_rotation;

        // Translate to center
        path.push(crate::ext::canvas::renderer::Point {
            x: x + rotated_x,
            y: y + rotated_y,
        });
    }
}

/// Tessellate a rounded rectangle into line segments and add to path
/// Tessellate a rounded rectangle with four independent corner radii
/// per the HTML Canvas spec (`roundRect(x, y, w, h, [tl, tr, br, bl])`).
///
/// Each radius is clamped individually to `min(width/2, height/2)`. A
/// corner with radius 0 is emitted as a sharp point, preserving the
/// square-corner case inline.
fn tessellate_rounded_rect_to_path(
    path: &mut Vec<crate::ext::canvas::renderer::Point>,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    radii: [f64; 4],
) {
    // Coerce negatives / NaN to 0 before applying the spec scaling.
    let sanitize = |r: f64| if r.is_finite() && r > 0.0 { r } else { 0.0 };
    let r_tl = sanitize(radii[0]);
    let r_tr = sanitize(radii[1]);
    let r_br = sanitize(radii[2]);
    let r_bl = sanitize(radii[3]);

    // HTML spec scaling: if adjacent corner radii along any side would
    // overlap, scale ALL four radii by the smallest factor needed so the
    // overlap just disappears. This is looser than per-corner clamping
    // to min(w/2, h/2) — a tall-and-thin rectangle can still have a
    // large corner radius on its long edges as long as the short edge's
    // neighbor is small.
    //
    // top    = tl + tr     <= width
    // bottom = br + bl     <= width
    // left   = tl + bl     <= height
    // right  = tr + br     <= height
    let top = r_tl + r_tr;
    let right = r_tr + r_br;
    let bottom = r_br + r_bl;
    let left = r_bl + r_tl;
    let mut scale = 1.0f64;
    let check = |sum: f64, limit: f64, scale: &mut f64| {
        if sum > 0.0 && limit > 0.0 && limit / sum < *scale {
            *scale = limit / sum;
        }
    };
    check(top, width, &mut scale);
    check(bottom, width, &mut scale);
    check(left, height, &mut scale);
    check(right, height, &mut scale);
    let tl = r_tl * scale;
    let tr = r_tr * scale;
    let br = r_br * scale;
    let bl = r_bl * scale;

    if tl <= 0.0 && tr <= 0.0 && br <= 0.0 && bl <= 0.0 {
        // Fast path: plain rectangle.
        path.push(crate::ext::canvas::renderer::Point { x, y });
        path.push(crate::ext::canvas::renderer::Point { x: x + width, y });
        path.push(crate::ext::canvas::renderer::Point {
            x: x + width,
            y: y + height,
        });
        path.push(crate::ext::canvas::renderer::Point { x, y: y + height });
        path.push(crate::ext::canvas::renderer::Point { x, y }); // close
        return;
    }

    const CORNER_SEGMENTS: usize = 8;

    // Emit a quarter-circle centered at (cx, cy) from `start_angle` to
    // `start_angle + π/2`. Degenerate to a single point when r == 0.
    let push_corner = |path: &mut Vec<crate::ext::canvas::renderer::Point>,
                       cx: f64,
                       cy: f64,
                       r: f64,
                       start_angle: f64| {
        if r <= 0.0 {
            path.push(crate::ext::canvas::renderer::Point { x: cx, y: cy });
            return;
        }
        for i in 0..=CORNER_SEGMENTS {
            let a = start_angle + i as f64 * std::f64::consts::FRAC_PI_2 / CORNER_SEGMENTS as f64;
            path.push(crate::ext::canvas::renderer::Point {
                x: cx + r * a.cos(),
                y: cy + r * a.sin(),
            });
        }
    };

    // Top-left corner: arc from π → 3π/2, centered at (x+tl, y+tl).
    push_corner(path, x + tl, y + tl, tl, std::f64::consts::PI);

    // Top edge.
    path.push(crate::ext::canvas::renderer::Point {
        x: x + width - tr,
        y,
    });

    // Top-right corner: arc from 3π/2 → 2π, centered at (x+w-tr, y+tr).
    push_corner(
        path,
        x + width - tr,
        y + tr,
        tr,
        -std::f64::consts::FRAC_PI_2,
    );

    // Right edge.
    path.push(crate::ext::canvas::renderer::Point {
        x: x + width,
        y: y + height - br,
    });

    // Bottom-right corner: arc from 0 → π/2, centered at (x+w-br, y+h-br).
    push_corner(path, x + width - br, y + height - br, br, 0.0);

    // Bottom edge.
    path.push(crate::ext::canvas::renderer::Point {
        x: x + bl,
        y: y + height,
    });

    // Bottom-left corner: arc from π/2 → π, centered at (x+bl, y+h-bl).
    push_corner(
        path,
        x + bl,
        y + height - bl,
        bl,
        std::f64::consts::FRAC_PI_2,
    );

    // Left edge + close.
    path.push(crate::ext::canvas::renderer::Point { x, y: y + tl });
}

/// Helper function to tessellate arc and add to path. Delegates to the
/// ellipse tessellator with equal x/y radii and zero rotation; the
/// `counter_clockwise` flag flows through so `arc()` honors the sweep
/// direction the same way `ellipse()` does.
fn tessellate_arc_to_path(
    path: &mut Vec<crate::ext::canvas::renderer::Point>,
    x: f64,
    y: f64,
    radius: f64,
    start_angle: f64,
    end_angle: f64,
    counter_clockwise: bool,
) {
    tessellate_ellipse_to_path(
        path,
        x,
        y,
        radius,
        radius,
        0.0,
        start_angle,
        end_angle,
        counter_clockwise,
    );
}

/// Tessellate a cubic Bezier curve into line segments and add to path
fn tessellate_bezier_curve_to_path(
    path: &mut Vec<crate::ext::canvas::renderer::Point>,
    start: crate::ext::canvas::renderer::Point,
    cp1: crate::ext::canvas::renderer::Point,
    cp2: crate::ext::canvas::renderer::Point,
    end: crate::ext::canvas::renderer::Point,
) {
    const SEGMENTS: usize = 16; // Number of segments for curve approximation

    for i in 1..=SEGMENTS {
        let t = i as f64 / SEGMENTS as f64;
        let one_minus_t = 1.0 - t;

        // Cubic Bezier formula: (1-t)³P₀ + 3(1-t)²tP₁ + 3(1-t)t²P₂ + t³P₃
        let x = one_minus_t * one_minus_t * one_minus_t * start.x
            + 3.0 * one_minus_t * one_minus_t * t * cp1.x
            + 3.0 * one_minus_t * t * t * cp2.x
            + t * t * t * end.x;
        let y = one_minus_t * one_minus_t * one_minus_t * start.y
            + 3.0 * one_minus_t * one_minus_t * t * cp1.y
            + 3.0 * one_minus_t * t * t * cp2.y
            + t * t * t * end.y;

        path.push(crate::ext::canvas::renderer::Point { x, y });
    }
}

/// Generate stroke path from current path with line width
/// Turn a polyline into the triangle list for its stroke, honoring the
/// current line width, cap style, join style, miter limit, and line
/// dash pattern.
pub fn generate_stroke_path_public(
    path: &[crate::ext::canvas::renderer::Point],
    line_width: f64,
    dash: &[f64],
    dash_offset: f64,
    line_cap: LineCap,
    line_join: LineJoin,
    miter_limit: f64,
) -> Vec<crate::ext::canvas::renderer::Point> {
    generate_stroke_path(
        path,
        line_width,
        dash,
        dash_offset,
        line_cap,
        line_join,
        miter_limit,
    )
}

fn generate_stroke_path(
    path: &[crate::ext::canvas::renderer::Point],
    line_width: f64,
    dash: &[f64],
    dash_offset: f64,
    line_cap: LineCap,
    line_join: LineJoin,
    miter_limit: f64,
) -> Vec<crate::ext::canvas::renderer::Point> {
    if path.len() < 2 {
        return Vec::new();
    }
    let half_width = line_width / 2.0;
    let mut out: Vec<crate::ext::canvas::renderer::Point> = Vec::new();

    // Corners of a single segment's rectangle: (outer_from, inner_from, outer_to, inner_to).
    type SegmentCorners = ((f64, f64), (f64, f64), (f64, f64), (f64, f64));

    let pt = |x: f64, y: f64| crate::ext::canvas::renderer::Point { x, y };
    let push_segment = |out: &mut Vec<crate::ext::canvas::renderer::Point>,
                        ax: f64,
                        ay: f64,
                        bx: f64,
                        by: f64|
     -> Option<SegmentCorners> {
        let dx = bx - ax;
        let dy = by - ay;
        let len = (dx * dx + dy * dy).sqrt();
        if len <= 0.0 {
            return None;
        }
        let nx = -dy / len * half_width;
        let ny = dx / len * half_width;
        let tl = (ax + nx, ay + ny);
        let bl = (ax - nx, ay - ny);
        let tr = (bx + nx, by + ny);
        let br = (bx - nx, by - ny);
        out.push(pt(tl.0, tl.1));
        out.push(pt(bl.0, bl.1));
        out.push(pt(tr.0, tr.1));
        out.push(pt(bl.0, bl.1));
        out.push(pt(br.0, br.1));
        out.push(pt(tr.0, tr.1));
        Some((tl, bl, tr, br))
    };

    // Fan a disk (or half-disk) around `center` between two unit
    // directions, used by round caps and round joins.
    let round_fan = |out: &mut Vec<crate::ext::canvas::renderer::Point>,
                     cx: f64,
                     cy: f64,
                     start_angle: f64,
                     sweep: f64| {
        // Segment the sweep so each chord is at most ~a few pixels on a
        // unit-radius curve. 12 segments for a half turn reads as smooth
        // under 4× MSAA without ballooning vertex counts.
        let n = ((sweep.abs() / std::f64::consts::PI) * 12.0)
            .ceil()
            .max(3.0) as usize;
        let mut prev = (
            cx + start_angle.cos() * half_width,
            cy + start_angle.sin() * half_width,
        );
        for i in 1..=n {
            let t = i as f64 / n as f64;
            let a = start_angle + sweep * t;
            let nxt = (cx + a.cos() * half_width, cy + a.sin() * half_width);
            out.push(pt(cx, cy));
            out.push(pt(prev.0, prev.1));
            out.push(pt(nxt.0, nxt.1));
            prev = nxt;
        }
    };

    // Butt: no extra geometry. Round: half-disk around `center`
    // on the `outward` side. Square: rectangle extending half_width in
    // the `outward` direction.
    //
    // `outward_angle` points away from the segment (opposite of the
    // segment direction for a start cap; along the segment direction
    // for an end cap).
    let draw_cap = |out: &mut Vec<crate::ext::canvas::renderer::Point>,
                    cx: f64,
                    cy: f64,
                    outward_dx: f64,
                    outward_dy: f64| {
        match line_cap {
            LineCap::Butt => {}
            LineCap::Round => {
                let base_angle = outward_dy.atan2(outward_dx);
                // Half-disk from +90° to -90° around the outward axis.
                round_fan(
                    out,
                    cx,
                    cy,
                    base_angle - std::f64::consts::FRAC_PI_2,
                    std::f64::consts::PI,
                );
            }
            LineCap::Square => {
                let px = -outward_dy * half_width;
                let py = outward_dx * half_width;
                let ex = outward_dx * half_width;
                let ey = outward_dy * half_width;
                let a = (cx + px, cy + py);
                let b = (cx - px, cy - py);
                let c = (cx + px + ex, cy + py + ey);
                let d = (cx - px + ex, cy - py + ey);
                out.push(pt(a.0, a.1));
                out.push(pt(b.0, b.1));
                out.push(pt(c.0, c.1));
                out.push(pt(b.0, b.1));
                out.push(pt(d.0, d.1));
                out.push(pt(c.0, c.1));
            }
        }
    };

    // Fill the corner between two adjacent segments sharing a vertex.
    // `(ux1, uy1)` is the direction of the incoming segment; `(ux2,
    // uy2)` is the direction of the outgoing segment. Both are unit.
    let draw_join = |out: &mut Vec<crate::ext::canvas::renderer::Point>,
                     cx: f64,
                     cy: f64,
                     ux1: f64,
                     uy1: f64,
                     ux2: f64,
                     uy2: f64| {
        // Cross product tells us which side the corner sticks out on.
        let cross = ux1 * uy2 - uy1 * ux2;
        if cross.abs() < 1e-9 {
            return; // collinear — no gap to fill
        }
        // Outer-edge normals.
        let n1x = -uy1 * half_width;
        let n1y = ux1 * half_width;
        let n2x = -uy2 * half_width;
        let n2y = ux2 * half_width;
        // Pick the outside side based on cross sign.
        let (out_n1x, out_n1y, out_n2x, out_n2y) = if cross > 0.0 {
            (-n1x, -n1y, -n2x, -n2y)
        } else {
            (n1x, n1y, n2x, n2y)
        };
        let outer_from = (cx + out_n1x, cy + out_n1y);
        let outer_to = (cx + out_n2x, cy + out_n2y);

        match line_join {
            LineJoin::Bevel => {
                out.push(pt(cx, cy));
                out.push(pt(outer_from.0, outer_from.1));
                out.push(pt(outer_to.0, outer_to.1));
            }
            LineJoin::Round => {
                let a1 = out_n1y.atan2(out_n1x);
                let a2 = out_n2y.atan2(out_n2x);
                let mut sweep = a2 - a1;
                // Take the shortest path around the outside.
                while sweep > std::f64::consts::PI {
                    sweep -= 2.0 * std::f64::consts::PI;
                }
                while sweep < -std::f64::consts::PI {
                    sweep += 2.0 * std::f64::consts::PI;
                }
                // Flip so we sweep on the OUTSIDE of the corner, not
                // across the inside.
                if (cross > 0.0 && sweep > 0.0) || (cross < 0.0 && sweep < 0.0) {
                    sweep = if sweep > 0.0 {
                        sweep - 2.0 * std::f64::consts::PI
                    } else {
                        sweep + 2.0 * std::f64::consts::PI
                    };
                }
                let _ = a2;
                round_fan(out, cx, cy, a1, sweep);
            }
            LineJoin::Miter => {
                // Compute the miter apex by intersecting the two outer
                // edges. If it exceeds miter_limit * half_width from the
                // vertex, fall back to bevel per the HTML spec.
                let denom = ux1 * uy2 - uy1 * ux2;
                if denom.abs() < 1e-9 {
                    return;
                }
                // Edge 1: start = outer_from, dir = (ux1, uy1)
                // Edge 2: start = outer_to,   dir = (ux2, uy2)
                // Solve outer_from + t1*(ux1, uy1) = outer_to + t2*(ux2, uy2)
                let dx = outer_to.0 - outer_from.0;
                let dy = outer_to.1 - outer_from.1;
                let t1 = (dx * uy2 - dy * ux2) / denom;
                let miter_x = outer_from.0 + ux1 * t1;
                let miter_y = outer_from.1 + uy1 * t1;
                let mdx = miter_x - cx;
                let mdy = miter_y - cy;
                let miter_len = (mdx * mdx + mdy * mdy).sqrt();
                if miter_len > miter_limit * half_width {
                    // Beyond the miter limit — fall back to bevel.
                    out.push(pt(cx, cy));
                    out.push(pt(outer_from.0, outer_from.1));
                    out.push(pt(outer_to.0, outer_to.1));
                } else {
                    // Two triangles fan from the center to the apex.
                    out.push(pt(cx, cy));
                    out.push(pt(outer_from.0, outer_from.1));
                    out.push(pt(miter_x, miter_y));
                    out.push(pt(cx, cy));
                    out.push(pt(miter_x, miter_y));
                    out.push(pt(outer_to.0, outer_to.1));
                }
            }
        }
    };

    let dash_normalized: Vec<f64> = if dash.is_empty() {
        Vec::new()
    } else if dash.len() % 2 == 1 {
        let mut v = Vec::with_capacity(dash.len() * 2);
        v.extend_from_slice(dash);
        v.extend_from_slice(dash);
        v
    } else {
        dash.to_vec()
    };

    // Helper: segment direction as a unit vector.
    let seg_dir = |a: &crate::ext::canvas::renderer::Point,
                   b: &crate::ext::canvas::renderer::Point|
     -> Option<(f64, f64)> {
        let dx = b.x - a.x;
        let dy = b.y - a.y;
        let len = (dx * dx + dy * dy).sqrt();
        if len <= 0.0 {
            None
        } else {
            Some((dx / len, dy / len))
        }
    };

    if dash_normalized.is_empty() {
        // Emit segment quads + joins at interior vertices + caps at
        // the two ends. Skip zero-length segments but preserve vertex
        // indexing for joins.
        let mut last_dir: Option<(f64, f64)> = None;
        for i in 0..path.len() - 1 {
            let a = &path[i];
            let b = &path[i + 1];
            if push_segment(&mut out, a.x, a.y, b.x, b.y).is_some() {
                let dir = seg_dir(a, b).unwrap();
                if let Some((pux, puy)) = last_dir {
                    draw_join(&mut out, a.x, a.y, pux, puy, dir.0, dir.1);
                } else {
                    // First segment: start cap pointing opposite the
                    // direction of travel.
                    draw_cap(&mut out, a.x, a.y, -dir.0, -dir.1);
                }
                last_dir = Some(dir);
            }
        }
        // End cap on the final segment's endpoint.
        if let Some((ux, uy)) = last_dir {
            let last = &path[path.len() - 1];
            draw_cap(&mut out, last.x, last.y, ux, uy);
        }
        return out;
    }

    let total_dash: f64 = dash_normalized.iter().sum();
    if total_dash <= 0.0 {
        return out;
    }
    let phase = dash_offset.rem_euclid(total_dash);
    let mut dash_idx = 0usize;
    let mut remaining_in_slot = dash_normalized[0];
    let mut walked = 0.0;
    while walked + remaining_in_slot <= phase {
        walked += remaining_in_slot;
        dash_idx = (dash_idx + 1) % dash_normalized.len();
        remaining_in_slot = dash_normalized[dash_idx];
    }
    remaining_in_slot -= phase - walked;
    let mut pen_on = dash_idx.is_multiple_of(2);

    for i in 0..path.len() - 1 {
        let a = &path[i];
        let b = &path[i + 1];
        let Some((ux, uy)) = seg_dir(a, b) else {
            continue;
        };
        let seg_len = {
            let dx = b.x - a.x;
            let dy = b.y - a.y;
            (dx * dx + dy * dy).sqrt()
        };
        let mut consumed = 0.0f64;
        let mut cur_x = a.x;
        let mut cur_y = a.y;
        while consumed < seg_len {
            let take = remaining_in_slot.min(seg_len - consumed);
            let nx = cur_x + ux * take;
            let ny = cur_y + uy * take;
            if pen_on {
                push_segment(&mut out, cur_x, cur_y, nx, ny);
                draw_cap(&mut out, cur_x, cur_y, -ux, -uy);
                draw_cap(&mut out, nx, ny, ux, uy);
            }
            cur_x = nx;
            cur_y = ny;
            consumed += take;
            remaining_in_slot -= take;
            if remaining_in_slot <= f64::EPSILON {
                dash_idx = (dash_idx + 1) % dash_normalized.len();
                remaining_in_slot = dash_normalized[dash_idx];
                pen_on = !pen_on;
            }
        }
    }

    let _ = line_join;
    let _ = miter_limit;

    out
}
