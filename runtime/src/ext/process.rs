// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use andromeda_core::{Extension, ExtensionOp};
use nova_vm::{
    ecmascript::{
        builtins::{ArgumentsList, Array},
        execution::{Agent, JsResult},
        types::{IntoValue, Value},
    },
    engine::context::{Bindable, GcScope},
};
use std::env;

/// Process extension for Andromeda.
/// This extension provides access to internal functions relating to the process.
#[derive(Default)]
pub struct ProcessExt;

impl ProcessExt {
    pub fn new_extension() -> Extension {
        Extension {
            name: "process",
            ops: vec![
                ExtensionOp::new("internal_get_cli_args", Self::internal_get_cli_args, 0),
                ExtensionOp::new("internal_get_env", Self::internal_get_env, 1),
                ExtensionOp::new("internal_set_env", Self::internal_set_env, 2),
                ExtensionOp::new("internal_delete_env", Self::internal_delete_env, 1),
                ExtensionOp::new("internal_get_env_keys", Self::internal_get_env_keys, 0),
            ],
            storage: None,
            files: vec![],
        }
    }

    fn internal_get_cli_args<'gc>(
        agent: &mut Agent,
        _this: Value,
        _: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let args = env::args().skip(1).collect::<Vec<String>>();
        let args = args.iter().map(|s| s.as_str()).collect::<Vec<&str>>();
        let args = args
            .iter()
            .map(|s| {
                nova_vm::ecmascript::types::String::from_string(agent, s.to_string(), gc.nogc())
                    .into_value()
            })
            .collect::<Vec<_>>();

        Ok(Array::from_slice(agent, args.as_slice(), gc.nogc())
            .unbind()
            .into())
    }

    fn internal_get_env<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let key = args.get(0);
        let key = key.to_string(agent, gc.reborrow()).unbind()?;
        match env::var(key.as_str(agent)) {
            Ok(value) => {
                Ok(
                    nova_vm::ecmascript::types::String::from_string(agent, value, gc.nogc())
                        .unbind()
                        .into(),
                )
            }
            _ => Ok(Value::Undefined),
        }
    }

    fn internal_set_env<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let key = args.get(0);
        let key = key.to_string(agent, gc.reborrow()).unbind()?;

        let value = args.get(1);
        let value = value.to_string(agent, gc.reborrow()).unbind().unbind()?;

        unsafe {
            env::set_var(key.as_str(agent), value.as_str(agent));
        }

        Ok(Value::Undefined)
    }

    fn internal_delete_env<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let key = args.get(0);
        let key = key.to_string(agent, gc.reborrow()).unbind()?;

        unsafe {
            env::remove_var(key.as_str(agent));
        }

        Ok(Value::Undefined)
    }

    fn internal_get_env_keys<'gc>(
        agent: &mut Agent,
        _this: Value,
        _: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let keys = env::vars()
            .map(|(k, _)| k)
            .map(|s| {
                nova_vm::ecmascript::types::String::from_string(agent, s, gc.nogc()).into_value()
            })
            .collect::<Vec<_>>();

        Ok(Array::from_slice(agent, keys.as_slice(), gc.nogc())
            .unbind()
            .into())
    }
}
