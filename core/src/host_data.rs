use std::{
    cell::RefCell,
    future::Future,
    sync::{
        atomic::{AtomicU32, Ordering},
        mpsc::{Receiver, Sender},
        Arc,
    },
};

use anymap::AnyMap;

use crate::MacroTask;

/// Data created and used by the Runtime.
pub struct HostData {
    /// Storage used by the built-in functions.
    pub storage: RefCell<AnyMap>,
    /// Send macro tasks to the event loop.
    pub macro_task_tx: Sender<MacroTask>,
    /// Counter of active macro tasks.
    pub macro_task_count: Arc<AtomicU32>,
}

impl HostData {
    pub fn new() -> (Self, Receiver<MacroTask>) {
        let (macro_task_tx, rx) = std::sync::mpsc::channel();
        (
            Self {
                storage: RefCell::new(AnyMap::new()),
                macro_task_tx,
                macro_task_count: Arc::new(AtomicU32::new(0)),
            },
            rx,
        )
    }

    /// Get an owned senderto the macro tasks event loop.
    pub fn macro_task_tx(&self) -> Sender<MacroTask> {
        self.macro_task_tx.clone()
    }

    /// Spawn an async task in the Tokio Runtime that self-registers and unregisters automatically.
    pub fn spawn_macro_task<F>(&self, future: F)
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        let macro_task_count = self.macro_task_count.clone();
        macro_task_count.fetch_add(1, Ordering::Relaxed);
        tokio::spawn(async move {
            future.await;
            macro_task_count.fetch_sub(1, Ordering::Relaxed);
        });
    }
}
