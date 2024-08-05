use std::env;
use andromeda_core::{Extension, ExtensionOp};
use nova_vm::ecmascript::{
    builtins::ArgumentsList,
    execution::{Agent, JsResult},
    types::Value,
};

#[derive(Default)]
pub struct CLIExt;

impl CLIExt {
    pub fn new_extension() -> Extension {
        Extension {
            name: "cli",
            ops: vec![
                ExtensionOp::new("internal_get_cli_args", Self::internal_get_cli_args, 0),
            ],
            storage: None,
        }
    }

    fn internal_get_cli_args(agent: &mut Agent, _this: Value, _: ArgumentsList) -> JsResult<Value> {
        let args = env::args().skip(1).collect::<Vec<String>>();
        // TODO: This should be an array
        Ok(Value::from_string(agent, args[0].clone()))
    }

    
}
