use crate::helper::exit_with_parse_errors;
use nova_vm::ecmascript::{
    execution::Agent,
    scripts_and_modules::script::{parse_script, script_evaluation},
};
use oxc_allocator::Allocator;

pub fn attach_builtins(allocator: &Allocator, agent: &mut Agent, no_strict: bool) {
    let realm = agent.current_realm_id();
    let paths = vec![
        include_str!("../../runtime/console.ts"),
        include_str!("../../runtime/mod.ts"),
    ];
    for path in paths {
        let script = match parse_script(allocator, path.into(), realm, !no_strict, None) {
            Ok(script) => script,
            Err((file, errors)) => exit_with_parse_errors(errors, "<runtime>", &file),
        };
        match script_evaluation(agent, script) {
            Ok(_) => (),
            Err(_) => println!("Error in runtime"),
        }
    }
}
