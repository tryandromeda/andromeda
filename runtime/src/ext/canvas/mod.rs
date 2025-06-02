// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use andromeda_core::{Extension, ExtensionOp, HostData, OpsStorage, ResourceTable, Rid};
mod context2d;
mod renderer;
use crate::ext::canvas::context2d::{
    internal_canvas_begin_path, internal_canvas_bezier_curve_to, internal_canvas_close_path,
};

use self::context2d::{
    internal_canvas_arc, internal_canvas_arc_to, internal_canvas_clear_rect,
    internal_canvas_fill_rect,
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

/// Represents different fill styles for Canvas 2D operations
#[derive(Clone, Debug)]
pub enum FillStyle {
    /// Solid color specified as RGBA values (0.0-1.0)
    Color { r: f32, g: f32, b: f32, a: f32 },
    /// Linear gradient (placeholder for future implementation)
    LinearGradient,
    /// Radial gradient (placeholder for future implementation)
    RadialGradient,
    /// Pattern (placeholder for future implementation)
    Pattern,
}

impl Default for FillStyle {
    fn default() -> Self {
        // Default to black color
        FillStyle::Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 }
    }
}

impl FillStyle {
    /// Parse a CSS color string into a FillStyle
    pub fn from_css_color(color_str: &str) -> Result<Self, String> {
        let color_str = color_str.trim();
        
        // Handle hex colors
        if color_str.starts_with('#') {
            return Self::parse_hex_color(color_str);
        }
        
        // Handle rgb() and rgba() colors
        if color_str.starts_with("rgb(") || color_str.starts_with("rgba(") {
            return Self::parse_rgb_color(color_str);
        }
        
        // Handle named colors
        Self::parse_named_color(color_str)
    }
    
    fn parse_hex_color(hex: &str) -> Result<Self, String> {
        let hex = hex.trim_start_matches('#');
        
        match hex.len() {
            3 => {
                // Short hex format like #RGB
                let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).map_err(|_| "Invalid hex color")?;
                let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).map_err(|_| "Invalid hex color")?;
                let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).map_err(|_| "Invalid hex color")?;
                Ok(FillStyle::Color {
                    r: r as f32 / 255.0,
                    g: g as f32 / 255.0,
                    b: b as f32 / 255.0,
                    a: 1.0,
                })
            }
            6 => {
                // Full hex format like #RRGGBB
                let r = u8::from_str_radix(&hex[0..2], 16).map_err(|_| "Invalid hex color")?;
                let g = u8::from_str_radix(&hex[2..4], 16).map_err(|_| "Invalid hex color")?;
                let b = u8::from_str_radix(&hex[4..6], 16).map_err(|_| "Invalid hex color")?;
                Ok(FillStyle::Color {
                    r: r as f32 / 255.0,
                    g: g as f32 / 255.0,
                    b: b as f32 / 255.0,
                    a: 1.0,
                })
            }
            8 => {
                // Hex with alpha like #RRGGBBAA
                let r = u8::from_str_radix(&hex[0..2], 16).map_err(|_| "Invalid hex color")?;
                let g = u8::from_str_radix(&hex[2..4], 16).map_err(|_| "Invalid hex color")?;
                let b = u8::from_str_radix(&hex[4..6], 16).map_err(|_| "Invalid hex color")?;
                let a = u8::from_str_radix(&hex[6..8], 16).map_err(|_| "Invalid hex color")?;
                Ok(FillStyle::Color {
                    r: r as f32 / 255.0,
                    g: g as f32 / 255.0,
                    b: b as f32 / 255.0,
                    a: a as f32 / 255.0,
                })
            }
            _ => Err("Invalid hex color length".to_string()),
        }
    }
    
    fn parse_rgb_color(rgb: &str) -> Result<Self, String> {
        let is_rgba = rgb.starts_with("rgba(");
        let inner = if is_rgba {
            rgb.trim_start_matches("rgba(").trim_end_matches(')')
        } else {
            rgb.trim_start_matches("rgb(").trim_end_matches(')')
        };
        
        let parts: Vec<&str> = inner.split(',').map(|s| s.trim()).collect();
        
        if (!is_rgba && parts.len() != 3) || (is_rgba && parts.len() != 4) {
            return Err("Invalid rgb/rgba format".to_string());
        }
        
        let r = parts[0].parse::<f32>().map_err(|_| "Invalid red value")? / 255.0;
        let g = parts[1].parse::<f32>().map_err(|_| "Invalid green value")? / 255.0;
        let b = parts[2].parse::<f32>().map_err(|_| "Invalid blue value")? / 255.0;
        let a = if is_rgba {
            parts[3].parse::<f32>().map_err(|_| "Invalid alpha value")?
        } else {
            1.0
        };
        
        Ok(FillStyle::Color { r, g, b, a })
    }
    
    fn parse_named_color(name: &str) -> Result<Self, String> {
        match name.to_lowercase().as_str() {
            "black" => Ok(FillStyle::Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 }),
            "white" => Ok(FillStyle::Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 }),
            "red" => Ok(FillStyle::Color { r: 1.0, g: 0.0, b: 0.0, a: 1.0 }),
            "green" => Ok(FillStyle::Color { r: 0.0, g: 1.0, b: 0.0, a: 1.0 }),
            "blue" => Ok(FillStyle::Color { r: 0.0, g: 0.0, b: 1.0, a: 1.0 }),
            "yellow" => Ok(FillStyle::Color { r: 1.0, g: 1.0, b: 0.0, a: 1.0 }),
            "magenta" | "fuchsia" => Ok(FillStyle::Color { r: 1.0, g: 0.0, b: 1.0, a: 1.0 }),
            "cyan" | "aqua" => Ok(FillStyle::Color { r: 0.0, g: 1.0, b: 1.0, a: 1.0 }),
            "orange" => Ok(FillStyle::Color { r: 1.0, g: 0.65, b: 0.0, a: 1.0 }),
            "purple" => Ok(FillStyle::Color { r: 0.5, g: 0.0, b: 0.5, a: 1.0 }),
            "brown" => Ok(FillStyle::Color { r: 0.65, g: 0.16, b: 0.16, a: 1.0 }),
            "pink" => Ok(FillStyle::Color { r: 1.0, g: 0.75, b: 0.8, a: 1.0 }),
            "gray" | "grey" => Ok(FillStyle::Color { r: 0.5, g: 0.5, b: 0.5, a: 1.0 }),
            "silver" => Ok(FillStyle::Color { r: 0.75, g: 0.75, b: 0.75, a: 1.0 }),
            _ => Err(format!("Unknown color name: {}", name)),
        }
    }
    
    /// Get the RGBA color values for rendering
    pub fn get_rgba(&self) -> (f32, f32, f32, f32) {
        match self {
            FillStyle::Color { r, g, b, a } => (*r, *g, *b, *a),
            _ => (0.0, 0.0, 0.0, 1.0), // Default to black for unsupported types
        }
    }
}

/// A Canvas extension
#[derive(Clone)]
struct CanvasData<'gc> {
    width: u32,
    height: u32,
    commands: Vec<context2d::CanvasCommand<'gc>>,
    fill_style: FillStyle,
}

struct CanvasResources<'gc> {
    canvases: ResourceTable<CanvasData<'gc>>,
    images: ResourceTable<ImageData>,
    renderers: ResourceTable<renderer::Renderer>,
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
                ExtensionOp::new("internal_canvas_get_fill_style", Self::internal_canvas_get_fill_style, 1),
                ExtensionOp::new("internal_canvas_set_fill_style", Self::internal_canvas_set_fill_style, 2),
                ExtensionOp::new("internal_canvas_render", Self::internal_canvas_render, 1),
                ExtensionOp::new("internal_canvas_save_as_png", Self::internal_canvas_save_as_png, 2),
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
            ],
            storage: Some(Box::new(|storage: &mut OpsStorage| {
                storage.insert(CanvasResources {
                    canvases: ResourceTable::new(),
                    images: ResourceTable::new(),
                    renderers: ResourceTable::new(),
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
        let res: &mut CanvasResources = storage.get_mut().unwrap();
        
        // Create canvas data
        let canvas_rid = res.canvases.push(CanvasData {
            width,
            height,
            commands: Vec::new(),
            fill_style: FillStyle::default(),
        });
        
        // Create renderer with GPU device
        let (device, queue) = create_wgpu_device_sync();
        let dimensions = renderer::Dimensions { width, height };
        let format = wgpu::TextureFormat::Bgra8UnormSrgb; // Common format for canvas
        let renderer = renderer::Renderer::new(device, queue, format, dimensions);
        let _renderer_rid = res.renderers.push(renderer);
        
        Ok(Value::Integer(SmallInteger::from(canvas_rid.index() as i32)))
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
        let _path = binding.as_str(agent);
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
        let path = path_str.as_str(agent);
        
        let host_data = agent
            .get_host_data()
            .downcast_ref::<HostData<crate::RuntimeMacroTask>>()
            .unwrap();
        let mut storage = host_data.storage.borrow_mut();
        let res: &mut CanvasResources = storage.get_mut().unwrap();
        
        // Try to save with GPU renderer if available
        if let Some(mut renderer) = res.renderers.get_mut(rid) {
            // Since we can't use async in this context, we'll use a blocking approach
            let path_owned = path.to_owned();
            
            // First render all pending operations
            renderer.render_all();
            
            // Use tokio to handle the async save operation
            let rt = match tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build() {
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
        
        // Convert fill style back to CSS string representation
        let css_string = match &data.fill_style {
            FillStyle::Color { r, g, b, a } => {
                if *a == 1.0 {
                    // RGB format for opaque colors
                    format!("rgb({}, {}, {})", 
                        (*r * 255.0) as u8, 
                        (*g * 255.0) as u8, 
                        (*b * 255.0) as u8)
                } else {
                    // RGBA format for transparent colors
                    format!("rgba({}, {}, {}, {})", 
                        (*r * 255.0) as u8, 
                        (*g * 255.0) as u8, 
                        (*b * 255.0) as u8, 
                        a)
                }
            }
            _ => "rgb(0, 0, 0)".to_string(), // Default fallback
        };
        
        // Drop storage borrow before creating string
        drop(storage);
        
        Ok(Value::from_string(agent, css_string, gc.nogc()).unbind())
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
        
        // Convert the JavaScript value to a string
        let style_str = style_val.to_string(agent, gc.reborrow()).unbind().unwrap();
        let style_string = style_str.as_str(agent);
        
        let host_data = agent
            .get_host_data()
            .downcast_ref::<HostData<crate::RuntimeMacroTask>>()
            .unwrap();
        let mut storage = host_data.storage.borrow_mut();
        let res: &mut CanvasResources = storage.get_mut().unwrap();
        let mut data = res.canvases.get_mut(rid).unwrap();
        
        // Parse the CSS color and update fill style
        match FillStyle::from_css_color(style_string) {
            Ok(fill_style) => {
                data.fill_style = fill_style;
                Ok(Value::Boolean(true))
            }
            Err(_) => {
                // Invalid color, keep current style
                Ok(Value::Boolean(false))
            }
        }
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
        .request_device(
            &wgpu::DeviceDescriptor {
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                label: None,
                memory_hints: Default::default(),
                trace: wgpu::Trace::default(),
            },
        )
        .await
        .unwrap();
    
    (device, queue)
}

fn create_wgpu_device_sync() -> (wgpu::Device, wgpu::Queue) {
    // Use a simple blocking executor - we'll create a simpler version for now
    // TODO: Replace with proper async runtime integration
    use std::sync::{Arc, Mutex};
    use std::thread;
    
    let result = Arc::new(Mutex::new(None));
    let result_clone = result.clone();
    
    thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let device_queue = rt.block_on(create_wgpu_device());
        *result_clone.lock().unwrap() = Some(device_queue);
    }).join().unwrap();
    
    result.lock().unwrap().take().unwrap()
}
