use nova_vm::ecmascript::{
    execution::Agent,
    scripts_and_modules::script::{parse_script, script_evaluation},
    types::Object,
};
use oxc_allocator::Allocator;
use oxc_diagnostics::OxcDiagnostic;

#[cfg(feature = "unsafe-sqlite")]
use crate::ext::SQliteExt;
use crate::{ext_loader::AgentExtLoader, ConsoleExt, FsExt, TimeExt};

pub fn initialize_recommended_extensions(agent: &mut Agent, global_object: Object) {
    agent.load_ext(global_object, FsExt);
    agent.load_ext(global_object, ConsoleExt);
    agent.load_ext(global_object, TimeExt);
    #[cfg(feature = "unsafe-sqlite")]
    agent.load_ext(global_object, SQliteExt);
}

pub fn initialize_recommended_builtins(allocator: &Allocator, agent: &mut Agent, no_strict: bool) {
    let realm = agent.current_realm_id();
    let builtins = vec![
        include_str!("../../runtime/console.ts"),
        include_str!("../../runtime/mod.ts"),
    ];
    for builtin in builtins {
        let script = match parse_script(allocator, builtin.into(), realm, !no_strict, None) {
            Ok(script) => script,
            Err((file, errors)) => exit_with_parse_errors(errors, "<runtime>", &file),
        };
        match script_evaluation(agent, script) {
            Ok(_) => (),
            Err(_) => println!("Error in runtime"),
        }
    }
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
