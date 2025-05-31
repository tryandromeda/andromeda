// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use andromeda_core::{Extension, HostData};
use nova_vm::ecmascript::execution::agent::{GcAgent, RealmRoot};

use crate::{
    CanvasExt, ConsoleExt, FsExt, HeadersExt, ProcessExt, RuntimeMacroTask, TimeExt, URLExt, WebExt,
};

pub fn recommended_extensions() -> Vec<Extension> {
    vec![
        FsExt::new_extension(),
        ConsoleExt::new_extension(),
        TimeExt::new_extension(),
        ProcessExt::new_extension(),
        URLExt::new_extension(),
        WebExt::new_extension(),
        HeadersExt::new_extension(),
        CanvasExt::new_extension(),
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
    }
}
