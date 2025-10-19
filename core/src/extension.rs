// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use nova_vm::{
    ecmascript::{
        builtins::{Behaviour, BuiltinFunctionArgs, RegularFn, create_builtin_function},
        execution::Agent,
        scripts_and_modules::{
            module::module_semantics::source_text_module_records::parse_module, script::HostDefined,
        },
        types::{InternalMethods, IntoValue, Object, PropertyDescriptor, PropertyKey},
    },
    engine::context::{Bindable, GcScope},
};

use crate::{AndromedaError, HostData, OpsStorage, exit_with_parse_errors, print_enhanced_error};

pub type ExtensionStorageInit = Box<dyn FnOnce(&mut OpsStorage)>;

/// Global function part of a larger [Extension].
pub struct ExtensionOp {
    pub name: &'static str,
    pub function: RegularFn,
    pub args: u32,
    pub global: bool,
}

impl ExtensionOp {
    pub fn new(name: &'static str, function: RegularFn, args: u32, global: bool) -> Self {
        Self {
            name,
            args,
            function,
            global,
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

#[cfg_attr(feature = "hotpath", hotpath::measure_all)]
impl Extension {
    pub(crate) fn load<UserMacroTask: 'static>(
        &mut self,
        agent: &mut Agent,
        global_object: Object,
        andromeda_object: Object,
        gc: &mut GcScope<'_, '_>,
    ) {
        for (idx, file_source) in self.files.iter().enumerate() {
            let specifier = format!("<ext:{}:{}>", self.name, idx);
            let source_text =
                nova_vm::ecmascript::types::String::from_str(agent, file_source, gc.nogc());

            let module = match parse_module(
                agent,
                source_text,
                agent.current_realm(gc.nogc()),
                Some(std::rc::Rc::new(specifier.clone()) as HostDefined),
                gc.nogc(),
            ) {
                Ok(module) => module,
                Err(diagnostics) => exit_with_parse_errors(diagnostics, &specifier, file_source),
            };

            let eval_result = agent
                .run_parsed_module(module.unbind(), None, gc.reborrow())
                .unbind();
            if let Err(e) = eval_result {
                let error_value = e.value();
                let message = error_value
                    .string_repr(agent, gc.reborrow())
                    .as_str(agent)
                    .unwrap_or("<non-string error>")
                    .to_string();
                let err = AndromedaError::runtime_error(message);
                print_enhanced_error(&err);
            }
        }
        for op in &self.ops {
            let function = create_builtin_function(
                agent,
                Behaviour::Regular(op.function),
                BuiltinFunctionArgs::new(op.args, op.name),
                gc.nogc(),
            );
            function.unbind();
            let property_key = PropertyKey::from_static_str(agent, op.name, gc.nogc());
            if op.global {
                global_object
                    .internal_define_own_property(
                        agent,
                        property_key.unbind(),
                        PropertyDescriptor {
                            value: Some(function.into_value().unbind()),
                            ..Default::default()
                        },
                        gc.reborrow(),
                    )
                    .unwrap();
            } else {
                andromeda_object
                    .internal_define_own_property(
                        agent,
                        property_key.unbind(),
                        PropertyDescriptor {
                            value: Some(function.into_value().unbind()),
                            ..Default::default()
                        },
                        gc.reborrow(),
                    )
                    .unwrap();
            }
        }

        if let Some(storage_hook) = self.storage.take() {
            let host_data = agent.get_host_data();
            let host_data: &HostData<UserMacroTask> = host_data.downcast_ref().unwrap();
            let mut storage = host_data.storage.borrow_mut();
            (storage_hook)(&mut storage)
        }
    }
}
