use andromeda_core::{
    debug, internal_exit, internal_read, internal_read_line, internal_read_text_file,
    internal_write, internal_write_line, internal_write_text_file,
};
use nova_vm::ecmascript::{
    builtins::{create_builtin_function, Behaviour, BuiltinFunctionArgs},
    execution::Agent,
    types::{InternalMethods, IntoValue, Object, PropertyDescriptor, PropertyKey},
};
use oxc_diagnostics::OxcDiagnostic;

/// Initialize the global object with the built-in functions.
pub fn initialize_global_object(agent: &mut Agent, global: Object) {
    let function = create_builtin_function(
        agent,
        Behaviour::Regular(debug),
        BuiltinFunctionArgs::new(1, "debug", agent.current_realm_id()),
    );
    let property_key = PropertyKey::from_static_str(agent, "debug");

    let internal_read_text_file = create_builtin_function(
        agent,
        Behaviour::Regular(internal_read_text_file),
        BuiltinFunctionArgs::new(1, "internal_read_text_file", agent.current_realm_id()),
    );
    let internal_read_text_file_key =
        PropertyKey::from_static_str(agent, "internal_read_text_file");

    let internal_write_text_file = create_builtin_function(
        agent,
        Behaviour::Regular(internal_write_text_file),
        BuiltinFunctionArgs::new(2, "internal_write_text_file", agent.current_realm_id()),
    );
    let internal_write_text_file_key_ =
        PropertyKey::from_static_str(agent, "internal_write_text_file");

    let internal_exit = create_builtin_function(
        agent,
        Behaviour::Regular(internal_exit),
        BuiltinFunctionArgs::new(1, "internal_exit", agent.current_realm_id()),
    );
    let internal_exit_key = PropertyKey::from_static_str(agent, "internal_exit");

    let internal_read_line = create_builtin_function(
        agent,
        Behaviour::Regular(internal_read_line),
        BuiltinFunctionArgs::new(0, "internal_read_line", agent.current_realm_id()),
    );
    let internal_read_line_key = PropertyKey::from_static_str(agent, "internal_read_line");

    let internal_write_line = create_builtin_function(
        agent,
        Behaviour::Regular(internal_write_line),
        BuiltinFunctionArgs::new(1, "internal_write_line", agent.current_realm_id()),
    );
    let internal_write_line_key = PropertyKey::from_static_str(agent, "internal_write_line");

    let internal_read = create_builtin_function(
        agent,
        Behaviour::Regular(internal_read),
        BuiltinFunctionArgs::new(0, "internal_read", agent.current_realm_id()),
    );
    let internal_read_key = PropertyKey::from_static_str(agent, "internal_read");

    let internal_write = create_builtin_function(
        agent,
        Behaviour::Regular(internal_write),
        BuiltinFunctionArgs::new(1, "internal_write", agent.current_realm_id()),
    );

    let internal_write_key = PropertyKey::from_static_str(agent, "internal_write");

    global
        .internal_define_own_property(
            agent,
            internal_read_line_key,
            PropertyDescriptor {
                value: Some(internal_read_line.into_value()),
                ..Default::default()
            },
        )
        .unwrap();

    global
        .internal_define_own_property(
            agent,
            internal_write_line_key,
            PropertyDescriptor {
                value: Some(internal_write_line.into_value()),
                ..Default::default()
            },
        )
        .unwrap();

    global
        .internal_define_own_property(
            agent,
            internal_read_key,
            PropertyDescriptor {
                value: Some(internal_read.into_value()),
                ..Default::default()
            },
        )
        .unwrap();

    global
        .internal_define_own_property(
            agent,
            internal_write_key,
            PropertyDescriptor {
                value: Some(internal_write.into_value()),
                ..Default::default()
            },
        )
        .unwrap();
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
            internal_read_text_file_key,
            PropertyDescriptor {
                value: Some(internal_read_text_file.into_value()),
                ..Default::default()
            },
        )
        .unwrap();
    global
        .internal_define_own_property(
            agent,
            internal_write_text_file_key_,
            PropertyDescriptor {
                value: Some(internal_write_text_file.into_value()),
                ..Default::default()
            },
        )
        .unwrap();

    global
        .internal_define_own_property(
            agent,
            internal_exit_key,
            PropertyDescriptor {
                value: Some(internal_exit.into_value()),
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
