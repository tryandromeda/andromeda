// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::RuntimeMacroTask;
use andromeda_core::{Extension, ExtensionOp, HostData, MacroTask, OpsStorage};
use nova_vm::{
    ecmascript::{
        builtins::ArgumentsList,
        execution::{Agent, JsResult},
        types::Value,
    },
    engine::{
        Global,
        context::{Bindable, GcScope},
    },
};
use std::collections::HashMap;
use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CronError {
    #[error("Cron name cannot exceed 64 characters: current length {0}")]
    NameExceeded(usize),
    #[error(
        "Invalid cron name: only alphanumeric characters, whitespace, hyphens, and underscores are allowed"
    )]
    NameInvalid,
    #[error("Cron with this name already exists")]
    AlreadyExists,
    #[error("Too many crons")]
    TooManyCrons,
    #[error("Invalid cron schedule")]
    InvalidCron,
    #[error("Invalid backoff schedule")]
    InvalidBackoff,
    #[error("Acquire error: {0}")]
    AcquireError(#[from] tokio::sync::AcquireError),
    #[error("Other error: {0}")]
    Other(String),
}

/// Specification for a cron job containing schedule and retry configuration
#[derive(Clone, Debug)]
pub struct CronSpec {
    /// Name of the cron job
    pub name: String,
    /// Cron schedule expression (e.g., "0 0 * * *")
    pub cron_schedule: String,
    /// Optional backoff schedule for retries (milliseconds)
    pub backoff_schedule: Option<Vec<u32>>,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct CronId(u32);

impl CronId {
    pub fn from_index(index: u32) -> Self {
        Self(index)
    }

    pub fn index(self) -> u32 {
        self.0
    }

    pub fn run(
        &self,
        agent: &mut nova_vm::ecmascript::execution::agent::GcAgent,
        host_data: &HostData<RuntimeMacroTask>,
        realm_root: &nova_vm::ecmascript::execution::agent::RealmRoot,
    ) {
        let storage = host_data.storage.borrow();
        if let Some(crons_storage) = storage.get::<CronsStorage>()
            && let Some(cron_job) = crons_storage.get_cron(self)
        {
            cron_job.run(agent, host_data, realm_root);
        }
    }

    pub fn clear_and_abort(&self, host_data: &HostData<RuntimeMacroTask>) {
        let mut storage = host_data.storage.borrow_mut();
        if let Some(crons_storage) = storage.get_mut::<CronsStorage>() {
            crons_storage.crons.remove(self);
        }
    }
}

pub struct CronJob {
    pub id: CronId,
    pub spec: CronSpec,
    pub callback: Global<Value<'static>>,
    pub next_deadline: Option<u64>,
    pub success: bool,
    pub retries: u32,
}

impl CronJob {
    pub fn run(
        &self,
        agent: &mut nova_vm::ecmascript::execution::agent::GcAgent,
        _host_data: &HostData<RuntimeMacroTask>,
        realm_root: &nova_vm::ecmascript::execution::agent::RealmRoot,
    ) {
        let global_callback = &self.callback;
        agent.run_in_realm(realm_root, |agent, mut gc| {
            let callback = global_callback.get(agent, gc.nogc());
            let callback_function: nova_vm::ecmascript::types::Function =
                callback.try_into().unwrap();
            callback_function
                .call(agent, Value::Undefined, &mut [], gc.reborrow())
                .unwrap();
        });
    }

    pub fn clear_and_abort(&self, host_data: &HostData<RuntimeMacroTask>) {
        let mut storage = host_data.storage.borrow_mut();
        let crons_storage: &mut CronsStorage = storage.get_mut().unwrap();
        crons_storage.crons.remove(&self.id);
    }
}

#[derive(Default)]
pub struct CronsStorage {
    pub crons: HashMap<CronId, CronJob>,
    pub next_id: u32,
}

impl CronsStorage {
    pub fn create_cron(&mut self, spec: CronSpec, callback: Global<Value<'static>>) -> CronId {
        let id = CronId(self.next_id);
        self.next_id += 1;

        let cron_job = CronJob {
            id,
            spec,
            callback,
            next_deadline: None,
            success: true,
            retries: 0,
        };

        self.crons.insert(id, cron_job);
        id
    }

    pub fn get_cron(&self, id: &CronId) -> Option<&CronJob> {
        self.crons.get(id)
    }

    pub fn get_cron_mut(&mut self, id: &CronId) -> Option<&mut CronJob> {
        self.crons.get_mut(id)
    }
}

#[derive(Default)]
pub struct CronExt;

impl CronExt {
    pub fn new_extension() -> Extension {
        Extension {
            name: "cron",
            ops: vec![ExtensionOp::new("cron", Self::cron, 3, false)],
            storage: Some(Box::new(|storage: &mut OpsStorage| {
                storage.insert(CronsStorage::default());
            })),
            files: vec![],
        }
    }

    pub fn cron<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        if args.len() < 3 {
            // Return undefined if arguments are insufficient
            return Ok(Value::Undefined);
        }

        // Validate name
        let name = args[0]
            .to_string(agent, gc.reborrow())
            .unbind()?
            .as_str(agent)
            .expect("String is not valid UTF-8")
            .to_string();

        if name.is_empty() {
            return Ok(Value::Undefined);
        }

        if name.len() > 64 {
            return Ok(Value::Undefined);
        }

        if !name
            .chars()
            .all(|c| c.is_ascii_whitespace() || c.is_ascii_alphanumeric() || c == '_' || c == '-')
        {
            return Ok(Value::Undefined);
        }

        // Validate schedule
        let schedule = args[1]
            .to_string(agent, gc.reborrow())
            .unbind()?
            .as_str(agent)
            .expect("String is not valid UTF-8")
            .to_string();

        // Validate cron expression
        if schedule.parse::<saffron::Cron>().is_err() {
            return Ok(Value::Undefined);
        }

        // Validate handler
        let handler_value = args[2];
        if !handler_value.is_function() {
            return Ok(Value::Undefined);
        }

        let callback = Global::new(agent, handler_value.unbind());
        let host_data = agent.get_host_data();
        let host_data: &HostData<RuntimeMacroTask> = host_data.downcast_ref().unwrap();

        let spec = CronSpec {
            name: name.clone(),
            cron_schedule: schedule,
            backoff_schedule: None,
        };

        // Create cron job
        let mut storage = host_data.storage.borrow_mut();
        let crons_storage: &mut CronsStorage = storage.get_mut().unwrap();
        let cron_id = crons_storage.create_cron(spec.clone(), callback);
        drop(storage);

        // Start the cron job scheduling
        let macro_task_tx = host_data.macro_task_tx();
        let cron_schedule = spec.cron_schedule.clone();

        host_data.spawn_macro_task(async move {
            let cron = cron_schedule.parse::<saffron::Cron>().unwrap();

            loop {
                let now = chrono::Utc::now();
                if let Some(next_deadline) = cron.next_after(now) {
                    let duration =
                        (next_deadline.timestamp_millis() - now.timestamp_millis()) as u64;

                    tokio::time::sleep(Duration::from_millis(duration)).await;

                    let _ = macro_task_tx.send(MacroTask::User(RuntimeMacroTask::RunCron(cron_id)));
                } else {
                    break;
                }
            }
        });

        // Return undefined - cron job is registered
        Ok(Value::Undefined)
    }
}
