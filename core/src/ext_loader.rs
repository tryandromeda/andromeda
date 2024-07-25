use nova_vm::ecmascript::{execution::Agent, types::Object};

use crate::ext_interface::{Ext, ExtLoader};

pub trait AgentExtLoader {
    fn load_ext(&mut self, global_object: Object, ext: impl Ext);
}

impl AgentExtLoader for Agent {
    fn load_ext(&mut self, global_object: Object, ext: impl Ext) {
        ext.load(ExtLoader {
            agent: self,
            global_object,
        });
    }
}
