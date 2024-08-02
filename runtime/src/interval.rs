use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc,
    },
    time::Duration,
};

use andromeda_core::{HostData, TaskId};
use nova_vm::ecmascript::{
    execution::agent::{GcAgent, RealmRoot},
    types::{Function, Global, Value},
};

use crate::RuntimeMacroTask;

#[derive(Default)]
pub struct IntervalsStorage {
    intervals: HashMap<IntervalId, Interval>,

    count: Arc<AtomicU32>,
}

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

    /// Remove and abort the interval.
    pub fn clear_and_abort(self, host_data: &HostData<RuntimeMacroTask>) {
        let mut host_data_storage = host_data.storage.borrow_mut();
        let intervals_storage: &mut IntervalsStorage = host_data_storage.get_mut().unwrap();
        let interval = intervals_storage.intervals.remove(&self).unwrap();
        host_data.abort_macro_task(interval.task_id);
        host_data.clear_macro_task(interval.task_id);
    }

    /// Execute the Interval callback.
    pub fn run(
        self,
        agent: &mut GcAgent,
        host_data: &HostData<RuntimeMacroTask>,
        realm_root: &RealmRoot,
    ) {
        Interval::with(host_data, &self, |interval| {
            let global_callback = &interval.callback;
            agent.run_in_realm(realm_root, |agent| {
                let callback = global_callback.get(agent);
                let callback_function: Function = callback.try_into().unwrap();
                callback_function
                    .call(agent, Value::Undefined, &[])
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
        host_data: &HostData<RuntimeMacroTask>,
        period: Duration,
        callback: Global<Value>,
        task_id: impl FnOnce(IntervalId) -> TaskId,
    ) -> IntervalId {
        let mut host_data_storage = host_data.storage.borrow_mut();
        let intervals_storage: &mut IntervalsStorage = host_data_storage.get_mut().unwrap();
        let id = intervals_storage.count.fetch_add(1, Ordering::Relaxed);
        let interval_id = IntervalId(id);
        let task_id = task_id(interval_id);
        let interval = Self {
            period,
            callback,
            task_id,
        };

        intervals_storage.intervals.insert(interval_id, interval);

        interval_id
    }

    /// Run a closure with a reference to the [Interval].
    pub fn with(
        host_data: &HostData<RuntimeMacroTask>,
        interval_id: &IntervalId,
        run: impl FnOnce(&Self),
    ) {
        let mut host_data_storage = host_data.storage.borrow_mut();
        let intervals_storage: &mut IntervalsStorage = host_data_storage.get_mut().unwrap();
        let interval = intervals_storage.intervals.get(interval_id).unwrap();
        run(interval);
    }
}
