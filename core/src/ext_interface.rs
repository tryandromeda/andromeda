use nova_vm::ecmascript::{
    builtins::{create_builtin_function, Behaviour, BuiltinFunctionArgs, RegularFn},
    execution::Agent,
    types::{InternalMethods, IntoValue, Object, PropertyDescriptor, PropertyKey},
};

use crate::HostData;

pub struct ExtLoader<'a> {
    pub agent: &'a mut Agent,
    pub global_object: Object,
}

impl ExtLoader<'_> {
    pub fn init_storage<T: 'static>(&mut self, value: T) {
        let host_data = self.agent.get_host_data();
        let host_data: &HostData = host_data.downcast_ref().unwrap();
        let mut storage = host_data.storage.borrow_mut();
        storage.insert(value);
    }

    pub fn load_op(&mut self, name: &'static str, function: RegularFn, args: u32) {
        let function = create_builtin_function(
            self.agent,
            Behaviour::Regular(function),
            BuiltinFunctionArgs::new(args, name, self.agent.current_realm_id()),
        );
        let property_key = PropertyKey::from_static_str(self.agent, name);
        self.global_object
            .internal_define_own_property(
                self.agent,
                property_key,
                PropertyDescriptor {
                    value: Some(function.into_value()),
                    ..Default::default()
                },
            )
            .unwrap();
    }
}

pub trait Ext {
    fn load(&self, loader: ExtLoader);
}
