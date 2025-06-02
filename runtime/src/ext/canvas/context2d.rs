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
    let mut data = res.canvases.get_mut(rid).unwrap();

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
        match res.renderers.get_mut(rid) {
            Some(_) => true,
            None => false,
        }
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
