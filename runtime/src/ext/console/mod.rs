// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::io::{Write, stdout};

use andromeda_core::{AndromedaError, ErrorReporter, Extension, ExtensionOp};
use nova_vm::{
    ecmascript::{
        builtins::ArgumentsList,
        execution::{Agent, JsResult},
        types::Value,
    },
    engine::context::{Bindable, GcScope},
};

#[derive(Default)]
pub struct ConsoleExt;

impl ConsoleExt {
    pub fn new_extension() -> Extension {
        Extension {
            name: "console",
            ops: vec![
                ExtensionOp::new("internal_read", Self::internal_read, 1),
                ExtensionOp::new("internal_read_line", Self::internal_read_line, 1),
                ExtensionOp::new("internal_write", Self::internal_write, 1),
                ExtensionOp::new("internal_write_line", Self::internal_write_line, 1),
                ExtensionOp::new("internal_print", Self::internal_print, 1),
                ExtensionOp::new("internal_exit", Self::internal_exit, 1),
            ],
            storage: None,
            files: vec![include_str!("./mod.ts")],
        }
    }
    /// Print function that prints the first argument to the console.
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
                .as_bytes(),
        ) {
            let error = AndromedaError::runtime_error(format!("Failed to write to stdout: {}", e));
            ErrorReporter::print_error(&error);
        }
        if let Err(e) = stdout().flush() {
            let error = AndromedaError::runtime_error(format!("Failed to flush stdout: {}", e));
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
                    AndromedaError::runtime_error(format!("Failed to read from stdin: {}", e));
                let error_msg = ErrorReporter::format_error(&error);
                Ok(Value::from_string(agent, format!("Error: {}", error_msg), gc.nogc()).unbind())
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
                    AndromedaError::runtime_error(format!("Failed to read line from stdin: {}", e));
                let error_msg = ErrorReporter::format_error(&error);
                Ok(Value::from_string(agent, format!("Error: {}", error_msg), gc.nogc()).unbind())
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
                arg.to_string(agent, gc.reborrow()).unbind()?.as_str(agent)
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
                arg.to_string(agent, gc.reborrow()).unbind()?.as_str(agent)
            );
        }
        println!();
        Ok(Value::Undefined)
    }
}
