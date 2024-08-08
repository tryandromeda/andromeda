use nova_vm::ecmascript::{
    builtins::{create_builtin_function, Behaviour, BuiltinFunctionArgs, RegularFn},
    execution::Agent,
    types::{InternalMethods, IntoValue, Object, PropertyDescriptor, PropertyKey},
};

use crate::{HostData, OpsStorage};

pub type ExtensionStorageInit = Box<dyn FnOnce(&mut OpsStorage)>;

/// Global function part of a larger [Extension].
pub struct ExtensionOp {
    pub name: &'static str,
    pub args: u32,
    pub function: RegularFn,
}

impl ExtensionOp {
    pub fn new(name: &'static str, function: RegularFn, args: u32) -> Self {
        Self {
            name,
            args,
            function,
        }
    }
}

/// Group of global functions. Usually every extension has it's own topic, e.g: fs, network, ffi, etc.
pub struct Extension {
    /// Name of the extension.
    pub name: &'static str,
    /// List of [ExtensionOp] pertaining to this [Extension].
    pub ops: Vec<ExtensionOp>,
    /// Storage initializer for this extension.
    /// Used to share values between the different [ExtensionOp] and multiple calls.
    pub storage: Option<ExtensionStorageInit>,

    // JavaScript or Typescript files that are loaded by this extension.
    pub files: Vec<String>,
}

impl Extension {
    pub(crate) fn load<UserMacroTask: 'static>(
        &mut self,
        agent: &mut Agent,
        global_object: Object,
    ) {
        for op in &self.ops {
            let function = create_builtin_function(
                agent,
                Behaviour::Regular(op.function),
                BuiltinFunctionArgs::new(op.args, op.name, agent.current_realm_id()),
            );
            let property_key = PropertyKey::from_static_str(agent, op.name);
            global_object
                .internal_define_own_property(
                    agent,
                    property_key,
                    PropertyDescriptor {
                        value: Some(function.into_value()),
                        ..Default::default()
                    },
                )
                .unwrap();
        }

        if let Some(storage_hook) = self.storage.take() {
            let host_data = agent.get_host_data();
            let host_data: &HostData<UserMacroTask> = host_data.downcast_ref().unwrap();
            let mut storage = host_data.storage.borrow_mut();
            (storage_hook)(&mut storage)
        }
    }
}
