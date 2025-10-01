// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use super::CanvasResources;
use super::FillStyle;
use super::Rid;
use super::renderer::{Point, Rect};
use crate::RuntimeMacroTask;
use crate::ext::canvas::renderer::{LineCap, LineJoin, RenderState};
use andromeda_core::HostData;
use nova_vm::ecmascript::types::Number;
use nova_vm::{
    ecmascript::{
        builtins::ArgumentsList,
        execution::{Agent, JsResult},
        types::Value,
    },
    engine::context::{Bindable, GcScope},
};

/// A command to be executed on the canvas
#[derive(Clone)]
#[allow(dead_code)]
pub enum CanvasCommand<'gc> {
    Arc {
        x: Number<'gc>,
        y: Number<'gc>,
        radius: Number<'gc>,
        start_angle: Number<'gc>,
        end_angle: Number<'gc>,
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
    );

    data.commands.push(CanvasCommand::Arc {
        x,
        y,
        radius,
        start_angle,
        end_angle,
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

        // Get the current fill style color
        let data = res.canvases.get(rid).unwrap();
        res.renderers.get_mut(rid).unwrap().render_rect(
            rect,
            &RenderState {
                fill_style: data.fill_style,
                global_alpha: data.global_alpha,
                transform: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
                line_cap: LineCap::default(),
                line_join: LineJoin::default(),
                miter_limit: 10.0,
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

    // Start a new subpath in the current path
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

        // data;

        if data.current_path.len() >= 3 {
            renderer.render_polygon(
                data.current_path.clone(),
                &RenderState {
                    fill_style: data.fill_style,
                    global_alpha: data.global_alpha,
                    transform: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
                    line_cap: LineCap::default(),
                    line_join: LineJoin::default(),
                    miter_limit: 10.0,
                },
            );
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

        if data.current_path.len() >= 2 {
            // Convert path to stroke polygon using line width
            let stroke_path = generate_stroke_path(&data.current_path, data.line_width);

            // Render the stroke as a polygon using the GPU renderer
            renderer.render_polygon(
                stroke_path,
                &RenderState {
                    fill_style: data.stroke_style,
                    global_alpha: data.global_alpha,
                    transform: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
                    line_cap: LineCap::default(),
                    line_join: LineJoin::default(),
                    miter_limit: 10.0,
                },
            );
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

    // Add rectangle to current path as four corners
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

    // Convert style to string first to avoid borrowing conflicts
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
    let mut data = res.canvases.get_mut(rid).unwrap();

    if let Some(style_str_obj) = style_string {
        let style_str = style_str_obj
            .as_str(agent)
            .expect("String is not valid UTF-8");
        if let Ok(parsed_style) =
            FillStyle::from_css_color(style_str).map_err(|_| "Invalid color format")
        {
            data.stroke_style = parsed_style.clone();
            data.commands
                .push(CanvasCommand::SetStrokeStyle(parsed_style));
        }
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

    // Convert Nova VM Numbers to f64 for renderer
    let x_f64 = x.into_f64(agent);
    let y_f64 = y.into_f64(agent);
    let width_f64 = width.into_f64(agent);
    let height_f64 = height.into_f64(agent);
    let radius_f64 = radius.into_f64(agent);

    // Tessellate rounded rectangle to current path
    tessellate_rounded_rect_to_path(
        &mut data.current_path,
        x_f64,
        y_f64,
        width_f64,
        height_f64,
        radius_f64,
    );

    data.commands.push(CanvasCommand::RoundRect {
        x,
        y,
        width,
        height,
        radius,
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
        line_cap: LineCap::default(),
        line_join: LineJoin::default(),
        miter_limit: 10.0,
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

    // Restore state from stack if available
    if let Some(saved_state) = data.state_stack.pop() {
        data.fill_style = saved_state.fill_style;
        data.stroke_style = saved_state.stroke_style;
        data.line_width = saved_state.line_width;
        data.global_alpha = saved_state.global_alpha;
        data.transform = saved_state.transform;
        data.line_dash = saved_state.line_dash;
        data.line_dash_offset = saved_state.line_dash_offset;
    }

    // Add restore command to command list
    data.commands.push(CanvasCommand::Restore);

    Ok(Value::Undefined)
}

/// Process all saved commands and render them using the renderer
#[allow(dead_code, clippy::needless_lifetimes)]
pub fn process_all_commands<'gc>(
    commands: &Vec<CanvasCommand<'gc>>,
    renderer: &mut crate::ext::canvas::renderer::Renderer,
    agent: &mut Agent,
    fill_style: &crate::ext::canvas::FillStyle,
    stroke_style: &crate::ext::canvas::FillStyle,
    line_width: f64,
    global_alpha: f32,
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
                    radius_f64,
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
                            fill_style: fill_style.clone(),
                            global_alpha,
                            transform: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
                            line_cap: LineCap::default(),
                            line_join: LineJoin::default(),
                            miter_limit: 10.0,
                        },
                    );
                }
            }
            CanvasCommand::Stroke => {
                if !current_path.is_empty() {
                    renderer.render_polyline(
                        current_path.clone(),
                        &RenderState {
                            fill_style: stroke_style.clone(),
                            global_alpha,
                            transform: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
                            line_cap: LineCap::default(),
                            line_join: LineJoin::default(),
                            miter_limit: 10.0,
                        },
                        line_width,
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
                        fill_style: fill_style.clone(),
                        global_alpha,
                        transform: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
                        line_cap: LineCap::default(),
                        line_join: LineJoin::default(),
                        miter_limit: 10.0,
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
                        fill_style: stroke_style.clone(),
                        global_alpha,
                        transform: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
                        line_cap: LineCap::default(),
                        line_join: LineJoin::default(),
                        miter_limit: 10.0,
                    },
                    line_width,
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
            CanvasCommand::Clip { path: _ } => {
                // Apply clipping to the current renderer state
                // TODO: Implement proper clipping in the renderer
                // For now, this is a placeholder that stores the clipping path
            }
            CanvasCommand::CreateLinearGradient { .. }
            | CanvasCommand::CreateRadialGradient { .. }
            | CanvasCommand::CreateConicGradient { .. } => {
                // These commands would need more complex state management
                // For now, we'll skip them in this basic implementation
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
fn tessellate_rounded_rect_to_path(
    path: &mut Vec<crate::ext::canvas::renderer::Point>,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    radius: f64,
) {
    let radius = radius.min(width / 2.0).min(height / 2.0).max(0.0);

    if radius <= 0.0 {
        // Simple rectangle
        path.push(crate::ext::canvas::renderer::Point { x, y });
        path.push(crate::ext::canvas::renderer::Point { x: x + width, y });
        path.push(crate::ext::canvas::renderer::Point {
            x: x + width,
            y: y + height,
        });
        path.push(crate::ext::canvas::renderer::Point { x, y: y + height });
        path.push(crate::ext::canvas::renderer::Point { x, y }); // Close
        return;
    }

    const CORNER_SEGMENTS: usize = 8; // Segments per corner

    // Top-left corner (start at right edge of arc)
    for i in 0..=CORNER_SEGMENTS {
        let angle =
            std::f64::consts::PI + i as f64 * std::f64::consts::PI / (2.0 * CORNER_SEGMENTS as f64);
        let corner_x = x + radius + radius * angle.cos();
        let corner_y = y + radius + radius * angle.sin();
        path.push(crate::ext::canvas::renderer::Point {
            x: corner_x,
            y: corner_y,
        });
    }

    // Top edge
    path.push(crate::ext::canvas::renderer::Point {
        x: x + width - radius,
        y,
    });

    // Top-right corner
    for i in 0..=CORNER_SEGMENTS {
        let angle = -std::f64::consts::PI / 2.0
            + i as f64 * std::f64::consts::PI / (2.0 * CORNER_SEGMENTS as f64);
        let corner_x = x + width - radius + radius * angle.cos();
        let corner_y = y + radius + radius * angle.sin();
        path.push(crate::ext::canvas::renderer::Point {
            x: corner_x,
            y: corner_y,
        });
    }

    // Right edge
    path.push(crate::ext::canvas::renderer::Point {
        x: x + width,
        y: y + height - radius,
    });

    // Bottom-right corner
    for i in 0..=CORNER_SEGMENTS {
        let angle = 0.0 + i as f64 * std::f64::consts::PI / (2.0 * CORNER_SEGMENTS as f64);
        let corner_x = x + width - radius + radius * angle.cos();
        let corner_y = y + height - radius + radius * angle.sin();
        path.push(crate::ext::canvas::renderer::Point {
            x: corner_x,
            y: corner_y,
        });
    }

    // Bottom edge
    path.push(crate::ext::canvas::renderer::Point {
        x: x + radius,
        y: y + height,
    });

    // Bottom-left corner
    for i in 0..=CORNER_SEGMENTS {
        let angle = std::f64::consts::PI / 2.0
            + i as f64 * std::f64::consts::PI / (2.0 * CORNER_SEGMENTS as f64);
        let corner_x = x + radius + radius * angle.cos();
        let corner_y = y + height - radius + radius * angle.sin();
        path.push(crate::ext::canvas::renderer::Point {
            x: corner_x,
            y: corner_y,
        });
    }

    // Left edge and close
    path.push(crate::ext::canvas::renderer::Point { x, y: y + radius });
}

/// Helper function to tessellate arc and add to path (for existing arc function)
fn tessellate_arc_to_path(
    path: &mut Vec<crate::ext::canvas::renderer::Point>,
    x: f64,
    y: f64,
    radius: f64,
    start_angle: f64,
    end_angle: f64,
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
        false,
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
fn generate_stroke_path(
    path: &[crate::ext::canvas::renderer::Point],
    line_width: f64,
) -> Vec<crate::ext::canvas::renderer::Point> {
    if path.len() < 2 {
        return Vec::new();
    }

    let half_width = line_width / 2.0;
    let mut stroke_vertices = Vec::new();

    // Simple stroke generation - create parallel lines on both sides
    for i in 0..path.len() - 1 {
        let a = &path[i];
        let b = &path[i + 1];

        // Calculate perpendicular vector
        let dx = b.x - a.x;
        let dy = b.y - a.y;
        let length = (dx * dx + dy * dy).sqrt();

        if length > 0.0 {
            let nx = -dy / length * half_width; // Perpendicular x
            let ny = dx / length * half_width; // Perpendicular y

            // Add vertices for the stroke quad
            stroke_vertices.push(crate::ext::canvas::renderer::Point {
                x: a.x + nx,
                y: a.y + ny,
            });
            stroke_vertices.push(crate::ext::canvas::renderer::Point {
                x: a.x - nx,
                y: a.y - ny,
            });
            stroke_vertices.push(crate::ext::canvas::renderer::Point {
                x: b.x + nx,
                y: b.y + ny,
            });
            stroke_vertices.push(crate::ext::canvas::renderer::Point {
                x: b.x - nx,
                y: b.y - ny,
            });
        }
    }

    stroke_vertices
}
