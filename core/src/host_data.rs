use std::{
    cell::RefCell,
    sync::mpsc::{Receiver, Sender},
};

use anymap::AnyMap;

use crate::MacroTask;

/// Data created and used by the Runtime.
pub struct HostData {
    /// Storage used by the built-in functions.
    pub storage: RefCell<AnyMap>,
    /// Message sender for the Runtime event loop.
    pub macro_task_tx: Sender<MacroTask>,
}

impl HostData {
    pub fn new() -> (Self, Receiver<MacroTask>) {
        let (tx, rx) = std::sync::mpsc::channel();
        (
            Self {
                storage: RefCell::new(AnyMap::new()),
                macro_task_tx: tx,
            },
            rx,
        )
    }

    /// Get an owned senderto the macro tasks event loop.
    pub fn macro_task_tx(&self) -> Sender<MacroTask> {
        self.macro_task_tx.clone()
    }
}
