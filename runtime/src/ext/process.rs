// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use andromeda_core::{AndromedaError, ErrorReporter, Extension, ExtensionOp, HostData, OpsStorage};
use nova_vm::{
    ecmascript::{
        builtins::{ArgumentsList, Array},
        execution::{Agent, JsResult},
        types::{IntoValue, Value},
    },
    engine::context::{Bindable, GcScope},
};
use std::{collections::HashMap, env};
use tokio::task::JoinHandle;

#[cfg(unix)]
use signal_hook::consts::*;
#[cfg(windows)]
use signal_hook::consts::{SIGBREAK, SIGINT};

use crate::RuntimeMacroTask;

#[derive(Default)]
struct ProcessExtResources {
    handlers: HashMap<i32, JoinHandle<()>>,
}

/// Process extension for Andromeda.
/// This extension provides access to internal functions relating to the process.
#[derive(Default)]
pub struct ProcessExt;

#[cfg_attr(feature = "hotpath", hotpath::measure_all)]
impl ProcessExt {
    pub fn new_extension() -> Extension {
        Extension {
            name: "process",
            ops: vec![
                ExtensionOp::new(
                    "internal_get_cli_args",
                    Self::internal_get_cli_args,
                    0,
                    false,
                ),
                ExtensionOp::new("internal_get_env", Self::internal_get_env, 1, false),
                ExtensionOp::new("internal_set_env", Self::internal_set_env, 2, false),
                ExtensionOp::new("internal_delete_env", Self::internal_delete_env, 1, false),
                ExtensionOp::new(
                    "internal_get_env_keys",
                    Self::internal_get_env_keys,
                    0,
                    false,
                ),
                ExtensionOp::new(
                    "internal_add_signal_listener",
                    Self::internal_add_signal_listener,
                    2,
                    false,
                ),
                ExtensionOp::new(
                    "internal_remove_signal_listener",
                    Self::internal_remove_signal_listener,
                    2,
                    false,
                ),
            ],
            storage: Some(Box::new(|storage: &mut OpsStorage| {
                storage.insert(ProcessExtResources::default());
            })),
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
        let key_str = key.as_str(agent).expect("String is not valid UTF-8");

        match env::var(key_str) {
            Ok(value) => {
                Ok(
                    nova_vm::ecmascript::types::String::from_string(agent, value, gc.nogc())
                        .unbind()
                        .into(),
                )
            }
            Err(env::VarError::NotPresent) => Ok(Value::Undefined),
            Err(env::VarError::NotUnicode(_)) => {
                let error = AndromedaError::encoding_error(
                    "UTF-8",
                    format!("Environment variable '{key_str}' contains invalid Unicode"),
                );
                let error_msg = ErrorReporter::format_error(&error);
                Ok(nova_vm::ecmascript::types::String::from_string(
                    agent,
                    format!("Error: {error_msg}"),
                    gc.nogc(),
                )
                .unbind()
                .into())
            }
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
            env::set_var(
                key.as_str(agent).expect("String is not valid UTF-8"),
                value.as_str(agent).expect("String is not valid UTF-8"),
            );
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
            env::remove_var(key.as_str(agent).expect("String is not valid UTF-8"));
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
    fn internal_add_signal_listener<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let signal_name = args.get(0);
        let signal_name = signal_name.to_string(agent, gc.reborrow()).unbind()?;
        let signal_name_str = signal_name
            .as_str(agent)
            .expect("String is not valid UTF-8");
        let callback = args.get(1);
        if !callback.is_function() {
            let error = AndromedaError::runtime_error("Callback must be a function");
            let error_msg = ErrorReporter::format_error(&error);
            return Ok(
                Value::from_string(agent, format!("Error: {error_msg}"), gc.nogc()).unbind(),
            );
        }

        let signal_num = match signal_name_str {
            #[cfg(unix)]
            "SIGTERM" => SIGTERM,
            #[cfg(unix)]
            "SIGINT" => SIGINT,
            #[cfg(unix)]
            "SIGHUP" => SIGHUP,
            #[cfg(unix)]
            "SIGQUIT" => SIGQUIT,
            #[cfg(unix)]
            "SIGUSR1" => SIGUSR1,
            #[cfg(unix)]
            "SIGUSR2" => SIGUSR2,
            #[cfg(windows)]
            "SIGINT" => SIGINT,
            #[cfg(windows)]
            "SIGBREAK" => SIGBREAK,
            _ => {
                #[cfg(windows)]
                {
                    let error_msg = format!(
                        "Signal '{signal_name_str}' is not supported on Windows. Only SIGINT and SIGBREAK are supported."
                    );
                    let error = AndromedaError::runtime_error(error_msg);
                    let error_formatted = ErrorReporter::format_error(&error);
                    return Ok(Value::from_string(
                        agent,
                        format!("Error: {error_formatted}"),
                        gc.nogc(),
                    )
                    .unbind());
                }
                #[cfg(unix)]
                {
                    let error_msg = format!("Unsupported signal: {signal_name_str}");
                    let error = AndromedaError::runtime_error(error_msg);
                    let error_formatted = ErrorReporter::format_error(&error);
                    return Ok(Value::from_string(
                        agent,
                        format!("Error: {error_formatted}"),
                        gc.nogc(),
                    )
                    .unbind());
                }
            }
        };
        let host_data = agent.get_host_data();
        let host_data: &HostData<RuntimeMacroTask> = host_data.downcast_ref().unwrap();
        let mut storage = host_data.storage.borrow_mut();
        let resources: &mut ProcessExtResources = storage.get_mut().unwrap();

        if let Some(handle) = resources.handlers.remove(&signal_num) {
            handle.abort();
        }
        let handle = tokio::task::spawn_blocking(move || {
            #[cfg(unix)]
            {
                use signal_hook::iterator::Signals;
                if let Ok(mut signals) = Signals::new([signal_num]) {
                    for _signal in signals.forever() {
                        eprintln!("Signal {signal_num} received");
                        // TODO: Dispatch to JavaScript event loop
                    }
                }
            }
            #[cfg(windows)]
            {
                eprintln!("Signal handler registered for signal {signal_num}");
            }
        });
        resources.handlers.insert(signal_num, handle);

        Ok(Value::Undefined)
    }

    fn internal_remove_signal_listener<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let signal_name = args.get(0);
        let signal_name = signal_name.to_string(agent, gc.reborrow()).unbind()?;
        let signal_name_str = signal_name
            .as_str(agent)
            .expect("String is not valid UTF-8");

        let signal_num = match signal_name_str {
            #[cfg(unix)]
            "SIGTERM" => SIGTERM,
            #[cfg(unix)]
            "SIGINT" => SIGINT,
            #[cfg(unix)]
            "SIGHUP" => SIGHUP,
            #[cfg(unix)]
            "SIGQUIT" => SIGQUIT,
            #[cfg(unix)]
            "SIGUSR1" => SIGUSR1,
            #[cfg(unix)]
            "SIGUSR2" => SIGUSR2,
            #[cfg(windows)]
            "SIGINT" => SIGINT,
            #[cfg(windows)]
            "SIGBREAK" => SIGBREAK,
            _ => {
                return Ok(Value::Undefined);
            }
        };

        let host_data = agent.get_host_data();
        let host_data: &HostData<RuntimeMacroTask> = host_data.downcast_ref().unwrap();
        let mut storage = host_data.storage.borrow_mut();
        let resources: &mut ProcessExtResources = storage.get_mut().unwrap();
        if let Some(handle) = resources.handlers.remove(&signal_num) {
            handle.abort();
        }

        Ok(Value::Undefined)
    }
}
