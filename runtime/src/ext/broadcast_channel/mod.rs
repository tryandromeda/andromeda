// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use andromeda_core::{Extension, ExtensionOp};
use nova_vm::{
    ecmascript::{
        builtins::ArgumentsList,
        execution::{Agent, JsResult},
        types::Value,
    },
    engine::context::{Bindable, GcScope},
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};

// Global storage for broadcast channels
lazy_static::lazy_static! {
    static ref BROADCAST_CHANNELS: Arc<RwLock<HashMap<String, Vec<BroadcastChannelHandle>>>> =
        Arc::new(RwLock::new(HashMap::new()));
    static ref NEXT_RID: Arc<Mutex<u32>> = Arc::new(Mutex::new(1));
}

#[derive(Clone)]
struct BroadcastChannelHandle {
    rid: u32,
}

#[derive(Default)]
pub struct BroadcastChannelExt;

impl BroadcastChannelExt {
    #[cfg_attr(feature = "hotpath", hotpath::measure)]
    pub fn new_extension() -> Extension {
        Extension {
            name: "broadcast_channel",
            ops: vec![
                ExtensionOp::new(
                    "op_broadcast_subscribe",
                    Self::op_broadcast_subscribe,
                    0,
                    false,
                ),
                ExtensionOp::new(
                    "op_broadcast_unsubscribe",
                    Self::op_broadcast_unsubscribe,
                    1,
                    false,
                ),
                ExtensionOp::new("op_broadcast_send", Self::op_broadcast_send, 3, false),
                ExtensionOp::new("op_broadcast_recv", Self::op_broadcast_recv, 1, false),
            ],
            storage: None,
            files: vec![include_str!("./broadcast_channel.ts")],
        }
    }
    /// Subscribe to broadcast channel and return a resource ID
    pub fn op_broadcast_subscribe<'gc>(
        agent: &mut Agent,
        _this: Value,
        _args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let mut next_rid = NEXT_RID.lock().unwrap();
        let rid = *next_rid;
        *next_rid += 1;
        drop(next_rid);

        Ok(Value::from_f64(agent, rid as f64, gc.nogc()).unbind())
    }

    /// Unsubscribe from broadcast channel
    pub fn op_broadcast_unsubscribe<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let rid_arg = args.get(0);
        let rid = rid_arg.to_number(agent, gc.reborrow()).unbind()?;
        let rid = rid.into_f64(agent) as u32;

        // Remove this RID from all channels
        let mut channels = BROADCAST_CHANNELS.write().unwrap();
        for (_name, handles) in channels.iter_mut() {
            handles.retain(|handle| handle.rid != rid);
        }

        Ok(Value::Undefined)
    }

    /// Send a message to a broadcast channel
    pub fn op_broadcast_send<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let rid_arg = args.get(0);
        let name_arg = args.get(1);
        let _data_arg = args.get(2);

        let rid = rid_arg.to_number(agent, gc.reborrow()).unbind()?;
        let _rid = rid.into_f64(agent) as u32;

        let name = name_arg.to_string(agent, gc.reborrow()).unbind()?;
        let _name_str = name
            .as_str(agent)
            .expect("String is not valid UTF-8")
            .to_string();

        // For now, this is a placeholder implementation
        // In a full implementation, this would serialize and send the message
        // to other processes/workers

        Ok(Value::Undefined)
    }
    /// Receive a message from broadcast channel (async)
    pub fn op_broadcast_recv<'gc>(
        _agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        _gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let _rid_arg = args.get(0);

        // This is a placeholder for async message receiving
        // In a full implementation, this would be an async operation
        // that waits for messages and returns them
        Ok(Value::Undefined)
    }
}
