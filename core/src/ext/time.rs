use std::time::Duration;

use nova_vm::ecmascript::{
    builtins::{
        promise_objects::promise_abstract_operations::promise_capability_records::PromiseCapability,
        ArgumentsList,
    },
    execution::{Agent, JsResult},
    types::{Global, IntoValue, Value},
};
use tokio::time::interval;

use crate::{
    ext_interface::{Ext, ExtLoader},
    HostData, Interval, IntervalId, MacroTask, Timeout, TimeoutId,
};

#[derive(Default)]
pub struct TimeExt;

impl Ext for TimeExt {
    fn load(&self, mut loader: ExtLoader) {
        loader.load_op("internal_sleep", Self::internal_sleep, 1);
        loader.load_op("setInterval", Self::set_interval, 2);
        loader.load_op("clearInterval", Self::clear_interval, 1);
        loader.load_op("setTimeout", Self::set_timeout, 2);
        loader.load_op("clearTimeout", Self::clear_timeout, 1);
    }
}

impl TimeExt {
    pub fn internal_sleep(agent: &mut Agent, _this: Value, args: ArgumentsList) -> JsResult<Value> {
        let promise_capability = PromiseCapability::new(agent);
        let time_ms = args[0].to_uint32(agent).unwrap();
        let duration = Duration::from_millis(time_ms as u64);

        let root_value = Global::new(agent, promise_capability.promise().into_value());
        let host_data = agent.get_host_data();
        let host_data: &HostData = host_data.downcast_ref().unwrap();
        let macro_task_tx = host_data.macro_task_tx();

        host_data.spawn_macro_task(async move {
            tokio::time::sleep(duration).await;
            macro_task_tx.send(MacroTask::ResolvePromise(root_value))
        });

        Ok(Value::Promise(promise_capability.promise()))
    }

    pub fn set_interval(agent: &mut Agent, _this: Value, args: ArgumentsList) -> JsResult<Value> {
        let callback = args[0];
        let time_ms = args[1].to_uint32(agent).unwrap();
        let period = Duration::from_millis(time_ms as u64);

        let root_callback = Global::new(agent, callback);
        let host_data = agent.get_host_data();
        let host_data: &HostData = host_data.downcast_ref().unwrap();
        let macro_task_tx = host_data.macro_task_tx();

        let interval_id = Interval::create(host_data, period, root_callback, |interval_id| {
            host_data.spawn_macro_task(async move {
                let mut interval = interval(period);
                loop {
                    interval.tick().await;
                    macro_task_tx
                        .send(MacroTask::RunInterval(interval_id))
                        .unwrap();
                }
            })
        });

        let interval_id_value = Value::from_f64(agent, interval_id.index() as f64);

        Ok(interval_id_value)
    }

    pub fn clear_interval(agent: &mut Agent, _this: Value, args: ArgumentsList) -> JsResult<Value> {
        let interval_id_value = args[0];
        let interval_id_u32 = interval_id_value.to_uint32(agent).unwrap();
        let interval_id = IntervalId::from_index(interval_id_u32);

        let host_data = agent.get_host_data();
        let host_data: &HostData = host_data.downcast_ref().unwrap();

        interval_id.request_clear(host_data);

        Ok(Value::Undefined)
    }

    pub fn set_timeout(agent: &mut Agent, _this: Value, args: ArgumentsList) -> JsResult<Value> {
        let callback = args[0];
        let time_ms = args[1].to_uint32(agent).unwrap();
        let duration = Duration::from_millis(time_ms as u64);

        let root_callback = Global::new(agent, callback);
        let host_data = agent.get_host_data();
        let host_data: &HostData = host_data.downcast_ref().unwrap();
        let macro_task_tx = host_data.macro_task_tx();

        let timeout_id = Timeout::create(host_data, duration, root_callback, |timeout_id| {
            host_data.spawn_macro_task(async move {
                tokio::time::sleep(duration).await;
                macro_task_tx
                    .send(MacroTask::RunAndClearTimeout(timeout_id))
                    .unwrap();
            })
        });

        let timeout_id_value = Value::from_f64(agent, timeout_id.index() as f64);

        Ok(timeout_id_value)
    }

    pub fn clear_timeout(agent: &mut Agent, _this: Value, args: ArgumentsList) -> JsResult<Value> {
        let timeout_id_value = args[0];
        let timeout_id_u32 = timeout_id_value.to_uint32(agent).unwrap();
        let timeout_id = TimeoutId::from_index(timeout_id_u32);

        let host_data = agent.get_host_data();
        let host_data: &HostData = host_data.downcast_ref().unwrap();

        timeout_id.request_clear(host_data);

        Ok(Value::Undefined)
    }
}
