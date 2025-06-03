// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use super::CanvasResources;
use super::Rid;
use crate::RuntimeMacroTask;
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
        tessellate_bezier_to_path(
            &mut data.current_path,
            last_point.x,
            last_point.y,
            cp1x_f64,
            cp1y_f64,
            cp2x_f64,
            cp2y_f64,
            x_f64,
            y_f64,
        );
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
        let transparent_color = [0.0, 0.0, 0.0, 0.0]; // Transparent black
        renderer.render_rect(clear_rect, transparent_color);
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
        use super::renderer::{Point, Rect};
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
        let color = match &data.fill_style {
            super::FillStyle::Color { r, g, b, a } => [*r, *g, *b, *a],
            _ => [0.0, 0.0, 0.0, 1.0], // Default black
        };

        res.renderers.get_mut(rid).unwrap().render_rect(rect, color);
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

        if data.current_path.len() >= 3 {
            // Get the current fill style color
            let color = match &data.fill_style {
                super::FillStyle::Color { r, g, b, a } => [*r, *g, *b, *a],
                _ => [0.0, 0.0, 0.0, 1.0], // Default black
            };

            // Render the polygon using the GPU renderer
            renderer.render_polygon(data.current_path.clone(), color);
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
            // Get the current stroke style color
            let color = match &data.stroke_style {
                super::FillStyle::Color { r, g, b, a } => [*r, *g, *b, *a],
                _ => [0.0, 0.0, 0.0, 1.0], // Default black
            };

            // Convert path to stroke polygon using line width
            let stroke_path = generate_stroke_path(&data.current_path, data.line_width);

            // Render the stroke as a polygon using the GPU renderer
            renderer.render_polygon(stroke_path, color);
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
    let color_str = args
        .get(1)
        .to_string(agent, gc.reborrow())
        .unbind()
        .unwrap();
    let color_str = color_str.as_str(agent);

    let host_data = agent
        .get_host_data()
        .downcast_ref::<HostData<RuntimeMacroTask>>()
        .unwrap();
    let mut storage = host_data.storage.borrow_mut();
    let res: &mut CanvasResources = storage.get_mut().unwrap();

    let mut data = res.canvases.get_mut(rid).unwrap();

    // Parse color string (simplified - just handle basic hex colors)
    if let Some(color) = parse_color_string(color_str) {
        data.stroke_style = color;
    }

    Ok(Value::Undefined)
}

/// Simple color string parser (handles basic hex colors like "#RRGGBB" and "#RRGGBBAA")
fn parse_color_string(color_str: &str) -> Option<super::FillStyle> {
    if color_str.starts_with('#') && color_str.len() >= 7 {
        let hex = &color_str[1..];

        // Parse RGB components
        if let (Ok(r), Ok(g), Ok(b)) = (
            u8::from_str_radix(&hex[0..2], 16),
            u8::from_str_radix(&hex[2..4], 16),
            u8::from_str_radix(&hex[4..6], 16),
        ) {
            let a = if hex.len() >= 8 {
                u8::from_str_radix(&hex[6..8], 16).unwrap_or(255)
            } else {
                255
            };

            return Some(super::FillStyle::Color {
                r: r as f32 / 255.0,
                g: g as f32 / 255.0,
                b: b as f32 / 255.0,
                a: a as f32 / 255.0,
            });
        }
    }

    match color_str {
        "aliceblue" => Some(super::FillStyle::Color {
            r: 0.941,
            g: 0.973,
            b: 1.0,
            a: 1.0,
        }),
        "antiquewhite" => Some(super::FillStyle::Color {
            r: 0.98,
            g: 0.92,
            b: 0.84,
            a: 1.0,
        }),
        "aqua" => Some(super::FillStyle::Color {
            r: 0.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        }),
        "aquamarine" => Some(super::FillStyle::Color {
            r: 0.498,
            g: 1.0,
            b: 0.831,
            a: 1.0,
        }),
        "azure" => Some(super::FillStyle::Color {
            r: 0.941,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        }),
        "beige" => Some(super::FillStyle::Color {
            r: 0.96,
            g: 0.96,
            b: 0.86,
            a: 1.0,
        }),
        "bisque" => Some(super::FillStyle::Color {
            r: 1.0,
            g: 0.894,
            b: 0.769,
            a: 1.0,
        }),
        "black" => Some(super::FillStyle::Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        }),
        "blanchedalmond" => Some(super::FillStyle::Color {
            r: 1.0,
            g: 0.922,
            b: 0.804,
            a: 1.0,
        }),
        "blue" => Some(super::FillStyle::Color {
            r: 0.0,
            g: 0.0,
            b: 1.0,
            a: 1.0,
        }),
        "blueviolet" => Some(super::FillStyle::Color {
            r: 0.541,
            g: 0.169,
            b: 0.886,
            a: 1.0,
        }),
        "brown" => Some(super::FillStyle::Color {
            r: 0.647,
            g: 0.165,
            b: 0.165,
            a: 1.0,
        }),
        "burlywood" => Some(super::FillStyle::Color {
            r: 0.871,
            g: 0.722,
            b: 0.529,
            a: 1.0,
        }),
        "cadetblue" => Some(super::FillStyle::Color {
            r: 0.373,
            g: 0.62,
            b: 0.627,
            a: 1.0,
        }),
        "cameo" => Some(super::FillStyle::Color {
            r: 0.937,
            g: 0.867,
            b: 0.702,
            a: 1.0,
        }),
        "chartreuse" => Some(super::FillStyle::Color {
            r: 0.498,
            g: 1.0,
            b: 0.0,
            a: 1.0,
        }),
        "chocolate" => Some(super::FillStyle::Color {
            r: 0.824,
            g: 0.412,
            b: 0.118,
            a: 1.0,
        }),
        "coral" => Some(super::FillStyle::Color {
            r: 1.0,
            g: 0.498,
            b: 0.314,
            a: 1.0,
        }),
        "cornflowerblue" => Some(super::FillStyle::Color {
            r: 0.392,
            g: 0.584,
            b: 0.929,
            a: 1.0,
        }),
        "crimson" => Some(super::FillStyle::Color {
            r: 0.863,
            g: 0.078,
            b: 0.235,
            a: 1.0,
        }),
        "cyan" => Some(super::FillStyle::Color {
            r: 0.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        }),
        "darkblue" => Some(super::FillStyle::Color {
            r: 0.0,
            g: 0.0,
            b: 0.545,
            a: 1.0,
        }),
        "darkcyan" => Some(super::FillStyle::Color {
            r: 0.0,
            g: 0.545,
            b: 0.545,
            a: 1.0,
        }),
        "darkgoldenrod" => Some(super::FillStyle::Color {
            r: 0.722,
            g: 0.525,
            b: 0.043,
            a: 1.0,
        }),
        "darkgray" => Some(super::FillStyle::Color {
            r: 0.663,
            g: 0.663,
            b: 0.663,
            a: 1.0,
        }),
        "darkgreen" => Some(super::FillStyle::Color {
            r: 0.0,
            g: 0.392,
            b: 0.0,
            a: 1.0,
        }),
        "darkkhaki" => Some(super::FillStyle::Color {
            r: 0.741,
            g: 0.717,
            b: 0.419,
            a: 1.0,
        }),
        "darkmagenta" => Some(super::FillStyle::Color {
            r: 0.545,
            g: 0.0,
            b: 0.545,
            a: 1.0,
        }),
        "darkolivegreen" => Some(super::FillStyle::Color {
            r: 0.333,
            g: 0.42,
            b: 0.184,
            a: 1.0,
        }),
        "darkorange" => Some(super::FillStyle::Color {
            r: 1.0,
            g: 0.549,
            b: 0.0,
            a: 1.0,
        }),
        "darkorchid" => Some(super::FillStyle::Color {
            r: 0.6,
            g: 0.196,
            b: 0.8,
            a: 1.0,
        }),
        "darkred" => Some(super::FillStyle::Color {
            r: 0.545,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        }),
        "darksalmon" => Some(super::FillStyle::Color {
            r: 0.914,
            g: 0.588,
            b: 0.478,
            a: 1.0,
        }),
        "darkseagreen" => Some(super::FillStyle::Color {
            r: 0.561,
            g: 0.737,
            b: 0.561,
            a: 1.0,
        }),
        "darkslateblue" => Some(super::FillStyle::Color {
            r: 0.282,
            g: 0.239,
            b: 0.545,
            a: 1.0,
        }),
        "darkslategray" => Some(super::FillStyle::Color {
            r: 0.184,
            g: 0.31,
            b: 0.31,
            a: 1.0,
        }),
        "darkturquoise" => Some(super::FillStyle::Color {
            r: 0.0,
            g: 0.807,
            b: 0.819,
            a: 1.0,
        }),
        "darkviolet" => Some(super::FillStyle::Color {
            r: 0.58,
            g: 0.0,
            b: 0.827,
            a: 1.0,
        }),
        "deeppink" => Some(super::FillStyle::Color {
            r: 1.0,
            g: 0.078,
            b: 0.576,
            a: 1.0,
        }),
        "deepskyblue" => Some(super::FillStyle::Color {
            r: 0.0,
            g: 0.749,
            b: 1.0,
            a: 1.0,
        }),
        "dimgray" => Some(super::FillStyle::Color {
            r: 0.412,
            g: 0.412,
            b: 0.412,
            a: 1.0,
        }),
        "dimpurple" => Some(super::FillStyle::Color {
            r: 0.415,
            g: 0.313,
            b: 0.494,
            a: 1.0,
        }),
        "dodgerblue" => Some(super::FillStyle::Color {
            r: 0.118,
            g: 0.565,
            b: 1.0,
            a: 1.0,
        }),
        "firebrick" => Some(super::FillStyle::Color {
            r: 0.698,
            g: 0.132,
            b: 0.203,
            a: 1.0,
        }),
        "floralwhite" => Some(super::FillStyle::Color {
            r: 1.0,
            g: 0.98,
            b: 0.941,
            a: 1.0,
        }),
        "forestgreen" => Some(super::FillStyle::Color {
            r: 0.133,
            g: 0.545,
            b: 0.133,
            a: 1.0,
        }),
        "fuchsia" => Some(super::FillStyle::Color {
            r: 1.0,
            g: 0.0,
            b: 1.0,
            a: 1.0,
        }),
        "gainsboro" => Some(super::FillStyle::Color {
            r: 0.863,
            g: 0.863,
            b: 0.863,
            a: 1.0,
        }),
        "ghostwhite" => Some(super::FillStyle::Color {
            r: 0.973,
            g: 0.973,
            b: 1.0,
            a: 1.0,
        }),
        "gold" => Some(super::FillStyle::Color {
            r: 1.0,
            g: 0.843,
            b: 0.0,
            a: 1.0,
        }),
        "goldenrod" => Some(super::FillStyle::Color {
            r: 0.855,
            g: 0.647,
            b: 0.125,
            a: 1.0,
        }),
        "gray" => Some(super::FillStyle::Color {
            r: 0.5,
            g: 0.5,
            b: 0.5,
            a: 1.0,
        }),
        "green" => Some(super::FillStyle::Color {
            r: 0.0,
            g: 1.0,
            b: 0.0,
            a: 1.0,
        }),
        "greenyellow" => Some(super::FillStyle::Color {
            r: 0.678,
            g: 1.0,
            b: 0.184,
            a: 1.0,
        }),
        "honeydew" => Some(super::FillStyle::Color {
            r: 0.941,
            g: 1.0,
            b: 0.941,
            a: 1.0,
        }),
        "hotpink" => Some(super::FillStyle::Color {
            r: 1.0,
            g: 0.412,
            b: 0.706,
            a: 1.0,
        }),
        "indianred" => Some(super::FillStyle::Color {
            r: 0.804,
            g: 0.361,
            b: 0.361,
            a: 1.0,
        }),
        "indigo" => Some(super::FillStyle::Color {
            r: 0.294,
            g: 0.0,
            b: 0.51,
            a: 1.0,
        }),
        "ivory" => Some(super::FillStyle::Color {
            r: 1.0,
            g: 1.0,
            b: 0.941,
            a: 1.0,
        }),
        "khaki" => Some(super::FillStyle::Color {
            r: 0.941,
            g: 0.902,
            b: 0.549,
            a: 1.0,
        }),
        "lavender" => Some(super::FillStyle::Color {
            r: 0.902,
            g: 0.902,
            b: 0.98,
            a: 1.0,
        }),
        "lavenderblush" => Some(super::FillStyle::Color {
            r: 1.0,
            g: 0.941,
            b: 0.961,
            a: 1.0,
        }),
        "lawngreen" => Some(super::FillStyle::Color {
            r: 0.486,
            g: 0.988,
            b: 0.0,
            a: 1.0,
        }),
        "lemonchiffon" => Some(super::FillStyle::Color {
            r: 1.0,
            g: 0.98,
            b: 0.804,
            a: 1.0,
        }),
        "lightblue" => Some(super::FillStyle::Color {
            r: 0.678,
            g: 0.847,
            b: 0.902,
            a: 1.0,
        }),
        "lightcoral" => Some(super::FillStyle::Color {
            r: 0.941,
            g: 0.502,
            b: 0.502,
            a: 1.0,
        }),
        "lightcyan" => Some(super::FillStyle::Color {
            r: 0.878,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        }),
        "lightgoldenrodyellow" => Some(super::FillStyle::Color {
            r: 0.980,
            g: 0.980,
            b: 0.824,
            a: 1.0,
        }),
        "lightgray" => Some(super::FillStyle::Color {
            r: 0.827,
            g: 0.827,
            b: 0.827,
            a: 1.0,
        }),
        "lightgreen" => Some(super::FillStyle::Color {
            r: 0.565,
            g: 0.933,
            b: 0.565,
            a: 1.0,
        }),
        "lightpink" => Some(super::FillStyle::Color {
            r: 1.0,
            g: 0.714,
            b: 0.757,
            a: 1.0,
        }),
        "lightsalmon" => Some(super::FillStyle::Color {
            r: 1.0,
            g: 0.627,
            b: 0.478,
            a: 1.0,
        }),
        "lightseagreen" => Some(super::FillStyle::Color {
            r: 0.125,
            g: 0.698,
            b: 0.667,
            a: 1.0,
        }),
        "lightskyblue" => Some(super::FillStyle::Color {
            r: 0.529,
            g: 0.808,
            b: 0.98,
            a: 1.0,
        }),
        "lightslategray" => Some(super::FillStyle::Color {
            r: 0.467,
            g: 0.533,
            b: 0.6,
            a: 1.0,
        }),
        "lightsteelblue" => Some(super::FillStyle::Color {
            r: 0.690,
            g: 0.769,
            b: 0.871,
            a: 1.0,
        }),
        "lightyellow" => Some(super::FillStyle::Color {
            r: 1.0,
            g: 1.0,
            b: 0.878,
            a: 1.0,
        }),
        "lime" => Some(super::FillStyle::Color {
            r: 0.0,
            g: 1.0,
            b: 0.0,
            a: 1.0,
        }),
        "limegreen" => Some(super::FillStyle::Color {
            r: 0.196,
            g: 0.804,
            b: 0.196,
            a: 1.0,
        }),
        "linen" => Some(super::FillStyle::Color {
            r: 0.98,
            g: 0.941,
            b: 0.902,
            a: 1.0,
        }),
        "magenta" => Some(super::FillStyle::Color {
            r: 1.0,
            g: 0.0,
            b: 1.0,
            a: 1.0,
        }),
        "maroon" => Some(super::FillStyle::Color {
            r: 0.5,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        }),
        "mediumaquamarine" => Some(super::FillStyle::Color {
            r: 0.4,
            g: 0.804,
            b: 0.667,
            a: 1.0,
        }),
        "mediumblue" => Some(super::FillStyle::Color {
            r: 0.0,
            g: 0.0,
            b: 0.804,
            a: 1.0,
        }),
        "mediumorchid" => Some(super::FillStyle::Color {
            r: 0.729,
            g: 0.333,
            b: 0.827,
            a: 1.0,
        }),
        "mediumpurple" => Some(super::FillStyle::Color {
            r: 0.576,
            g: 0.439,
            b: 0.859,
            a: 1.0,
        }),
        "mediumseagreen" => Some(super::FillStyle::Color {
            r: 0.235,
            g: 0.702,
            b: 0.443,
            a: 1.0,
        }),
        "mediumslateblue" => Some(super::FillStyle::Color {
            r: 0.482,
            g: 0.408,
            b: 0.933,
            a: 1.0,
        }),
        "mediumspringgreen" => Some(super::FillStyle::Color {
            r: 0.0,
            g: 0.98,
            b: 0.604,
            a: 1.0,
        }),
        "mediumturquoise" => Some(super::FillStyle::Color {
            r: 0.282,
            g: 0.82,
            b: 0.8,
            a: 1.0,
        }),
        "mediumvioletred" => Some(super::FillStyle::Color {
            r: 0.78,
            g: 0.082,
            b: 0.522,
            a: 1.0,
        }),
        "midnightblue" => Some(super::FillStyle::Color {
            r: 0.098,
            g: 0.098,
            b: 0.439,
            a: 1.0,
        }),
        "mintcream" => Some(super::FillStyle::Color {
            r: 0.961,
            g: 1.0,
            b: 0.98,
            a: 1.0,
        }),
        "mistyrose" => Some(super::FillStyle::Color {
            r: 1.0,
            g: 0.894,
            b: 0.882,
            a: 1.0,
        }),
        "mocha" => Some(super::FillStyle::Color {
            r: 0.824,
            g: 0.706,
            b: 0.549,
            a: 1.0,
        }),
        "navajowhite" => Some(super::FillStyle::Color {
            r: 1.0,
            g: 0.871,
            b: 0.678,
            a: 1.0,
        }),
        "navy" => Some(super::FillStyle::Color {
            r: 0.0,
            g: 0.0,
            b: 0.5,
            a: 1.0,
        }),
        "oldlace" => Some(super::FillStyle::Color {
            r: 0.992,
            g: 0.961,
            b: 0.902,
            a: 1.0,
        }),
        "olive" => Some(super::FillStyle::Color {
            r: 0.5,
            g: 0.5,
            b: 0.0,
            a: 1.0,
        }),
        "olivedrab" => Some(super::FillStyle::Color {
            r: 0.42,
            g: 0.557,
            b: 0.137,
            a: 1.0,
        }),
        "orange" => Some(super::FillStyle::Color {
            r: 1.0,
            g: 0.647,
            b: 0.0,
            a: 1.0,
        }),
        "orangered" => Some(super::FillStyle::Color {
            r: 1.0,
            g: 0.271,
            b: 0.0,
            a: 1.0,
        }),
        "orchid" => Some(super::FillStyle::Color {
            r: 0.855,
            g: 0.439,
            b: 0.839,
            a: 1.0,
        }),
        "palegoldenrod" => Some(super::FillStyle::Color {
            r: 0.933,
            g: 0.91,
            b: 0.667,
            a: 1.0,
        }),
        "palegreen" => Some(super::FillStyle::Color {
            r: 0.596,
            g: 0.984,
            b: 0.596,
            a: 1.0,
        }),
        "paleturquoise" => Some(super::FillStyle::Color {
            r: 0.686,
            g: 0.933,
            b: 0.933,
            a: 1.0,
        }),
        "palevioletred" => Some(super::FillStyle::Color {
            r: 0.859,
            g: 0.439,
            b: 0.576,
            a: 1.0,
        }),
        "papayawhip" => Some(super::FillStyle::Color {
            r: 1.0,
            g: 0.937,
            b: 0.835,
            a: 1.0,
        }),
        "peachpuff" => Some(super::FillStyle::Color {
            r: 1.0,
            g: 0.855,
            b: 0.725,
            a: 1.0,
        }),
        "peru" => Some(super::FillStyle::Color {
            r: 0.804,
            g: 0.522,
            b: 0.247,
            a: 1.0,
        }),
        "pink" => Some(super::FillStyle::Color {
            r: 1.0,
            g: 0.753,
            b: 0.796,
            a: 1.0,
        }),
        "plum" => Some(super::FillStyle::Color {
            r: 0.867,
            g: 0.627,
            b: 0.867,
            a: 1.0,
        }),
        "powderblue" => Some(super::FillStyle::Color {
            r: 0.69,
            g: 0.878,
            b: 0.902,
            a: 1.0,
        }),
        "purple" => Some(super::FillStyle::Color {
            r: 0.5,
            g: 0.0,
            b: 0.5,
            a: 1.0,
        }),
        "red" => Some(super::FillStyle::Color {
            r: 1.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        }),
        "rosybrown" => Some(super::FillStyle::Color {
            r: 0.737,
            g: 0.561,
            b: 0.561,
            a: 1.0,
        }),
        "royalblue" => Some(super::FillStyle::Color {
            r: 0.255,
            g: 0.412,
            b: 0.882,
            a: 1.0,
        }),
        "saddlebrown" => Some(super::FillStyle::Color {
            r: 0.545,
            g: 0.271,
            b: 0.075,
            a: 1.0,
        }),
        "salmon" => Some(super::FillStyle::Color {
            r: 0.98,
            g: 0.502,
            b: 0.447,
            a: 1.0,
        }),
        "sandybrown" => Some(super::FillStyle::Color {
            r: 0.957,
            g: 0.643,
            b: 0.376,
            a: 1.0,
        }),
        "seagreen" => Some(super::FillStyle::Color {
            r: 0.18,
            g: 0.545,
            b: 0.341,
            a: 1.0,
        }),
        "seashell" => Some(super::FillStyle::Color {
            r: 1.0,
            g: 0.961,
            b: 0.933,
            a: 1.0,
        }),
        "sienna" => Some(super::FillStyle::Color {
            r: 0.627,
            g: 0.322,
            b: 0.176,
            a: 1.0,
        }),
        "silver" => Some(super::FillStyle::Color {
            r: 0.75,
            g: 0.75,
            b: 0.75,
            a: 1.0,
        }),
        "skyblue" => Some(super::FillStyle::Color {
            r: 0.529,
            g: 0.808,
            b: 0.922,
            a: 1.0,
        }),
        "slateblue" => Some(super::FillStyle::Color {
            r: 0.416,
            g: 0.353,
            b: 0.804,
            a: 1.0,
        }),
        "slategray" => Some(super::FillStyle::Color {
            r: 0.439,
            g: 0.502,
            b: 0.565,
            a: 1.0,
        }),
        "snow" => Some(super::FillStyle::Color {
            r: 1.0,
            g: 0.98,
            b: 0.98,
            a: 1.0,
        }),
        "springgreen" => Some(super::FillStyle::Color {
            r: 0.0,
            g: 1.0,
            b: 0.498,
            a: 1.0,
        }),
        "steelblue" => Some(super::FillStyle::Color {
            r: 0.275,
            g: 0.51,
            b: 0.706,
            a: 1.0,
        }),
        "tan" => Some(super::FillStyle::Color {
            r: 0.824,
            g: 0.706,
            b: 0.549,
            a: 1.0,
        }),
        "teal" => Some(super::FillStyle::Color {
            r: 0.0,
            g: 0.5,
            b: 0.5,
            a: 1.0,
        }),
        "thistle" => Some(super::FillStyle::Color {
            r: 0.847,
            g: 0.749,
            b: 0.847,
            a: 1.0,
        }),
        "tomato" => Some(super::FillStyle::Color {
            r: 1.0,
            g: 0.388,
            b: 0.278,
            a: 1.0,
        }),
        "turquoise" => Some(super::FillStyle::Color {
            r: 0.251,
            g: 0.878,
            b: 0.816,
            a: 1.0,
        }),
        "violet" => Some(super::FillStyle::Color {
            r: 0.933,
            g: 0.51,
            b: 0.933,
            a: 1.0,
        }),
        "wheat" => Some(super::FillStyle::Color {
            r: 0.961,
            g: 0.871,
            b: 0.702,
            a: 1.0,
        }),
        "white" => Some(super::FillStyle::Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        }),
        "whitesmoke" => Some(super::FillStyle::Color {
            r: 0.961,
            g: 0.961,
            b: 0.961,
            a: 1.0,
        }),
        "yellow" => Some(super::FillStyle::Color {
            r: 1.0,
            g: 1.0,
            b: 0.0,
            a: 1.0,
        }),
        "yellowgreen" => Some(super::FillStyle::Color {
            r: 0.604,
            g: 0.804,
            b: 0.196,
            a: 1.0,
        }),

        _ => None,
    }
}

/// Tessellate an arc into line segments and add to path
fn tessellate_arc_to_path(
    path: &mut Vec<crate::ext::canvas::renderer::Point>,
    center_x: f64,
    center_y: f64,
    radius: f64,
    start_angle: f64,
    end_angle: f64,
) {
    const SEGMENTS: usize = 32; // Number of segments for arc approximation

    let angle_diff = end_angle - start_angle;
    let step = angle_diff / SEGMENTS as f64;

    for i in 0..=SEGMENTS {
        let angle = start_angle + (i as f64 * step);
        let x = center_x + radius * angle.cos();
        let y = center_y + radius * angle.sin();
        path.push(crate::ext::canvas::renderer::Point { x, y });
    }
}

#[allow(clippy::too_many_arguments)]
/// Tessellate a cubic Bezier curve into line segments and add to path
fn tessellate_bezier_to_path(
    path: &mut Vec<crate::ext::canvas::renderer::Point>,
    start_x: f64,
    start_y: f64,
    cp1x: f64,
    cp1y: f64,
    cp2x: f64,
    cp2y: f64,
    end_x: f64,
    end_y: f64,
) {
    const SEGMENTS: usize = 20; // Number of segments for curve approximation

    for i in 0..=SEGMENTS {
        let t = i as f64 / SEGMENTS as f64;
        let t2 = t * t;
        let t3 = t2 * t;
        let mt = 1.0 - t;
        let mt2 = mt * mt;
        let mt3 = mt2 * mt;

        // Cubic Bezier formula: B(t) = (1-t)³P₀ + 3(1-t)²tP₁ + 3(1-t)t²P₂ + t³P₃
        let x = mt3 * start_x + 3.0 * mt2 * t * cp1x + 3.0 * mt * t2 * cp2x + t3 * end_x;
        let y = mt3 * start_y + 3.0 * mt2 * t * cp1y + 3.0 * mt * t2 * cp2y + t3 * end_y;

        path.push(crate::ext::canvas::renderer::Point { x, y });
    }
}

/// Generate a stroke path from a line path by creating a polygon with the specified width
fn generate_stroke_path(
    path: &[crate::ext::canvas::renderer::Point],
    line_width: f64,
) -> Vec<crate::ext::canvas::renderer::Point> {
    if path.len() < 2 {
        return Vec::new();
    }

    let half_width = line_width / 2.0;
    let mut stroke_polygon = Vec::new();

    // For each line segment, create perpendicular lines for the stroke width
    for window in path.windows(2) {
        let start = &window[0];
        let end = &window[1];

        // Calculate perpendicular vector
        let dx = end.x - start.x;
        let dy = end.y - start.y;
        let length = (dx * dx + dy * dy).sqrt();

        if length > 0.0 {
            let perpendicular_x = -dy / length * half_width;
            let perpendicular_y = dx / length * half_width;

            // Add points for this segment (simplified approach - creates rectangles)
            stroke_polygon.push(crate::ext::canvas::renderer::Point {
                x: start.x + perpendicular_x,
                y: start.y + perpendicular_y,
            });
            stroke_polygon.push(crate::ext::canvas::renderer::Point {
                x: start.x - perpendicular_x,
                y: start.y - perpendicular_y,
            });
            stroke_polygon.push(crate::ext::canvas::renderer::Point {
                x: end.x - perpendicular_x,
                y: end.y - perpendicular_y,
            });
            stroke_polygon.push(crate::ext::canvas::renderer::Point {
                x: end.x + perpendicular_x,
                y: end.y + perpendicular_y,
            });
        }
    }

    stroke_polygon
}
