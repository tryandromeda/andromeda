use nova_vm::ecmascript::{
    builtins::{create_builtin_function, ArgumentsList, Behaviour, BuiltinFunctionArgs},
    execution::{Agent, JsResult},
    types::{InternalMethods, IntoValue, Object, PropertyDescriptor, PropertyKey, Value},
};
use oxc_diagnostics::OxcDiagnostic;
use std::borrow::BorrowMut;

/// Initialize the global object with the built-in functions.
pub fn initialize_global_object(agent: &mut Agent, global: Object) {
    // Define the `debug` function.
    fn debug(agent: &mut Agent, _this: Value, args: ArgumentsList) -> JsResult<Value> {
        if args.len() == 0 {
            println!();
        } else {
            println!("{}", args[0].to_string(agent)?.as_str(agent));
        }
        Ok(Value::Undefined)
    }
    fn _internal_read_file(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
    ) -> JsResult<Value> {
        let binding = args.get(0).to_string(agent)?;
        let path = binding.as_str(agent);
        let content = match std::fs::read_to_string(path) {
            Ok(content) => content,
            Err(e) => {
                return Ok(Value::from_string(
                    agent,
                    format!("Error: {}", e.to_string()),
                ));
            }
        };
        Ok(Value::from_string(agent, content))
    }
    fn _internal_write_text_file(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
    ) -> JsResult<Value> {
        let binding = args.get(0).to_string(agent)?;
        let content = args.get(1).to_string(agent.borrow_mut())?;
        match std::fs::write(binding.as_str(agent), content.as_str(agent)) {
            Ok(_) => Ok(Value::from_string(agent, "Success".to_string())),
            Err(e) => Ok(Value::from_string(
                agent,
                format!("Error: {}", e.to_string()),
            )),
        }
    }
    let function = create_builtin_function(
        agent,
        Behaviour::Regular(debug),
        BuiltinFunctionArgs::new(1, "debug", agent.current_realm_id()),
    );
    let property_key = PropertyKey::from_static_str(agent, "debug");

    let _internal_read_file = create_builtin_function(
        agent,
        Behaviour::Regular(_internal_read_file),
        BuiltinFunctionArgs::new(1, "_internal_read_file", agent.current_realm_id()),
    );
    let _internal_read_file_key = PropertyKey::from_static_str(agent, "_internal_read_file");

    let _internal_write_text_file = create_builtin_function(
        agent,
        Behaviour::Regular(_internal_write_text_file),
        BuiltinFunctionArgs::new(2, "_internal_write_text_file", agent.current_realm_id()),
    );
    let _internal_write_text_file_key_ =
        PropertyKey::from_static_str(agent, "_internal_write_text_file");

    global
        .internal_define_own_property(
            agent,
            property_key,
            PropertyDescriptor {
                value: Some(function.into_value()),
                ..Default::default()
            },
        )
        .unwrap();

    global
        .internal_define_own_property(
            agent,
            _internal_read_file_key,
            PropertyDescriptor {
                value: Some(_internal_read_file.into_value()),
                ..Default::default()
            },
        )
        .unwrap();
    global
        .internal_define_own_property(
            agent,
            _internal_write_text_file_key_,
            PropertyDescriptor {
                value: Some(_internal_write_text_file.into_value()),
                ..Default::default()
            },
        )
        .unwrap();
}

/// Exit the program with parse errors.
pub fn exit_with_parse_errors(errors: Vec<OxcDiagnostic>, source_path: &str, source: &str) -> ! {
    assert!(!errors.is_empty());

    // This seems to be needed for color and Unicode output.
    miette::set_hook(Box::new(|_| {
        Box::new(oxc_diagnostics::GraphicalReportHandler::new())
    }))
    .unwrap();

    eprintln!("Parse errors:");

    // SAFETY: This function never returns, so `source`'s lifetime must last for
    // the duration of the program.
    let source: &'static str = unsafe { std::mem::transmute(source) };
    let named_source = miette::NamedSource::new(source_path, source);

    for error in errors {
        let report = error.with_source_code(named_source.clone());
        eprint!("{:?}", report);
    }
    eprintln!();

    std::process::exit(1);
}
