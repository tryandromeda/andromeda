use std::{sync::atomic::Ordering, time::Duration};

use nova_vm::ecmascript::{
    builtins::ArgumentsList,
    execution::agent::{GcAgent, RealmRoot},
    types::{Function, Global, InternalMethods, Value},
};

use crate::{HostData, TaskId};

/// An Id representing a [Timeout].
#[derive(Debug, PartialEq, Hash, Eq, Clone, Copy)]
pub struct TimeoutId(u32);

impl TimeoutId {
    pub fn index(&self) -> u32 {
        self.0
    }

    pub fn from_index(index: u32) -> Self {
        Self(index)
    }

    /// Remove the timeout.
    pub fn clear(self, host_data: &HostData) {
        let timeout = host_data.timeouts.borrow_mut().remove(&self).unwrap();
        host_data.clear_macro_task(timeout.task_id);
    }

    /// Remove and abort the timeout.
    pub fn clear_and_abort(self, host_data: &HostData) {
        let timeout = host_data.timeouts.borrow_mut().remove(&self).unwrap();
        host_data.abort_macro_task(timeout.task_id);
        host_data.clear_macro_task(timeout.task_id);
    }

    /// Execute the Timeout callback.
    pub fn run_and_clear(self, agent: &mut GcAgent, host_data: &HostData, realm_root: &RealmRoot) {
        Timeout::with(host_data, &self, |timeout| {
            let global_callback = &timeout.callback;
            agent.run_in_realm(realm_root, |agent| {
                let callback = global_callback.get(agent);
                let callback_function: Function = callback.try_into().unwrap();
                callback_function
                    .internal_call(agent, Value::Undefined, ArgumentsList(&[]))
                    .unwrap();
            });
        });
        self.clear(host_data);
    }
}

#[derive(Debug, PartialEq)]
pub struct Timeout {
    pub(crate) period: Duration,
    pub(crate) callback: Global<Value>,
    pub(crate) task_id: TaskId,
}

impl Timeout {
    /// Create a new [Timeout] and return its [TimeoutId].
    pub fn create(
        host_data: &HostData,
        period: Duration,
        callback: Global<Value>,
        task_id: impl FnOnce(TimeoutId) -> TaskId,
    ) -> TimeoutId {
        let id = host_data.timeout_count.fetch_add(1, Ordering::Relaxed);
        let timeout_id = TimeoutId(id);
        let task_id = task_id(timeout_id);
        let timeout = Self {
            period,
            callback,
            task_id,
        };

        host_data.timeouts.borrow_mut().insert(timeout_id, timeout);

        timeout_id
    }

    /// Run a closure with a reference to the [Timeout].
    pub fn with(host_data: &HostData, timeout_id: &TimeoutId, run: impl FnOnce(&Self)) {
        let timeouts = host_data.timeouts.borrow();
        let timeout = timeouts.get(timeout_id).unwrap();
        run(timeout);
    }
}
