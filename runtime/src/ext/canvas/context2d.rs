// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use super::Rid;
use super::{CanvasCommand, CanvasResources};
use crate::RuntimeMacroTask;
use andromeda_core::HostData;
use nova_vm::{
    ecmascript::{
        builtins::ArgumentsList,
        execution::{Agent, JsResult},
        types::Value,
    },
    engine::context::{Bindable, GcScope},
};

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
    let mut data = res.canvases.get_mut(rid).unwrap();
    data.commands.push(CanvasCommand::FillRect {
        x,
        y,
        width,
        height,
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
    let mut data = res.canvases.get_mut(rid).unwrap();
    data.commands.push(CanvasCommand::ClearRect {
        x,
        y,
        width,
        height,
    });
    Ok(Value::Undefined)
}
