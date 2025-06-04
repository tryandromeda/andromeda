// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use andromeda_core::{HostData, RuntimeHostHooks, exit_with_parse_errors};
use andromeda_runtime::{recommended_builtins, recommended_extensions};
use cliclack::{input, intro, set_theme};
use nova_vm::{
    ecmascript::{
        builtins::{ArgumentsList, Behaviour, BuiltinFunctionArgs, create_builtin_function},
        execution::{
            Agent, JsResult,
            agent::{GcAgent, Options},
        },
        scripts_and_modules::script::{parse_script, script_evaluation},
        types::{
            self, InternalMethods, IntoValue, Object, OrdinaryObject, PropertyDescriptor,
            PropertyKey, Value,
        },
    },
    engine::{
        context::{Bindable, GcScope},
        rootable::Scopable,
    },
};
use std::sync::mpsc;

use crate::styles::DefaultTheme;

pub fn run_repl(
    expose_internals: bool,
    print_internals: bool,
    disable_gc: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let (_macro_task_tx, _macro_task_rx) = mpsc::channel();
    let host_data = HostData::new(_macro_task_tx);

    let host_hooks = RuntimeHostHooks::new(host_data);
    let host_hooks: &RuntimeHostHooks<()> = &*Box::leak(Box::new(host_hooks));

    let mut agent = GcAgent::new(
        Options {
            disable_gc,
            print_internals,
        },
        host_hooks,
    );

    let create_global_object: Option<for<'a> fn(&mut Agent, GcScope<'a, '_>) -> Object<'a>> = None;
    let create_global_this_value: Option<for<'a> fn(&mut Agent, GcScope<'a, '_>) -> Object<'a>> =
        None;

    let initialize_global: Option<fn(&mut Agent, Object, GcScope)> = if expose_internals {
        Some(initialize_global_object_with_internals)
    } else {
        Some(initialize_global_object)
    };

    let realm = agent.create_realm(
        create_global_object,
        create_global_this_value,
        initialize_global,
    );

    // Load builtin JavaScript sources
    agent.run_in_realm(&realm, |agent, mut gc| {
        for builtin in recommended_builtins() {
            let realm_obj = agent.current_realm(gc.nogc());
            let source_text = types::String::from_str(agent, builtin, gc.nogc());
            let script = match parse_script(agent, source_text, realm_obj, true, None, gc.nogc()) {
                Ok(script) => script,
                Err(errors) => exit_with_parse_errors(errors, "<builtin>", builtin),
            };
            match script_evaluation(agent, script.unbind(), gc.reborrow()) {
                Ok(_) => (),
                Err(_) => eprintln!("Error loading builtin"),
            }
        }
    });

    set_theme(DefaultTheme);
    println!("\n");
    let mut placeholder = "Enter a line of JavaScript".to_string();

    let _ = ctrlc::set_handler(|| {
        std::process::exit(0);
    });

    loop {
        intro("Andromeda REPL")?;
        let input: String = input("").placeholder(&placeholder).interact()?;

        if input.trim() == "exit" {
            std::process::exit(0);
        } else if input.trim() == "gc" {
            agent.gc();
            continue;
        }

        placeholder = input.clone();
        agent.run_in_realm(&realm, |agent, mut gc| {
            let realm_obj = agent.current_realm(gc.nogc());
            let source_text = types::String::from_string(agent, input, gc.nogc());
            let script = match parse_script(agent, source_text, realm_obj, true, None, gc.nogc()) {
                Ok(script) => script,
                Err(errors) => {
                    exit_with_parse_errors(errors, "<stdin>", &placeholder);
                }
            };
            let result = script_evaluation(agent, script.unbind(), gc.reborrow()).unbind();
            match result {
                Ok(result) => match result.to_string(agent, gc) {
                    Ok(val) => {
                        println!("{}", val.as_str(agent));
                    }
                    Err(_) => {
                        eprintln!("Error converting result to string");
                    }
                },
                Err(error) => {
                    eprintln!("Uncaught exception: {error:?}");
                }
            }
        });
    }
}

fn initialize_global_object(agent: &mut Agent, global_object: Object, mut gc: GcScope) {
    let mut extensions = recommended_extensions();
    for extension in &mut extensions {
        // Load extension JavaScript/TypeScript files
        for file in &extension.files {
            let source_text = types::String::from_str(agent, file, gc.nogc());
            let script = match parse_script(
                agent,
                source_text,
                agent.current_realm(gc.nogc()),
                true,
                None,
                gc.nogc(),
            ) {
                Ok(script) => script,
                Err(errors) => exit_with_parse_errors(errors, "<extension>", file),
            };
            match script_evaluation(agent, script.unbind(), gc.reborrow()) {
                Ok(_) => (),
                Err(_) => eprintln!("Error loading extension"),
            }
        }

        // Load extension ops (native functions)
        for op in &extension.ops {
            let function = create_builtin_function(
                agent,
                Behaviour::Regular(op.function),
                BuiltinFunctionArgs::new(op.args, op.name),
                gc.nogc(),
            );
            let property_key = PropertyKey::from_static_str(agent, op.name, gc.nogc());
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
        }
    }
}

fn initialize_global_object_with_internals(agent: &mut Agent, global: Object, mut gc: GcScope) {
    // `detachArrayBuffer` function
    fn detach_array_buffer<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let args = args.bind(gc.nogc());
        let Value::ArrayBuffer(array_buffer) = args.get(0) else {
            return Err(agent.throw_exception_with_static_message(
                nova_vm::ecmascript::execution::agent::ExceptionType::Error,
                "Cannot detach non ArrayBuffer argument",
                gc.into_nogc(),
            ));
        };
        array_buffer.detach(agent, None, gc.nogc()).unbind()?;
        Ok(Value::Undefined)
    }

    fn create_realm<'gc>(
        agent: &mut Agent,
        _this: Value,
        _args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let create_global_object: Option<for<'a> fn(&mut Agent, GcScope<'a, '_>) -> Object<'a>> =
            None;
        let create_global_this_value: Option<
            for<'a> fn(&mut Agent, GcScope<'a, '_>) -> Object<'a>,
        > = None;
        let realm = agent
            .create_realm(
                create_global_object,
                create_global_this_value,
                Some(initialize_global_object_with_internals),
                gc,
            )
            .unbind();
        Ok(realm.global_object(agent).into_value().unbind())
    }
    initialize_global_object(agent, global, gc.reborrow());
    ().unbind();
    let obj = OrdinaryObject::create_empty_object(agent, gc.nogc()).unbind();
    let nova_obj = obj.scope(agent, gc.nogc());
    let property_key = PropertyKey::from_static_str(agent, "__nova__", gc.nogc());
    global
        .internal_define_own_property(
            agent,
            property_key.unbind(),
            PropertyDescriptor {
                value: Some(nova_obj.get(agent).into_value()),
                writable: Some(true),
                enumerable: Some(false),
                configurable: Some(true),
                ..Default::default()
            },
            gc.reborrow(),
        )
        .unwrap();

    let function = create_builtin_function(
        agent,
        Behaviour::Regular(detach_array_buffer),
        BuiltinFunctionArgs::new(1, "detachArrayBuffer"),
        gc.nogc(),
    );
    let property_key = PropertyKey::from_static_str(agent, "detachArrayBuffer", gc.nogc());
    nova_obj
        .get(agent)
        .internal_define_own_property(
            agent,
            property_key.unbind(),
            PropertyDescriptor {
                value: Some(function.into_value().unbind()),
                writable: Some(true),
                enumerable: Some(false),
                configurable: Some(true),
                ..Default::default()
            },
            gc.reborrow(),
        )
        .unwrap();

    let function = create_builtin_function(
        agent,
        Behaviour::Regular(create_realm),
        BuiltinFunctionArgs::new(1, "createRealm"),
        gc.nogc(),
    );
    let property_key = PropertyKey::from_static_str(agent, "createRealm", gc.nogc());
    nova_obj
        .get(agent)
        .internal_define_own_property(
            agent,
            property_key.unbind(),
            PropertyDescriptor {
                value: Some(function.into_value().unbind()),
                writable: Some(true),
                enumerable: Some(false),
                configurable: Some(true),
                ..Default::default()
            },
            gc.reborrow(),
        )
        .unwrap();
}
