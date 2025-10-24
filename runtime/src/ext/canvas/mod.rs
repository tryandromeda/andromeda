// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

pub mod context2d;
pub mod fill_style;
pub mod font_system;
pub mod path2d;
pub mod renderer;
pub mod state;
pub mod text;
pub mod text_metrics;
use andromeda_core::{Extension, ExtensionOp, HostData, OpsStorage, ResourceTable, Rid};
use std::ops::DerefMut;

use crate::ext::canvas::context2d::{
    internal_canvas_begin_path, internal_canvas_bezier_curve_to, internal_canvas_close_path,
};
use crate::ext::canvas::fill_style::{ConicGradient, LinearGradient, RadialGradient};
use crate::ext::canvas::path2d::{FillRule, Path2D};
use crate::ext::canvas::renderer::{ColorStop, LineCap, LineJoin, Point, Rect, RenderState};
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

// Helper functions for text rendering
fn calculate_baseline_offset(
    baseline: &state::TextBaseline,
    font_descriptor: &font_system::FontDescriptor,
    text_height: f64,
) -> f64 {
    // Note: Y coordinate in canvas is positive downward
    // The baseline offset adjusts where the text bitmap is placed relative to the Y coordinate
    match baseline {
        state::TextBaseline::Top => 0.0, // Top of text at Y
        state::TextBaseline::Hanging => font_descriptor.size as f64 * 0.2, // Slightly below top
        state::TextBaseline::Middle => text_height / 2.0 - font_descriptor.size as f64 * 0.8, // Center vertically
        state::TextBaseline::Alphabetic => -font_descriptor.size as f64 * 0.2, // Baseline is ~20% up from bottom
        state::TextBaseline::Ideographic => 0.0, // Similar to bottom for CJK
        state::TextBaseline::Bottom => -text_height, // Bottom of text at Y
    }
}

fn calculate_alignment_offset(
    align: &state::TextAlign,
    _direction: &state::Direction,
    text_width: f64,
) -> f64 {
    match align {
        state::TextAlign::Left | state::TextAlign::Start => 0.0,
        state::TextAlign::Center => -text_width / 2.0,
        state::TextAlign::Right | state::TextAlign::End => -text_width,
    }
}

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
    // Line dash state (segments and offset)
    line_dash: Vec<f64>,
    line_dash_offset: f64,
    // Line style properties
    line_cap: renderer::LineCap,
    line_join: renderer::LineJoin,
    miter_limit: f64,
    // Shadow properties
    shadow_blur: f64,
    shadow_color: FillStyle,
    shadow_offset_x: f64,
    shadow_offset_y: f64,
    // Path state for renderer
    current_path: Vec<renderer::Point>,
    path_started: bool,
    // State stack for save/restore functionality
    state_stack: Vec<state::CanvasState>,
    // Transformation matrix [a, b, c, d, e, f]
    transform: [f64; 6],
    // Composite operation for blending
    composite_operation: renderer::CompositeOperation,
    // Text properties
    font: String,
    text_align: state::TextAlign,
    text_baseline: state::TextBaseline,
    direction: state::Direction,
}

struct CanvasResources<'gc> {
    canvases: ResourceTable<CanvasData<'gc>>,
    path2ds: ResourceTable<Path2D>,
    images: ResourceTable<ImageData>,
    renderers: ResourceTable<renderer::Renderer>,
    fill_styles: ResourceTable<FillStyle>,
    next_texture_id: u32,
}

/// Stored image dimensions and pixel data
#[derive(Clone)]
struct ImageData {
    width: u32,
    height: u32,
    /// RGBA pixel data (4 bytes per pixel)
    data: Option<Vec<u8>>,
}

#[derive(Default)]
pub struct CanvasExt;

impl CanvasExt {
    #[cfg_attr(feature = "hotpath", hotpath::measure)]
    pub fn new_extension() -> Extension {
        Extension {
            name: "canvas",
            ops: vec![
                // Internal operations for Canvas API
                ExtensionOp::new(
                    "internal_canvas_create",
                    Self::internal_canvas_create,
                    2,
                    false,
                ),
                ExtensionOp::new(
                    "internal_canvas_get_width",
                    Self::internal_canvas_get_width,
                    1,
                    false,
                ),
                ExtensionOp::new(
                    "internal_canvas_get_height",
                    Self::internal_canvas_get_height,
                    1,
                    false,
                ),
                // Context2D operations
                ExtensionOp::new("internal_canvas_arc", internal_canvas_arc, 5, false),
                ExtensionOp::new("internal_canvas_arc_to", internal_canvas_arc_to, 5, false),
                ExtensionOp::new(
                    "internal_canvas_bezier_curve_to",
                    internal_canvas_bezier_curve_to,
                    7,
                    false,
                ),
                ExtensionOp::new(
                    "internal_canvas_begin_path",
                    internal_canvas_begin_path,
                    1,
                    false,
                ),
                ExtensionOp::new(
                    "internal_canvas_clear_rect",
                    internal_canvas_clear_rect,
                    5,
                    false,
                ),
                ExtensionOp::new(
                    "internal_canvas_close_path",
                    internal_canvas_close_path,
                    1,
                    false,
                ),
                ExtensionOp::new(
                    "internal_canvas_fill_rect",
                    internal_canvas_fill_rect,
                    5,
                    false,
                ),
                ExtensionOp::new("internal_canvas_move_to", internal_canvas_move_to, 3, false),
                ExtensionOp::new("internal_canvas_line_to", internal_canvas_line_to, 3, false),
                ExtensionOp::new("internal_canvas_fill", internal_canvas_fill, 1, false),
                ExtensionOp::new("internal_canvas_stroke", internal_canvas_stroke, 1, false),
                ExtensionOp::new("internal_canvas_rect", internal_canvas_rect, 5, false),
                ExtensionOp::new(
                    "internal_canvas_quadratic_curve_to",
                    internal_canvas_quadratic_curve_to,
                    5,
                    false,
                ),
                ExtensionOp::new("internal_canvas_ellipse", internal_canvas_ellipse, 9, false),
                ExtensionOp::new(
                    "internal_canvas_round_rect",
                    internal_canvas_round_rect,
                    6,
                    false,
                ),
                ExtensionOp::new(
                    "internal_canvas_set_line_width",
                    internal_canvas_set_line_width,
                    2,
                    false,
                ),
                ExtensionOp::new(
                    "internal_canvas_set_stroke_style",
                    internal_canvas_set_stroke_style,
                    2,
                    false,
                ),
                ExtensionOp::new(
                    "internal_canvas_stroke_rect",
                    Self::internal_canvas_stroke_rect,
                    5,
                    false,
                ),
                ExtensionOp::new(
                    "internal_canvas_get_stroke_style",
                    Self::internal_canvas_get_stroke_style,
                    1,
                    false,
                ),
                ExtensionOp::new(
                    "internal_canvas_get_line_width",
                    Self::internal_canvas_get_line_width,
                    1,
                    false,
                ),
                ExtensionOp::new(
                    "internal_canvas_get_fill_style",
                    Self::internal_canvas_get_fill_style,
                    1,
                    false,
                ),
                ExtensionOp::new(
                    "internal_canvas_set_fill_style",
                    Self::internal_canvas_set_fill_style,
                    2,
                    false,
                ),
                ExtensionOp::new(
                    "internal_canvas_set_line_dash",
                    Self::internal_canvas_set_line_dash,
                    2,
                    false,
                ),
                ExtensionOp::new(
                    "internal_canvas_get_line_dash",
                    Self::internal_canvas_get_line_dash,
                    1,
                    false,
                ),
                // Line style properties
                ExtensionOp::new(
                    "internal_canvas_set_line_cap",
                    Self::internal_canvas_set_line_cap,
                    2,
                    false,
                ),
                ExtensionOp::new(
                    "internal_canvas_get_line_cap",
                    Self::internal_canvas_get_line_cap,
                    1,
                    false,
                ),
                ExtensionOp::new(
                    "internal_canvas_set_line_join",
                    Self::internal_canvas_set_line_join,
                    2,
                    false,
                ),
                ExtensionOp::new(
                    "internal_canvas_get_line_join",
                    Self::internal_canvas_get_line_join,
                    1,
                    false,
                ),
                ExtensionOp::new(
                    "internal_canvas_set_miter_limit",
                    Self::internal_canvas_set_miter_limit,
                    2,
                    false,
                ),
                ExtensionOp::new(
                    "internal_canvas_get_miter_limit",
                    Self::internal_canvas_get_miter_limit,
                    1,
                    false,
                ),
                // Shadow properties
                ExtensionOp::new(
                    "internal_canvas_set_shadow_blur",
                    Self::internal_canvas_set_shadow_blur,
                    2,
                    false,
                ),
                ExtensionOp::new(
                    "internal_canvas_get_shadow_blur",
                    Self::internal_canvas_get_shadow_blur,
                    1,
                    false,
                ),
                ExtensionOp::new(
                    "internal_canvas_set_shadow_color",
                    Self::internal_canvas_set_shadow_color,
                    2,
                    false,
                ),
                ExtensionOp::new(
                    "internal_canvas_get_shadow_color",
                    Self::internal_canvas_get_shadow_color,
                    1,
                    false,
                ),
                ExtensionOp::new(
                    "internal_canvas_set_shadow_offset_x",
                    Self::internal_canvas_set_shadow_offset_x,
                    2,
                    false,
                ),
                ExtensionOp::new(
                    "internal_canvas_get_shadow_offset_x",
                    Self::internal_canvas_get_shadow_offset_x,
                    1,
                    false,
                ),
                ExtensionOp::new(
                    "internal_canvas_set_shadow_offset_y",
                    Self::internal_canvas_set_shadow_offset_y,
                    2,
                    false,
                ),
                ExtensionOp::new(
                    "internal_canvas_get_shadow_offset_y",
                    Self::internal_canvas_get_shadow_offset_y,
                    1,
                    false,
                ),
                // Text properties
                ExtensionOp::new(
                    "internal_canvas_set_font",
                    Self::internal_canvas_set_font,
                    2,
                    false,
                ),
                ExtensionOp::new(
                    "internal_canvas_get_font",
                    Self::internal_canvas_get_font,
                    1,
                    false,
                ),
                ExtensionOp::new(
                    "internal_canvas_set_text_align",
                    Self::internal_canvas_set_text_align,
                    2,
                    false,
                ),
                ExtensionOp::new(
                    "internal_canvas_get_text_align",
                    Self::internal_canvas_get_text_align,
                    1,
                    false,
                ),
                ExtensionOp::new(
                    "internal_canvas_set_text_baseline",
                    Self::internal_canvas_set_text_baseline,
                    2,
                    false,
                ),
                ExtensionOp::new(
                    "internal_canvas_get_text_baseline",
                    Self::internal_canvas_get_text_baseline,
                    1,
                    false,
                ),
                ExtensionOp::new(
                    "internal_canvas_set_direction",
                    Self::internal_canvas_set_direction,
                    2,
                    false,
                ),
                ExtensionOp::new(
                    "internal_canvas_get_direction",
                    Self::internal_canvas_get_direction,
                    1,
                    false,
                ),
                ExtensionOp::new(
                    "internal_canvas_measure_text",
                    Self::internal_canvas_measure_text,
                    2,
                    false,
                ),
                ExtensionOp::new(
                    "internal_canvas_fill_text",
                    Self::internal_canvas_fill_text,
                    4,
                    false,
                ),
                ExtensionOp::new(
                    "internal_canvas_stroke_text",
                    Self::internal_canvas_stroke_text,
                    4,
                    false,
                ),
                // Pattern operations
                ExtensionOp::new(
                    "internal_canvas_create_pattern",
                    Self::internal_canvas_create_pattern,
                    2,
                    false,
                ),
                ExtensionOp::new(
                    "internal_canvas_get_global_alpha",
                    Self::internal_canvas_get_global_alpha,
                    1,
                    false,
                ),
                ExtensionOp::new(
                    "internal_canvas_set_global_alpha",
                    Self::internal_canvas_set_global_alpha,
                    2,
                    false,
                ),
                ExtensionOp::new(
                    "internal_canvas_get_global_composite_operation",
                    Self::internal_canvas_get_global_composite_operation,
                    1,
                    false,
                ),
                ExtensionOp::new(
                    "internal_canvas_set_global_composite_operation",
                    Self::internal_canvas_set_global_composite_operation,
                    2,
                    false,
                ),
                ExtensionOp::new(
                    "internal_canvas_render",
                    Self::internal_canvas_render,
                    1,
                    false,
                ),
                ExtensionOp::new(
                    "internal_canvas_save_as_png",
                    Self::internal_canvas_save_as_png,
                    2,
                    false,
                ),
                ExtensionOp::new("internal_canvas_save", internal_canvas_save, 1, false),
                ExtensionOp::new("internal_canvas_restore", internal_canvas_restore, 1, false),
                // Transformation operations
                ExtensionOp::new(
                    "internal_canvas_rotate",
                    Self::internal_canvas_rotate,
                    2,
                    false,
                ),
                ExtensionOp::new(
                    "internal_canvas_scale",
                    Self::internal_canvas_scale,
                    3,
                    false,
                ),
                ExtensionOp::new(
                    "internal_canvas_translate",
                    Self::internal_canvas_translate,
                    3,
                    false,
                ),
                ExtensionOp::new(
                    "internal_canvas_transform",
                    Self::internal_canvas_transform,
                    7,
                    false,
                ),
                ExtensionOp::new(
                    "internal_canvas_set_transform",
                    Self::internal_canvas_set_transform,
                    7,
                    false,
                ),
                ExtensionOp::new(
                    "internal_canvas_reset_transform",
                    Self::internal_canvas_reset_transform,
                    1,
                    false,
                ),
                ExtensionOp::new(
                    "internal_canvas_get_transform",
                    Self::internal_canvas_get_transform,
                    1,
                    false,
                ),
                // ImageBitmap API
                ExtensionOp::new(
                    "internal_image_bitmap_create",
                    Self::internal_image_bitmap_create,
                    1,
                    false,
                ),
                ExtensionOp::new(
                    "internal_image_bitmap_get_width",
                    Self::internal_image_bitmap_get_width,
                    1,
                    false,
                ),
                ExtensionOp::new(
                    "internal_image_bitmap_get_height",
                    Self::internal_image_bitmap_get_height,
                    1,
                    false,
                ),
                // Image drawing operations
                ExtensionOp::new(
                    "internal_canvas_draw_image",
                    Self::internal_canvas_draw_image,
                    9,
                    false,
                ),
                ExtensionOp::new(
                    "internal_canvas_create_image_data",
                    Self::internal_canvas_create_image_data,
                    2,
                    false,
                ),
                ExtensionOp::new(
                    "internal_canvas_get_image_data",
                    Self::internal_canvas_get_image_data,
                    4,
                    false,
                ),
                ExtensionOp::new(
                    "internal_canvas_put_image_data",
                    Self::internal_canvas_put_image_data,
                    3,
                    false,
                ),
                ExtensionOp::new(
                    "internal_image_data_get_width",
                    Self::internal_image_data_get_width,
                    1,
                    false,
                ),
                ExtensionOp::new(
                    "internal_image_data_get_height",
                    Self::internal_image_data_get_height,
                    1,
                    false,
                ),
                ExtensionOp::new(
                    "internal_image_data_get_data",
                    Self::internal_image_data_get_data,
                    1,
                    false,
                ),
                // Gradient operations
                ExtensionOp::new(
                    "internal_canvas_create_linear_gradient",
                    Self::internal_canvas_create_linear_gradient,
                    4,
                    false,
                ),
                ExtensionOp::new(
                    "internal_canvas_create_radial_gradient",
                    Self::internal_canvas_create_radial_gradient,
                    6,
                    false,
                ),
                ExtensionOp::new(
                    "internal_canvas_create_conic_gradient",
                    Self::internal_canvas_create_conic_gradient,
                    3,
                    false,
                ),
                ExtensionOp::new(
                    "internal_canvas_gradient_add_color_stop",
                    Self::internal_canvas_gradient_add_color_stop,
                    2,
                    false,
                ),
                // Path2D operations
                ExtensionOp::new(
                    "internal_path2d_create",
                    Self::internal_path2d_create,
                    0,
                    false,
                ),
                ExtensionOp::new(
                    "internal_path2d_create_from_path",
                    Self::internal_path2d_create_from_path,
                    1,
                    false,
                ),
                ExtensionOp::new(
                    "internal_path2d_create_from_svg",
                    Self::internal_path2d_create_from_svg,
                    1,
                    false,
                ),
                ExtensionOp::new(
                    "internal_path2d_add_path",
                    Self::internal_path2d_add_path,
                    2,
                    false,
                ),
                ExtensionOp::new(
                    "internal_path2d_move_to",
                    Self::internal_path2d_move_to,
                    3,
                    false,
                ),
                ExtensionOp::new(
                    "internal_path2d_line_to",
                    Self::internal_path2d_line_to,
                    3,
                    false,
                ),
                ExtensionOp::new(
                    "internal_path2d_bezier_curve_to",
                    Self::internal_path2d_bezier_curve_to,
                    7,
                    false,
                ),
                ExtensionOp::new(
                    "internal_path2d_quadratic_curve_to",
                    Self::internal_path2d_quadratic_curve_to,
                    5,
                    false,
                ),
                ExtensionOp::new("internal_path2d_arc", Self::internal_path2d_arc, 7, false),
                ExtensionOp::new(
                    "internal_path2d_arc_to",
                    Self::internal_path2d_arc_to,
                    6,
                    false,
                ),
                ExtensionOp::new(
                    "internal_path2d_ellipse",
                    Self::internal_path2d_ellipse,
                    9,
                    false,
                ),
                ExtensionOp::new("internal_path2d_rect", Self::internal_path2d_rect, 5, false),
                ExtensionOp::new(
                    "internal_path2d_round_rect",
                    Self::internal_path2d_round_rect,
                    6,
                    false,
                ),
                ExtensionOp::new(
                    "internal_path2d_close_path",
                    Self::internal_path2d_close_path,
                    1,
                    false,
                ),
                ExtensionOp::new(
                    "internal_canvas_is_point_in_path",
                    Self::internal_canvas_is_point_in_path,
                    4,
                    false,
                ),
                ExtensionOp::new(
                    "internal_canvas_is_point_in_stroke",
                    Self::internal_canvas_is_point_in_stroke,
                    4,
                    false,
                ),
                ExtensionOp::new("internal_canvas_clip", Self::internal_canvas_clip, 2, false),
            ],
            storage: Some(Box::new(|storage: &mut OpsStorage| {
                storage.insert(CanvasResources {
                    canvases: ResourceTable::new(),
                    path2ds: ResourceTable::new(),
                    images: ResourceTable::new(),
                    renderers: ResourceTable::new(),
                    fill_styles: ResourceTable::new(),
                    next_texture_id: 1000, // Start from 1000 to avoid conflict with regular image RIDs
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
            // Initialize dash state
            line_dash: Vec::new(),
            line_dash_offset: 0.0,
            // Line style properties
            line_cap: renderer::LineCap::default(),
            line_join: renderer::LineJoin::default(),
            miter_limit: 10.0,
            // Shadow properties (defaults: no shadow)
            shadow_blur: 0.0,
            shadow_color: FillStyle::Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.0,
            },
            shadow_offset_x: 0.0,
            shadow_offset_y: 0.0,
            current_path: Vec::new(),
            path_started: false,
            state_stack: Vec::new(),
            // Identity transformation matrix
            transform: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
            // Default composite operation is source-over
            composite_operation: renderer::CompositeOperation::default(),
            // Text properties - Canvas2D defaults
            font: "10px sans-serif".to_string(),
            text_align: state::TextAlign::default(),
            text_baseline: state::TextBaseline::default(),
            direction: state::Direction::default(),
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

        Ok(Value::from_f64(agent, global_alpha as f64, gc.nogc()).unbind())
    }

    /// Internal op to get the current globalCompositeOperation of a canvas context
    #[allow(dead_code)]
    fn internal_canvas_get_global_composite_operation<'gc>(
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
        let composite_op_str = data.composite_operation.as_str();
        drop(storage);

        Ok(Value::from_string(agent, composite_op_str.to_string(), gc.nogc()).unbind())
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

    /// Internal op to set the globalCompositeOperation of a canvas context
    #[allow(dead_code)]
    fn internal_canvas_set_global_composite_operation<'gc>(
        agent: &mut Agent,
        _this: Value<'_>,
        args: ArgumentsList<'_, '_>,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let rid_val = args.get(0).to_int32(agent, gc.reborrow()).unbind().unwrap() as u32;
        let rid = Rid::from_index(rid_val);

        // Get the string value for the composite operation
        let op_str_val = args.get(1).to_string(agent, gc.reborrow()).unbind()?;
        let op_str = op_str_val.as_str(agent).map(|s| s.to_string());

        if let Some(op_string) = op_str
            && let Ok(composite_op) = op_string.parse::<renderer::CompositeOperation>()
        {
            let host_data = agent
                .get_host_data()
                .downcast_ref::<HostData<crate::RuntimeMacroTask>>()
                .unwrap();
            let mut storage = host_data.storage.borrow_mut();
            let res: &mut CanvasResources = storage.get_mut().unwrap();
            let mut data = res.canvases.get_mut(rid).unwrap();
            data.composite_operation = composite_op;
        }

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
        let path = binding.as_str(agent).expect("String is not valid UTF-8");

        // Load and decode the image
        let image_result = Self::load_image_from_path(path);

        let (width, height, data) = match image_result {
            Ok((w, h, d)) => (w, h, Some(d)),
            Err(e) => {
                eprintln!("Failed to load image '{}': {}", path, e);
                // Return invalid image with zero dimensions
                (0, 0, None)
            }
        };

        let host_data = agent
            .get_host_data()
            .downcast_ref::<HostData<crate::RuntimeMacroTask>>()
            .unwrap();
        let mut storage = host_data.storage.borrow_mut();
        let res: &mut CanvasResources = storage.get_mut().unwrap();

        let rid = res.images.push(ImageData {
            width,
            height,
            data: data.clone(),
        });

        // Note: Texture loading is now done lazily when drawImage is called
        // This avoids the problem of not knowing which renderer to use

        Ok(Value::Integer(SmallInteger::from(rid.index() as i32)))
    }

    /// Helper function to load and decode an image from a file path
    /// Similar to Deno's image decoding implementation
    fn load_image_from_path(path: &str) -> Result<(u32, u32, Vec<u8>), Box<dyn std::error::Error>> {
        // Read the file
        let image_bytes = std::fs::read(path)?;

        // Decode the image using the image crate (supports PNG, JPEG, GIF, WebP, etc.)
        let img = image::load_from_memory(&image_bytes)?;

        // Convert to RGBA8 format (8 bits per channel)
        let rgba = img.to_rgba8();
        let width = rgba.width();
        let height = rgba.height();

        // Get raw pixel data (already in RGBA format)
        let data = rgba.into_raw();

        Ok((width, height, data))
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

    fn internal_canvas_create_radial_gradient<'gc>(
        agent: &mut Agent,
        _this: Value<'_>,
        args: ArgumentsList<'_, '_>,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let x0 = args.get(0).to_number(agent, gc.reborrow()).unbind();
        let y0 = args.get(1).to_number(agent, gc.reborrow()).unbind();
        let r0 = args.get(2).to_number(agent, gc.reborrow()).unbind();
        let x1 = args.get(3).to_number(agent, gc.reborrow()).unbind();
        let y1 = args.get(4).to_number(agent, gc.reborrow()).unbind();
        let r1 = args.get(5).to_number(agent, gc.reborrow()).unbind();

        // For now, stub with zero dimensions
        let host_data = agent
            .get_host_data()
            .downcast_ref::<HostData<crate::RuntimeMacroTask>>()
            .unwrap();
        let mut storage = host_data.storage.borrow_mut();
        let res: &mut CanvasResources = storage.get_mut().unwrap();
        let rid = res
            .fill_styles
            .push(FillStyle::RadialGradient(RadialGradient {
                start: (x0.unwrap().into_f32(agent), y0.unwrap().into_f32(agent)),
                end: (x1.unwrap().into_f32(agent), y1.unwrap().into_f32(agent)),
                start_radius: r0.unwrap().into_f32(agent),
                end_radius: r1.unwrap().into_f32(agent),
                color_stops: vec![],
                rid: 0,
            }));
        let mut fill_style = res.fill_styles.get_mut(rid).unwrap();
        if let FillStyle::RadialGradient(gradient) = fill_style.deref_mut() {
            gradient.rid = rid.index();
        }
        Ok(Value::Integer(SmallInteger::from(rid.index() as i32)))
    }

    fn internal_canvas_create_conic_gradient<'gc>(
        agent: &mut Agent,
        _this: Value<'_>,
        args: ArgumentsList<'_, '_>,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let start_angle = args.get(0).to_number(agent, gc.reborrow()).unbind();
        let x = args.get(1).to_number(agent, gc.reborrow()).unbind();
        let y = args.get(2).to_number(agent, gc.reborrow()).unbind();

        // For now, stub with zero dimensions
        let host_data = agent
            .get_host_data()
            .downcast_ref::<HostData<crate::RuntimeMacroTask>>()
            .unwrap();
        let mut storage = host_data.storage.borrow_mut();
        let res: &mut CanvasResources = storage.get_mut().unwrap();
        let rid = res
            .fill_styles
            .push(FillStyle::ConicGradient(ConicGradient {
                center: (x.unwrap().into_f32(agent), y.unwrap().into_f32(agent)),
                start_angle: start_angle.unwrap().into_f32(agent),
                color_stops: vec![],
                rid: 0,
            }));
        let mut fill_style = res.fill_styles.get_mut(rid).unwrap();
        if let FillStyle::ConicGradient(gradient) = fill_style.deref_mut() {
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
        let color_stops = match data.deref_mut() {
            FillStyle::LinearGradient(gradient) => &mut gradient.color_stops,
            FillStyle::RadialGradient(gradient) => &mut gradient.color_stops,
            FillStyle::ConicGradient(gradient) => &mut gradient.color_stops,
            _ => unreachable!(),
        };
        let color_str = color.as_str(agent).expect("String is not valid UTF-8");
        let fill_style = FillStyle::from_css_color(color_str).unwrap();
        if let FillStyle::Color { r, g, b, a } = fill_style {
            color_stops.push(ColorStop {
                color: [r, g, b, a],
                offset,
            })
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
            FillStyle::RadialGradient(gradient) => {
                Ok(Value::from_i64(agent, gradient.rid as i64, gc.nogc()).unbind())
            }
            FillStyle::ConicGradient(gradient) => {
                Ok(Value::from_i64(agent, gradient.rid as i64, gc.nogc()).unbind())
            }
            FillStyle::Pattern { image_rid, .. } => {
                // Return the image RID as a number for now
                // In a full implementation, this would return a CanvasPattern object
                Ok(Value::from_i64(agent, *image_rid as i64, gc.nogc()).unbind())
            }
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
        let stroke_style = data.stroke_style.clone();

        // Drop storage borrow before creating string
        drop(storage);

        match &stroke_style {
            FillStyle::Color { r, g, b, a } => {
                let css_string = if *a == 1.0 {
                    format!(
                        "rgb({}, {}, {})",
                        (*r * 255.0) as u8,
                        (*g * 255.0) as u8,
                        (*b * 255.0) as u8
                    )
                } else {
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
            FillStyle::RadialGradient(gradient) => {
                Ok(Value::from_i64(agent, gradient.rid as i64, gc.nogc()).unbind())
            }
            FillStyle::ConicGradient(gradient) => {
                Ok(Value::from_i64(agent, gradient.rid as i64, gc.nogc()).unbind())
            }
            FillStyle::Pattern { image_rid, .. } => {
                // Return the image RID as a number for now
                Ok(Value::from_i64(agent, *image_rid as i64, gc.nogc()).unbind())
            }
        }
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

    /// Internal op to set the line dash pattern for a canvas context
    fn internal_canvas_set_line_dash<'gc>(
        agent: &mut Agent,
        _this: Value<'_>,
        args: ArgumentsList<'_, '_>,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let rid_val = args.get(0).to_int32(agent, gc.reborrow()).unbind().unwrap() as u32;
        let rid = Rid::from_index(rid_val);

        // Expect an array of numbers and optional offset (we accept an array and optional number)
        let pattern_val = args.get(1);

        // Parse pattern and optional offset before borrowing host storage to avoid
        // borrowing Agent mutably while storage is borrowed.
        let mut parsed_dash: Option<Vec<f64>> = None;
        if !pattern_val.is_undefined()
            && let Ok(sv) = pattern_val.to_string(agent, gc.reborrow())
            && let Some(s) = sv.as_str(agent)
        {
            let s_str = s.to_string();
            if let Ok(parsed) = serde_json::from_str::<Vec<f64>>(&s_str) {
                parsed_dash = Some(parsed);
            } else {
                let mut v = Vec::new();
                for part in s_str.split(',') {
                    let part = part.trim();
                    if part.is_empty() {
                        continue;
                    }
                    if let Ok(n) = part.parse::<f64>() {
                        v.push(n);
                    }
                }
                parsed_dash = Some(v);
            }
        }

        let parsed_offset: Option<f64> = if args.get(2).is_number() {
            if let Ok(offset_num) = args.get(2).to_number(agent, gc.reborrow()) {
                Some(offset_num.into_f64(agent))
            } else {
                None
            }
        } else {
            None
        };

        let host_data = agent
            .get_host_data()
            .downcast_ref::<HostData<crate::RuntimeMacroTask>>()
            .unwrap();
        let mut storage = host_data.storage.borrow_mut();
        let res: &mut CanvasResources = storage.get_mut().unwrap();
        let mut data = res.canvases.get_mut(rid).unwrap();

        // Apply parsed values to canvas data
        if let Some(dash) = parsed_dash {
            data.line_dash = dash;
        }
        if let Some(off) = parsed_offset {
            data.line_dash_offset = off;
        }

        Ok(Value::Undefined)
    }

    /// Internal op to get the current line dash array and offset
    fn internal_canvas_get_line_dash<'gc>(
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

        // Clone dash data out of storage then release the borrow so we can
        // safely interact with the Agent/VM when creating return values.
        let dash_clone = data.line_dash.clone();
        let offset_clone = data.line_dash_offset;

        drop(storage);

        // Return a JSON string describing dash array and offset, e.g. {"dash":[4,2],"offset":1}
        let mut s = String::from("{");
        s.push_str("\"dash\":[");
        for (i, v) in dash_clone.iter().enumerate() {
            if i > 0 {
                s.push(',');
            }
            s.push_str(&format!("{v}"));
        }
        s.push_str("],\"offset\":");
        s.push_str(&format!("{offset_clone}"));
        s.push('}');

        Ok(Value::from_string(agent, s, gc.nogc()).unbind())
    }

    /// Internal op to create a new Path2D object
    fn internal_path2d_create<'gc>(
        agent: &mut Agent,
        _this: Value<'_>,
        _args: ArgumentsList<'_, '_>,
        _gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let host_data = agent
            .get_host_data()
            .downcast_ref::<HostData<crate::RuntimeMacroTask>>()
            .unwrap();
        let mut storage = host_data.storage.borrow_mut();
        let res: &mut CanvasResources = storage.get_mut().unwrap();

        let path = Path2D::new();
        let path_rid = res.path2ds.push(path);

        Ok(Value::Integer(SmallInteger::from(path_rid.index() as i32)))
    }

    /// Internal op to create a Path2D from another path
    fn internal_path2d_create_from_path<'gc>(
        agent: &mut Agent,
        _this: Value<'_>,
        args: ArgumentsList<'_, '_>,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let other_rid_val = args.get(0).to_int32(agent, gc.reborrow()).unbind().unwrap() as u32;
        let other_rid = Rid::from_index(other_rid_val);

        let host_data = agent
            .get_host_data()
            .downcast_ref::<HostData<crate::RuntimeMacroTask>>()
            .unwrap();
        let mut storage = host_data.storage.borrow_mut();
        let res: &mut CanvasResources = storage.get_mut().unwrap();

        let other_path = res.path2ds.get(other_rid).unwrap();
        let new_path = Path2D::from_path(&other_path);
        let path_rid = res.path2ds.push(new_path);

        Ok(Value::Integer(SmallInteger::from(path_rid.index() as i32)))
    }

    /// Internal op to create a Path2D from SVG path data
    fn internal_path2d_create_from_svg<'gc>(
        agent: &mut Agent,
        _this: Value<'_>,
        args: ArgumentsList<'_, '_>,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let svg_data = args
            .get(0)
            .to_string(agent, gc.reborrow())
            .unbind()
            .unwrap();

        let host_data = agent
            .get_host_data()
            .downcast_ref::<HostData<crate::RuntimeMacroTask>>()
            .unwrap();
        let mut storage = host_data.storage.borrow_mut();
        let res: &mut CanvasResources = storage.get_mut().unwrap();

        match Path2D::from_svg_path_data(svg_data.as_str(agent).unwrap()) {
            Ok(path) => {
                let path_rid = res.path2ds.push(path);
                Ok(Value::Integer(SmallInteger::from(path_rid.index() as i32)))
            }
            Err(_) => {
                // Return null for invalid SVG path data
                Ok(Value::Null)
            }
        }
    }

    /// Internal op to add a path to another path
    fn internal_path2d_add_path<'gc>(
        agent: &mut Agent,
        _this: Value<'_>,
        args: ArgumentsList<'_, '_>,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let target_rid_val = args.get(0).to_int32(agent, gc.reborrow()).unbind().unwrap() as u32;
        let source_rid_val = args.get(1).to_int32(agent, gc.reborrow()).unbind().unwrap() as u32;
        let target_rid = Rid::from_index(target_rid_val);
        let source_rid = Rid::from_index(source_rid_val);

        let host_data = agent
            .get_host_data()
            .downcast_ref::<HostData<crate::RuntimeMacroTask>>()
            .unwrap();
        let mut storage = host_data.storage.borrow_mut();
        let res: &mut CanvasResources = storage.get_mut().unwrap();

        let source_path = res.path2ds.get(source_rid).unwrap().clone();
        res.path2ds
            .get_mut(target_rid)
            .unwrap()
            .add_path(&source_path, None); // TODO: Add transform support

        Ok(Value::Undefined)
    }

    /// Internal op to test if a point is in a path
    fn internal_canvas_is_point_in_path<'gc>(
        agent: &mut Agent,
        _this: Value<'_>,
        args: ArgumentsList<'_, '_>,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let path_rid_val = args.get(0).to_int32(agent, gc.reborrow()).unbind().unwrap() as u32;
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
        let fill_rule_str = args
            .get(3)
            .to_string(agent, gc.reborrow())
            .unbind()
            .unwrap();

        let path_rid = Rid::from_index(path_rid_val);
        let fill_rule = match fill_rule_str.as_str(agent) {
            Some("evenodd") => FillRule::EvenOdd,
            _ => FillRule::NonZero,
        };

        let host_data = agent
            .get_host_data()
            .downcast_ref::<HostData<crate::RuntimeMacroTask>>()
            .unwrap();
        let storage = host_data.storage.borrow();
        let res: &CanvasResources = storage.get().unwrap();

        let path = res.path2ds.get(path_rid).unwrap();
        let result = path.is_point_in_path(
            x.into_f32(agent) as f64,
            y.into_f32(agent) as f64,
            fill_rule,
        );

        Ok(Value::Boolean(result))
    }

    /// Internal op to test if a point is in a path stroke
    fn internal_canvas_is_point_in_stroke<'gc>(
        agent: &mut Agent,
        _this: Value<'_>,
        args: ArgumentsList<'_, '_>,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let path_rid_val = args.get(0).to_int32(agent, gc.reborrow()).unbind().unwrap() as u32;
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
        let line_width = args
            .get(3)
            .to_number(agent, gc.reborrow())
            .unbind()
            .unwrap();

        let path_rid = Rid::from_index(path_rid_val);

        let host_data = agent
            .get_host_data()
            .downcast_ref::<HostData<crate::RuntimeMacroTask>>()
            .unwrap();
        let storage = host_data.storage.borrow();
        let res: &CanvasResources = storage.get().unwrap();

        let path = res.path2ds.get(path_rid).unwrap();
        let result = path.is_point_in_stroke(
            x.into_f32(agent) as f64,
            y.into_f32(agent) as f64,
            line_width.into_f32(agent) as f64,
        );

        Ok(Value::Boolean(result))
    }

    /// Internal op to clip using a path
    fn internal_canvas_clip<'gc>(
        agent: &mut Agent,
        _this: Value<'_>,
        args: ArgumentsList<'_, '_>,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let canvas_rid_val = args.get(0).to_int32(agent, gc.reborrow()).unbind().unwrap() as u32;
        let path_rid_val = args.get(1).to_int32(agent, gc.reborrow()).unbind().unwrap() as u32;

        let canvas_rid = Rid::from_index(canvas_rid_val);
        let path_rid = Rid::from_index(path_rid_val);

        let host_data = agent
            .get_host_data()
            .downcast_ref::<HostData<crate::RuntimeMacroTask>>()
            .unwrap();
        let mut storage = host_data.storage.borrow_mut();
        let res: &mut CanvasResources = storage.get_mut().unwrap();

        // Get the path data
        let path = res.path2ds.get(path_rid).unwrap().clone();

        // Add a clip command to the canvas
        res.canvases
            .get_mut(canvas_rid)
            .unwrap()
            .commands
            .push(context2d::CanvasCommand::Clip {
                path: path.get_all_points(),
            });

        Ok(Value::Undefined)
    }

    /// Internal op to move to a point on a Path2D
    fn internal_path2d_move_to<'gc>(
        agent: &mut Agent,
        _this: Value<'_>,
        args: ArgumentsList<'_, '_>,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let path_rid_val = args.get(0).to_int32(agent, gc.reborrow()).unbind().unwrap() as u32;
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

        let path_rid = Rid::from_index(path_rid_val);

        let host_data = agent
            .get_host_data()
            .downcast_ref::<HostData<crate::RuntimeMacroTask>>()
            .unwrap();
        let mut storage = host_data.storage.borrow_mut();
        let res: &mut CanvasResources = storage.get_mut().unwrap();

        res.path2ds
            .get_mut(path_rid)
            .unwrap()
            .move_to(x.into_f64(agent), y.into_f64(agent));

        Ok(Value::Undefined)
    }

    /// Internal op to add a line to a Path2D
    fn internal_path2d_line_to<'gc>(
        agent: &mut Agent,
        _this: Value<'_>,
        args: ArgumentsList<'_, '_>,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let path_rid_val = args.get(0).to_int32(agent, gc.reborrow()).unbind().unwrap() as u32;
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

        let path_rid = Rid::from_index(path_rid_val);

        let host_data = agent
            .get_host_data()
            .downcast_ref::<HostData<crate::RuntimeMacroTask>>()
            .unwrap();
        let mut storage = host_data.storage.borrow_mut();
        let res: &mut CanvasResources = storage.get_mut().unwrap();

        res.path2ds
            .get_mut(path_rid)
            .unwrap()
            .line_to(x.into_f64(agent), y.into_f64(agent));

        Ok(Value::Undefined)
    }

    /// Internal op to add a bezier curve to a Path2D
    fn internal_path2d_bezier_curve_to<'gc>(
        agent: &mut Agent,
        _this: Value<'_>,
        args: ArgumentsList<'_, '_>,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let path_rid_val = args.get(0).to_int32(agent, gc.reborrow()).unbind().unwrap() as u32;
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

        let path_rid = Rid::from_index(path_rid_val);

        let host_data = agent
            .get_host_data()
            .downcast_ref::<HostData<crate::RuntimeMacroTask>>()
            .unwrap();
        let mut storage = host_data.storage.borrow_mut();
        let res: &mut CanvasResources = storage.get_mut().unwrap();

        res.path2ds.get_mut(path_rid).unwrap().bezier_curve_to(
            cp1x.into_f64(agent),
            cp1y.into_f64(agent),
            cp2x.into_f64(agent),
            cp2y.into_f64(agent),
            x.into_f64(agent),
            y.into_f64(agent),
        );

        Ok(Value::Undefined)
    }

    /// Internal op to add a quadratic curve to a Path2D
    fn internal_path2d_quadratic_curve_to<'gc>(
        agent: &mut Agent,
        _this: Value<'_>,
        args: ArgumentsList<'_, '_>,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let path_rid_val = args.get(0).to_int32(agent, gc.reborrow()).unbind().unwrap() as u32;
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

        let path_rid = Rid::from_index(path_rid_val);

        let host_data = agent
            .get_host_data()
            .downcast_ref::<HostData<crate::RuntimeMacroTask>>()
            .unwrap();
        let mut storage = host_data.storage.borrow_mut();
        let res: &mut CanvasResources = storage.get_mut().unwrap();

        res.path2ds.get_mut(path_rid).unwrap().quadratic_curve_to(
            cpx.into_f64(agent),
            cpy.into_f64(agent),
            x.into_f64(agent),
            y.into_f64(agent),
        );

        Ok(Value::Undefined)
    }

    /// Internal op to add an arc to a Path2D
    fn internal_path2d_arc<'gc>(
        agent: &mut Agent,
        _this: Value<'_>,
        args: ArgumentsList<'_, '_>,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let path_rid_val = args.get(0).to_int32(agent, gc.reborrow()).unbind().unwrap() as u32;
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
        let anticlockwise = args.get(6).is_true();

        let path_rid = Rid::from_index(path_rid_val);

        let host_data = agent
            .get_host_data()
            .downcast_ref::<HostData<crate::RuntimeMacroTask>>()
            .unwrap();
        let mut storage = host_data.storage.borrow_mut();
        let res: &mut CanvasResources = storage.get_mut().unwrap();

        res.path2ds.get_mut(path_rid).unwrap().arc(
            x.into_f64(agent),
            y.into_f64(agent),
            radius.into_f64(agent),
            start_angle.into_f64(agent),
            end_angle.into_f64(agent),
            anticlockwise,
        );

        Ok(Value::Undefined)
    }

    /// Internal op to add an arcTo to a Path2D
    fn internal_path2d_arc_to<'gc>(
        agent: &mut Agent,
        _this: Value<'_>,
        args: ArgumentsList<'_, '_>,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let path_rid_val = args.get(0).to_int32(agent, gc.reborrow()).unbind().unwrap() as u32;
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

        let path_rid = Rid::from_index(path_rid_val);

        let host_data = agent
            .get_host_data()
            .downcast_ref::<HostData<crate::RuntimeMacroTask>>()
            .unwrap();
        let mut storage = host_data.storage.borrow_mut();
        let res: &mut CanvasResources = storage.get_mut().unwrap();

        res.path2ds.get_mut(path_rid).unwrap().arc_to(
            x1.into_f64(agent),
            y1.into_f64(agent),
            x2.into_f64(agent),
            y2.into_f64(agent),
            radius.into_f64(agent),
        );

        Ok(Value::Undefined)
    }

    /// Internal op to add an ellipse to a Path2D
    fn internal_path2d_ellipse<'gc>(
        agent: &mut Agent,
        _this: Value<'_>,
        args: ArgumentsList<'_, '_>,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let path_rid_val = args.get(0).to_int32(agent, gc.reborrow()).unbind().unwrap() as u32;
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
        let anticlockwise = args.get(8).is_true();

        let path_rid = Rid::from_index(path_rid_val);

        let host_data = agent
            .get_host_data()
            .downcast_ref::<HostData<crate::RuntimeMacroTask>>()
            .unwrap();
        let mut storage = host_data.storage.borrow_mut();
        let res: &mut CanvasResources = storage.get_mut().unwrap();

        res.path2ds.get_mut(path_rid).unwrap().ellipse(
            x.into_f64(agent),
            y.into_f64(agent),
            radius_x.into_f64(agent),
            radius_y.into_f64(agent),
            rotation.into_f64(agent),
            start_angle.into_f64(agent),
            end_angle.into_f64(agent),
            anticlockwise,
        );

        Ok(Value::Undefined)
    }

    /// Internal op to add a rectangle to a Path2D
    fn internal_path2d_rect<'gc>(
        agent: &mut Agent,
        _this: Value<'_>,
        args: ArgumentsList<'_, '_>,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let path_rid_val = args.get(0).to_int32(agent, gc.reborrow()).unbind().unwrap() as u32;
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

        let path_rid = Rid::from_index(path_rid_val);

        let host_data = agent
            .get_host_data()
            .downcast_ref::<HostData<crate::RuntimeMacroTask>>()
            .unwrap();
        let mut storage = host_data.storage.borrow_mut();
        let res: &mut CanvasResources = storage.get_mut().unwrap();

        res.path2ds.get_mut(path_rid).unwrap().rect(
            x.into_f64(agent),
            y.into_f64(agent),
            width.into_f64(agent),
            height.into_f64(agent),
        );

        Ok(Value::Undefined)
    }

    /// Internal op to add a rounded rectangle to a Path2D
    fn internal_path2d_round_rect<'gc>(
        agent: &mut Agent,
        _this: Value<'_>,
        args: ArgumentsList<'_, '_>,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let path_rid_val = args.get(0).to_int32(agent, gc.reborrow()).unbind().unwrap() as u32;
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

        // Parse radii array from JavaScript
        let radii_array = args.get(5);
        let mut radii = Vec::new();

        // Handle different radii input formats
        if !radii_array.is_undefined()
            && let Ok(sv) = radii_array.to_string(agent, gc.reborrow())
            && let Some(s) = sv.as_str(agent)
        {
            let s_str = s.to_string();
            if let Ok(parsed) = serde_json::from_str::<Vec<f64>>(&s_str) {
                radii = parsed;
            } else {
                // Try comma-separated values
                for part in s_str.split(',') {
                    let part = part.trim();
                    if !part.is_empty()
                        && let Ok(n) = part.parse::<f64>()
                    {
                        radii.push(n);
                    }
                }
            }
        }

        // If no array was parsed, treat as single number
        if radii.is_empty() {
            if let Ok(num) = radii_array.to_number(agent, gc.reborrow()) {
                radii.push(num.into_f64(agent));
            } else {
                radii.push(0.0); // Default radius
            }
        }

        let path_rid = Rid::from_index(path_rid_val);

        let host_data = agent
            .get_host_data()
            .downcast_ref::<HostData<crate::RuntimeMacroTask>>()
            .unwrap();
        let mut storage = host_data.storage.borrow_mut();
        let res: &mut CanvasResources = storage.get_mut().unwrap();

        res.path2ds.get_mut(path_rid).unwrap().round_rect_web_api(
            x.into_f64(agent),
            y.into_f64(agent),
            width.into_f64(agent),
            height.into_f64(agent),
            &radii,
        );

        Ok(Value::Undefined)
    }

    /// Internal op to close a path on a Path2D
    fn internal_path2d_close_path<'gc>(
        agent: &mut Agent,
        _this: Value<'_>,
        args: ArgumentsList<'_, '_>,
        _gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let path_rid_val = args.get(0).to_int32(agent, _gc).unbind().unwrap() as u32;
        let path_rid = Rid::from_index(path_rid_val);

        let host_data = agent
            .get_host_data()
            .downcast_ref::<HostData<crate::RuntimeMacroTask>>()
            .unwrap();
        let mut storage = host_data.storage.borrow_mut();
        let res: &mut CanvasResources = storage.get_mut().unwrap();

        res.path2ds.get_mut(path_rid).unwrap().close_path();

        Ok(Value::Undefined)
    }

    /// Internal op to stroke a rectangle on a canvas by Rid
    fn internal_canvas_stroke_rect<'gc>(
        agent: &mut Agent,
        _this: Value<'_>,
        args: ArgumentsList<'_, '_>,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let rid_val = args.get(0).to_int32(agent, gc.reborrow()).unbind().unwrap() as u32;
        let rid = Rid::from_index(rid_val);
        let x = args.get(1).to_number(agent, gc.reborrow()).unbind()?;
        let y = args.get(2).to_number(agent, gc.reborrow()).unbind()?;
        let width = args.get(3).to_number(agent, gc.reborrow()).unbind()?;
        let height = args.get(4).to_number(agent, gc.reborrow()).unbind()?;

        let host_data = agent
            .get_host_data()
            .downcast_ref::<HostData<crate::RuntimeMacroTask>>()
            .unwrap();
        let mut storage = host_data.storage.borrow_mut();
        let res: &mut CanvasResources = storage.get_mut().unwrap();

        let canvas = res.canvases.get(rid).unwrap();
        let stroke_style = canvas.stroke_style.clone();
        let line_width = canvas.line_width;
        drop(storage);

        // Render the stroke rectangle
        let mut storage = host_data.storage.borrow_mut();
        let res: &mut CanvasResources = storage.get_mut().unwrap();
        if let Some(mut renderer) = res.renderers.get_mut(rid) {
            let rect = Rect {
                start: Point {
                    x: x.into_f64(agent),
                    y: y.into_f64(agent),
                },
                end: Point {
                    x: x.into_f64(agent) + width.into_f64(agent),
                    y: y.into_f64(agent) + height.into_f64(agent),
                },
            };

            let (r, g, b, a) = stroke_style.get_rgba();
            let stroke_color = [r, g, b, a];

            let canvas = res.canvases.get(rid).unwrap();
            let render_state = RenderState {
                fill_style: stroke_style,
                global_alpha: canvas.global_alpha,
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
                composite_operation: renderer::CompositeOperation::default(),
                clip_path: None,
            };

            renderer.render_stroke_rect(rect, &render_state, stroke_color, line_width as f32);
        }

        Ok(Value::Undefined)
    }

    /// Internal op to rotate the transformation matrix
    fn internal_canvas_rotate<'gc>(
        agent: &mut Agent,
        _this: Value<'_>,
        args: ArgumentsList<'_, '_>,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let rid_val = args.get(0).to_int32(agent, gc.reborrow()).unbind().unwrap() as u32;
        let rid = Rid::from_index(rid_val);
        let angle = args
            .get(1)
            .to_number(agent, gc.reborrow())
            .unbind()?
            .into_f64(agent);

        let host_data = agent
            .get_host_data()
            .downcast_ref::<HostData<crate::RuntimeMacroTask>>()
            .unwrap();
        let mut storage = host_data.storage.borrow_mut();
        let res: &mut CanvasResources = storage.get_mut().unwrap();

        if let Some(mut canvas) = res.canvases.get_mut(rid) {
            let cos = angle.cos();
            let sin = angle.sin();
            let [a, b, c, d, e, f] = canvas.transform;

            // Multiply current transform by rotation matrix
            canvas.transform = [
                a * cos + c * sin,
                b * cos + d * sin,
                a * -sin + c * cos,
                b * -sin + d * cos,
                e,
                f,
            ];
        }

        Ok(Value::Undefined)
    }

    /// Internal op to scale the transformation matrix
    fn internal_canvas_scale<'gc>(
        agent: &mut Agent,
        _this: Value<'_>,
        args: ArgumentsList<'_, '_>,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let rid_val = args.get(0).to_int32(agent, gc.reborrow()).unbind().unwrap() as u32;
        let rid = Rid::from_index(rid_val);
        let x = args
            .get(1)
            .to_number(agent, gc.reborrow())
            .unbind()?
            .into_f64(agent);
        let y = args
            .get(2)
            .to_number(agent, gc.reborrow())
            .unbind()?
            .into_f64(agent);

        let host_data = agent
            .get_host_data()
            .downcast_ref::<HostData<crate::RuntimeMacroTask>>()
            .unwrap();
        let mut storage = host_data.storage.borrow_mut();
        let res: &mut CanvasResources = storage.get_mut().unwrap();

        if let Some(mut canvas) = res.canvases.get_mut(rid) {
            let [a, b, c, d, e, f] = canvas.transform;

            // Multiply current transform by scale matrix
            canvas.transform = [a * x, b * x, c * y, d * y, e, f];
        }

        Ok(Value::Undefined)
    }

    /// Internal op to translate the transformation matrix
    fn internal_canvas_translate<'gc>(
        agent: &mut Agent,
        _this: Value<'_>,
        args: ArgumentsList<'_, '_>,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let rid_val = args.get(0).to_int32(agent, gc.reborrow()).unbind().unwrap() as u32;
        let rid = Rid::from_index(rid_val);
        let x = args
            .get(1)
            .to_number(agent, gc.reborrow())
            .unbind()?
            .into_f64(agent);
        let y = args
            .get(2)
            .to_number(agent, gc.reborrow())
            .unbind()?
            .into_f64(agent);

        let host_data = agent
            .get_host_data()
            .downcast_ref::<HostData<crate::RuntimeMacroTask>>()
            .unwrap();
        let mut storage = host_data.storage.borrow_mut();
        let res: &mut CanvasResources = storage.get_mut().unwrap();

        if let Some(mut canvas) = res.canvases.get_mut(rid) {
            let [a, b, c, d, e, f] = canvas.transform;

            // Multiply current transform by translation matrix
            canvas.transform = [a, b, c, d, e + a * x + c * y, f + b * x + d * y];
        }

        Ok(Value::Undefined)
    }

    /// Internal op to transform (multiply) the transformation matrix
    fn internal_canvas_transform<'gc>(
        agent: &mut Agent,
        _this: Value<'_>,
        args: ArgumentsList<'_, '_>,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let rid_val = args.get(0).to_int32(agent, gc.reborrow()).unbind().unwrap() as u32;
        let rid = Rid::from_index(rid_val);
        let a2 = args
            .get(1)
            .to_number(agent, gc.reborrow())
            .unbind()?
            .into_f64(agent);
        let b2 = args
            .get(2)
            .to_number(agent, gc.reborrow())
            .unbind()?
            .into_f64(agent);
        let c2 = args
            .get(3)
            .to_number(agent, gc.reborrow())
            .unbind()?
            .into_f64(agent);
        let d2 = args
            .get(4)
            .to_number(agent, gc.reborrow())
            .unbind()?
            .into_f64(agent);
        let e2 = args
            .get(5)
            .to_number(agent, gc.reborrow())
            .unbind()?
            .into_f64(agent);
        let f2 = args
            .get(6)
            .to_number(agent, gc.reborrow())
            .unbind()?
            .into_f64(agent);

        let host_data = agent
            .get_host_data()
            .downcast_ref::<HostData<crate::RuntimeMacroTask>>()
            .unwrap();
        let mut storage = host_data.storage.borrow_mut();
        let res: &mut CanvasResources = storage.get_mut().unwrap();

        if let Some(mut canvas) = res.canvases.get_mut(rid) {
            let [a1, b1, c1, d1, e1, f1] = canvas.transform;

            // Multiply transformation matrices
            canvas.transform = [
                a1 * a2 + c1 * b2,
                b1 * a2 + d1 * b2,
                a1 * c2 + c1 * d2,
                b1 * c2 + d1 * d2,
                a1 * e2 + c1 * f2 + e1,
                b1 * e2 + d1 * f2 + f1,
            ];
        }

        Ok(Value::Undefined)
    }

    /// Internal op to set the transformation matrix (replace, not multiply)
    fn internal_canvas_set_transform<'gc>(
        agent: &mut Agent,
        _this: Value<'_>,
        args: ArgumentsList<'_, '_>,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let rid_val = args.get(0).to_int32(agent, gc.reborrow()).unbind().unwrap() as u32;
        let rid = Rid::from_index(rid_val);
        let a = args
            .get(1)
            .to_number(agent, gc.reborrow())
            .unbind()?
            .into_f64(agent);
        let b = args
            .get(2)
            .to_number(agent, gc.reborrow())
            .unbind()?
            .into_f64(agent);
        let c = args
            .get(3)
            .to_number(agent, gc.reborrow())
            .unbind()?
            .into_f64(agent);
        let d = args
            .get(4)
            .to_number(agent, gc.reborrow())
            .unbind()?
            .into_f64(agent);
        let e = args
            .get(5)
            .to_number(agent, gc.reborrow())
            .unbind()?
            .into_f64(agent);
        let f = args
            .get(6)
            .to_number(agent, gc.reborrow())
            .unbind()?
            .into_f64(agent);

        let host_data = agent
            .get_host_data()
            .downcast_ref::<HostData<crate::RuntimeMacroTask>>()
            .unwrap();
        let mut storage = host_data.storage.borrow_mut();
        let res: &mut CanvasResources = storage.get_mut().unwrap();

        if let Some(mut canvas) = res.canvases.get_mut(rid) {
            // Replace transformation matrix
            canvas.transform = [a, b, c, d, e, f];
        }

        Ok(Value::Undefined)
    }

    /// Internal op to reset the transformation matrix to identity
    fn internal_canvas_reset_transform<'gc>(
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
        let mut storage = host_data.storage.borrow_mut();
        let res: &mut CanvasResources = storage.get_mut().unwrap();

        if let Some(mut canvas) = res.canvases.get_mut(rid) {
            // Reset to identity matrix
            canvas.transform = [1.0, 0.0, 0.0, 1.0, 0.0, 0.0];
        }

        Ok(Value::Undefined)
    }

    /// Internal op to get the current transformation matrix
    fn internal_canvas_get_transform<'gc>(
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
        let canvas = res.canvases.get(rid).unwrap();
        let [a, b, c, d, e, f] = canvas.transform;
        drop(storage);

        // Return as JSON string with the transform values
        let json = format!(
            "{{\"a\":{},\"b\":{},\"c\":{},\"d\":{},\"e\":{},\"f\":{}}}",
            a, b, c, d, e, f
        );
        Ok(Value::from_string(agent, json, gc.nogc()).unbind())
    }

    /// Internal op to draw an image onto the canvas
    /// Arguments: canvas_rid, image_rid, sx, sy, s_width, s_height, dx, dy, d_width, d_height
    fn internal_canvas_draw_image<'gc>(
        agent: &mut Agent,
        _this: Value<'_>,
        args: ArgumentsList<'_, '_>,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let canvas_rid =
            Rid::from_index(args.get(0).to_int32(agent, gc.reborrow()).unbind()? as u32);
        let image_rid = args.get(1).to_int32(agent, gc.reborrow()).unbind()? as u32;
        let sx = args
            .get(2)
            .to_number(agent, gc.reborrow())
            .unbind()?
            .into_f64(agent);
        let sy = args
            .get(3)
            .to_number(agent, gc.reborrow())
            .unbind()?
            .into_f64(agent);
        let s_width = args
            .get(4)
            .to_number(agent, gc.reborrow())
            .unbind()?
            .into_f64(agent);
        let s_height = args
            .get(5)
            .to_number(agent, gc.reborrow())
            .unbind()?
            .into_f64(agent);
        let dx = args
            .get(6)
            .to_number(agent, gc.reborrow())
            .unbind()?
            .into_f64(agent);
        let dy = args
            .get(7)
            .to_number(agent, gc.reborrow())
            .unbind()?
            .into_f64(agent);
        let d_width = args
            .get(8)
            .to_number(agent, gc.reborrow())
            .unbind()?
            .into_f64(agent);
        let d_height = args
            .get(9)
            .to_number(agent, gc.reborrow())
            .unbind()?
            .into_f64(agent);

        let host_data = agent
            .get_host_data()
            .downcast_ref::<HostData<crate::RuntimeMacroTask>>()
            .unwrap();
        let mut storage = host_data.storage.borrow_mut();
        let res: &mut CanvasResources = storage.get_mut().unwrap();

        // Try to render directly with GPU if renderer exists
        let has_renderer = res.renderers.get_mut(canvas_rid).is_some();

        if has_renderer {
            // First, load the texture if not already loaded
            let image_data = res.images.get(Rid::from_index(image_rid));

            if let Some(img_data) = image_data
                && let Some(ref pixel_data) = img_data.data
            {
                // Load texture into cache if not already there
                if let Some(mut renderer) = res.renderers.get_mut(canvas_rid)
                    && !renderer.texture_cache.contains_key(&image_rid)
                {
                    renderer.load_image_texture(
                        image_rid,
                        pixel_data,
                        img_data.width,
                        img_data.height,
                    );
                }
            }

            // Get the current canvas state for rendering
            let data = res.canvases.get(canvas_rid).unwrap();
            let render_state = renderer::RenderState {
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

            // Render the image
            res.renderers.get_mut(canvas_rid).unwrap().render_image(
                image_rid,
                sx,
                sy,
                s_width,
                s_height,
                dx,
                dy,
                d_width,
                d_height,
                &render_state,
            );
        } else {
            // Fallback to command storage if no renderer
            if let Some(mut canvas) = res.canvases.get_mut(canvas_rid) {
                canvas.commands.push(context2d::CanvasCommand::DrawImage {
                    image_rid,
                    sx,
                    sy,
                    s_width,
                    s_height,
                    dx,
                    dy,
                    d_width,
                    d_height,
                });
            }
        }

        Ok(Value::Undefined)
    }

    /// Internal op to create an ImageData object
    fn internal_canvas_create_image_data<'gc>(
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
        let res: &mut CanvasResources = storage.get_mut().unwrap();

        // Create blank RGBA data (4 bytes per pixel, all transparent black)
        let pixel_count = (width * height * 4) as usize;
        let data = vec![0u8; pixel_count];

        let rid = res.images.push(ImageData {
            width,
            height,
            data: Some(data),
        });

        Ok(Value::Integer(SmallInteger::from(rid.index() as i32)))
    }

    /// Internal op to get image data from canvas
    fn internal_canvas_get_image_data<'gc>(
        agent: &mut Agent,
        _this: Value<'_>,
        args: ArgumentsList<'_, '_>,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let canvas_rid =
            Rid::from_index(args.get(0).to_int32(agent, gc.reborrow()).unbind()? as u32);
        let sx = args.get(1).to_int32(agent, gc.reborrow()).unbind()? as u32;
        let sy = args.get(2).to_int32(agent, gc.reborrow()).unbind()? as u32;
        let sw = args.get(3).to_int32(agent, gc.reborrow()).unbind()? as u32;
        let sh = args.get(4).to_int32(agent, gc.reborrow()).unbind()? as u32;

        let host_data = agent
            .get_host_data()
            .downcast_ref::<HostData<crate::RuntimeMacroTask>>()
            .unwrap();
        let mut storage = host_data.storage.borrow_mut();
        let res: &mut CanvasResources = storage.get_mut().unwrap();

        // Get renderer and read pixel data
        let renderer_rid = res.canvases.get(canvas_rid).map(|_| Rid::from_index(0));

        let pixel_data = if let Some(renderer_rid) = renderer_rid {
            if let Some(mut renderer) = res.renderers.get_mut(renderer_rid) {
                // Render all pending commands first
                renderer.render_all();

                // Read pixels from GPU (this requires async, so we'll use block_on)
                let bitmap = futures::executor::block_on(renderer.create_bitmap());

                // Extract the requested region
                extract_image_region(
                    &bitmap,
                    renderer.dimensions.width,
                    renderer.dimensions.height,
                    sx,
                    sy,
                    sw,
                    sh,
                )
            } else {
                vec![0u8; (sw * sh * 4) as usize]
            }
        } else {
            vec![0u8; (sw * sh * 4) as usize]
        };

        let rid = res.images.push(ImageData {
            width: sw,
            height: sh,
            data: Some(pixel_data),
        });

        Ok(Value::Integer(SmallInteger::from(rid.index() as i32)))
    }

    /// Internal op to put image data onto canvas
    fn internal_canvas_put_image_data<'gc>(
        agent: &mut Agent,
        _this: Value<'_>,
        args: ArgumentsList<'_, '_>,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let canvas_rid =
            Rid::from_index(args.get(0).to_int32(agent, gc.reborrow()).unbind()? as u32);
        let image_data_rid =
            Rid::from_index(args.get(1).to_int32(agent, gc.reborrow()).unbind()? as u32);
        let dx = args
            .get(2)
            .to_number(agent, gc.reborrow())
            .unbind()?
            .into_f64(agent);
        let dy = args
            .get(3)
            .to_number(agent, gc.reborrow())
            .unbind()?
            .into_f64(agent);

        let host_data = agent
            .get_host_data()
            .downcast_ref::<HostData<crate::RuntimeMacroTask>>()
            .unwrap();
        let mut storage = host_data.storage.borrow_mut();
        let res: &mut CanvasResources = storage.get_mut().unwrap();

        // Get image data and load it as a texture
        if let Some(image_data) = res.images.get(image_data_rid) {
            let width = image_data.width;
            let height = image_data.height;

            if let Some(data) = &image_data.data {
                // Load the image data into a temporary texture
                let renderer_rid = Rid::from_index(0); // Assume single renderer for now
                if let Some(mut renderer) = res.renderers.get_mut(renderer_rid) {
                    let temp_image_rid = u32::MAX; // Use special ID for temp texture
                    renderer.load_image_texture(temp_image_rid, data, width, height);
                }

                // Add draw command
                if let Some(mut canvas) = res.canvases.get_mut(canvas_rid) {
                    canvas.commands.push(context2d::CanvasCommand::DrawImage {
                        image_rid: u32::MAX,
                        sx: 0.0,
                        sy: 0.0,
                        s_width: width as f64,
                        s_height: height as f64,
                        dx,
                        dy,
                        d_width: width as f64,
                        d_height: height as f64,
                    });
                }
            }
        }

        Ok(Value::Undefined)
    }

    /// Internal op to get ImageData width
    fn internal_image_data_get_width<'gc>(
        agent: &mut Agent,
        _this: Value<'_>,
        args: ArgumentsList<'_, '_>,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let rid = Rid::from_index(args.get(0).to_int32(agent, gc.reborrow()).unbind()? as u32);

        let host_data = agent
            .get_host_data()
            .downcast_ref::<HostData<crate::RuntimeMacroTask>>()
            .unwrap();
        let storage = host_data.storage.borrow();
        let res: &CanvasResources = storage.get().unwrap();

        let width = res.images.get(rid).map(|img| img.width).unwrap_or(0);
        Ok(Value::Integer(SmallInteger::from(width as i32)))
    }

    /// Internal op to get ImageData height
    fn internal_image_data_get_height<'gc>(
        agent: &mut Agent,
        _this: Value<'_>,
        args: ArgumentsList<'_, '_>,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let rid = Rid::from_index(args.get(0).to_int32(agent, gc.reborrow()).unbind()? as u32);

        let host_data = agent
            .get_host_data()
            .downcast_ref::<HostData<crate::RuntimeMacroTask>>()
            .unwrap();
        let storage = host_data.storage.borrow();
        let res: &CanvasResources = storage.get().unwrap();

        let height = res.images.get(rid).map(|img| img.height).unwrap_or(0);
        Ok(Value::Integer(SmallInteger::from(height as i32)))
    }

    /// Internal op to get ImageData pixel data as a Uint8ClampedArray
    fn internal_image_data_get_data<'gc>(
        agent: &mut Agent,
        _this: Value<'_>,
        args: ArgumentsList<'_, '_>,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let rid = Rid::from_index(args.get(0).to_int32(agent, gc.reborrow()).unbind()? as u32);

        let host_data = agent
            .get_host_data()
            .downcast_ref::<HostData<crate::RuntimeMacroTask>>()
            .unwrap();
        let storage = host_data.storage.borrow();
        let res: &CanvasResources = storage.get().unwrap();

        // For now, return empty array - full implementation would return Uint8ClampedArray
        // This requires proper TypedArray support in the runtime
        if let Some(image_data) = res.images.get(rid)
            && let Some(data) = &image_data.data
        {
            // Convert to JSON array string as a temporary solution
            let json = format!(
                "[{}]",
                data.iter()
                    .map(|b| b.to_string())
                    .collect::<Vec<_>>()
                    .join(",")
            );
            drop(storage); // Drop the borrow before creating the string
            return Ok(Value::from_string(agent, json, gc.nogc()).unbind());
        }
        drop(storage); // Drop the borrow

        Ok(Value::Undefined)
    }

    // ========== PHASE 2 IMPLEMENTATIONS: LINE STYLES ==========

    /// Internal op to set lineCap property
    fn internal_canvas_set_line_cap<'gc>(
        agent: &mut Agent,
        _this: Value<'_>,
        args: ArgumentsList<'_, '_>,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let rid_val = args.get(0).to_int32(agent, gc.reborrow()).unbind()? as u32;
        let rid = Rid::from_index(rid_val);
        let cap_str = args.get(1).to_string(agent, gc.reborrow()).unbind()?;

        let host_data = agent
            .get_host_data()
            .downcast_ref::<HostData<crate::RuntimeMacroTask>>()
            .unwrap();
        let mut storage = host_data.storage.borrow_mut();
        let res: &mut CanvasResources = storage.get_mut().unwrap();

        if let Some(mut canvas) = res.canvases.get_mut(rid) {
            let cap_string = cap_str.as_str(agent).unwrap_or("butt");
            canvas.line_cap = match cap_string {
                "round" => renderer::LineCap::Round,
                "square" => renderer::LineCap::Square,
                _ => renderer::LineCap::Butt,
            };
        }

        Ok(Value::Undefined)
    }

    /// Internal op to get lineCap property
    fn internal_canvas_get_line_cap<'gc>(
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
        let canvas = res.canvases.get(rid).unwrap();

        let cap_str = match canvas.line_cap {
            renderer::LineCap::Butt => "butt",
            renderer::LineCap::Round => "round",
            renderer::LineCap::Square => "square",
        };

        drop(storage);
        Ok(Value::from_string(agent, cap_str.to_string(), gc.nogc()).unbind())
    }

    /// Internal op to set lineJoin property
    fn internal_canvas_set_line_join<'gc>(
        agent: &mut Agent,
        _this: Value<'_>,
        args: ArgumentsList<'_, '_>,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let rid_val = args.get(0).to_int32(agent, gc.reborrow()).unbind()? as u32;
        let rid = Rid::from_index(rid_val);
        let join_str = args.get(1).to_string(agent, gc.reborrow()).unbind()?;

        let host_data = agent
            .get_host_data()
            .downcast_ref::<HostData<crate::RuntimeMacroTask>>()
            .unwrap();
        let mut storage = host_data.storage.borrow_mut();
        let res: &mut CanvasResources = storage.get_mut().unwrap();

        if let Some(mut canvas) = res.canvases.get_mut(rid) {
            let join_string = join_str.as_str(agent).unwrap_or("miter");
            canvas.line_join = match join_string {
                "round" => renderer::LineJoin::Round,
                "bevel" => renderer::LineJoin::Bevel,
                _ => renderer::LineJoin::Miter,
            };
        }

        Ok(Value::Undefined)
    }

    /// Internal op to get lineJoin property
    fn internal_canvas_get_line_join<'gc>(
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
        let canvas = res.canvases.get(rid).unwrap();

        let join_str = match canvas.line_join {
            renderer::LineJoin::Miter => "miter",
            renderer::LineJoin::Round => "round",
            renderer::LineJoin::Bevel => "bevel",
        };

        drop(storage);
        Ok(Value::from_string(agent, join_str.to_string(), gc.nogc()).unbind())
    }

    /// Internal op to set miterLimit property
    fn internal_canvas_set_miter_limit<'gc>(
        agent: &mut Agent,
        _this: Value<'_>,
        args: ArgumentsList<'_, '_>,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let rid_val = args.get(0).to_int32(agent, gc.reborrow()).unbind()? as u32;
        let rid = Rid::from_index(rid_val);
        let limit = args
            .get(1)
            .to_number(agent, gc.reborrow())
            .unbind()?
            .into_f64(agent);

        let host_data = agent
            .get_host_data()
            .downcast_ref::<HostData<crate::RuntimeMacroTask>>()
            .unwrap();
        let mut storage = host_data.storage.borrow_mut();
        let res: &mut CanvasResources = storage.get_mut().unwrap();

        if let Some(mut canvas) = res.canvases.get_mut(rid)
            && limit > 0.0
        {
            canvas.miter_limit = limit;
        }

        Ok(Value::Undefined)
    }

    /// Internal op to get miterLimit property
    fn internal_canvas_get_miter_limit<'gc>(
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
        let canvas = res.canvases.get(rid).unwrap();
        let limit = canvas.miter_limit;

        drop(storage);
        Ok(Value::from_f64(agent, limit, gc.nogc()).unbind())
    }

    // ========== PHASE 2 IMPLEMENTATIONS: SHADOWS ==========

    /// Internal op to set shadowBlur property
    fn internal_canvas_set_shadow_blur<'gc>(
        agent: &mut Agent,
        _this: Value<'_>,
        args: ArgumentsList<'_, '_>,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let rid_val = args.get(0).to_int32(agent, gc.reborrow()).unbind()? as u32;
        let rid = Rid::from_index(rid_val);
        let blur = args
            .get(1)
            .to_number(agent, gc.reborrow())
            .unbind()?
            .into_f64(agent);

        let host_data = agent
            .get_host_data()
            .downcast_ref::<HostData<crate::RuntimeMacroTask>>()
            .unwrap();
        let mut storage = host_data.storage.borrow_mut();
        let res: &mut CanvasResources = storage.get_mut().unwrap();

        if let Some(mut canvas) = res.canvases.get_mut(rid) {
            canvas.shadow_blur = blur.max(0.0);
        }

        Ok(Value::Undefined)
    }

    /// Internal op to get shadowBlur property
    fn internal_canvas_get_shadow_blur<'gc>(
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
        let canvas = res.canvases.get(rid).unwrap();
        let blur = canvas.shadow_blur;

        drop(storage);
        Ok(Value::from_f64(agent, blur, gc.nogc()).unbind())
    }

    /// Internal op to set shadowColor property
    fn internal_canvas_set_shadow_color<'gc>(
        agent: &mut Agent,
        _this: Value<'_>,
        args: ArgumentsList<'_, '_>,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let rid_val = args.get(0).to_int32(agent, gc.reborrow()).unbind()? as u32;
        let rid = Rid::from_index(rid_val);
        let color_str = args.get(1).to_string(agent, gc.reborrow()).unbind()?;
        let color_string = color_str
            .as_str(agent)
            .unwrap_or("rgba(0,0,0,0)")
            .to_string();

        let host_data = agent
            .get_host_data()
            .downcast_ref::<HostData<crate::RuntimeMacroTask>>()
            .unwrap();
        let mut storage = host_data.storage.borrow_mut();
        let res: &mut CanvasResources = storage.get_mut().unwrap();

        if let Some(mut canvas) = res.canvases.get_mut(rid)
            && let Ok(shadow_color) = FillStyle::from_css_color(&color_string)
        {
            canvas.shadow_color = shadow_color;
        }

        Ok(Value::Undefined)
    }

    /// Internal op to get shadowColor property
    fn internal_canvas_get_shadow_color<'gc>(
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
        let canvas = res.canvases.get(rid).unwrap();

        let color_str = match &canvas.shadow_color {
            FillStyle::Color { r, g, b, a } => {
                if *a == 1.0 {
                    format!(
                        "rgb({}, {}, {})",
                        (*r * 255.0) as u8,
                        (*g * 255.0) as u8,
                        (*b * 255.0) as u8
                    )
                } else {
                    format!(
                        "rgba({}, {}, {}, {})",
                        (*r * 255.0) as u8,
                        (*g * 255.0) as u8,
                        (*b * 255.0) as u8,
                        a
                    )
                }
            }
            _ => "rgba(0, 0, 0, 0)".to_string(),
        };

        drop(storage);
        Ok(Value::from_string(agent, color_str, gc.nogc()).unbind())
    }

    /// Internal op to set shadowOffsetX property
    fn internal_canvas_set_shadow_offset_x<'gc>(
        agent: &mut Agent,
        _this: Value<'_>,
        args: ArgumentsList<'_, '_>,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let rid_val = args.get(0).to_int32(agent, gc.reborrow()).unbind()? as u32;
        let rid = Rid::from_index(rid_val);
        let offset_x = args
            .get(1)
            .to_number(agent, gc.reborrow())
            .unbind()?
            .into_f64(agent);

        let host_data = agent
            .get_host_data()
            .downcast_ref::<HostData<crate::RuntimeMacroTask>>()
            .unwrap();
        let mut storage = host_data.storage.borrow_mut();
        let res: &mut CanvasResources = storage.get_mut().unwrap();

        if let Some(mut canvas) = res.canvases.get_mut(rid) {
            canvas.shadow_offset_x = offset_x;
        }

        Ok(Value::Undefined)
    }

    /// Internal op to get shadowOffsetX property
    fn internal_canvas_get_shadow_offset_x<'gc>(
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
        let canvas = res.canvases.get(rid).unwrap();
        let offset_x = canvas.shadow_offset_x;

        drop(storage);
        Ok(Value::from_f64(agent, offset_x, gc.nogc()).unbind())
    }

    /// Internal op to set shadowOffsetY property
    fn internal_canvas_set_shadow_offset_y<'gc>(
        agent: &mut Agent,
        _this: Value<'_>,
        args: ArgumentsList<'_, '_>,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let rid_val = args.get(0).to_int32(agent, gc.reborrow()).unbind()? as u32;
        let rid = Rid::from_index(rid_val);
        let offset_y = args
            .get(1)
            .to_number(agent, gc.reborrow())
            .unbind()?
            .into_f64(agent);

        let host_data = agent
            .get_host_data()
            .downcast_ref::<HostData<crate::RuntimeMacroTask>>()
            .unwrap();
        let mut storage = host_data.storage.borrow_mut();
        let res: &mut CanvasResources = storage.get_mut().unwrap();

        if let Some(mut canvas) = res.canvases.get_mut(rid) {
            canvas.shadow_offset_y = offset_y;
        }

        Ok(Value::Undefined)
    }

    /// Internal op to get shadowOffsetY property
    fn internal_canvas_get_shadow_offset_y<'gc>(
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
        let canvas = res.canvases.get(rid).unwrap();
        let offset_y = canvas.shadow_offset_y;

        drop(storage);
        Ok(Value::from_f64(agent, offset_y, gc.nogc()).unbind())
    }

    // ========== TEXT PROPERTIES ==========

    /// Internal op to set font property
    fn internal_canvas_set_font<'gc>(
        agent: &mut Agent,
        _this: Value<'_>,
        args: ArgumentsList<'_, '_>,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let rid_val = args.get(0).to_int32(agent, gc.reborrow()).unbind()? as u32;
        let rid = Rid::from_index(rid_val);
        let font_str = args.get(1).to_string(agent, gc.reborrow()).unbind()?;
        let font_string = font_str
            .as_str(agent)
            .unwrap_or("10px sans-serif")
            .to_string();

        let host_data = agent
            .get_host_data()
            .downcast_ref::<HostData<crate::RuntimeMacroTask>>()
            .unwrap();
        let mut storage = host_data.storage.borrow_mut();
        let res: &mut CanvasResources = storage.get_mut().unwrap();

        if let Some(mut canvas) = res.canvases.get_mut(rid) {
            // Validate font string by attempting to parse it
            if font_system::FontManager::parse_font_string(&font_string).is_ok() {
                canvas.font = font_string;
            }
        }

        Ok(Value::Undefined)
    }

    /// Internal op to get font property
    fn internal_canvas_get_font<'gc>(
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
        let canvas = res.canvases.get(rid).unwrap();
        let font = canvas.font.clone();

        drop(storage);
        Ok(Value::from_string(agent, font, gc.nogc()).unbind())
    }

    /// Internal op to set textAlign property
    fn internal_canvas_set_text_align<'gc>(
        agent: &mut Agent,
        _this: Value<'_>,
        args: ArgumentsList<'_, '_>,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let rid_val = args.get(0).to_int32(agent, gc.reborrow()).unbind()? as u32;
        let rid = Rid::from_index(rid_val);
        let align_str = args.get(1).to_string(agent, gc.reborrow()).unbind()?;

        let host_data = agent
            .get_host_data()
            .downcast_ref::<HostData<crate::RuntimeMacroTask>>()
            .unwrap();
        let mut storage = host_data.storage.borrow_mut();
        let res: &mut CanvasResources = storage.get_mut().unwrap();

        if let Some(mut canvas) = res.canvases.get_mut(rid) {
            let align_string = align_str.as_str(agent).unwrap_or("start");
            canvas.text_align = match align_string {
                "left" => state::TextAlign::Left,
                "right" => state::TextAlign::Right,
                "center" => state::TextAlign::Center,
                "start" => state::TextAlign::Start,
                "end" => state::TextAlign::End,
                _ => state::TextAlign::Start,
            };
        }

        Ok(Value::Undefined)
    }

    /// Internal op to get textAlign property
    fn internal_canvas_get_text_align<'gc>(
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
        let canvas = res.canvases.get(rid).unwrap();

        let align_str = match canvas.text_align {
            state::TextAlign::Start => "start",
            state::TextAlign::End => "end",
            state::TextAlign::Left => "left",
            state::TextAlign::Right => "right",
            state::TextAlign::Center => "center",
        };

        drop(storage);
        Ok(Value::from_string(agent, align_str.to_string(), gc.nogc()).unbind())
    }

    /// Internal op to set textBaseline property
    fn internal_canvas_set_text_baseline<'gc>(
        agent: &mut Agent,
        _this: Value<'_>,
        args: ArgumentsList<'_, '_>,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let rid_val = args.get(0).to_int32(agent, gc.reborrow()).unbind()? as u32;
        let rid = Rid::from_index(rid_val);
        let baseline_str = args.get(1).to_string(agent, gc.reborrow()).unbind()?;

        let host_data = agent
            .get_host_data()
            .downcast_ref::<HostData<crate::RuntimeMacroTask>>()
            .unwrap();
        let mut storage = host_data.storage.borrow_mut();
        let res: &mut CanvasResources = storage.get_mut().unwrap();

        if let Some(mut canvas) = res.canvases.get_mut(rid) {
            let baseline_string = baseline_str.as_str(agent).unwrap_or("alphabetic");
            canvas.text_baseline = match baseline_string {
                "top" => state::TextBaseline::Top,
                "hanging" => state::TextBaseline::Hanging,
                "middle" => state::TextBaseline::Middle,
                "alphabetic" => state::TextBaseline::Alphabetic,
                "ideographic" => state::TextBaseline::Ideographic,
                "bottom" => state::TextBaseline::Bottom,
                _ => state::TextBaseline::Alphabetic,
            };
        }

        Ok(Value::Undefined)
    }

    /// Internal op to get textBaseline property
    fn internal_canvas_get_text_baseline<'gc>(
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
        let canvas = res.canvases.get(rid).unwrap();

        let baseline_str = match canvas.text_baseline {
            state::TextBaseline::Top => "top",
            state::TextBaseline::Hanging => "hanging",
            state::TextBaseline::Middle => "middle",
            state::TextBaseline::Alphabetic => "alphabetic",
            state::TextBaseline::Ideographic => "ideographic",
            state::TextBaseline::Bottom => "bottom",
        };

        drop(storage);
        Ok(Value::from_string(agent, baseline_str.to_string(), gc.nogc()).unbind())
    }

    /// Internal op to set direction property
    fn internal_canvas_set_direction<'gc>(
        agent: &mut Agent,
        _this: Value<'_>,
        args: ArgumentsList<'_, '_>,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let rid_val = args.get(0).to_int32(agent, gc.reborrow()).unbind()? as u32;
        let rid = Rid::from_index(rid_val);
        let direction_str = args.get(1).to_string(agent, gc.reborrow()).unbind()?;

        let host_data = agent
            .get_host_data()
            .downcast_ref::<HostData<crate::RuntimeMacroTask>>()
            .unwrap();
        let mut storage = host_data.storage.borrow_mut();
        let res: &mut CanvasResources = storage.get_mut().unwrap();

        if let Some(mut canvas) = res.canvases.get_mut(rid) {
            let direction_string = direction_str.as_str(agent).unwrap_or("inherit");
            canvas.direction = match direction_string {
                "ltr" => state::Direction::Ltr,
                "rtl" => state::Direction::Rtl,
                "inherit" => state::Direction::Inherit,
                _ => state::Direction::Inherit,
            };
        }

        Ok(Value::Undefined)
    }

    /// Internal op to get direction property
    fn internal_canvas_get_direction<'gc>(
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
        let canvas = res.canvases.get(rid).unwrap();

        let direction_str = match canvas.direction {
            state::Direction::Ltr => "ltr",
            state::Direction::Rtl => "rtl",
            state::Direction::Inherit => "inherit",
        };

        drop(storage);
        Ok(Value::from_string(agent, direction_str.to_string(), gc.nogc()).unbind())
    }

    /// Internal op to measure text and return TextMetrics
    fn internal_canvas_measure_text<'gc>(
        agent: &mut Agent,
        _this: Value<'_>,
        args: ArgumentsList<'_, '_>,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let rid_val = args.get(0).to_int32(agent, gc.reborrow()).unbind()? as u32;
        let rid = Rid::from_index(rid_val);
        let text_str = args.get(1).to_string(agent, gc.reborrow()).unbind()?;
        let text = text_str.as_str(agent).unwrap_or("");

        let host_data = agent
            .get_host_data()
            .downcast_ref::<HostData<crate::RuntimeMacroTask>>()
            .unwrap();
        let storage = host_data.storage.borrow();
        let res: &CanvasResources = storage.get().unwrap();
        let canvas = res.canvases.get(rid).unwrap();
        let font_string = canvas.font.clone();
        drop(storage);

        // Parse the font string
        let font_descriptor = match font_system::FontManager::parse_font_string(&font_string) {
            Ok(descriptor) => descriptor,
            Err(_) => {
                // Return default metrics on parse error
                let json = r#"{"width":0,"actualBoundingBoxLeft":0,"actualBoundingBoxRight":0,"fontBoundingBoxAscent":0,"fontBoundingBoxDescent":0,"actualBoundingBoxAscent":0,"actualBoundingBoxDescent":0,"emHeightAscent":0,"emHeightDescent":0,"hangingBaseline":0,"alphabeticBaseline":0,"ideographicBaseline":0}"#;
                return Ok(Value::from_string(agent, json.to_string(), gc.nogc()).unbind());
            }
        };

        // Create a font manager for measurement
        let mut font_manager = font_system::FontManager::new();

        // Measure the text
        let metrics = match text_metrics::TextMetrics::measure(
            text,
            &mut font_manager,
            &font_descriptor,
        ) {
            Ok(metrics) => metrics,
            Err(_) => {
                // Return default metrics on measurement error
                let json = r#"{"width":0,"actualBoundingBoxLeft":0,"actualBoundingBoxRight":0,"fontBoundingBoxAscent":0,"fontBoundingBoxDescent":0,"actualBoundingBoxAscent":0,"actualBoundingBoxDescent":0,"emHeightAscent":0,"emHeightDescent":0,"hangingBaseline":0,"alphabeticBaseline":0,"ideographicBaseline":0}"#;
                return Ok(Value::from_string(agent, json.to_string(), gc.nogc()).unbind());
            }
        };

        // Serialize metrics to JSON
        let json = format!(
            r#"{{"width":{},"actualBoundingBoxLeft":{},"actualBoundingBoxRight":{},"fontBoundingBoxAscent":{},"fontBoundingBoxDescent":{},"actualBoundingBoxAscent":{},"actualBoundingBoxDescent":{},"emHeightAscent":{},"emHeightDescent":{},"hangingBaseline":{},"alphabeticBaseline":{},"ideographicBaseline":{}}}"#,
            metrics.width,
            metrics.actual_bounding_box_left,
            metrics.actual_bounding_box_right,
            metrics.font_bounding_box_ascent,
            metrics.font_bounding_box_descent,
            metrics.actual_bounding_box_ascent,
            metrics.actual_bounding_box_descent,
            metrics.em_height_ascent,
            metrics.em_height_descent,
            metrics.hanging_baseline,
            metrics.alphabetic_baseline,
            metrics.ideographic_baseline
        );

        Ok(Value::from_string(agent, json, gc.nogc()).unbind())
    }

    /// Internal op to render filled text on the canvas
    fn internal_canvas_fill_text<'gc>(
        agent: &mut Agent,
        _this: Value<'_>,
        args: ArgumentsList<'_, '_>,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let rid_val = args.get(0).to_int32(agent, gc.reborrow()).unbind().unwrap() as u32;
        let rid = Rid::from_index(rid_val);

        let text_str = args.get(1).to_string(agent, gc.reborrow()).unbind()?;
        let text = text_str.as_str(agent).unwrap().to_string();

        let x = args
            .get(2)
            .to_number(agent, gc.reborrow())
            .unbind()?
            .into_f64(agent);
        let y = args
            .get(3)
            .to_number(agent, gc.reborrow())
            .unbind()?
            .into_f64(agent);

        // Optional max_width parameter (not fully implemented yet, but parsed for future use)
        let _max_width = if args.len() > 4 {
            Some(
                args.get(4)
                    .to_number(agent, gc.reborrow())
                    .unbind()?
                    .into_f64(agent),
            )
        } else {
            None
        };

        let host_data = agent
            .get_host_data()
            .downcast_ref::<HostData<crate::RuntimeMacroTask>>()
            .unwrap();
        let mut storage = host_data.storage.borrow_mut();
        let res: &mut CanvasResources = storage.get_mut().unwrap();

        if let Some(canvas) = res.canvases.get(rid) {
            // Get current font and parse it
            let font_descriptor = match font_system::FontManager::parse_font_string(&canvas.font) {
                Ok(desc) => desc,
                Err(_) => return Ok(Value::Undefined), // Silently fail on invalid font
            };

            // Create FontManager and TextRenderer
            let mut text_renderer = text::TextRenderer::new();

            // Get fill color from fill_style (defaulting to black)
            let color = match &canvas.fill_style {
                FillStyle::Color { r, g, b, a } => [
                    (r * 255.0) as u8,
                    (g * 255.0) as u8,
                    (b * 255.0) as u8,
                    (a * 255.0) as u8,
                ],
                _ => [0, 0, 0, 255], // Default to black for gradients/patterns
            };

            // Render text to bitmap
            let (bitmap, width, height) =
                match text_renderer.render_text_to_bitmap(&text, &font_descriptor, color) {
                    Ok(result) => result,
                    Err(_) => return Ok(Value::Undefined), // Silently fail on render error
                };

            if width == 0 || height == 0 {
                return Ok(Value::Undefined); // Nothing to render
            }

            // Upload bitmap as texture and render it
            if let Some(mut renderer) = res.renderers.get_mut(rid) {
                // Generate a unique texture ID for this text
                let texture_id = res.next_texture_id;
                res.next_texture_id += 1;

                // Upload bitmap to GPU
                renderer.load_image_texture(texture_id, &bitmap, width, height);

                // Create render state from canvas
                let render_state = renderer::RenderState {
                    fill_style: canvas.fill_style.clone(),
                    global_alpha: canvas.global_alpha,
                    transform: canvas.transform,
                    line_cap: canvas.line_cap,
                    line_join: canvas.line_join,
                    miter_limit: canvas.miter_limit,
                    shadow_blur: canvas.shadow_blur,
                    shadow_color: canvas.shadow_color.clone(),
                    shadow_offset_x: canvas.shadow_offset_x,
                    shadow_offset_y: canvas.shadow_offset_y,
                    composite_operation: canvas.composite_operation,
                    clip_path: None,
                };

                // Calculate baseline adjustment
                let baseline_offset = calculate_baseline_offset(
                    &canvas.text_baseline,
                    &font_descriptor,
                    height as f64,
                );

                // Calculate alignment adjustment
                let align_offset =
                    calculate_alignment_offset(&canvas.text_align, &canvas.direction, width as f64);

                // Render text bitmap as textured rectangle
                renderer.render_image(
                    texture_id,
                    0.0,
                    0.0,
                    width as f64,
                    height as f64,
                    x + align_offset,
                    y + baseline_offset,
                    width as f64,
                    height as f64,
                    &render_state,
                );
            }
        }

        Ok(Value::Undefined)
    }

    /// Internal op to render stroked (outlined) text on the canvas
    fn internal_canvas_stroke_text<'gc>(
        agent: &mut Agent,
        _this: Value<'_>,
        args: ArgumentsList<'_, '_>,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let rid_val = args.get(0).to_int32(agent, gc.reborrow()).unbind().unwrap() as u32;
        let rid = Rid::from_index(rid_val);

        let text_str = args.get(1).to_string(agent, gc.reborrow()).unbind()?;
        let text = text_str.as_str(agent).unwrap().to_string();

        let x = args
            .get(2)
            .to_number(agent, gc.reborrow())
            .unbind()?
            .into_f64(agent);
        let y = args
            .get(3)
            .to_number(agent, gc.reborrow())
            .unbind()?
            .into_f64(agent);

        // Optional max_width parameter
        let _max_width = if args.len() > 4 {
            Some(
                args.get(4)
                    .to_number(agent, gc.reborrow())
                    .unbind()?
                    .into_f64(agent),
            )
        } else {
            None
        };

        let host_data = agent
            .get_host_data()
            .downcast_ref::<HostData<crate::RuntimeMacroTask>>()
            .unwrap();
        let mut storage = host_data.storage.borrow_mut();
        let res: &mut CanvasResources = storage.get_mut().unwrap();

        if let Some(canvas) = res.canvases.get(rid) {
            // Get current font and parse it
            let font_descriptor = match font_system::FontManager::parse_font_string(&canvas.font) {
                Ok(desc) => desc,
                Err(_) => return Ok(Value::Undefined),
            };

            // For now, stroke text is rendered the same as fill text
            // TODO: Implement actual text stroking with line width
            let mut text_renderer = text::TextRenderer::new();

            // Get stroke color from stroke_style (defaulting to black)
            let color = match &canvas.stroke_style {
                FillStyle::Color { r, g, b, a } => [
                    (r * 255.0) as u8,
                    (g * 255.0) as u8,
                    (b * 255.0) as u8,
                    (a * 255.0) as u8,
                ],
                _ => [0, 0, 0, 255], // Default to black for gradients/patterns
            };

            let (bitmap, width, height) =
                match text_renderer.render_text_to_bitmap(&text, &font_descriptor, color) {
                    Ok(result) => result,
                    Err(_) => return Ok(Value::Undefined),
                };

            if width == 0 || height == 0 {
                return Ok(Value::Undefined);
            }

            if let Some(mut renderer) = res.renderers.get_mut(rid) {
                let texture_id = res.next_texture_id;
                res.next_texture_id += 1;

                renderer.load_image_texture(texture_id, &bitmap, width, height);

                let render_state = renderer::RenderState {
                    fill_style: canvas.stroke_style.clone(), // Use stroke style for stroked text
                    global_alpha: canvas.global_alpha,
                    transform: canvas.transform,
                    line_cap: canvas.line_cap,
                    line_join: canvas.line_join,
                    miter_limit: canvas.miter_limit,
                    shadow_blur: canvas.shadow_blur,
                    shadow_color: canvas.shadow_color.clone(),
                    shadow_offset_x: canvas.shadow_offset_x,
                    shadow_offset_y: canvas.shadow_offset_y,
                    composite_operation: canvas.composite_operation,
                    clip_path: None,
                };

                let baseline_offset = calculate_baseline_offset(
                    &canvas.text_baseline,
                    &font_descriptor,
                    height as f64,
                );

                let align_offset =
                    calculate_alignment_offset(&canvas.text_align, &canvas.direction, width as f64);

                renderer.render_image(
                    texture_id,
                    0.0,
                    0.0,
                    width as f64,
                    height as f64,
                    x + align_offset,
                    y + baseline_offset,
                    width as f64,
                    height as f64,
                    &render_state,
                );
            }
        }

        Ok(Value::Undefined)
    }

    // ========== PHASE 2 IMPLEMENTATIONS: PATTERNS ==========

    /// Internal op to create a pattern from an image with repetition mode
    fn internal_canvas_create_pattern<'gc>(
        agent: &mut Agent,
        _this: Value<'_>,
        args: ArgumentsList<'_, '_>,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let image_rid = args.get(0).to_int32(agent, gc.reborrow()).unbind()? as u32;
        let repetition_str = args.get(1).to_string(agent, gc.reborrow()).unbind()?;
        let repetition_string = repetition_str.as_str(agent).unwrap_or("repeat").to_string();

        let host_data = agent
            .get_host_data()
            .downcast_ref::<HostData<crate::RuntimeMacroTask>>()
            .unwrap();
        let mut storage = host_data.storage.borrow_mut();
        let res: &mut CanvasResources = storage.get_mut().unwrap();

        let repetition = repetition_string
            .parse::<fill_style::PatternRepetition>()
            .unwrap_or(fill_style::PatternRepetition::Repeat);

        let pattern = FillStyle::Pattern {
            image_rid,
            repetition,
        };

        let pattern_rid = res.fill_styles.push(pattern);

        Ok(Value::Integer(SmallInteger::from(
            pattern_rid.index() as i32
        )))
    }
}

/// Helper function to extract a region from bitmap data
fn extract_image_region(
    bitmap: &[u8],
    full_width: u32,
    full_height: u32,
    sx: u32,
    sy: u32,
    sw: u32,
    sh: u32,
) -> Vec<u8> {
    let mut result = Vec::with_capacity((sw * sh * 4) as usize);

    for y in 0..sh {
        let src_y = sy + y;
        if src_y >= full_height {
            // Out of bounds, fill with transparent
            result.extend(vec![0u8; (sw * 4) as usize]);
            continue;
        }

        for x in 0..sw {
            let src_x = sx + x;
            if src_x >= full_width {
                // Out of bounds, transparent pixel
                result.extend_from_slice(&[0, 0, 0, 0]);
            } else {
                let idx = ((src_y * full_width + src_x) * 4) as usize;
                if idx + 3 < bitmap.len() {
                    result.extend_from_slice(&bitmap[idx..idx + 4]);
                } else {
                    result.extend_from_slice(&[0, 0, 0, 0]);
                }
            }
        }
    }

    result
}

fn create_wgpu_device_sync() -> (wgpu::Device, wgpu::Queue) {
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
    });

    let adapter =
        futures::executor::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: None,
            force_fallback_adapter: false,
        }))
        .unwrap();

    let (device, queue) =
        futures::executor::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            label: None,
            memory_hints: Default::default(),
            trace: wgpu::Trace::default(),
            experimental_features: wgpu::ExperimentalFeatures::default(),
        }))
        .unwrap();

    (device, queue)
}
