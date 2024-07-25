use andromeda_core::{
    exit_with_parse_errors, initialize_recommended_builtins, initialize_recommended_extensions,
    HostData, RuntimeHostHooks,
};
use cliclack::{input, intro, set_theme};
use nova_vm::ecmascript::{
    execution::{agent::Options, initialize_host_defined_realm, Agent, Realm},
    scripts_and_modules::script::{parse_script, script_evaluation},
    types::Object,
};

use crate::repl_theme::DefaultTheme;

pub fn repl(verbose: bool) {
    let allocator = Default::default();
    let (host_data, _macro_task_rx) = HostData::new();
    let host_hooks: RuntimeHostHooks = RuntimeHostHooks::new(host_data);
    let host_hooks: &RuntimeHostHooks = &*Box::leak(Box::new(host_hooks));
    let mut agent = Agent::new(
        Options {
            disable_gc: false,
            print_internals: verbose,
        },
        host_hooks,
    );
    {
        let create_global_object: Option<fn(&mut Realm) -> Object> = None;
        let create_global_this_value: Option<fn(&mut Realm) -> Object> = None;
        initialize_host_defined_realm(
            &mut agent,
            create_global_object,
            create_global_this_value,
            Some(initialize_recommended_extensions),
        );
    }
    let realm = agent.current_realm_id();
    initialize_recommended_builtins(&allocator, &mut agent, false);

    set_theme(DefaultTheme);
    println!("\n\n");
    let mut placeholder = "Enter a line of Javascript".to_string();

    loop {
        intro("Nova Repl (type exit or ctrl+c to exit)").unwrap();
        let input: String = input("").placeholder(&placeholder).interact().unwrap();

        if input.matches("exit").count() == 1 {
            std::process::exit(0);
        }
        placeholder = input.to_string();
        let script = match parse_script(&allocator, input.into(), realm, true, None) {
            Ok(script) => script,
            Err((file, errors)) => {
                exit_with_parse_errors(errors, "<stdin>", &file);
            }
        };
        let result = script_evaluation(&mut agent, script);
        match result {
            Ok(result) => {
                println!("{:?}\n", result);
            }
            Err(error) => {
                eprintln!(
                    "Uncaught exception: {}",
                    error.value().string_repr(&mut agent).as_str(&agent)
                );
            }
        }
    }
}
