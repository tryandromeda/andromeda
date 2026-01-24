// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::config::{AndromedaConfig, ConfigManager};
use crate::error::{CliError, CliResult, print_error};
use crate::repl::highlighter::JsHighlighter;
use crate::repl::prompt::ReplPrompt;
use crate::repl::validator::JsValidator;
use crate::styles::format_js_value;
use andromeda_core::{HostData, RuntimeHostHooks};
use andromeda_runtime::{RuntimeMacroTask, recommended_builtins, recommended_extensions};
use console::Style;
use nova_vm::ecmascript::types::IntoObject;
use nova_vm::{
    ecmascript::{
        builtins::{ArgumentsList, Behaviour, BuiltinFunctionArgs, create_builtin_function},
        execution::{
            Agent, JsResult,
            agent::{GcAgent, Options},
        },
        scripts_and_modules::{
            module::module_semantics::source_text_module_records::parse_module,
            script::{parse_script, script_evaluation},
        },
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
use oxc_diagnostics::OxcDiagnostic;
use reedline::{Reedline, Signal};
use std::sync::mpsc;

mod highlighter;
mod prompt;
mod validator;

/// Handle parse errors in REPL with beautiful formatting
fn handle_parse_errors(errors: Vec<OxcDiagnostic>, source_path: &str, source: &str) {
    let error = CliError::parse_error(errors, source_path.to_string(), source.to_string());
    print_error(error);
}

/// Handle runtime errors in REPL with beautiful formatting
fn handle_runtime_error_with_message(error_message: String) {
    let error =
        CliError::runtime_error(error_message, Some("<repl>".to_string()), None, None, None);
    print_error(error);
}

#[allow(clippy::result_large_err)]
#[hotpath::measure]
pub fn run_repl_with_config(
    expose_internals: bool,
    print_internals: bool,
    disable_gc: bool,
    config_override: Option<AndromedaConfig>,
) -> CliResult<()> {
    // Load configuration
    let config = config_override.unwrap_or_else(|| ConfigManager::load_or_default(None));

    // Apply CLI overrides to config
    let effective_expose_internals = expose_internals || config.runtime.expose_internals;
    let effective_print_internals = print_internals || config.runtime.print_internals;
    let effective_disable_gc = disable_gc || config.runtime.disable_gc;

    let (_macro_task_tx, _macro_task_rx) = mpsc::channel();
    let host_data = HostData::new(_macro_task_tx);

    let host_hooks = RuntimeHostHooks::new(host_data);
    let host_hooks: &RuntimeHostHooks<RuntimeMacroTask> = &*Box::leak(Box::new(host_hooks));

    let mut agent = GcAgent::new(
        Options {
            no_block: false,
            disable_gc: effective_disable_gc,
            print_internals: effective_print_internals,
        },
        host_hooks,
    );

    let create_global_object: Option<for<'a> fn(&mut Agent, GcScope<'a, '_>) -> Object<'a>> = None;
    let create_global_this_value: Option<for<'a> fn(&mut Agent, GcScope<'a, '_>) -> Object<'a>> =
        None;

    let initialize_global: Option<fn(&mut Agent, Object, GcScope)> = if effective_expose_internals {
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
                Err(errors) => {
                    handle_parse_errors(errors, "<builtin>", builtin);
                    std::process::exit(1);
                }
            };
            if script_evaluation(agent, script.unbind(), gc.reborrow()).is_err() {
                eprintln!("⚠️  Warning: Error loading builtin module");
                handle_runtime_error_with_message("Script evaluation failed".to_string());
            }
        }
    });

    let welcome_style = Style::new().cyan().bold();
    let version_style = Style::new().dim();
    let help_style = Style::new().yellow();

    println!(
        "\n{} {}",
        welcome_style.apply_to("Andromeda"),
        env!("CARGO_PKG_VERSION")
    );
    println!(
        "{}",
        version_style.apply_to("JavaScript/TypeScript Runtime powered by Nova")
    );
    println!(
        "{}",
        help_style.apply_to("Type 'help' for commands, 'exit' or Ctrl+C to quit")
    );
    println!();

    show_startup_tip();

    let mut line_editor = Reedline::create()
        .with_validator(Box::new(JsValidator))
        .with_highlighter(Box::new(JsHighlighter));

    let mut evaluation_count = 1;
    let mut command_history: Vec<String> = Vec::new();

    loop {
        let prompt = ReplPrompt::new(evaluation_count);

        let sig = line_editor.read_line(&prompt);
        let input = match sig {
            Ok(Signal::Success(buffer)) => buffer,
            Ok(Signal::CtrlD) | Ok(Signal::CtrlC) => {
                std::process::exit(0);
            }
            Err(err) => {
                println!("Error reading input: {err}");
                continue;
            }
        };

        let input_trimmed = input.trim();

        match input_trimmed {
            "exit" | "quit" => {
                std::process::exit(0);
            }
            "help" => {
                print_help();
                continue;
            }
            "clear" => {
                print!("\x1B[2J\x1B[1;1H");
                continue;
            }
            "history" => {
                print_history(&command_history);
                continue;
            }
            "gc" => {
                let gc_style = Style::new().yellow();
                println!("{}", gc_style.apply_to("🗑️  Running garbage collection..."));
                agent.gc();
                println!(
                    "{}",
                    Style::new()
                        .green()
                        .apply_to("✅ Garbage collection completed")
                );
                continue;
            }
            "" => continue,
            _ => {}
        }

        #[allow(clippy::unnecessary_map_or)]
        if !command_history.last().map_or(false, |last| last == &input) {
            command_history.push(input.clone());
            if command_history.len() > 100 {
                command_history.remove(0);
            }
        }
        let start_time = std::time::Instant::now();
        agent.run_in_realm(&realm, |agent, mut gc| {
            let realm_obj = agent.current_realm(gc.nogc());
            let source_text = types::String::from_string(agent, input.clone(), gc.nogc());
            let script = match parse_script(agent, source_text, realm_obj, true, None, gc.nogc()) {
                Ok(script) => script,
                Err(errors) => {
                    handle_parse_errors(errors, "<repl>", &input);
                    return;
                }
            };
            let result = script_evaluation(agent, script.unbind(), gc.reborrow()).unbind();
            let elapsed = start_time.elapsed();

            match result {
                Ok(result) => match result.to_string(agent, gc) {
                    Ok(val) => {
                        let result_style = Style::new().green();
                        let time_style = Style::new().dim();
                        let type_style = Style::new().dim().italic();
                        let output = val.as_str(agent).expect("String is not valid UTF-8");

                        if !output.is_empty() && output != "undefined" {
                            let (formatted_value, value_type) = format_js_value(output);
                            println!(
                                "{} {} {}",
                                result_style.apply_to("←"),
                                formatted_value,
                                type_style.apply_to(format!("({value_type})"))
                            );
                        } else if output == "undefined" {
                            let (formatted_value, _) = format_js_value(output);
                            println!("{} {}", Style::new().dim().apply_to("←"), formatted_value);
                        }
                        println!(
                            "{}",
                            time_style.apply_to(format!("  {}ms", elapsed.as_millis()))
                        );
                    }
                    Err(_) => {
                        let error_style = Style::new().red().bold();
                        println!(
                            "{} {}",
                            error_style.apply_to("✗"),
                            error_style.apply_to("Error converting result to string")
                        );
                    }
                },
                Err(error) => {
                    let error_value = error.value();
                    let error_message = error_value
                        .string_repr(agent, gc.reborrow())
                        .as_str(agent)
                        .expect("String is not valid UTF-8")
                        .to_string();
                    handle_runtime_error_with_message(error_message);
                }
            }
        });

        evaluation_count += 1;
        println!();
    }
}

fn initialize_global_object(agent: &mut Agent, global_object: Object, mut gc: GcScope) {
    let andromeda_obj =
        OrdinaryObject::create_empty_object(agent, gc.nogc()).scope(agent, gc.nogc());
    let property_key = PropertyKey::from_static_str(agent, "__andromeda__", gc.nogc());
    global_object
        .internal_define_own_property(
            agent,
            property_key.unbind(),
            PropertyDescriptor {
                value: Some(andromeda_obj.get(agent).into_value()),
                writable: Some(true),
                enumerable: Some(false),
                configurable: Some(true),
                ..Default::default()
            },
            gc.reborrow(),
        )
        .unwrap();

    let mut extensions = recommended_extensions();
    for extension in &mut extensions {
        for (idx, file) in extension.files.iter().enumerate() {
            let specifier = format!("<ext:{}:{}>", extension.name, idx);
            let source_text = types::String::from_str(agent, file, gc.nogc());
            let module = match parse_module(
                agent,
                source_text,
                agent.current_realm(gc.nogc()),
                Some(std::rc::Rc::new(specifier.clone())),
                gc.nogc(),
            ) {
                Ok(module) => module,
                Err(errors) => {
                    handle_parse_errors(errors, &specifier, file);
                    std::process::exit(1);
                }
            };
            if agent
                .run_parsed_module(module.unbind(), None, gc.reborrow())
                .unbind()
                .is_err()
            {
                eprintln!("⚠️  Warning: Error loading extension {}", specifier);
                handle_runtime_error_with_message("Module evaluation failed".to_string());
            }
        }
        // Register native ops
        for op in &extension.ops {
            let function = create_builtin_function(
                agent,
                Behaviour::Regular(op.function),
                BuiltinFunctionArgs::new(op.args, op.name),
                gc.nogc(),
            );
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
                andromeda_obj
                    .get(agent)
                    .into_object()
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

        // Run storage initializer for the extension (if any) so extension storage types are inserted
        if let Some(storage_hook) = extension.storage.take() {
            let host_data = agent.get_host_data();
            let host_data: &HostData<RuntimeMacroTask> = host_data.downcast_ref().unwrap();
            let mut storage = host_data.storage.borrow_mut();
            (storage_hook)(&mut storage)
        }
    }
}

fn initialize_global_object_with_internals(agent: &mut Agent, global: Object, mut gc: GcScope) {
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

fn show_startup_tip() {
    let tips = [
        "console.log('Hello, World!')",
        "Math.sqrt(16)",
        "new Date().toISOString()",
        "[1, 2, 3].map(x => x * 2)",
        "JSON.stringify({name: 'Andromeda'})",
        "'hello'.toUpperCase()",
        "Array.from({length: 5}, (_, i) => i)",
        "Promise.resolve(42).then(console.log)",
        "const obj = { x: 1, y: 2 }",
    ];

    let code_style = Style::new().yellow();
    let multiline_style = Style::new().dim();
    let random_tip = tips[std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize
        % tips.len()];

    println!("Try: {}", code_style.apply_to(random_tip));

    if std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        .is_multiple_of(3)
    {
        println!(
            "{}",
            multiline_style.apply_to(
                "   Multiline: Start typing function/object syntax, it detects automatically!"
            )
        );
    }

    println!();
}

fn print_help() {
    let help_style = Style::new().cyan().bold();
    let command_style = Style::new().yellow();
    let desc_style = Style::new().dim();

    println!("{}", help_style.apply_to("Available Commands:"));
    println!(
        "  {}  {}",
        command_style.apply_to("help"),
        desc_style.apply_to("Show this help message")
    );
    println!(
        "  {}  {}",
        command_style.apply_to("exit, quit"),
        desc_style.apply_to("Exit the REPL")
    );
    println!(
        "  {}  {}",
        command_style.apply_to("clear"),
        desc_style.apply_to("Clear the screen")
    );
    println!(
        "  {}  {}",
        command_style.apply_to("history"),
        desc_style.apply_to("Show command history")
    );
    println!(
        "  {}  {}",
        command_style.apply_to("gc"),
        desc_style.apply_to("Run garbage collection")
    );
    println!();
    println!("{}", help_style.apply_to("Multiline Support:"));
    println!(
        "  • {} {}",
        command_style.apply_to("Auto-detection:"),
        desc_style.apply_to("Incomplete syntax triggers multiline mode")
    );
    println!(
        "  • {} {}",
        command_style.apply_to("Manual finish:"),
        desc_style.apply_to("Press Enter on complete syntax")
    );
    println!(
        "  • {} {}",
        command_style.apply_to("Examples:"),
        desc_style.apply_to("function declarations, objects, arrays")
    );
    println!();
    println!(
        "{}",
        desc_style
            .apply_to("Tip: Use arrow keys to navigate history, syntax highlighting included!")
    );
    println!();
}

fn print_history(history: &[String]) {
    let history_style = Style::new().cyan().bold();
    let number_style = Style::new().dim();
    let command_style = Style::new().bright();

    if history.is_empty() {
        println!("{}", Style::new().dim().apply_to("No command history yet"));
        return;
    }

    println!("{}", history_style.apply_to("Command History:"));
    for (i, cmd) in history.iter().enumerate().rev().take(20) {
        println!(
            "  {} {}",
            number_style.apply_to(format!("{:2}.", i + 1)),
            command_style.apply_to(cmd)
        );
    }

    if history.len() > 20 {
        println!(
            "  {}",
            Style::new()
                .dim()
                .apply_to(format!("... and {} more", history.len() - 20))
        );
    }
    println!();
}
