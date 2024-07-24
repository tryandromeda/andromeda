use std::time::Duration;

use nova_vm::ecmascript::{
    builtins::{
        promise_objects::promise_abstract_operations::promise_capability_records::PromiseCapability,
        ArgumentsList,
    },
    execution::{Agent, JsResult},
    types::{GlobalValue, Value},
};

use crate::{
    ext_interface::{Ext, ExtLoader},
    HostData, MacroTask,
};

#[derive(Default)]
pub struct ConsolExt;

impl Ext for ConsolExt {
    fn load(&self, mut loader: ExtLoader) {
        loader.load_op("internal_read", Self::internal_read, 1);
        loader.load_op("internal_read_line", Self::internal_read_line, 1);
        loader.load_op("internal_write", Self::internal_write, 1);
        loader.load_op("internal_write_line", Self::internal_write_line, 1);
        loader.load_op("debug", Self::debug, 1);
        loader.load_op("internal_exit", Self::internal_exit, 1);
        loader.load_op("internal_sleep", Self::internal_sleep, 1); // TODO: Move to another ext
    }
}

impl ConsolExt {
    /// Debug function that prints the first argument to the console.
    fn debug(agent: &mut Agent, _this: Value, args: ArgumentsList) -> JsResult<Value> {
        if args.len() == 0 {
            println!();
        } else {
            println!("{}", args[0].to_string(agent)?.as_str(agent));
        }
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

    pub fn internal_sleep(agent: &mut Agent, _this: Value, args: ArgumentsList) -> JsResult<Value> {
        let promise_capability = PromiseCapability::new(agent);
        let time_ms = args[0].to_uint32(agent).unwrap();
        let duration = Duration::from_millis(time_ms as u64);

        let root_value = GlobalValue::new(agent, promise_capability.promise());
        let host_data = agent.get_host_data();
        let host_data: &HostData = host_data.downcast_ref().unwrap();
        let macro_task_tx = host_data.macro_task_tx();

        host_data.spawn_macro_task(async move {
            tokio::time::sleep(duration).await;
            macro_task_tx.send(MacroTask::ResolvePromise(root_value))
        });

        Ok(Value::Promise(promise_capability.promise()))
    }
}
