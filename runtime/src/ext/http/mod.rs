use andromeda_core::{Extension, ExtensionOp};
use nova_vm::{
    ecmascript::{
        builtins::ArgumentsList,
        execution::{Agent, JsResult},
        types::Value,
    },
    engine::context::GcScope,
};

#[derive(Default)]
pub struct ServeExt;

impl ServeExt {
    pub fn new_extension() -> Extension {
        Extension {
            name: "serve",
            ops: vec![ExtensionOp::new(
                "internal_serve",
                Self::internal_serve,
                0,
                false,
            )],
            storage: None,
            files: vec![],
        }
    }

    fn internal_serve<'gc>(
        agent: &mut Agent,
        _this: Value,
        _args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        println!("hello world");
        Ok(Value::from_string(
            agent,
            "Success".to_string(),
            gc.into_nogc(),
        ))
    }
}
