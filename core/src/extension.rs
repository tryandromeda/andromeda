use nova_vm::{
    ecmascript::{
        builtins::{Behaviour, BuiltinFunctionArgs, RegularFn, create_builtin_function},
        execution::Agent,
        scripts_and_modules::script::{parse_script, script_evaluation},
        types::{InternalMethods, IntoValue, Object, PropertyDescriptor, PropertyKey},
    },
    engine::context::GcScope,
};

use crate::{HostData, OpsStorage, exit_with_parse_errors};

pub type ExtensionStorageInit = Box<dyn FnOnce(&mut OpsStorage)>;

/// Global function part of a larger [Extension].
pub struct ExtensionOp {
    pub name: &'static str,
    pub function: RegularFn,
    pub args: u32,
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
    pub files: Vec<&'static str>,
}

impl Extension {
    pub(crate) fn load<UserMacroTask: 'static>(
        &mut self,
        agent: &mut Agent,
        global_object: Object,
        gc: &mut GcScope<'_, '_>,
    ) {
        for file in &self.files {
            let source_text = nova_vm::ecmascript::types::String::from_str(agent, file, gc.nogc());
            let script = match parse_script(
                agent,
                source_text,
                agent.current_realm_id(),
                true,
                None,
                gc.nogc(),
            ) {
                Ok(script) => script,
                Err(diagnostics) => exit_with_parse_errors(diagnostics, "<runtime>", file),
            };
            match script_evaluation(agent, script, gc.reborrow()) {
                Ok(_) => (),
                Err(_) => println!("Error in runtime"),
            }
        }
        for op in &self.ops {
            let function = create_builtin_function(
                agent,
                Behaviour::Regular(op.function),
                BuiltinFunctionArgs::new(op.args, op.name, agent.current_realm_id()),
                gc.nogc(),
            );
            function.unbind();
            let property_key = PropertyKey::from_static_str(agent, op.name, gc.nogc());
            global_object
                .internal_define_own_property(
                    agent,
                    property_key.unbind(),
                    PropertyDescriptor {
                        value: Some(function.into_value()),
                        ..Default::default()
                    },
                    gc.reborrow(),
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
