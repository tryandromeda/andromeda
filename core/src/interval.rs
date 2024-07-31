use std::{sync::atomic::Ordering, time::Duration};

use nova_vm::ecmascript::{
    builtins::ArgumentsList,
    execution::agent::{GcAgent, RealmRoot},
    types::{Function, Global, InternalMethods, Value},
};

use crate::{HostData, MacroTask, TaskId};

/// An Id representing an [Interval].
#[derive(Debug, PartialEq, Hash, Eq, Clone, Copy)]
pub struct IntervalId(u32);

impl IntervalId {
    pub fn index(&self) -> u32 {
        self.0
    }

    pub fn from_index(index: u32) -> Self {
        Self(index)
    }

    /// Request to clear the interval to the event loop.
    pub fn request_clear(self, host_data: &HostData) {
        let macro_task_tx = host_data.macro_task_tx();
        macro_task_tx.send(MacroTask::ClearInterval(self)).unwrap();
    }

    /// Remove and abort the interval.
    pub fn clear_and_abort(self, host_data: &HostData) {
        let interval = host_data.intervals.borrow_mut().remove(&self).unwrap();
        host_data.abort_macro_task(interval.task_id);
        host_data.clear_macro_task(interval.task_id);
    }

    /// Execute the Interval callback.
    pub fn run(self, agent: &mut GcAgent, host_data: &HostData, realm_root: &RealmRoot) {
        Interval::with(host_data, &self, |interval| {
            let global_callback = &interval.callback;
            agent.run_in_realm(realm_root, |agent| {
                let callback = global_callback.get(agent);
                let callback_function: Function = callback.try_into().unwrap();
                callback_function
                    .internal_call(agent, Value::Undefined, ArgumentsList(&[]))
                    .unwrap();
            });
        });
    }
}

#[derive(Debug, PartialEq)]
pub struct Interval {
    pub(crate) period: Duration,
    pub(crate) callback: Global<Value>,
    pub(crate) task_id: TaskId,
}

impl Interval {
    /// Create a new [Interval] and return its [IntervalId].
    pub fn create(
        host_data: &HostData,
        period: Duration,
        callback: Global<Value>,
        task_id: impl FnOnce(IntervalId) -> TaskId,
    ) -> IntervalId {
        let id = host_data.interval_count.fetch_add(1, Ordering::Relaxed);
        let interval_id = IntervalId(id);
        let task_id = task_id(interval_id);
        let interval = Self {
            period,
            callback,
            task_id,
        };

        host_data
            .intervals
            .borrow_mut()
            .insert(interval_id, interval);

        interval_id
    }

    /// Run a closure with a reference to the [Interval].
    pub fn with(host_data: &HostData, interval_id: &IntervalId, run: impl FnOnce(&Self)) {
        let intervals = host_data.intervals.borrow();
        let interval = intervals.get(interval_id).unwrap();
        run(interval);
    }
}
