// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use andromeda_core::{HostData};
use nova_vm::{
    ecmascript::{execution::Agent, types::Value},
    engine::context::GcScope,
};
use crate::RuntimeMacroTask;

/// A 2D rendering context for Canvas
pub struct CanvasRenderingContext2D {
    rid: u32,
}

impl CanvasRenderingContext2D {
    pub fn new(rid: u32) -> Self {
        Self { rid }
    }

    pub fn fill_rect<'gc>(
        &self,
        agent: &mut Agent,
        _this: Value,
        args: super::ArgumentsList,
        _gc: GcScope<'gc, '_>,
    ) -> super::JsResult<'gc, Value<'gc>> {
        let x = args.get(0).to_number(agent, _gc.reborrow()).unwrap();
        let y = args.get(1).to_number(agent, _gc.reborrow()).unwrap();
        let width = args.get(2).to_number(agent, _gc.reborrow()).unwrap();
        let height = args.get(3).to_number(agent, _gc.reborrow()).unwrap();
        internal_canvas_fill_rect(self.rid, x, y, width, height);
        Ok(Value::Undefined)
    }

    pub fn clear_rect<'gc>(
        &self,
        agent: &mut Agent,
        _this: Value,
        args: super::ArgumentsList,
        _gc: GcScope<'gc, '_>,
    ) -> super::JsResult<'gc, Value<'gc>> {
        let x = args.get(0).to_number(agent, _gc.reborrow()).unwrap();
        let y = args.get(1).to_number(agent, _gc.reborrow()).unwrap();
        let width = args.get(2).to_number(agent, _gc.reborrow()).unwrap();
        let height = args.get(3).to_number(agent, _gc.reborrow()).unwrap();
        internal_canvas_clear_rect(self.rid, x, y, width, height);
        Ok(Value::Undefined)
    }
}
