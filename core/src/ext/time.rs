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
pub struct TimeExt;

impl Ext for TimeExt {
    fn load(&self, mut loader: ExtLoader) {
        loader.load_op("internal_sleep", Self::internal_sleep, 1);
    }
}

impl TimeExt {
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
