mod event_loop;
mod ext;
mod ext_interface;
mod helper;
mod host_data;
mod resource_table;

use ext_interface::{Ext, ExtLoader};
use nova_vm::ecmascript::{execution::Agent, types::Object};

pub use event_loop::*;
pub use ext::*;
pub use helper::*;
pub use host_data::*;

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
