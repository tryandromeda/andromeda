use std::{
    cell::RefCell,
    collections::HashMap,
    future::Future,
    sync::{
        Arc,
        atomic::{AtomicU32, Ordering},
        mpsc::{Receiver, Sender},
    },
};

use anymap::AnyMap;
use tokio::task::JoinHandle;

use crate::{MacroTask, TaskId};

pub type OpsStorage = AnyMap;

pub type LocalOpsStorage = RefCell<OpsStorage>;

/// Data created and used by the Runtime.
pub struct HostData<UserMacroTask> {
    /// Storage used by the built-in functions.
    pub storage: LocalOpsStorage,
    /// Send macro tasks to the event loop.
    pub macro_task_tx: Sender<MacroTask<UserMacroTask>>,
    /// Counter of active macro tasks.
    pub macro_task_count: Arc<AtomicU32>,
    /// Registry of async tasks.
    pub tasks: RefCell<HashMap<TaskId, JoinHandle<()>>>,
    /// Counter of accumulative created async tasks. Used for ID generation.
    pub task_count: Arc<AtomicU32>,
}

impl<UserMacroTask> HostData<UserMacroTask> {
    pub fn new() -> (Self, Receiver<MacroTask<UserMacroTask>>) {
        let (macro_task_tx, rx) = std::sync::mpsc::channel();
        (
            Self {
                storage: RefCell::new(AnyMap::new()),
                macro_task_tx,
                macro_task_count: Arc::new(AtomicU32::new(0)),
                tasks: RefCell::default(),
                task_count: Arc::default(),
            },
            rx,
        )
    }

    /// Get an owned senderto the macro tasks event loop.
    pub fn macro_task_tx(&self) -> Sender<MacroTask<UserMacroTask>> {
        self.macro_task_tx.clone()
    }

    /// Spawn an async task in the Tokio Runtime that self-registers and unregisters automatically.
    /// It's given [TaskId] is returned.
    pub fn spawn_macro_task<F>(&self, future: F) -> TaskId
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        let macro_task_count = self.macro_task_count.clone();
        macro_task_count.fetch_add(1, Ordering::Relaxed);

        let task_handle = tokio::spawn(async move {
            future.await;
            macro_task_count.fetch_sub(1, Ordering::Relaxed);
        });

        let task_id = TaskId::from_index(self.task_count.fetch_add(1, Ordering::Relaxed));
        self.tasks.borrow_mut().insert(task_id, task_handle);

        task_id
    }

    /// Abort a MacroTask execution given it's [TaskId].
    pub fn abort_macro_task(&self, task_id: TaskId) {
        let tasks = self.tasks.borrow();
        let task = tasks.get(&task_id).unwrap();
        task.abort();

        // Manually decrease the macro tasks counter as the task was aborted.
        self.macro_task_count.fetch_sub(1, Ordering::Relaxed);
    }

    /// Clear a MacroTask given it's [TaskId].
    pub fn clear_macro_task(&self, task_id: TaskId) {
        self.tasks.borrow_mut().remove(&task_id).unwrap();
    }
}
