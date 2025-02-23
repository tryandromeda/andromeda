use andromeda_core::{Extension, ExtensionOp};
use nova_vm::{
    ecmascript::{
        builtins::ArgumentsList,
        execution::{agent::ExceptionType, Agent, JsResult},
        types::Value,
    },
    engine::context::GcScope,
};

#[derive(Default)]
pub struct WebExt;

impl WebExt {
    pub fn new_extension() -> Extension {
        Extension {
            name: "web",
            ops: vec![ExtensionOp::new("internal_btoa", Self::internal_btoa, 1)],
            storage: None,
            files: vec![],
        }
    }

    pub fn internal_btoa(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'_, '_>,
    ) -> JsResult<Value> {
        let input = args.get(0).to_string(agent, gc.reborrow())?;
        let rust_string = input.as_str(agent).to_string();
        for c in rust_string.chars() {
            if c as u32 > 0xFF {
                // TODO: Returning an InvalidCharacterError is the correct behavior.
                // ref: https://html.spec.whatwg.org/multipage/webappapis.html#atob
                return Err(agent.throw_exception(ExceptionType::Error, format!(
                    "InvalidCharacterError: The string to be encoded contains characters outside of the Latin1 range. Found: '{}'",
                    c
                ), gc.nogc()));
            }
        }
        Ok(Self::forgiving_base64_encode(
            agent,
            rust_string.as_bytes(),
            gc,
        ))
    }

    /// See <https://infra.spec.whatwg.org/#forgiving-base64>
    pub fn forgiving_base64_encode(agent: &mut Agent, s: &[u8], gc: GcScope<'_, '_>) -> Value {
        let encoded_str = base64_simd::STANDARD.encode_to_string(s);
        Value::from_string(agent, encoded_str, gc.nogc())
    }
}
