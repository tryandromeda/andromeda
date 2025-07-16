// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::error::{AndromedaError, Result, print_error};
use crate::styles::format_js_value;
use andromeda_core::{HostData, RuntimeHostHooks};
use andromeda_runtime::{RuntimeMacroTask, recommended_builtins, recommended_extensions};
use console::Style;
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
use oxc_diagnostics::OxcDiagnostic;
use reedline::{
    Highlighter, Prompt, PromptHistorySearch, PromptHistorySearchStatus, Reedline, Signal,
    StyledText, ValidationResult, Validator,
};
use std::sync::mpsc;

#[derive(Clone)]
struct ReplPrompt {
    evaluation_count: usize,
}

impl ReplPrompt {
    fn new(count: usize) -> Self {
        Self {
            evaluation_count: count,
        }
    }
}

impl Prompt for ReplPrompt {
    fn render_prompt_left(&self) -> std::borrow::Cow<str> {
        let count_style = Style::new().dim();
        format!(
            "{} > ",
            count_style.apply_to(format!("{}", self.evaluation_count))
        )
        .into()
    }

    fn render_prompt_right(&self) -> std::borrow::Cow<str> {
        "".into()
    }

    fn render_prompt_indicator(
        &self,
        _prompt_mode: reedline::PromptEditMode,
    ) -> std::borrow::Cow<str> {
        "".into()
    }

    fn render_prompt_multiline_indicator(&self) -> std::borrow::Cow<str> {
        let style = Style::new().dim();
        format!("{}", style.apply_to("...")).into()
    }

    fn render_prompt_history_search_indicator(
        &self,
        history_search: PromptHistorySearch,
    ) -> std::borrow::Cow<str> {
        let prefix = match history_search.status {
            PromptHistorySearchStatus::Passing => "",
            PromptHistorySearchStatus::Failing => "failing ",
        };
        format!("({}reverse-search: {}) ", prefix, history_search.term).into()
    }
}

// JavaScript syntax validator for multiline input
#[derive(Clone)]
struct JsValidator;

impl Validator for JsValidator {
    fn validate(&self, line: &str) -> ValidationResult {
        let trimmed = line.trim();

        // Special commands are always complete
        if matches!(
            trimmed,
            "help" | "exit" | "quit" | "clear" | "history" | "gc" | ""
        ) {
            return ValidationResult::Complete;
        }

        if is_incomplete_js(line) {
            ValidationResult::Incomplete
        } else {
            ValidationResult::Complete
        }
    }
}

fn is_incomplete_js(code: &str) -> bool {
    let mut paren_count = 0;
    let mut brace_count = 0;
    let mut bracket_count = 0;
    let mut in_string = false;
    let mut string_char = '\0';
    let mut escape_next = false;
    let mut in_single_comment = false;
    let mut in_multi_comment = false;

    let chars: Vec<char> = code.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let ch = chars[i];

        if escape_next {
            escape_next = false;
            i += 1;
            continue;
        }

        if in_single_comment {
            if ch == '\n' {
                in_single_comment = false;
            }
            i += 1;
            continue;
        }

        if in_multi_comment {
            if ch == '*' && i + 1 < chars.len() && chars[i + 1] == '/' {
                in_multi_comment = false;
                i += 2;
                continue;
            }
            i += 1;
            continue;
        }

        if in_string {
            if ch == '\\' {
                escape_next = true;
            } else if ch == string_char {
                in_string = false;
                string_char = '\0';
            }
            i += 1;
            continue;
        }

        match ch {
            '"' | '\'' | '`' => {
                in_string = true;
                string_char = ch;
            }
            '/' if i + 1 < chars.len() => {
                if chars[i + 1] == '/' {
                    in_single_comment = true;
                    i += 1;
                } else if chars[i + 1] == '*' {
                    in_multi_comment = true;
                    i += 1;
                }
            }
            '(' => paren_count += 1,
            ')' => paren_count -= 1,
            '{' => brace_count += 1,
            '}' => brace_count -= 1,
            '[' => bracket_count += 1,
            ']' => bracket_count -= 1,
            _ => {}
        }

        i += 1;
    }

    paren_count > 0 || brace_count > 0 || bracket_count > 0 || in_string || in_multi_comment
}

// JavaScript syntax highlighter for the REPL
#[derive(Clone)]
pub struct JsHighlighter;

impl Highlighter for JsHighlighter {
    fn highlight(&self, line: &str, _cursor: usize) -> StyledText {
        let mut styled = StyledText::new();

        // Handle special REPL commands first
        let trimmed = line.trim();
        if matches!(
            trimmed,
            "help" | "exit" | "quit" | "clear" | "history" | "gc"
        ) {
            styled.push((
                nu_ansi_term::Style::new().fg(nu_ansi_term::Color::Yellow),
                trimmed.to_string(),
            ));
            return styled;
        }

        // Tokenize and highlight JavaScript
        let chars: Vec<char> = line.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            let ch = chars[i];

            // Skip whitespace
            if ch.is_whitespace() {
                styled.push((nu_ansi_term::Style::new(), ch.to_string()));
                i += 1;
                continue;
            }

            // Handle strings
            if ch == '"' || ch == '\'' || ch == '`' {
                let string_start = i;
                let quote = ch;
                i += 1;

                // Find the end of the string
                while i < chars.len() {
                    if chars[i] == '\\' && i + 1 < chars.len() {
                        i += 2; // Skip escaped character
                    } else if chars[i] == quote {
                        i += 1; // Include closing quote
                        break;
                    } else {
                        i += 1;
                    }
                }

                let string_content: String = chars[string_start..i].iter().collect();
                let style = if quote == '`' {
                    nu_ansi_term::Style::new().fg(nu_ansi_term::Color::Cyan) // Template literals
                } else {
                    nu_ansi_term::Style::new().fg(nu_ansi_term::Color::Green) // Regular strings
                };
                styled.push((style, string_content));
                continue;
            }

            // Handle comments
            if ch == '/' && i + 1 < chars.len() {
                if chars[i + 1] == '/' {
                    // Single line comment
                    let comment: String = chars[i..].iter().collect();
                    styled.push((
                        nu_ansi_term::Style::new().fg(nu_ansi_term::Color::DarkGray),
                        comment,
                    ));
                    break;
                } else if chars[i + 1] == '*' {
                    // Multi-line comment
                    let comment_start = i;
                    i += 2;

                    while i + 1 < chars.len() {
                        if chars[i] == '*' && chars[i + 1] == '/' {
                            i += 2;
                            break;
                        }
                        i += 1;
                    }

                    let comment: String = chars[comment_start..i].iter().collect();
                    styled.push((
                        nu_ansi_term::Style::new().fg(nu_ansi_term::Color::DarkGray),
                        comment,
                    ));
                    continue;
                }
            }

            // Handle numbers
            if ch.is_ascii_digit()
                || (ch == '.' && i + 1 < chars.len() && chars[i + 1].is_ascii_digit())
            {
                let number_start = i;

                // Handle decimal numbers
                while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                    i += 1;
                }

                let number: String = chars[number_start..i].iter().collect();
                styled.push((
                    nu_ansi_term::Style::new().fg(nu_ansi_term::Color::Blue),
                    number,
                ));
                continue;
            }

            // Handle type annotations (TypeScript)
            if ch == ':' && i + 1 < chars.len() && chars[i + 1].is_whitespace() {
                // This is likely a type annotation, so highlight the colon and continue
                styled.push((
                    nu_ansi_term::Style::new().fg(nu_ansi_term::Color::Cyan),
                    ch.to_string(),
                ));
                i += 1;
                continue;
            }

            // Handle angle brackets for generic types
            if ch == '<' || ch == '>' {
                // Check if this might be a generic type by looking for surrounding context
                let is_likely_generic = if ch == '<' {
                    // Look backwards for function/class/interface names or 'new'
                    let mut j = i.saturating_sub(1);
                    while j > 0 && chars[j].is_whitespace() {
                        j = j.saturating_sub(1);
                    }
                    j > 0 && (chars[j].is_alphanumeric() || chars[j] == '_' || chars[j] == '$')
                } else {
                    // Look ahead for typical post-generic syntax
                    i + 1 < chars.len() && (chars[i + 1] == '(' || chars[i + 1].is_whitespace())
                };

                let style = if is_likely_generic {
                    nu_ansi_term::Style::new().fg(nu_ansi_term::Color::Cyan)
                } else {
                    nu_ansi_term::Style::new().fg(nu_ansi_term::Color::Red) // Comparison operators
                };
                styled.push((style, ch.to_string()));
                i += 1;
                continue;
            }

            if ch.is_alphabetic() || ch == '_' || ch == '$' {
                let word_start = i;

                while i < chars.len()
                    && (chars[i].is_alphanumeric() || chars[i] == '_' || chars[i] == '$')
                {
                    i += 1;
                }

                let word: String = chars[word_start..i].iter().collect();

                let style = match word.as_str() {
                    // JavaScript keywords
                    "const" | "let" | "var" | "function" | "return" | "if" | "else" | "for"
                    | "while" | "do" | "switch" | "case" | "default" | "break" | "continue"
                    | "try" | "catch" | "finally" | "throw" | "new" | "this" | "super"
                    | "class" | "extends" | "import" | "export" | "from" | "as" | "async"
                    | "await" | "yield" | "typeof" | "instanceof" | "in" | "of" | "delete"
                    | "void" => nu_ansi_term::Style::new()
                        .fg(nu_ansi_term::Color::Purple)
                        .bold(),

                    // TypeScript-specific keywords
                    "interface" | "type" | "enum" | "namespace" | "module" | "declare"
                    | "abstract" | "implements" | "private" | "protected" | "public"
                    | "readonly" | "static" | "override" | "keyof" | "infer" | "is" | "asserts"
                    | "satisfies" | "using" | "out" | "accessor" => nu_ansi_term::Style::new()
                        .fg(nu_ansi_term::Color::Purple)
                        .bold(),

                    // Built-in TypeScript types
                    "string" | "number" | "boolean" | "object" | "bigint" | "symbol"
                    | "undefined" | "any" | "unknown" | "never" => nu_ansi_term::Style::new()
                        .fg(nu_ansi_term::Color::Blue)
                        .bold(),

                    // Utility types
                    "Partial"
                    | "Required"
                    | "Readonly"
                    | "Record"
                    | "Pick"
                    | "Omit"
                    | "Exclude"
                    | "Extract"
                    | "NonNullable"
                    | "ReturnType"
                    | "InstanceType"
                    | "Parameters"
                    | "ConstructorParameters"
                    | "ThisParameterType"
                    | "OmitThisParameter"
                    | "ThisType"
                    | "Uppercase"
                    | "Lowercase"
                    | "Capitalize"
                    | "Uncapitalize"
                    | "NoInfer"
                    | "Awaited" => nu_ansi_term::Style::new().fg(nu_ansi_term::Color::Blue),

                    // Boolean and null values
                    "true" | "false" | "null" => nu_ansi_term::Style::new()
                        .fg(nu_ansi_term::Color::Red)
                        .bold(),

                    // Built-in objects and global functions
                    "Andromeda" | "Event" | "OffscreenCanvas" | "console" | "Math" | "Date"
                    | "Array" | "Object" | "String" | "Number" | "Boolean" | "RegExp" | "Error"
                    | "Promise" | "JSON" | "parseInt" | "parseFloat" | "isNaN" | "isFinite"
                    | "encodeURIComponent" | "decodeURIComponent" | "Map" | "Set" | "WeakMap"
                    | "WeakSet" | "ArrayBuffer" | "DataView" | "Int8Array" | "Uint8Array"
                    | "Int16Array" | "Uint16Array" | "Int32Array" | "Uint32Array"
                    | "Float32Array" | "Float64Array" | "BigInt64Array" | "BigUint64Array" => {
                        nu_ansi_term::Style::new().fg(nu_ansi_term::Color::Magenta)
                    }

                    // Check if it's a type annotation context (follows a colon)
                    _ => {
                        // Look backwards to see if this word follows a colon (type annotation)
                        let mut j = word_start.saturating_sub(1);
                        while j > 0 && chars[j].is_whitespace() {
                            j = j.saturating_sub(1);
                        }
                        if j > 0 && chars[j] == ':' {
                            // This is likely a type annotation
                            nu_ansi_term::Style::new().fg(nu_ansi_term::Color::Blue)
                        } else {
                            // Check if it looks like a type (PascalCase)
                            if word.chars().next().is_some_and(|c| c.is_uppercase()) {
                                nu_ansi_term::Style::new().fg(nu_ansi_term::Color::Blue)
                            } else {
                                nu_ansi_term::Style::new()
                            }
                        }
                    }
                };

                styled.push((style, word));
                continue;
            }

            // Handle multi-character TypeScript operators
            if ch == '=' && i + 1 < chars.len() && chars[i + 1] == '>' {
                // Arrow function
                styled.push((
                    nu_ansi_term::Style::new().fg(nu_ansi_term::Color::Purple),
                    "=>".to_string(),
                ));
                i += 2;
                continue;
            }

            if ch == '?' && i + 1 < chars.len() {
                if chars[i + 1] == '?' {
                    // Nullish coalescing operator
                    styled.push((
                        nu_ansi_term::Style::new().fg(nu_ansi_term::Color::Red),
                        "??".to_string(),
                    ));
                    i += 2;
                    continue;
                } else if chars[i + 1] == '.' {
                    // Optional chaining
                    styled.push((
                        nu_ansi_term::Style::new().fg(nu_ansi_term::Color::Red),
                        "?.".to_string(),
                    ));
                    i += 2;
                    continue;
                }
            }

            if ch == '!' && i + 1 < chars.len() && chars[i + 1] == '!' {
                // Non-null assertion (TypeScript)
                styled.push((
                    nu_ansi_term::Style::new().fg(nu_ansi_term::Color::Red),
                    "!!".to_string(),
                ));
                i += 2;
                continue;
            }

            // Handle pipe for union types
            if ch == '|' && i > 0 && i + 1 < chars.len() {
                // Check if this is a union type (surrounded by type context)
                let prev_non_space = chars[..i].iter().rposition(|&c| !c.is_whitespace());
                let next_non_space = chars[i + 1..]
                    .iter()
                    .position(|&c| !c.is_whitespace())
                    .map(|pos| i + 1 + pos);

                let is_union_type = match (prev_non_space, next_non_space) {
                    (Some(prev_idx), Some(next_idx)) => {
                        // Simple heuristic: if surrounded by alphanumeric characters or angle brackets, likely a union type
                        (chars[prev_idx].is_alphanumeric()
                            || chars[prev_idx] == '>'
                            || chars[prev_idx] == ']')
                            && (chars[next_idx].is_alphanumeric()
                                || chars[next_idx] == '<'
                                || chars[next_idx] == '('
                                || chars[next_idx] == '{')
                    }
                    _ => false,
                };

                let style = if is_union_type {
                    nu_ansi_term::Style::new().fg(nu_ansi_term::Color::Cyan)
                } else {
                    nu_ansi_term::Style::new().fg(nu_ansi_term::Color::Red) // Bitwise OR
                };
                styled.push((style, ch.to_string()));
                i += 1;
                continue;
            }

            // Handle ampersand for intersection types
            if ch == '&' && i > 0 && i + 1 < chars.len() && chars[i + 1] != '&' {
                // Similar heuristic for intersection types
                let prev_non_space = chars[..i].iter().rposition(|&c| !c.is_whitespace());
                let next_non_space = chars[i + 1..]
                    .iter()
                    .position(|&c| !c.is_whitespace())
                    .map(|pos| i + 1 + pos);

                let is_intersection_type = match (prev_non_space, next_non_space) {
                    (Some(prev_idx), Some(next_idx)) => {
                        (chars[prev_idx].is_alphanumeric()
                            || chars[prev_idx] == '>'
                            || chars[prev_idx] == ']')
                            && (chars[next_idx].is_alphanumeric()
                                || chars[next_idx] == '<'
                                || chars[next_idx] == '('
                                || chars[next_idx] == '{')
                    }
                    _ => false,
                };

                let style = if is_intersection_type {
                    nu_ansi_term::Style::new().fg(nu_ansi_term::Color::Cyan)
                } else {
                    nu_ansi_term::Style::new().fg(nu_ansi_term::Color::Red) // Bitwise AND
                };
                styled.push((style, ch.to_string()));
                i += 1;
                continue;
            }

            let style = match ch {
                '(' | ')' | '[' | ']' | '{' | '}' => {
                    nu_ansi_term::Style::new().fg(nu_ansi_term::Color::Yellow)
                }
                '+' | '-' | '*' | '/' | '%' | '=' | '!' | '<' | '>' | '&' | '|' | '^' | '~'
                | '?' => nu_ansi_term::Style::new().fg(nu_ansi_term::Color::Red),
                ';' | ',' | '.' => nu_ansi_term::Style::new().fg(nu_ansi_term::Color::DarkGray),
                ':' => {
                    // TypeScript type annotations
                    nu_ansi_term::Style::new().fg(nu_ansi_term::Color::Cyan)
                }
                _ => nu_ansi_term::Style::new(),
            };

            styled.push((style, ch.to_string()));
            i += 1;
        }

        styled
    }
}

/// Handle parse errors in REPL with beautiful formatting
fn handle_parse_errors(errors: Vec<OxcDiagnostic>, source_path: &str, source: &str) {
    let error = AndromedaError::parse_error(errors, source_path.to_string(), source.to_string());
    print_error(error);
}

/// Handle runtime errors in REPL with beautiful formatting
fn handle_runtime_error_with_message(error_message: String) {
    let error =
        AndromedaError::runtime_error(error_message, Some("<repl>".to_string()), None, None, None);
    print_error(error);
}

#[allow(clippy::result_large_err)]
pub fn run_repl(expose_internals: bool, print_internals: bool, disable_gc: bool) -> Result<()> {
    let (_macro_task_tx, _macro_task_rx) = mpsc::channel();
    let host_data = HostData::new(_macro_task_tx);

    let host_hooks = RuntimeHostHooks::new(host_data);
    let host_hooks: &RuntimeHostHooks<RuntimeMacroTask> = &*Box::leak(Box::new(host_hooks));

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
        "\n{}",
        welcome_style.apply_to("🚀 Welcome to Andromeda REPL")
    );
    println!(
        "{}",
        version_style.apply_to("   JavaScript/TypeScript Runtime powered by Nova")
    );
    println!(
        "{}",
        help_style.apply_to("   Type 'help' for commands, 'exit' or Ctrl+C to quit")
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
                println!("\n{}", Style::new().dim().apply_to("👋 Goodbye!"));
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
                println!("{}", Style::new().dim().apply_to("👋 Goodbye!"));
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
                            time_style.apply_to(format!("  ⏱️  {}ms", elapsed.as_millis()))
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
                Err(errors) => {
                    handle_parse_errors(errors, "<extension>", file);
                    std::process::exit(1);
                }
            };
            if script_evaluation(agent, script.unbind(), gc.reborrow()).is_err() {
                eprintln!("⚠️  Warning: Error loading extension");
                handle_runtime_error_with_message("Script evaluation failed".to_string());
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

    let tip_style = Style::new().blue();
    let code_style = Style::new().yellow();
    let multiline_style = Style::new().dim();
    let random_tip = tips[std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize
        % tips.len()];

    println!(
        "{} Try: {}",
        tip_style.apply_to("💡"),
        code_style.apply_to(random_tip)
    );

    // Occasionally show multiline tip
    if std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        % 3
        == 0
    {
        println!(
            "{}",
            multiline_style.apply_to(
                "   💭 Multiline: Start typing function/object syntax, it detects automatically!"
            )
        );
    }

    println!();
}

fn print_help() {
    let help_style = Style::new().cyan().bold();
    let command_style = Style::new().yellow();
    let desc_style = Style::new().dim();

    println!("{}", help_style.apply_to("📚 Available Commands:"));
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
    println!("{}", help_style.apply_to("🔧 Multiline Support:"));
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
            .apply_to("💡 Tip: Use arrow keys to navigate history, syntax highlighting included!")
    );
    println!();
}

fn print_history(history: &[String]) {
    let history_style = Style::new().cyan().bold();
    let number_style = Style::new().dim();
    let command_style = Style::new().bright();

    if history.is_empty() {
        println!(
            "{}",
            Style::new().dim().apply_to("📝 No command history yet")
        );
        return;
    }

    println!("{}", history_style.apply_to("📝 Command History:"));
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
