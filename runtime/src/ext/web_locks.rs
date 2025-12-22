// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::{
    collections::{HashMap, VecDeque},
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
        mpsc::Sender,
    },
    time::Instant,
};

use andromeda_core::{Extension, ExtensionOp, HostData, MacroTask, OpsStorage, TaskId};
use nova_vm::{
    ecmascript::{
        builtins::ArgumentsList,
        execution::{Agent, JsResult},
        types::{IntoValue, Value},
    },
    engine::{
        Global,
        context::{Bindable, GcScope},
    },
};

use crate::RuntimeMacroTask;

/// Parameters for spawning an acquire task
struct AcquireTaskParams<'gc> {
    promise: Value<'gc>,
    lock_id: u64,
    name: String,
    mode: LockMode,
    macro_task_tx: Sender<MacroTask<RuntimeMacroTask>>,
}

/// Parameters for requesting a lock
struct RequestLockParams<'gc> {
    name: String,
    mode: LockMode,
    if_available: bool,
    steal: bool,
    promise: Option<Value<'gc>>,
    macro_task_tx: Sender<MacroTask<RuntimeMacroTask>>,
}

// Lock modes as defined by the Web Locks API
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LockMode {
    Exclusive,
    Shared,
}

impl std::fmt::Display for LockMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LockMode::Exclusive => write!(f, "exclusive"),
            LockMode::Shared => write!(f, "shared"),
        }
    }
}

// Simplified lock request for queuing
#[derive(Debug)]
pub struct LockRequest {
    pub id: u64,
    pub name: String,
    pub mode: LockMode,
    pub request_time: Instant,
    pub aborted: bool,           // Track if this request has been aborted
    pub task_id: Option<TaskId>, // Associated task for async operations
}

// Represents an active lock
#[derive(Debug, Clone)]
pub struct ActiveLock {
    pub id: u64,
    pub name: String,
    pub mode: LockMode,
    pub acquired_time: Instant,
}

// State for a specific lock name
#[derive(Debug)]
struct LockState {
    // Currently held locks (exclusive: max 1, shared: multiple allowed)
    active_locks: Vec<ActiveLock>,
    // Queue of pending requests
    pending_requests: VecDeque<LockRequest>,
}

impl LockState {
    fn new() -> Self {
        Self {
            active_locks: Vec::new(),
            pending_requests: VecDeque::new(),
        }
    }

    fn has_exclusive_lock(&self) -> bool {
        self.active_locks
            .iter()
            .any(|lock| lock.mode == LockMode::Exclusive)
    }

    fn can_acquire(&self, mode: LockMode) -> bool {
        if self.active_locks.is_empty() {
            return true;
        }

        match mode {
            LockMode::Exclusive => self.active_locks.is_empty(),
            LockMode::Shared => !self.has_exclusive_lock(),
        }
    }

    fn add_active_lock(&mut self, lock: ActiveLock) {
        self.active_locks.push(lock);
    }

    fn remove_active_lock(&mut self, lock_id: u64) -> Option<ActiveLock> {
        if let Some(pos) = self.active_locks.iter().position(|lock| lock.id == lock_id) {
            Some(self.active_locks.remove(pos))
        } else {
            None
        }
    }
}

// Global lock manager state with task system integration
pub struct WebLocksManager {
    // Maps lock names to their state
    locks: std::sync::Mutex<HashMap<String, LockState>>,
    // Counter for generating unique lock IDs
    next_lock_id: AtomicU64,
}

impl WebLocksManager {
    fn new() -> Self {
        Self {
            locks: std::sync::Mutex::new(HashMap::new()),
            next_lock_id: AtomicU64::new(1),
        }
    }

    fn next_id(&self) -> u64 {
        self.next_lock_id.fetch_add(1, Ordering::SeqCst)
    }

    /// Spawn a task to acquire a lock asynchronously
    fn spawn_acquire_task<'gc>(
        &self,
        agent: &mut Agent,
        params: AcquireTaskParams<'gc>,
    ) -> Result<(), String> {
        let promise_global = Global::new(agent, params.promise.into_value().unbind());
        let task = MacroTask::User(RuntimeMacroTask::AcquireLock {
            promise: promise_global,
            lock_id: params.lock_id,
            name: params.name,
            mode: params.mode,
        });
        params
            .macro_task_tx
            .send(task)
            .map_err(|_| "Failed to send acquire task to event loop".to_string())?;
        Ok(())
    }

    /// Spawn a task to release a lock and process pending requests
    fn spawn_release_task(
        &self,
        name: String,
        lock_id: u64,
        macro_task_tx: &Sender<MacroTask<RuntimeMacroTask>>,
    ) -> Result<(), String> {
        let task = MacroTask::User(RuntimeMacroTask::ReleaseLock { name, lock_id });
        macro_task_tx
            .send(task)
            .map_err(|_| "Failed to send release task to event loop".to_string())?;
        Ok(())
    }

    /// Spawn a task to abort a lock request
    fn spawn_abort_task(
        &self,
        name: String,
        lock_id: u64,
        macro_task_tx: &Sender<MacroTask<RuntimeMacroTask>>,
    ) -> Result<(), String> {
        let task = MacroTask::User(RuntimeMacroTask::AbortLockRequest { name, lock_id });
        macro_task_tx
            .send(task)
            .map_err(|_| "Failed to send abort task to event loop".to_string())?;
        Ok(())
    }

    /// Request lock with direct macro_task_tx to avoid borrowing conflicts
    fn request_lock_with_tx<'gc>(
        &self,
        agent: &mut Agent,
        params: RequestLockParams<'gc>,
    ) -> Result<u64, String> {
        let mut locks = self.locks.lock().unwrap();
        let lock_state = locks
            .entry(params.name.clone())
            .or_insert_with(LockState::new);

        // Handle "steal" option - preempt existing locks
        if params.steal {
            // Clear all existing locks and pending requests
            lock_state.active_locks.clear();
            lock_state.pending_requests.clear();

            // Grant this request immediately, regardless of any other conditions
            let lock_id = self.next_id();
            let active_lock = ActiveLock {
                id: lock_id,
                name: params.name.clone(),
                mode: params.mode,
                acquired_time: Instant::now(),
            };

            lock_state.add_active_lock(active_lock);
            return Ok(lock_id);
        }

        // Check if we can acquire immediately
        if lock_state.can_acquire(params.mode) {
            let lock_id = self.next_id();
            let active_lock = ActiveLock {
                id: lock_id,
                name: params.name.clone(),
                mode: params.mode,
                acquired_time: Instant::now(),
            };

            lock_state.add_active_lock(active_lock);
            return Ok(lock_id);
        }

        // Check "ifAvailable" option - fail if not immediately available
        if params.if_available {
            return Err("Lock not available and ifAvailable was specified".to_string());
        }

        // Queue the request for async processing
        let lock_id = self.next_id();
        let request = LockRequest {
            id: lock_id,
            name: params.name.clone(),
            mode: params.mode,
            request_time: Instant::now(),
            aborted: false,
            task_id: None, // Will be set when we implement proper task tracking
        };

        // Add to the pending queue
        lock_state.pending_requests.push_back(request);

        // If we have a promise, spawn an async task to handle it
        if let Some(promise_value) = params.promise {
            self.spawn_acquire_task(
                agent,
                AcquireTaskParams {
                    promise: promise_value,
                    lock_id,
                    name: params.name.clone(),
                    mode: params.mode,
                    macro_task_tx: params.macro_task_tx.clone(),
                },
            )?;
        }

        // Return the lock ID
        Ok(lock_id)
    }

    fn process_next_pending_request(&self, _name: &str, lock_state: &mut LockState) {
        // Check if there are any pending requests
        while !lock_state.pending_requests.is_empty() {
            // Look at the next request but don't remove it yet
            if let Some(next_request) = lock_state.pending_requests.front() {
                // Skip if the request has been aborted
                if next_request.aborted {
                    lock_state.pending_requests.pop_front();
                    continue;
                }

                // Check if it can be granted based on the mode
                if lock_state.can_acquire(next_request.mode) {
                    // Remove from pending queue
                    let request = lock_state.pending_requests.pop_front().unwrap();

                    // Create and add the active lock
                    let active_lock = ActiveLock {
                        id: request.id,
                        name: request.name.clone(),
                        mode: request.mode,
                        acquired_time: Instant::now(),
                    };

                    lock_state.add_active_lock(active_lock);

                    // TODO: Wake the task associated with this lock request
                    // This would involve resolving the promise that was created when the lock was requested
                    // For now, we just grant the lock synchronously

                    // If we granted a shared lock, we can potentially grant more
                    if request.mode == LockMode::Exclusive {
                        break; // Stop after granting an exclusive lock
                    }
                    // For shared locks, continue to the next iteration to try granting more
                } else {
                    break; // Can't grant the next request
                }
            } else {
                break; // No more pending requests
            }
        }
    }

    /// Release lock with direct macro_task_tx to avoid borrowing conflicts
    fn release_lock_with_tx(
        &self,
        name: &str,
        lock_id: u64,
        macro_task_tx: Sender<MacroTask<RuntimeMacroTask>>,
    ) -> Result<(), String> {
        let mut locks = self.locks.lock().unwrap();

        if let Some(lock_state) = locks.get_mut(name) {
            if lock_state.remove_active_lock(lock_id).is_some() {
                // Process next pending request if any and spawn wake tasks
                self.process_next_pending_request(name, lock_state);

                // Also spawn a release task to handle any follow-up processing
                drop(locks); // Release the mutex before spawning tasks
                self.spawn_release_task(name.to_string(), lock_id, &macro_task_tx)?;
                Ok(())
            } else {
                Err(format!("Lock with ID {} not found", lock_id))
            }
        } else {
            Err(format!("No locks found for name '{}'", name))
        }
    }

    /// Abort request with direct macro_task_tx to avoid borrowing conflicts
    fn abort_request_with_tx(
        &self,
        name: &str,
        lock_id: u64,
        macro_task_tx: Sender<MacroTask<RuntimeMacroTask>>,
    ) -> Result<(), String> {
        let mut locks = self.locks.lock().unwrap();

        if let Some(lock_state) = locks.get_mut(name) {
            // Check if the lock is already active
            if lock_state
                .active_locks
                .iter()
                .any(|lock| lock.id == lock_id)
            {
                // If it's already active, we'll release it
                drop(locks); // Release mutex before calling release_lock_with_tx
                return self.release_lock_with_tx(name, lock_id, macro_task_tx);
            }

            // Check if it's in the pending queue and get the task_id
            let mut task_id = None;
            for request in &mut lock_state.pending_requests {
                if request.id == lock_id {
                    // Mark it as aborted
                    request.aborted = true;
                    task_id = request.task_id;
                    break;
                }
            }

            // Release the mutex before spawning abort task
            drop(locks);

            if task_id.is_some() {
                // Spawn an abort task to handle async cancellation
                self.spawn_abort_task(name.to_string(), lock_id, &macro_task_tx)?;
                Ok(())
            } else {
                Err(format!("Lock request with ID {} not found", lock_id))
            }
        } else {
            Err(format!("No locks found for name '{}'", name))
        }
    }

    fn query_locks(&self) -> String {
        let locks = self.locks.lock().unwrap();
        let mut held = Vec::new();
        let mut pending = Vec::new();

        for (_name, state) in locks.iter() {
            // Add active locks to held
            for active_lock in &state.active_locks {
                held.push(format!(
                    r#"{{"name":"{}","mode":"{}","clientId":"lock_{}"}}"#,
                    active_lock.name, active_lock.mode, active_lock.id
                ));
            }

            // Add pending requests to pending
            for request in &state.pending_requests {
                pending.push(format!(
                    r#"{{"name":"{}","mode":"{}","clientId":"lock_{}"}}"#,
                    request.name, request.mode, request.id
                ));
            }
        }

        format!(
            r#"{{"held":[{}],"pending":[{}]}}"#,
            held.join(","),
            pending.join(",")
        )
    }
}

// Information about a lock for query results
#[derive(Debug, Clone)]
pub struct LockInfo {
    pub name: String,
    pub mode: LockMode,
    pub client_id: Option<String>,
}

// Resources managed by the WebLocks extension
pub struct WebLocksResources {
    pub manager: Arc<WebLocksManager>,
}

// Extension implementation
#[derive(Default)]
pub struct WebLocksExt;

impl WebLocksExt {
    #[hotpath::measure]
    pub fn new_extension() -> Extension {
        Extension {
            name: "weblocks",
            ops: vec![
                ExtensionOp::new(
                    "internal_locks_request",
                    Self::internal_locks_request,
                    4,
                    false,
                ),
                ExtensionOp::new(
                    "internal_locks_release",
                    Self::internal_locks_release,
                    2,
                    false,
                ),
                ExtensionOp::new("internal_locks_query", Self::internal_locks_query, 0, false),
                ExtensionOp::new("internal_locks_abort", Self::internal_locks_abort, 2, false),
            ],
            storage: Some(Box::new(|storage: &mut OpsStorage| {
                storage.insert(WebLocksResources {
                    manager: Arc::new(WebLocksManager::new()),
                });
            })),
            files: vec![include_str!("web/web_locks.ts")],
        }
    }

    // Request a lock: internal_locks_request(name, mode, ifAvailable, steal)
    fn internal_locks_request<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let name_binding = args.get(0).to_string(agent, gc.reborrow()).unbind()?;
        let mode_binding = args.get(1).to_string(agent, gc.reborrow()).unbind()?;
        let if_available_binding = args.get(2);
        let _steal_binding = args.get(3); // Steal not implemented yet

        let name = name_binding
            .as_str(agent)
            .expect("Name string is not valid UTF-8")
            .to_string();

        // Validate name (no leading '-')
        if name.starts_with('-') {
            return Ok(Value::from_str(agent, "error:Invalid lock name", gc.nogc())
                .into_value()
                .unbind());
        }

        let mode_str = mode_binding
            .as_str(agent)
            .expect("Mode string is not valid UTF-8");

        let mode = match mode_str {
            "shared" => LockMode::Shared,
            _ => LockMode::Exclusive,
        };

        let if_available = if_available_binding.is_true();
        let steal = _steal_binding.is_true();

        let (manager, macro_task_tx) = {
            let host_data = agent.get_host_data();
            let host_data: &HostData<RuntimeMacroTask> = host_data.downcast_ref().unwrap();
            let storage = host_data.storage.borrow();
            let resources: &WebLocksResources =
                storage.get().expect("WebLocks resources not initialized");
            (Arc::clone(&resources.manager), host_data.macro_task_tx())
        };

        // Pass macro_task_tx directly instead of host_data to avoid borrowing conflicts
        let result = manager.request_lock_with_tx(
            agent,
            RequestLockParams {
                name,
                mode,
                if_available,
                steal,
                promise: None,
                macro_task_tx,
            },
        );

        match result {
            Ok(lock_id) => {
                let lock_id_str = lock_id.to_string();
                Ok(Value::from_str(agent, &lock_id_str, gc.nogc())
                    .into_value()
                    .unbind())
            }
            Err(error) => {
                // Return "not_available" for ifAvailable failures
                if error.contains("ifAvailable") {
                    Ok(Value::from_str(agent, "not_available", gc.nogc())
                        .into_value()
                        .unbind())
                } else {
                    Ok(
                        Value::from_str(agent, &format!("error:{}", error), gc.nogc())
                            .into_value()
                            .unbind(),
                    )
                }
            }
        }
    }

    // Release a lock: internal_locks_release(name, lockId)
    fn internal_locks_release<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let name_binding = args.get(0).to_string(agent, gc.reborrow()).unbind()?;
        let lock_id_binding = args.get(1).to_string(agent, gc.reborrow()).unbind()?;

        let name = name_binding
            .as_str(agent)
            .expect("Name string is not valid UTF-8");

        let lock_id_str = lock_id_binding
            .as_str(agent)
            .expect("Lock ID string is not valid UTF-8");

        let lock_id: u64 = lock_id_str.parse().unwrap_or(0);

        let (manager, macro_task_tx) = {
            let host_data = agent.get_host_data();
            let host_data: &HostData<RuntimeMacroTask> = host_data.downcast_ref().unwrap();
            let storage = host_data.storage.borrow();
            let resources: &WebLocksResources =
                storage.get().expect("WebLocks resources not initialized");
            (Arc::clone(&resources.manager), host_data.macro_task_tx())
        };

        let result = manager.release_lock_with_tx(name, lock_id, macro_task_tx);

        match result {
            Ok(()) => Ok(Value::from_str(agent, "released", gc.nogc())
                .into_value()
                .unbind()),
            Err(error) => Ok(
                Value::from_str(agent, &format!("error:{}", error), gc.nogc())
                    .into_value()
                    .unbind(),
            ),
        }
    }

    // Query lock state: internal_locks_query()
    fn internal_locks_query<'gc>(
        agent: &mut Agent,
        _this: Value,
        _args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let host_data = agent.get_host_data();
        let host_data: &HostData<RuntimeMacroTask> = host_data.downcast_ref().unwrap();

        let result_json = {
            let storage = host_data.storage.borrow();
            let resources: &WebLocksResources =
                storage.get().expect("WebLocks resources not initialized");

            resources.manager.query_locks()
        };

        Ok(Value::from_str(agent, &result_json, gc.nogc())
            .into_value()
            .unbind())
    }

    // Abort a lock request: internal_locks_abort(name, lockId)
    fn internal_locks_abort<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let name_binding = args.get(0).to_string(agent, gc.reborrow()).unbind()?;
        let lock_id_binding = args.get(1).to_string(agent, gc.reborrow()).unbind()?;

        let name = name_binding
            .as_str(agent)
            .expect("Name string is not valid UTF-8");

        let lock_id_str = lock_id_binding
            .as_str(agent)
            .expect("Lock ID string is not valid UTF-8");

        let lock_id: u64 = lock_id_str.parse().unwrap_or(0);

        let host_data = agent.get_host_data();
        let host_data: &HostData<RuntimeMacroTask> = host_data.downcast_ref().unwrap();

        let (manager, macro_task_tx) = {
            let storage = host_data.storage.borrow();
            let resources: &WebLocksResources =
                storage.get().expect("WebLocks resources not initialized");
            (Arc::clone(&resources.manager), host_data.macro_task_tx())
        };

        let result = manager.abort_request_with_tx(name, lock_id, macro_task_tx);

        match result {
            Ok(()) => Ok(Value::from_str(agent, "aborted", gc.nogc())
                .into_value()
                .unbind()),
            Err(error) => Ok(
                Value::from_str(agent, &format!("error:{}", error), gc.nogc())
                    .into_value()
                    .unbind(),
            ),
        }
    }
}
