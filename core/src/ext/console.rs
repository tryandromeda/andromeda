use std::io::{stdout, Write};

use nova_vm::ecmascript::{
    builtins::ArgumentsList,
    execution::{Agent, JsResult},
    types::Value,
};

use crate::ext_interface::{Ext, ExtLoader};

#[derive(Default)]
pub struct ConsoleExt;

impl Ext for ConsoleExt {
    fn load(&self, mut loader: ExtLoader) {
        loader.load_op("internal_read", Self::internal_read, 1);
        loader.load_op("internal_read_line", Self::internal_read_line, 1);
        loader.load_op("internal_write", Self::internal_write, 1);
        loader.load_op("internal_write_line", Self::internal_write_line, 1);
        loader.load_op("internal_print", Self::internal_print, 1);
        loader.load_op("internal_exit", Self::internal_exit, 1);
    }
}

impl ConsoleExt {
    /// Print function that prints the first argument to the console.
    fn internal_print(agent: &mut Agent, _this: Value, args: ArgumentsList) -> JsResult<Value> {
        stdout()
            .write_all(args[0].to_string(agent)?.as_str(agent).as_bytes())
            .unwrap();
        stdout().flush().unwrap();
        Ok(Value::Undefined)
    }

    /// Exit the process with the given exit code.
    pub fn internal_exit(agent: &mut Agent, _this: Value, args: ArgumentsList) -> JsResult<Value> {
        std::process::exit(args[0].to_int32(agent)?);
    }

    /// Internal read for reading from the console.
    pub fn internal_read(agent: &mut Agent, _this: Value, _args: ArgumentsList) -> JsResult<Value> {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        Ok(Value::from_string(agent, input.trim_end().to_string()))
    }

    /// Internal read line for reading from the console with a newline.
    pub fn internal_read_line(
        agent: &mut Agent,
        _this: Value,
        _args: ArgumentsList,
    ) -> JsResult<Value> {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        Ok(Value::from_string(agent, input))
    }

    /// Internal write for writing to the console.
    pub fn internal_write(agent: &mut Agent, _this: Value, args: ArgumentsList) -> JsResult<Value> {
        for arg in args.iter() {
            print!("{}", arg.to_string(agent)?.as_str(agent));
        }
        Ok(Value::Undefined)
    }

    /// Internal write line for writing to the console with a newline.
    pub fn internal_write_line(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
    ) -> JsResult<Value> {
        for arg in args.iter() {
            print!("{}", arg.to_string(agent)?.as_str(agent));
        }
        println!();
        Ok(Value::Undefined)
    }
}
