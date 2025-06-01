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

/// A Canvas extension
#[derive(Clone)]
struct CanvasData<'gc> {
    width: u32,
    height: u32,
    commands: Vec<context2d::CanvasCommand<'gc>>,
}

struct CanvasResources<'gc> {
    canvases: ResourceTable<CanvasData<'gc>>,
    images: ResourceTable<ImageData>,
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
        let rid = res.canvases.push(CanvasData {
            width,
            height,
            commands: Vec::new(),
        });
        Ok(Value::Integer(SmallInteger::from(rid.index() as i32)))
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
}
