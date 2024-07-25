use nova_vm::ecmascript::{execution::Agent, types::Object};

use crate::{AgentExtLoader, ConsoleExt, FsExt, TimeExt};

pub fn initialize_recommended_extensions(agent: &mut Agent, global_object: Object) {
    agent.load_ext(global_object, FsExt);
    agent.load_ext(global_object, ConsoleExt);
    agent.load_ext(global_object, TimeExt);
}
