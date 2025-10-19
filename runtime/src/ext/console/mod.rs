// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::collections::HashMap;
use std::io::{Write, stderr, stdout};
use std::time::{SystemTime, UNIX_EPOCH};

use andromeda_core::{AndromedaError, ErrorReporter, Extension, ExtensionOp, HostData, OpsStorage};
use nova_vm::{
    ecmascript::{
        builtins::ArgumentsList,
        execution::{Agent, JsResult},
        types::Value,
    },
    engine::context::{Bindable, GcScope},
};

/// Storage for console state (timers, counters, group indentation)
#[derive(Default)]
pub struct ConsoleStorage {
    pub timers: HashMap<String, u128>,
    pub counters: HashMap<String, u32>,
    pub group_indent_level: usize,
}

#[derive(Default)]
pub struct ConsoleExt;

#[cfg_attr(feature = "hotpath", hotpath::measure_all)]
impl ConsoleExt {
    pub fn new_extension() -> Extension {
        Extension {
            name: "console",
            ops: vec![
                ExtensionOp::new("internal_print", Self::internal_print, 1, false),
                ExtensionOp::new("internal_print_err", Self::internal_print_err, 1, false),
                ExtensionOp::new("time_start", Self::time_start, 1, false),
                ExtensionOp::new("time_log", Self::time_log, 2, false),
                ExtensionOp::new("time_end", Self::time_end, 1, false),
                ExtensionOp::new("count", Self::count, 1, false),
                ExtensionOp::new("count_reset", Self::count_reset, 1, false),
                ExtensionOp::new("group_start", Self::group_start, 1, false),
                ExtensionOp::new("group_end", Self::group_end, 0, false),
                ExtensionOp::new("get_group_indent", Self::get_group_indent, 0, false),
                ExtensionOp::new("get_stack_trace", Self::get_stack_trace, 0, false),
                ExtensionOp::new("clear_console", Self::clear_console, 0, false),
                ExtensionOp::new("internal_read", Self::internal_read, 1, false),
                ExtensionOp::new("internal_read_line", Self::internal_read_line, 1, false),
                ExtensionOp::new("internal_write", Self::internal_write, 1, false),
                ExtensionOp::new("internal_write_line", Self::internal_write_line, 1, false),
                ExtensionOp::new("internal_css_to_ansi", Self::internal_css_to_ansi, 1, false),
                ExtensionOp::new("internal_exit", Self::internal_exit, 1, false),
            ],
            storage: Some(Box::new(|storage: &mut OpsStorage| {
                storage.insert(ConsoleStorage::default());
            })),
            files: vec![include_str!("./mod.ts")],
        }
    }

    /// Print function that prints to stdout
    fn internal_print<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        if let Err(e) = stdout().write_all(
            args[0]
                .to_string(agent, gc.reborrow())
                .unbind()?
                .as_str(agent)
                .expect("String is not valid UTF-8")
                .as_bytes(),
        ) {
            let error = AndromedaError::runtime_error(format!("Failed to write to stdout: {e}"));
            ErrorReporter::print_error(&error);
        }
        if let Err(e) = stdout().flush() {
            let error = AndromedaError::runtime_error(format!("Failed to flush stdout: {e}"));
            ErrorReporter::print_error(&error);
        }
        Ok(Value::Undefined)
    }

    /// Print function that prints to stderr
    fn internal_print_err<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        if let Err(e) = stderr().write_all(
            args[0]
                .to_string(agent, gc.reborrow())
                .unbind()?
                .as_str(agent)
                .expect("String is not valid UTF-8")
                .as_bytes(),
        ) {
            let error = AndromedaError::runtime_error(format!("Failed to write to stderr: {e}"));
            ErrorReporter::print_error(&error);
        }
        if let Err(e) = stderr().flush() {
            let error = AndromedaError::runtime_error(format!("Failed to flush stderr: {e}"));
            ErrorReporter::print_error(&error);
        }
        Ok(Value::Undefined)
    }

    /// Start a timer with the given label
    fn time_start<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let label = if !args.is_empty() {
            args[0]
                .to_string(agent, gc.reborrow())
                .unbind()?
                .as_str(agent)
                .expect("String is not valid UTF-8")
                .to_string()
        } else {
            "default".to_string()
        };

        let timer_exists = {
            let host_data = agent.get_host_data();
            let host_data: &HostData<crate::RuntimeMacroTask> = host_data.downcast_ref().unwrap();
            let storage = host_data.storage.borrow();
            let console_storage: &ConsoleStorage = storage.get().unwrap();
            console_storage.timers.contains_key(&label)
        };

        if timer_exists {
            let warning_msg = format!("Timer '{label}' already exists");
            return Ok(Value::from_string(agent, warning_msg, gc.nogc()).unbind());
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();

        // Store the timer in console storage
        {
            let host_data = agent.get_host_data();
            let host_data: &HostData<crate::RuntimeMacroTask> = host_data.downcast_ref().unwrap();
            let mut storage = host_data.storage.borrow_mut();
            let console_storage: &mut ConsoleStorage = storage.get_mut().unwrap();
            console_storage.timers.insert(label, now);
        }

        Ok(Value::Undefined)
    }

    /// Log time for a timer
    fn time_log<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let label = if !args.is_empty() {
            args[0]
                .to_string(agent, gc.reborrow())
                .unbind()?
                .as_str(agent)
                .expect("String is not valid UTF-8")
                .to_string()
        } else {
            "default".to_string()
        };

        let data = if args.len() > 1 {
            args[1]
                .to_string(agent, gc.reborrow())
                .unbind()?
                .as_str(agent)
                .expect("String is not valid UTF-8")
                .to_string()
        } else {
            String::new()
        };

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();

        // Access the timer from console storage
        let result = {
            let host_data = agent.get_host_data();
            let host_data: &HostData<crate::RuntimeMacroTask> = host_data.downcast_ref().unwrap();
            let storage = host_data.storage.borrow();
            let console_storage: &ConsoleStorage = storage.get().unwrap();

            if let Some(&start_time) = console_storage.timers.get(&label) {
                let elapsed = now - start_time;
                if data.is_empty() {
                    format!("{label}: {elapsed}ms")
                } else {
                    format!("{label}: {elapsed}ms {data}")
                }
            } else {
                // Return warning message per WHATWG spec
                format!("Timer '{label}' does not exist")
            }
        };

        Ok(Value::from_string(agent, result, gc.nogc()).unbind())
    }

    /// End a timer and return the elapsed time
    fn time_end<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let label = if !args.is_empty() {
            args[0]
                .to_string(agent, gc.reborrow())
                .unbind()?
                .as_str(agent)
                .expect("String is not valid UTF-8")
                .to_string()
        } else {
            "default".to_string()
        };

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();

        // Remove the timer from console storage and calculate elapsed time
        let result = {
            let host_data = agent.get_host_data();
            let host_data: &HostData<crate::RuntimeMacroTask> = host_data.downcast_ref().unwrap();
            let mut storage = host_data.storage.borrow_mut();
            let console_storage: &mut ConsoleStorage = storage.get_mut().unwrap();

            if let Some(start_time) = console_storage.timers.remove(&label) {
                let elapsed = now - start_time;
                format!("{label}: {elapsed}ms")
            } else {
                format!("Timer '{label}' does not exist")
            }
        };

        Ok(Value::from_string(agent, result, gc.nogc()).unbind())
    }

    /// Count operation
    fn count<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let label = if !args.is_empty() {
            args[0]
                .to_string(agent, gc.reborrow())
                .unbind()?
                .as_str(agent)
                .expect("String is not valid UTF-8")
                .to_string()
        } else {
            "default".to_string()
        };

        // Access and increment the counter in console storage
        let result = {
            let host_data = agent.get_host_data();
            let host_data: &HostData<crate::RuntimeMacroTask> = host_data.downcast_ref().unwrap();
            let mut storage = host_data.storage.borrow_mut();
            let console_storage: &mut ConsoleStorage = storage.get_mut().unwrap();

            let count = console_storage.counters.entry(label.clone()).or_insert(0);
            *count += 1;
            format!("{label}: {count}")
        };

        Ok(Value::from_string(agent, result, gc.nogc()).unbind())
    }

    /// Reset counter
    fn count_reset<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let label = if !args.is_empty() {
            args[0]
                .to_string(agent, gc.reborrow())
                .unbind()?
                .as_str(agent)
                .expect("String is not valid UTF-8")
                .to_string()
        } else {
            "default".to_string()
        };

        // Reset the counter in console storage
        let result = {
            let host_data = agent.get_host_data();
            let host_data: &HostData<crate::RuntimeMacroTask> = host_data.downcast_ref().unwrap();
            let mut storage = host_data.storage.borrow_mut();
            let console_storage: &mut ConsoleStorage = storage.get_mut().unwrap();

            if console_storage.counters.contains_key(&label) {
                console_storage.counters.insert(label.clone(), 0);
                format!("Count for '{label}' reset")
            } else {
                format!("Count for '{label}' does not exist")
            }
        };

        Ok(Value::from_string(agent, result, gc.nogc()).unbind())
    }

    /// Start a group
    fn group_start<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let label = if !args.is_empty() {
            args[0]
                .to_string(agent, gc.reborrow())
                .unbind()?
                .as_str(agent)
                .expect("String is not valid UTF-8")
                .to_string()
        } else {
            String::new()
        };

        // Increment group indent level in console storage
        {
            let host_data = agent.get_host_data();
            let host_data: &HostData<crate::RuntimeMacroTask> = host_data.downcast_ref().unwrap();
            let mut storage = host_data.storage.borrow_mut();
            let console_storage: &mut ConsoleStorage = storage.get_mut().unwrap();
            console_storage.group_indent_level += 1;
        }

        Ok(Value::from_string(agent, label, gc.nogc()).unbind())
    }

    /// End a group
    fn group_end<'gc>(
        agent: &mut Agent,
        _this: Value,
        _args: ArgumentsList,
        _gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        // Decrement group indent level in console storage
        let host_data = agent.get_host_data();
        let host_data: &HostData<crate::RuntimeMacroTask> = host_data.downcast_ref().unwrap();
        let mut storage = host_data.storage.borrow_mut();
        let console_storage: &mut ConsoleStorage = storage.get_mut().unwrap();
        if console_storage.group_indent_level > 0 {
            console_storage.group_indent_level -= 1;
        }
        Ok(Value::Undefined)
    }

    /// Get the current group indent level
    fn get_group_indent<'gc>(
        agent: &mut Agent,
        _this: Value,
        _args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let indent_level = {
            let host_data = agent.get_host_data();
            let host_data: &HostData<crate::RuntimeMacroTask> = host_data.downcast_ref().unwrap();
            let storage = host_data.storage.borrow();
            match storage.get::<ConsoleStorage>() {
                Some(console_storage) => console_storage.group_indent_level,
                None => {
                    let error = AndromedaError::runtime_error(
                        "Console storage missing when querying group indent; defaulting to 0"
                            .to_string(),
                    );
                    ErrorReporter::print_error(&error);
                    0
                }
            }
        };
        Ok(Value::from_f64(agent, indent_level as f64, gc.nogc()).unbind())
    }

    /// Get stack trace
    fn get_stack_trace<'gc>(
        agent: &mut Agent,
        _this: Value,
        _args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        // I
        // TODO: capture the actual JavaScript call stack
        // For now, this simulate a realistic stack trace that could be enhanced
        // with actual frame walking in the future

        // Create a more realistic stack trace that reflects actual execution context
        let mut stack_trace = String::from("Error");

        // Generate stack frames that represent a typical JavaScript execution context
        // This could be enhanced to walk the actual call stack frames from Nova VM
        let frames = vec![
            "    at console.trace (andromeda:console)",
            "    at Object.<anonymous> (file:///unknown:1:1)",
        ];

        for frame in frames {
            stack_trace.push('\n');
            stack_trace.push_str(frame);
        }

        // Additional context that could be enhanced with real frame information
        if let Some(current_file) = Self::get_current_file_context() {
            stack_trace.push('\n');
            stack_trace.push_str(&format!(
                "    at {} ({}:1:1)",
                "Object.<anonymous>", current_file
            ));
        }

        Ok(Value::from_string(agent, stack_trace, gc.nogc()).unbind())
    }

    /// Get current file context (placeholder for future enhancement)
    /// In a production implementation, this would extract the current execution file
    /// from the Nova VM's execution context
    fn get_current_file_context() -> Option<String> {
        // This is where we would integrate with Nova VM's execution context
        // to get the actual file being executed
        None
    }

    /// Clear console
    fn clear_console<'gc>(
        agent: &mut Agent,
        _this: Value,
        _args: ArgumentsList,
        _gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        // Reset group indent level per WHATWG spec
        {
            let host_data = agent.get_host_data();
            let host_data: &HostData<crate::RuntimeMacroTask> = host_data.downcast_ref().unwrap();
            let mut storage = host_data.storage.borrow_mut();
            let console_storage: &mut ConsoleStorage = storage.get_mut().unwrap();
            console_storage.group_indent_level = 0;
        }

        // ANSI escape sequence to clear screen
        print!("\x1B[2J\x1B[H");
        if let Err(e) = stdout().flush() {
            let error = AndromedaError::runtime_error(format!("Failed to flush stdout: {e}"));
            ErrorReporter::print_error(&error);
        }
        Ok(Value::Undefined)
    }

    /// Exit the process with the given exit code.
    pub fn internal_exit<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        std::process::exit(args[0].to_int32(agent, gc.reborrow()).unbind()?);
    }

    /// Internal read for reading from the console.
    pub fn internal_read<'gc>(
        agent: &mut Agent,
        _this: Value,
        _args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let mut input = String::new();
        match std::io::stdin().read_line(&mut input) {
            Ok(_) => {
                Ok(Value::from_string(agent, input.trim_end().to_string(), gc.nogc()).unbind())
            }
            Err(e) => {
                let error =
                    AndromedaError::runtime_error(format!("Failed to read from stdin: {e}"));
                let error_msg = ErrorReporter::format_error(&error);
                Ok(Value::from_string(agent, format!("Error: {error_msg}"), gc.nogc()).unbind())
            }
        }
    }

    /// Internal read line for reading from the console with a newline.
    pub fn internal_read_line<'gc>(
        agent: &mut Agent,
        _this: Value,
        _args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let mut input = String::new();
        match std::io::stdin().read_line(&mut input) {
            Ok(_) => {
                Ok(Value::from_string(agent, input.trim_end().to_string(), gc.nogc()).unbind())
            }
            Err(e) => {
                let error =
                    AndromedaError::runtime_error(format!("Failed to read line from stdin: {e}"));
                let error_msg = ErrorReporter::format_error(&error);
                Ok(Value::from_string(agent, format!("Error: {error_msg}"), gc.nogc()).unbind())
            }
        }
    }

    /// Internal write for writing to the console.
    pub fn internal_write<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        for arg in args.iter() {
            print!(
                "{}",
                arg.to_string(agent, gc.reborrow())
                    .unbind()?
                    .as_str(agent)
                    .expect("String is not valid UTF-8")
            );
        }
        Ok(Value::Undefined)
    }

    /// Internal write line for writing to the console with a newline.
    pub fn internal_write_line<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        for arg in args.iter() {
            print!(
                "{}",
                arg.to_string(agent, gc.reborrow())
                    .unbind()?
                    .as_str(agent)
                    .expect("String is not valid UTF-8")
            );
        }
        println!();
        Ok(Value::Undefined)
    }

    /// Convert CSS styling to ANSI escape codes
    fn internal_css_to_ansi<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let css_text = if !args.is_empty() {
            args[0]
                .to_string(agent, gc.reborrow())
                .unbind()?
                .as_str(agent)
                .expect("String is not valid UTF-8")
                .to_string()
        } else {
            String::new()
        };

        let ansi_codes = Self::css_to_ansi(&css_text);
        Ok(Value::from_string(agent, ansi_codes, gc.nogc()).unbind())
    }

    /// Parse CSS and convert to ANSI escape codes
    fn css_to_ansi(css: &str) -> String {
        let mut ansi = String::new();

        // Parse CSS properties (simple parser for common cases)
        let properties: Vec<&str> = css.split(';').collect();

        for prop in properties {
            let prop = prop.trim();
            if prop.is_empty() {
                continue;
            }

            if let Some((key, value)) = prop.split_once(':') {
                let key = key.trim().to_lowercase();
                let value = value.trim().to_lowercase();

                match key.as_str() {
                    "color" => {
                        if let Some(color_code) = Self::color_to_ansi(&value, false) {
                            ansi.push_str(&color_code);
                        }
                    }
                    "background-color" => {
                        if let Some(color_code) = Self::color_to_ansi(&value, true) {
                            ansi.push_str(&color_code);
                        }
                    }
                    "font-weight" => {
                        if value == "bold" || value.parse::<i32>().unwrap_or(400) >= 700 {
                            ansi.push_str("\x1b[1m");
                        }
                    }
                    "font-style" => {
                        if value == "italic" {
                            ansi.push_str("\x1b[3m");
                        }
                    }
                    "text-decoration" => {
                        if value.contains("underline") {
                            ansi.push_str("\x1b[4m");
                        }
                    }
                    _ => {} // Ignore unsupported properties
                }
            }
        }

        ansi
    }

    /// Convert color value to ANSI escape code
    fn color_to_ansi(color: &str, is_background: bool) -> Option<String> {
        let base = if is_background { 40 } else { 30 };
        let bright_base = if is_background { 100 } else { 90 };

        match color {
            "black" => Some(format!("\x1b[{base}m")),
            "red" => Some(format!("\x1b[{}m", base + 1)),
            "green" => Some(format!("\x1b[{}m", base + 2)),
            "yellow" => Some(format!("\x1b[{}m", base + 3)),
            "blue" => Some(format!("\x1b[{}m", base + 4)),
            "magenta" => Some(format!("\x1b[{}m", base + 5)),
            "cyan" => Some(format!("\x1b[{}m", base + 6)),
            "white" => Some(format!("\x1b[{}m", base + 7)),
            "gray" | "grey" => Some(format!("\x1b[{bright_base}m")),
            _ => {
                // Try to parse hex colors
                if color.starts_with('#')
                    && color.len() == 7
                    && let Ok(r) = u8::from_str_radix(&color[1..3], 16)
                    && let Ok(g) = u8::from_str_radix(&color[3..5], 16)
                    && let Ok(b) = u8::from_str_radix(&color[5..7], 16)
                {
                    let code = if is_background { 48 } else { 38 };
                    return Some(format!("\x1b[{code};2;{r};{g};{b}m"));
                }

                // Try to parse rgb() colors
                if color.starts_with("rgb(") && color.ends_with(')') {
                    let rgb_str = &color[4..color.len() - 1];
                    let parts: Vec<&str> = rgb_str.split(',').map(|s| s.trim()).collect();
                    if parts.len() == 3
                        && let (Ok(r), Ok(g), Ok(b)) = (
                            parts[0].parse::<u8>(),
                            parts[1].parse::<u8>(),
                            parts[2].parse::<u8>(),
                        )
                    {
                        let code = if is_background { 48 } else { 38 };
                        return Some(format!("\x1b[{code};2;{r};{g};{b}m"));
                    }
                }

                None
            }
        }
    }
}
