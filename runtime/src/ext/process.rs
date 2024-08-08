use andromeda_core::{Extension, ExtensionOp};
use nova_vm::ecmascript::{
    builtins::{ArgumentsList, Array},
    execution::{Agent, JsResult},
    types::Value,
};
use std::env;

/// Process extension for Nova
/// This extension provides access to internal functions relating to the process.
#[derive(Default)]
pub struct ProcessExt;

impl ProcessExt {
    pub fn new_extension() -> Extension {
        Extension {
            name: "process",
            ops: vec![ExtensionOp::new(
                "internal_get_cli_args",
                Self::internal_get_cli_args,
                0,
            )],
            storage: None,
            files: vec![],
        }
    }

    fn internal_get_cli_args(agent: &mut Agent, _this: Value, _: ArgumentsList) -> JsResult<Value> {
        let args = env::args().skip(1).collect::<Vec<String>>();
        let args = args.iter().map(|s| s.as_str()).collect::<Vec<&str>>();
        let args = args
            .iter()
            .map(|s| nova_vm::ecmascript::types::String::from_string(agent, s.to_string()))
            .collect::<Vec<_>>();
        // TODO: This should be an array

        Ok(Array::from_slice(agent, &args).into())
    }
}
