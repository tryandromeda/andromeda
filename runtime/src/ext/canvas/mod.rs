// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use andromeda_core::{Extension, ExtensionOp, HostData, OpsStorage, ResourceTable, Rid};
mod context2d;
use self::context2d::{internal_canvas_fill_rect, internal_canvas_clear_rect};
use nova_vm::{
    SmallInteger,
    ecmascript::{
        builtins::ArgumentsList,
        execution::{Agent, JsResult},
        types::{Number, Value},
    },
    engine::context::{Bindable, GcScope},
};

/// A command to be executed on the canvas
#[derive(Clone)]
#[allow(dead_code)]
enum CanvasCommand<'gc> {
    FillRect {
        x: Number<'gc>,
        y: Number<'gc>,
        width: Number<'gc>,
        height: Number<'gc>,
    },
    ClearRect {
        x: Number<'gc>,
        y: Number<'gc>,
        width: Number<'gc>,
        height: Number<'gc>,
    },
}

/// A Canvas extension
#[derive(Clone)]
struct CanvasData<'gc> {
    width: u32,
    height: u32,
    commands: Vec<CanvasCommand<'gc>>,
}

struct CanvasResources<'gc> {
    canvases: ResourceTable<CanvasData<'gc>>,
}
#[derive(Default)]
pub struct CanvasExt;

impl CanvasExt {
    pub fn new_extension() -> Extension {
        Extension {
            name: "canvas",
            ops: vec![
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
                ExtensionOp::new("internal_canvas_fill_rect", internal_canvas_fill_rect, 5),
                ExtensionOp::new("internal_canvas_clear_rect", internal_canvas_clear_rect, 5),
            ],
            storage: Some(Box::new(|storage: &mut OpsStorage| {
                storage.insert(CanvasResources {
                    canvases: ResourceTable::new(),
                });
            })),
            files: vec![include_str!("./mod.ts")],
        }
    }
    fn internal_canvas_create<'a, 'b, 'c, 'd, 'e, 'gc>(
        agent: &'a mut Agent,
        _this: Value<'b>,
        args: ArgumentsList<'c, 'd>,
        mut gc: GcScope<'gc, 'e>,
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
    fn internal_canvas_get_width<'a, 'b, 'c, 'd, 'e, 'gc>(
        agent: &'a mut Agent,
        _this: Value<'b>,
        args: ArgumentsList<'c, 'd>,
        mut gc: GcScope<'gc, 'e>,
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
    fn internal_canvas_get_height<'a, 'b, 'c, 'd, 'e, 'gc>(
        agent: &'a mut Agent,
        _this: Value<'b>,
        args: ArgumentsList<'c, 'd>,
        mut gc: GcScope<'gc, 'e>,
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
}
