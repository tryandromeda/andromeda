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
    BroadcastChannelExt, ConsoleExt, CronExt, FetchExt, FfiExt, FileExt, NetExt, ProcessExt,
    RuntimeMacroTask, ServeExt, StreamsExt, TimeExt, TlsExt, URLExt, WebExt, WebLocksExt,
};

#[cfg(not(feature = "virtualfs"))]
use crate::FsExt;
#[cfg(feature = "virtualfs")]
use crate::VirtualFsExt;

#[cfg_attr(feature = "hotpath", hotpath::measure)]
pub fn recommended_extensions() -> Vec<Extension> {
    vec![
        #[cfg(not(feature = "virtualfs"))]
        FsExt::new_extension(),
        #[cfg(feature = "virtualfs")]
        VirtualFsExt::new_extension(),
        ConsoleExt::new_extension(),
        TimeExt::new_extension(),
        CronExt::new_extension(),
        ProcessExt::new_extension(),
        URLExt::new_extension(),
        WebExt::new_extension(),
        WebLocksExt::new_extension(),
        FileExt::new_extension(),
        BroadcastChannelExt::new_extension(),
        FetchExt::new_extension(),
        NetExt::new_extension(),
        StreamsExt::new_extension(),
        TlsExt::new_extension(),
        FfiExt::new_extension(),
        #[cfg(feature = "serve")]
        ServeExt::new_extension(),
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

#[cfg_attr(feature = "hotpath", hotpath::measure)]
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
        RuntimeMacroTask::ResolvePromiseWithBytes(root_value, bytes_value) => {
            let hex_string_global = agent
                .run_in_realm(realm_root, |agent, gc| {
                    let mut hex_content = String::new();
                    for b in &bytes_value {
                        use std::fmt::Write;
                        write!(&mut hex_content, "{b:02x}").unwrap();
                    }
                    let string_val = Value::from_string(agent, hex_content, gc.nogc());
                    Some(Global::new(agent, string_val.into_value().unbind()))
                })
                .unwrap();

            agent.run_in_realm(realm_root, |agent, gc| {
                let promise_value = root_value.take(agent);
                let string_value = hex_string_global.take(agent);
                if let Value::Promise(promise) = promise_value {
                    let promise_capability = PromiseCapability::from_promise(promise, false);
                    promise_capability.resolve(agent, string_value, gc);
                } else {
                    panic!("Attempted to resolve a non-promise value");
                }
            });
        }
        RuntimeMacroTask::RegisterTlsStream(root_value, tls_stream) => {
            // Insert the tls stream into the runtime storage resources and resolve the promise
            let rid = {
                let storage = host_data.storage.borrow();
                let resources: &crate::ext::tls::TlsResources = storage.get().unwrap();
                // Wrap tls_stream into Arc<Mutex<..>> so it can be shared across tasks
                // tls_stream is boxed in the macro task; move the boxed stream out and store in Arc/Mutex
                let boxed = std::sync::Arc::new(tokio::sync::Mutex::new(*tls_stream));
                resources
                    .streams
                    .push(crate::ext::tls::TlsResource::Client(boxed))
            };

            // Resolve the original promise with the numeric rid as string. Pre-create the string value
            // in a separate realm call to avoid borrow conflicts.
            let string_global = agent
                .run_in_realm(realm_root, |agent, gc| {
                    let rid_str = rid.index().to_string();
                    let string_val = Value::from_string(agent, rid_str, gc.nogc());
                    Some(Global::new(agent, string_val.into_value().unbind()))
                })
                .unwrap();

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
        RuntimeMacroTask::AcquireLock {
            promise,
            lock_id,
            name,
            mode,
        } => {
            // Handle lock acquisition - resolve the promise with the lock result
            // First, create the result string in a separate realm call
            let lock_info = format!(
                "{{\"lockId\":{},\"name\":\"{}\",\"mode\":\"{}\"}}",
                lock_id, name, mode
            );
            let string_global = agent
                .run_in_realm(realm_root, |agent, gc| {
                    let string_val = Value::from_string(agent, lock_info, gc.nogc());
                    Some(Global::new(agent, string_val.into_value().unbind()))
                })
                .unwrap();

            // Then resolve the promise with the pre-created string
            agent.run_in_realm(realm_root, |agent, gc| {
                let promise_value = promise.take(agent);
                let string_value = string_global.take(agent);
                if let Value::Promise(promise_obj) = promise_value {
                    let promise_capability = PromiseCapability::from_promise(promise_obj, false);
                    promise_capability.resolve(agent, string_value, gc);
                } else {
                    panic!("Attempted to resolve a non-promise value in AcquireLock");
                }
            });
        }
        RuntimeMacroTask::ReleaseLock { name, lock_id } => {
            // Handle lock release - trigger processing of pending requests
            // This would typically wake up any waiting tasks
            println!("Released lock {} for '{}'", lock_id, name);
            // TODO: Implement actual lock release logic that wakes pending tasks
        }
        RuntimeMacroTask::AbortLockRequest { name, lock_id } => {
            // Handle lock request abortion
            println!("Aborted lock request {} for '{}'", lock_id, name);
            // TODO: Implement actual abort logic that cancels the associated promise
        }
    }
}
