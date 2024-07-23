use nova_vm::ecmascript::{execution::Agent, types::Object};

use crate::{AgentExtLoader, ConsolExt, FsExt};

pub fn initialize_recommended_extensions(agent: &mut Agent, global_object: Object) {
    agent.load_ext(global_object, FsExt);
    agent.load_ext(global_object, ConsolExt);
}
