// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use andromeda_core::{Extension, HostData};
use nova_vm::{
    ecmascript::{
        builtins::promise_objects::promise_abstract_operations::promise_capability_records::PromiseCapability,
        execution::agent::{GcAgent, RealmRoot},
        types::{IntoValue, String as NovaString, Value},
    },
    engine::{Global, context::Bindable},
};

use crate::{
    BroadcastChannelExt, ConsoleExt, CronExt, FetchExt, FileExt, FsExt, HeadersExt, ProcessExt,
    RequestExt, ResponseExt, RuntimeMacroTask, StreamsExt, TimeExt, URLExt, WebExt,
};

pub fn recommended_extensions() -> Vec<Extension> {
    vec![
        FsExt::new_extension(),
        ConsoleExt::new_extension(),
        TimeExt::new_extension(),
        CronExt::new_extension(),
        ProcessExt::new_extension(),
        URLExt::new_extension(),
        WebExt::new_extension(),
        FileExt::new_extension(),
        HeadersExt::new_extension(),
        BroadcastChannelExt::new_extension(),
        RequestExt::new_extension(),
        ResponseExt::new_extension(),
        FetchExt::new_extension(),
        StreamsExt::new_extension(),
        #[cfg(feature = "canvas")]
        crate::CanvasExt::new_extension(),
        #[cfg(feature = "crypto")]
        crate::CryptoExt::new_extension(),
        #[cfg(feature = "storage")]
        crate::LocalStorageExt::new_extension(),
        #[cfg(feature = "storage")]
        crate::SqliteExt::new_extension(),
        #[cfg(feature = "storage")]
        crate::CacheStorageExt::new_extension(),
    ]
}

pub fn recommended_builtins() -> Vec<&'static str> {
    vec![include_str!("../../namespace/mod.ts")]
}

pub fn recommended_eventloop_handler(
    macro_task: RuntimeMacroTask,
    agent: &mut GcAgent,
    realm_root: &RealmRoot,
    host_data: &HostData<RuntimeMacroTask>,
) {
    match macro_task {
        RuntimeMacroTask::RunInterval(interval_id) => interval_id.run(agent, host_data, realm_root),
        RuntimeMacroTask::ClearInterval(interval_id) => {
            interval_id.clear_and_abort(host_data);
        }
        RuntimeMacroTask::RunAndClearTimeout(timeout_id) => {
            timeout_id.run_and_clear(agent, host_data, realm_root)
        }
        RuntimeMacroTask::ClearTimeout(timeout_id) => {
            timeout_id.clear_and_abort(host_data);
        }
        RuntimeMacroTask::RunCron(cron_id) => cron_id.run(agent, host_data, realm_root),
        RuntimeMacroTask::ClearCron(cron_id) => {
            cron_id.clear_and_abort(host_data);
        }
        RuntimeMacroTask::ResolvePromiseWithValue(promise_root, value_root) => {
            agent.run_in_realm(realm_root, |agent, mut gc| {
                let promise_value = promise_root.take(agent);
                let resolve_value = value_root.take(agent);
                if let Value::Promise(promise) = promise_value {
                    let promise_capability = PromiseCapability::from_promise(promise, false);
                    promise_capability.resolve(agent, resolve_value, gc.reborrow());
                } else {
                    panic!("Attempted to resolve a non-promise value");
                }
            });
        }
        RuntimeMacroTask::RejectPromise(root_value, error_message) => {
            agent.run_in_realm(realm_root, |agent, gc| {
                let value = root_value.take(agent);
                if let Value::Promise(promise) = value {
                    let promise_capability = PromiseCapability::from_promise(promise, false);
                    let error_val = NovaString::from_str(agent, &error_message, gc.nogc());
                    promise_capability.reject(agent, error_val.into_value(), gc.nogc());
                } else {
                    panic!("Attempted to reject a non-promise value");
                }
            });
        }
        RuntimeMacroTask::ResolvePromiseWithString(root_value, string_value) => {
            // First, create the string value in a separate realm call
            let string_global = agent
                .run_in_realm(realm_root, |agent, gc| {
                    let string_val = Value::from_string(agent, string_value, gc.nogc());
                    Some(Global::new(agent, string_val.into_value().unbind()))
                })
                .unwrap();

            // Then resolve the promise with the pre-created string
            agent.run_in_realm(realm_root, |agent, gc| {
                let promise_value = root_value.take(agent);
                let string_value = string_global.take(agent);
                if let Value::Promise(promise) = promise_value {
                    let promise_capability = PromiseCapability::from_promise(promise, false);
                    promise_capability.resolve(agent, string_value, gc);
                } else {
                    panic!("Attempted to resolve a non-promise value");
                }
            });
        }
        RuntimeMacroTask::ResolvePromiseWithBytes(root_value, _bytes_value) => {
            agent.run_in_realm(realm_root, |agent, gc| {
                let value = root_value.take(agent);
                if let Value::Promise(promise) = value {
                    let promise_capability = PromiseCapability::from_promise(promise, false);
                    // TODO: Implement bytes to Uint8Array conversion
                    let undefined_val = Value::Undefined;
                    promise_capability.resolve(agent, undefined_val, gc);
                } else {
                    panic!("Attempted to resolve a non-promise value");
                }
            });
        }
    }
}
