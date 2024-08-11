use andromeda_core::{Extension, ExtensionOp};
use nova_vm::ecmascript::{
    builtins::{ArgumentsList, Array},
    execution::{Agent, JsResult},
    types::Value,
};
use std::env;

/// Process extension for Andromeda.
/// This extension provides access to internal functions relating to the process.
#[derive(Default)]
pub struct ProcessExt;

impl ProcessExt {
    pub fn new_extension() -> Extension {
        Extension {
            name: "process",
            ops: vec![
                ExtensionOp::new("internal_get_cli_args", Self::internal_get_cli_args, 0),
                ExtensionOp::new("internal_get_env", Self::internal_get_env, 1),
                ExtensionOp::new("internal_set_env", Self::internal_set_env, 2),
                ExtensionOp::new("internal_delete_env", Self::internal_delete_env, 1),
                ExtensionOp::new("internal_get_env_keys", Self::internal_get_env_keys, 0),
            ],
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

        Ok(Array::from_slice(agent, &args).into())
    }

    fn internal_get_env(agent: &mut Agent, _this: Value, args: ArgumentsList) -> JsResult<Value> {
        let key = args.get(0);
        let key = key.to_string(agent)?;

        let value = env::var(key.as_str(agent)).unwrap_or_default();
        Ok(nova_vm::ecmascript::types::String::from_string(agent, value).into())
    }

    fn internal_set_env(agent: &mut Agent, _this: Value, args: ArgumentsList) -> JsResult<Value> {
        let key = args.get(0);
        let key = key.to_string(agent)?;

        let value = args.get(1);
        let value = value.to_string(agent)?;

        env::set_var(key.as_str(agent), value.as_str(agent));

        Ok(Value::Undefined)
    }

    fn internal_delete_env(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
    ) -> JsResult<Value> {
        let key = args.get(0);
        let key = key.to_string(agent)?;

        env::remove_var(key.as_str(agent));

        Ok(Value::Undefined)
    }

    fn internal_get_env_keys(agent: &mut Agent, _this: Value, _: ArgumentsList) -> JsResult<Value> {
        let keys = env::vars()
            .map(|(k, _)| k)
            .map(|s| nova_vm::ecmascript::types::String::from_string(agent, s))
            .collect::<Vec<_>>();

        Ok(Array::from_slice(agent, &keys).into())
    }
}
