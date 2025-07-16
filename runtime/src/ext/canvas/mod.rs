// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

mod context2d;
mod fill_style;
mod renderer;
mod state;
use andromeda_core::{Extension, ExtensionOp, HostData, OpsStorage, ResourceTable, Rid};
use std::ops::DerefMut;
use std::sync::{Arc, Mutex};
use std::thread;

use crate::ext::canvas::context2d::{
    internal_canvas_begin_path, internal_canvas_bezier_curve_to, internal_canvas_close_path,
};
use crate::ext::canvas::fill_style::LinearGradient;
use crate::ext::canvas::renderer::ColorStop;
pub use fill_style::FillStyle;

use crate::ext::canvas::context2d::{
    internal_canvas_arc, internal_canvas_arc_to, internal_canvas_clear_rect,
    internal_canvas_ellipse, internal_canvas_fill, internal_canvas_fill_rect,
    internal_canvas_line_to, internal_canvas_move_to, internal_canvas_quadratic_curve_to,
    internal_canvas_rect, internal_canvas_restore, internal_canvas_round_rect,
    internal_canvas_save, internal_canvas_set_line_width, internal_canvas_set_stroke_style,
    internal_canvas_stroke,
};
use nova_vm::{
    SmallInteger,
    ecmascript::{
        builtins::ArgumentsList,
        execution::{Agent, JsResult},
        types::Value,
    },
    engine::context::{Bindable, GcScope},
};

/// A Canvas extension
#[derive(Clone)]
struct CanvasData<'gc> {
    width: u32,
    height: u32,
    commands: Vec<context2d::CanvasCommand<'gc>>,
    fill_style: FillStyle,
    stroke_style: FillStyle,
    line_width: f64,
    global_alpha: f32,
    // Path state for renderer
    current_path: Vec<renderer::Point>,
    path_started: bool,
    // State stack for save/restore functionality
    state_stack: Vec<state::CanvasState>,
}

struct CanvasResources<'gc> {
    canvases: ResourceTable<CanvasData<'gc>>,
    images: ResourceTable<ImageData>,
    renderers: ResourceTable<renderer::Renderer>,
    fill_styles: ResourceTable<FillStyle>,
}

/// Stored image dimensions
#[derive(Clone)]
struct ImageData {
    width: u32,
    height: u32,
}

#[derive(Default)]
pub struct CanvasExt;

impl CanvasExt {
    pub fn new_extension() -> Extension {
        Extension {
            name: "canvas",
            ops: vec![
                // Internal operations for Canvas API
                ExtensionOp::new("internal_canvas_create", Self::internal_canvas_create, 2),
                ExtensionOp::new(
                    "internal_canvas_get_width",
                    Self::internal_canvas_get_width,
                    1,
                ),
                ExtensionOp::new(
                    "internal_canvas_get_height",
                    Self::internal_canvas_get_height,
                    1,
                ),
                // Context2D operations
                ExtensionOp::new("internal_canvas_arc", internal_canvas_arc, 5),
                ExtensionOp::new("internal_canvas_arc_to", internal_canvas_arc_to, 5),
                ExtensionOp::new(
                    "internal_canvas_bezier_curve_to",
                    internal_canvas_bezier_curve_to,
                    7,
                ),
                ExtensionOp::new("internal_canvas_begin_path", internal_canvas_begin_path, 1),
                ExtensionOp::new("internal_canvas_clear_rect", internal_canvas_clear_rect, 5),
                ExtensionOp::new("internal_canvas_close_path", internal_canvas_close_path, 1),
                ExtensionOp::new("internal_canvas_fill_rect", internal_canvas_fill_rect, 5),
                ExtensionOp::new("internal_canvas_move_to", internal_canvas_move_to, 3),
                ExtensionOp::new("internal_canvas_line_to", internal_canvas_line_to, 3),
                ExtensionOp::new("internal_canvas_fill", internal_canvas_fill, 1),
                ExtensionOp::new("internal_canvas_stroke", internal_canvas_stroke, 1),
                ExtensionOp::new("internal_canvas_rect", internal_canvas_rect, 5),
                ExtensionOp::new(
                    "internal_canvas_quadratic_curve_to",
                    internal_canvas_quadratic_curve_to,
                    5,
                ),
                ExtensionOp::new("internal_canvas_ellipse", internal_canvas_ellipse, 9),
                ExtensionOp::new("internal_canvas_round_rect", internal_canvas_round_rect, 6),
                ExtensionOp::new(
                    "internal_canvas_set_line_width",
                    internal_canvas_set_line_width,
                    2,
                ),
                ExtensionOp::new(
                    "internal_canvas_set_stroke_style",
                    internal_canvas_set_stroke_style,
                    2,
                ),
                ExtensionOp::new(
                    "internal_canvas_get_stroke_style",
                    Self::internal_canvas_get_stroke_style,
                    1,
                ),
                ExtensionOp::new(
                    "internal_canvas_get_line_width",
                    Self::internal_canvas_get_line_width,
                    1,
                ),
                ExtensionOp::new(
                    "internal_canvas_get_fill_style",
                    Self::internal_canvas_get_fill_style,
                    1,
                ),
                ExtensionOp::new(
                    "internal_canvas_set_fill_style",
                    Self::internal_canvas_set_fill_style,
                    2,
                ),
                ExtensionOp::new(
                    "internal_canvas_get_global_alpha",
                    Self::internal_canvas_get_global_alpha,
                    1,
                ),
                ExtensionOp::new(
                    "internal_canvas_set_global_alpha",
                    Self::internal_canvas_set_global_alpha,
                    2,
                ),
                ExtensionOp::new("internal_canvas_render", Self::internal_canvas_render, 1),
                ExtensionOp::new(
                    "internal_canvas_save_as_png",
                    Self::internal_canvas_save_as_png,
                    2,
                ),
                ExtensionOp::new("internal_canvas_save", internal_canvas_save, 1),
                ExtensionOp::new("internal_canvas_restore", internal_canvas_restore, 1),
                // ImageBitmap API
                ExtensionOp::new(
                    "internal_image_bitmap_create",
                    Self::internal_image_bitmap_create,
                    1,
                ),
                ExtensionOp::new(
                    "internal_image_bitmap_get_width",
                    Self::internal_image_bitmap_get_width,
                    1,
                ),
                ExtensionOp::new(
                    "internal_image_bitmap_get_height",
                    Self::internal_image_bitmap_get_height,
                    1,
                ),
                ExtensionOp::new(
                    "internal_canvas_create_linear_gradient",
                    Self::internal_canvas_create_linear_gradient,
                    4,
                ),
                ExtensionOp::new(
                    "internal_canvas_gradient_add_color_stop",
                    Self::internal_canvas_gradient_add_color_stop,
                    2,
                ),
            ],
            storage: Some(Box::new(|storage: &mut OpsStorage| {
                storage.insert(CanvasResources {
                    canvases: ResourceTable::new(),
                    images: ResourceTable::new(),
                    renderers: ResourceTable::new(),
                    fill_styles: ResourceTable::new(),
                });
            })),
            files: vec![include_str!("./mod.ts"), include_str!("./image.ts")],
        }
    }

    fn internal_canvas_create<'gc>(
        agent: &mut Agent,
        _this: Value<'_>,
        args: ArgumentsList<'_, '_>,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let width = args.get(0).to_int32(agent, gc.reborrow()).unbind()? as u32;
        let height = args.get(1).to_int32(agent, gc.reborrow()).unbind()? as u32;
        let host_data = agent
            .get_host_data()
            .downcast_ref::<HostData<crate::RuntimeMacroTask>>()
            .unwrap();
        let mut storage = host_data.storage.borrow_mut();
        let res: &mut CanvasResources = storage.get_mut().unwrap(); // Create canvas data
        let canvas_rid = res.canvases.push(CanvasData {
            width,
            height,
            commands: Vec::new(),
            fill_style: FillStyle::default(),
            stroke_style: FillStyle::default(),
            line_width: 1.0,
            global_alpha: 1.0,
            current_path: Vec::new(),
            path_started: false,
            state_stack: Vec::new(),
        });

        // Create renderer with GPU device
        let (device, queue) = create_wgpu_device_sync();
        let dimensions = renderer::Dimensions { width, height };
        let format = wgpu::TextureFormat::Bgra8UnormSrgb; // Common format for canvas
        let renderer = renderer::Renderer::new(device, queue, format, dimensions);
        let _renderer_rid = res.renderers.push(renderer);

        Ok(Value::Integer(
            SmallInteger::from(canvas_rid.index() as i32),
        ))
    }

    /// Internal op to get the current globalAlpha of a canvas context
    #[allow(dead_code)]
    fn internal_canvas_get_global_alpha<'gc>(
        agent: &mut Agent,
        _this: Value<'_>,
        args: ArgumentsList<'_, '_>,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let rid_val = args.get(0).to_int32(agent, gc.reborrow()).unbind().unwrap() as u32;
        let rid = Rid::from_index(rid_val);
        let host_data = agent
            .get_host_data()
            .downcast_ref::<HostData<crate::RuntimeMacroTask>>()
            .unwrap();
        let storage = host_data.storage.borrow();
        let res: &CanvasResources = storage.get().unwrap();
        let data = res.canvases.get(rid).unwrap();
        let global_alpha = data.global_alpha;
        drop(storage);

        // Convert global_alpha (0.0-1.0) to integer representation for return
        // We multiply by 1000 to preserve 3 decimal places precision
        let alpha_int = (global_alpha * 1000.0) as i32;
        Ok(Value::Integer(SmallInteger::from(alpha_int)))
    }

    /// Internal op to set the globalAlpha of a canvas context
    #[allow(dead_code)]
    fn internal_canvas_set_global_alpha<'gc>(
        agent: &mut Agent,
        _this: Value<'_>,
        args: ArgumentsList<'_, '_>,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let rid_val = args.get(0).to_int32(agent, gc.reborrow()).unbind().unwrap() as u32;
        let rid = Rid::from_index(rid_val);
        let alpha_val = args
            .get(1)
            .to_number(agent, gc.reborrow())
            .unbind()
            .unwrap();
        let alpha = alpha_val.into_f64(agent).clamp(0.0, 1.0) as f32;
        let host_data = agent
            .get_host_data()
            .downcast_ref::<HostData<crate::RuntimeMacroTask>>()
            .unwrap();
        let mut storage = host_data.storage.borrow_mut();
        let res: &mut CanvasResources = storage.get_mut().unwrap();
        let mut data = res.canvases.get_mut(rid).unwrap();
        data.global_alpha = alpha;
        Ok(Value::Undefined)
    }

    fn internal_canvas_get_width<'gc>(
        agent: &mut Agent,
        _this: Value<'_>,
        args: ArgumentsList<'_, '_>,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let rid_val = args.get(0).to_int32(agent, gc.reborrow()).unbind()? as u32;
        let rid = Rid::from_index(rid_val);
        let host_data = agent
            .get_host_data()
            .downcast_ref::<HostData<crate::RuntimeMacroTask>>()
            .unwrap();
        let storage = host_data.storage.borrow();
        let res: &CanvasResources = storage.get().unwrap();
        let data = res.canvases.get(rid).unwrap();
        Ok(Value::Integer(SmallInteger::from(data.width as i32)))
    }

    fn internal_canvas_get_height<'gc>(
        agent: &mut Agent,
        _this: Value<'_>,
        args: ArgumentsList<'_, '_>,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let rid_val = args.get(0).to_int32(agent, gc.reborrow()).unbind()? as u32;
        let rid = Rid::from_index(rid_val);
        let host_data = agent
            .get_host_data()
            .downcast_ref::<HostData<crate::RuntimeMacroTask>>()
            .unwrap();
        let storage = host_data.storage.borrow();
        let res: &CanvasResources = storage.get().unwrap();
        let data = res.canvases.get(rid).unwrap();
        Ok(Value::Integer(SmallInteger::from(data.height as i32)))
    }

    /// Internal op to create an ImageBitmap resource
    fn internal_image_bitmap_create<'gc>(
        agent: &mut Agent,
        _this: Value<'_>,
        args: ArgumentsList<'_, '_>,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let binding = args.get(0).to_string(agent, gc.reborrow()).unbind()?;
        let _path = binding.as_str(agent).expect("String is not valid UTF-8");
        // For now, stub with zero dimensions
        let host_data = agent
            .get_host_data()
            .downcast_ref::<HostData<crate::RuntimeMacroTask>>()
            .unwrap();
        let mut storage = host_data.storage.borrow_mut();
        let res: &mut CanvasResources = storage.get_mut().unwrap();
        let rid = res.images.push(ImageData {
            width: 0,
            height: 0,
        });
        Ok(Value::Integer(SmallInteger::from(rid.index() as i32)))
    }

    /// Internal op to get ImageBitmap width
    fn internal_image_bitmap_get_width<'gc>(
        agent: &mut Agent,
        _this: Value<'_>,
        args: ArgumentsList<'_, '_>,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let rid_val = args.get(0).to_int32(agent, gc.reborrow()).unbind()? as u32;
        let rid = Rid::from_index(rid_val);
        let host_data = agent
            .get_host_data()
            .downcast_ref::<HostData<crate::RuntimeMacroTask>>()
            .unwrap();
        let storage = host_data.storage.borrow();
        let res: &CanvasResources = storage.get().unwrap();
        let data = res.images.get(rid).unwrap();
        Ok(Value::Integer(SmallInteger::from(data.width as i32)))
    }

    /// Internal op to get ImageBitmap height
    fn internal_image_bitmap_get_height<'gc>(
        agent: &mut Agent,
        _this: Value<'_>,
        args: ArgumentsList<'_, '_>,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let rid_val = args.get(0).to_int32(agent, gc.reborrow()).unbind()? as u32;
        let rid = Rid::from_index(rid_val);
        let host_data = agent
            .get_host_data()
            .downcast_ref::<HostData<crate::RuntimeMacroTask>>()
            .unwrap();
        let storage = host_data.storage.borrow();
        let res: &CanvasResources = storage.get().unwrap();
        let data = res.images.get(rid).unwrap();
        Ok(Value::Integer(SmallInteger::from(data.height as i32)))
    }

    fn internal_canvas_create_linear_gradient<'gc>(
        agent: &mut Agent,
        _this: Value<'_>,
        args: ArgumentsList<'_, '_>,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let x0 = args.get(0).to_number(agent, gc.reborrow()).unbind();
        let y0 = args.get(1).to_number(agent, gc.reborrow()).unbind();
        let x1 = args.get(2).to_number(agent, gc.reborrow()).unbind();
        let y1 = args.get(3).to_number(agent, gc.reborrow()).unbind();

        // For now, stub with zero dimensions
        let host_data = agent
            .get_host_data()
            .downcast_ref::<HostData<crate::RuntimeMacroTask>>()
            .unwrap();
        let mut storage = host_data.storage.borrow_mut();
        let res: &mut CanvasResources = storage.get_mut().unwrap();
        let rid = res
            .fill_styles
            .push(FillStyle::LinearGradient(LinearGradient {
                start: (x0.unwrap().into_f32(agent), y0.unwrap().into_f32(agent)),
                end: (x1.unwrap().into_f32(agent), y1.unwrap().into_f32(agent)),
                color_stops: vec![],
                rid: 0,
            }));
        let mut fill_style = res.fill_styles.get_mut(rid).unwrap();
        if let FillStyle::LinearGradient(gradient) = fill_style.deref_mut() {
            gradient.rid = rid.index();
        }
        Ok(Value::Integer(SmallInteger::from(rid.index() as i32)))
    }

    fn internal_canvas_gradient_add_color_stop<'gc>(
        agent: &mut Agent,
        _this: Value<'_>,
        args: ArgumentsList<'_, '_>,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let rid_val = args.get(0).to_int32(agent, gc.reborrow()).unbind()? as u32;
        let offset_val = args.get(1).to_number(agent, gc.reborrow()).unbind()?;
        let rid = Rid::from_index(rid_val);
        let offset = offset_val.into_f32(agent);
        let color = args.get(2).to_string(agent, gc.reborrow()).unbind()?;

        let host_data = agent
            .get_host_data()
            .downcast_ref::<HostData<crate::RuntimeMacroTask>>()
            .unwrap();
        let storage = host_data.storage.borrow();
        let res: &CanvasResources = storage.get().unwrap();
        let mut data = res.fill_styles.get_mut(rid).unwrap();
        if let FillStyle::LinearGradient(gradient) = data.deref_mut() {
            let fill_style = FillStyle::from_css_color(color.as_str(agent)).unwrap();
            if let FillStyle::Color { r, g, b, a } = fill_style {
                gradient.color_stops.push(ColorStop {
                    color: [r, g, b, a],
                    offset,
                })
            }
        }
        Ok(Value::Undefined)
    }

    /// Internal op to render canvas to pixels (snapshot GPU canvas)
    fn internal_canvas_render<'gc>(
        agent: &mut Agent,
        _this: Value<'_>,
        args: ArgumentsList<'_, '_>,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let rid_val = args.get(0).to_int32(agent, gc.reborrow()).unbind()? as u32;
        let rid = Rid::from_index(rid_val);
        let host_data = agent
            .get_host_data()
            .downcast_ref::<HostData<crate::RuntimeMacroTask>>()
            .unwrap();
        let mut storage = host_data.storage.borrow_mut();
        let res: &mut CanvasResources = storage.get_mut().unwrap();

        // Try to render with GPU renderer if available
        if let Some(mut renderer) = res.renderers.get_mut(rid) {
            // Finalize all pending render operations
            renderer.render_all();

            // TODO: Add method to extract pixel data from GPU texture
            // For now, just indicate successful render
            Ok(Value::Boolean(true))
        } else {
            // No renderer available - can't render to pixels
            Ok(Value::Boolean(false))
        }
    }

    /// Internal op to save canvas as PNG file
    fn internal_canvas_save_as_png<'gc>(
        agent: &mut Agent,
        _this: Value<'_>,
        args: ArgumentsList<'_, '_>,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let rid_val = args.get(0).to_int32(agent, gc.reborrow()).unbind()? as u32;
        let rid = Rid::from_index(rid_val);
        let path_str = args.get(1).to_string(agent, gc.reborrow()).unbind()?;
        let path_owned = path_str
            .as_str(agent)
            .expect("String is not valid UTF-8")
            .to_owned();

        let host_data = agent
            .get_host_data()
            .downcast_ref::<HostData<crate::RuntimeMacroTask>>()
            .unwrap();
        let mut storage = host_data.storage.borrow_mut();
        let res: &mut CanvasResources = storage.get_mut().unwrap();

        // Try to save with GPU renderer if available
        if let Some(mut renderer) = res.renderers.get_mut(rid) {
            // Since we can't use async in this context, we'll use a blocking approach

            // First render all pending operations
            renderer.render_all();

            // Use tokio to handle the async save operation
            let rt = match tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            {
                Ok(rt) => rt,
                Err(_) => return Ok(Value::Boolean(false)),
            };

            match rt.block_on(renderer.save_as_png(&path_owned)) {
                Ok(_) => Ok(Value::Boolean(true)),
                Err(_) => Ok(Value::Boolean(false)),
            }
        } else {
            // No renderer available - can't save to PNG
            Ok(Value::Boolean(false))
        }
    }

    /// Internal op to get the current fill style of a canvas context
    fn internal_canvas_get_fill_style<'gc>(
        agent: &mut Agent,
        _this: Value<'_>,
        args: ArgumentsList<'_, '_>,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let rid_val = args.get(0).to_int32(agent, gc.reborrow()).unbind().unwrap() as u32;
        let rid = Rid::from_index(rid_val);
        let host_data = agent
            .get_host_data()
            .downcast_ref::<HostData<crate::RuntimeMacroTask>>()
            .unwrap();
        let storage = host_data.storage.borrow();
        let res: &CanvasResources = storage.get().unwrap();
        let data = res.canvases.get(rid).unwrap();
        let fill_style = data.fill_style.clone();

        // Drop storage borrow before creating string
        drop(storage);

        match &fill_style {
            FillStyle::Color { r, g, b, a } => {
                let css_string = if *a == 1.0 {
                    // RGB format for opaque colors
                    format!(
                        "rgb({}, {}, {})",
                        (*r * 255.0) as u8,
                        (*g * 255.0) as u8,
                        (*b * 255.0) as u8
                    )
                } else {
                    // RGBA format for transparent colors
                    format!(
                        "rgba({}, {}, {}, {})",
                        (*r * 255.0) as u8,
                        (*g * 255.0) as u8,
                        (*b * 255.0) as u8,
                        a
                    )
                };
                Ok(Value::from_string(agent, css_string, gc.nogc()).unbind())
            }
            FillStyle::LinearGradient(gradient) => {
                Ok(Value::from_i64(agent, gradient.rid as i64, gc.nogc()).unbind())
            }
            _ => unimplemented!(),
        }
    }

    /// Internal op to set the fill style of a canvas context
    fn internal_canvas_set_fill_style<'gc>(
        agent: &mut Agent,
        _this: Value<'_>,
        args: ArgumentsList<'_, '_>,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let rid_val = args.get(0).to_int32(agent, gc.reborrow()).unbind().unwrap() as u32;
        let rid = Rid::from_index(rid_val);
        let style_val = args.get(1);
        let mut fill_rid = 0;
        let mut style_str = String::new();
        if style_val.is_number() {
            fill_rid = style_val.to_uint32(agent, gc).unwrap();
        } else {
            let style_string = style_val.to_string(agent, gc.reborrow()).unbind().unwrap();
            style_str = style_string
                .as_str(agent)
                .expect("String is not valid UTF-8")
                .to_string();
        }

        let host_data = agent
            .get_host_data()
            .downcast_ref::<HostData<crate::RuntimeMacroTask>>()
            .unwrap();
        let mut storage = host_data.storage.borrow_mut();
        let res: &mut CanvasResources = storage.get_mut().unwrap();
        let fill_style = if style_val.is_number() {
            res.fill_styles.get(Rid::from_index(fill_rid))
        } else {
            FillStyle::from_css_color(style_str.as_str()).ok()
        };

        let mut data = res.canvases.get_mut(rid).unwrap();
        match fill_style {
            Some(fill_style) => {
                data.fill_style = fill_style;
                Ok(Value::Boolean(true))
            }
            None => Ok(Value::Boolean(false)),
        }
    }

    /// Internal op to get the current stroke style of a canvas context
    fn internal_canvas_get_stroke_style<'gc>(
        agent: &mut Agent,
        _this: Value<'_>,
        args: ArgumentsList<'_, '_>,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let rid_val = args.get(0).to_int32(agent, gc.reborrow()).unbind().unwrap() as u32;
        let rid = Rid::from_index(rid_val);
        let host_data = agent
            .get_host_data()
            .downcast_ref::<HostData<crate::RuntimeMacroTask>>()
            .unwrap();
        let storage = host_data.storage.borrow();
        let res: &CanvasResources = storage.get().unwrap();
        let data = res.canvases.get(rid).unwrap();

        // Convert stroke style back to CSS string representation
        let css_string = match &data.stroke_style {
            FillStyle::Color { r, g, b, a } => {
                if *a == 1.0 {
                    // RGB format for opaque colors
                    format!(
                        "rgb({}, {}, {})",
                        (*r * 255.0) as u8,
                        (*g * 255.0) as u8,
                        (*b * 255.0) as u8
                    )
                } else {
                    // RGBA format for transparent colors
                    format!(
                        "rgba({}, {}, {}, {})",
                        (*r * 255.0) as u8,
                        (*g * 255.0) as u8,
                        (*b * 255.0) as u8,
                        a
                    )
                }
            }
            _ => "rgb(0, 0, 0)".to_string(), // Default fallback
        };

        // Drop storage borrow before creating string
        drop(storage);

        Ok(Value::from_string(agent, css_string, gc.nogc()).unbind())
    }

    /// Internal op to get the current line width of a canvas context
    fn internal_canvas_get_line_width<'gc>(
        agent: &mut Agent,
        _this: Value<'_>,
        args: ArgumentsList<'_, '_>,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let rid_val = args.get(0).to_int32(agent, gc.reborrow()).unbind().unwrap() as u32;
        let rid = Rid::from_index(rid_val);
        let host_data = agent
            .get_host_data()
            .downcast_ref::<HostData<crate::RuntimeMacroTask>>()
            .unwrap();
        let storage = host_data.storage.borrow();
        let res: &CanvasResources = storage.get().unwrap();
        let data = res.canvases.get(rid).unwrap();

        let line_width = data.line_width;

        // Drop storage borrow before creating result
        drop(storage);

        Ok(Value::from_f64(agent, line_width, gc.nogc()).unbind())
    }
}

async fn create_wgpu_device() -> (wgpu::Device, wgpu::Queue) {
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
    });

    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: None,
            force_fallback_adapter: false,
        })
        .await
        .unwrap();

    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor {
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            label: None,
            memory_hints: Default::default(),
            trace: wgpu::Trace::default(),
        })
        .await
        .unwrap();

    (device, queue)
}

fn create_wgpu_device_sync() -> (wgpu::Device, wgpu::Queue) {
    // Use a simple blocking executor - we'll create a simpler version for now
    let result = Arc::new(Mutex::new(None));
    let result_clone = result.clone();

    thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let device_queue = rt.block_on(create_wgpu_device());
        *result_clone.lock().unwrap() = Some(device_queue);
    })
    .join()
    .unwrap();

    result.lock().unwrap().take().unwrap()
}
