// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use andromeda_core::{AndromedaError, ErrorReporter, Extension, HostData};
use nova_vm::{
    ecmascript::execution::agent::{GcAgent, RealmRoot},
    ecmascript::types::{Function, Value},
};

use crate::{ConsoleExt, FsExt, HeadersExt, ProcessExt, RuntimeMacroTask, TimeExt, URLExt, WebExt};

pub fn recommended_extensions() -> Vec<Extension> {
    vec![
        FsExt::new_extension(),
        ConsoleExt::new_extension(),
        TimeExt::new_extension(),
        ProcessExt::new_extension(),
        URLExt::new_extension(),
        WebExt::new_extension(),
        HeadersExt::new_extension(),
        #[cfg(feature = "canvas")]
        crate::CanvasExt::new_extension(),
        #[cfg(feature = "crypto")]
        crate::CryptoExt::new_extension(),
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
        RuntimeMacroTask::RunMicrotaskCallback(root_callback) => {
            // Execute the microtask callback - this is for queueMicrotask implementation
            agent.run_in_realm(realm_root, |agent, gc| {
                let callback = root_callback.get(agent, gc.nogc());
                if let Ok(callback_function) = Function::try_from(callback) {
                    // Call the callback with no arguments as per HTML spec
                    // If the callback throws an error, it should be reported but not stop execution
                    if let Err(error) = callback_function.call(agent, Value::Undefined, &mut [], gc)
                    {
                        // Report the error as per Web IDL "invoke a callback function" with "report" error handling
                        // This is the proper implementation of error reporting for queueMicrotask
                        report_microtask_error(error);
                    }
                }
            });
        }
    }
}

/// Report a microtask error
/// This is called when a queueMicrotask callback throws an exception.
fn report_microtask_error<E>(error: E)
where
    E: std::fmt::Debug,
{
    let error_message = format!("Uncaught error in microtask callback: {:?}", error);
    let andromeda_error = AndromedaError::runtime_error(error_message);
    ErrorReporter::print_error(&andromeda_error);
}
