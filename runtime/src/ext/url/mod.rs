use andromeda_core::{Extension, ExtensionOp};

use nova_vm::{
    ecmascript::{
        builtins::ArgumentsList,
        execution::{Agent, JsResult},
        types::Value,
        // types::Value,
    },
    engine::context::GcScope,
};
use url::Url;

#[derive(Default)]
pub struct URLExt;

impl URLExt {
    pub fn new_extension() -> Extension {
        Extension {
            name: "url",
            ops: vec![
                ExtensionOp::new("internal_url_parse", Self::internal_parse, 2),
                ExtensionOp::new(
                    "internal_url_parse_no_base",
                    Self::internal_parse_no_base,
                    1,
                ),
            ],
            storage: None,
            files: vec![include_str!("./mod.ts")],
        }
    }

    fn internal_parse(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        gc: &mut GcScope<'_, '_>,
    ) -> JsResult<Value> {
        let url = args.get(0).to_string(agent, gc.reborrow())?;

        let base_href = args.get(1).to_string(agent, gc.reborrow())?;

        let base_url = match Url::parse(base_href.as_str(agent)) {
            Ok(url) => url,
            Err(e) => {
                return Ok(Value::from_string(
                    agent,
                    format!("Error: {}", e),
                    gc.nogc(),
                ));
            }
        };

        let url = match base_url.join(url.as_str(agent)) {
            Ok(url) => url,
            Err(e) => {
                return Ok(Value::from_string(
                    agent,
                    format!("Error: {}", e),
                    gc.nogc(),
                ));
            }
        };

        Ok(Value::from_string(agent, url.to_string(), gc.nogc()))
    }

    fn internal_parse_no_base(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        gc: &mut GcScope<'_, '_>,
    ) -> JsResult<Value> {
        let url = args.get(0).to_string(agent, gc.reborrow())?;

        let url = match Url::parse(url.as_str(agent)) {
            Ok(url) => url,
            Err(e) => {
                return Ok(Value::from_string(
                    agent,
                    format!("Error: {}", e),
                    gc.nogc(),
                ));
            }
        };

        Ok(Value::from_string(agent, url.to_string(), gc.nogc()))
    }
}
