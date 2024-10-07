use andromeda_core::{Extension, HostData};
use nova_vm::ecmascript::execution::agent::{GcAgent, RealmRoot};

use crate::{ConsoleExt, FsExt, ProcessExt, RuntimeMacroTask, TimeExt, URLExt};

pub fn recommended_extensions() -> Vec<Extension> {
    vec![
        FsExt::new_extension(),
        ConsoleExt::new_extension(),
        TimeExt::new_extension(),
        ProcessExt::new_extension(),
        URLExt::new_extension(),
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
