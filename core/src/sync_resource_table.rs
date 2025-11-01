// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::{
    collections::HashMap,
    sync::{
        Arc, Mutex,
        atomic::{AtomicU32, Ordering},
    },
};

use crate::{AndromedaError, AndromedaResult, Rid};

// Allow retrieving resources; requires T: Clone
impl<T: Clone> SyncResourceTable<T> {
    /// Get a clone of the resource by Rid.
    pub fn get(&self, rid: Rid) -> Option<T> {
        self.table.lock().unwrap().get(&rid).cloned()
    }

    /// Get a clone of the resource by Rid with proper error handling.
    pub fn get_or_error(&self, rid: Rid, operation: &str) -> AndromedaResult<T> {
        self.table
            .lock()
            .unwrap()
            .get(&rid)
            .cloned()
            .ok_or_else(|| {
                Box::new(AndromedaError::resource_error(
                    rid.index(),
                    operation,
                    "Resource not found",
                ))
            })
    }
}

pub struct SyncResourceTable<T> {
    table: Arc<Mutex<HashMap<Rid, T>>>,
    next_rid: Arc<AtomicU32>,
}

impl<T> Default for SyncResourceTable<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Clone for SyncResourceTable<T> {
    fn clone(&self) -> Self {
        Self {
            table: self.table.clone(),
            next_rid: self.next_rid.clone(),
        }
    }
}

impl<T> SyncResourceTable<T> {
    pub fn new() -> Self {
        Self {
            table: Arc::default(),
            next_rid: Arc::default(),
        }
    }

    /// Returns true if the table contains the given Rid.
    pub fn contains(&self, rid: Rid) -> bool {
        self.table.lock().unwrap().contains_key(&rid)
    }

    /// Returns the number of resources currently stored.
    pub fn len(&self) -> usize {
        self.table.lock().unwrap().len()
    }

    /// Returns true if the resource table has no entries.
    pub fn is_empty(&self) -> bool {
        self.table.lock().unwrap().is_empty()
    }

    pub fn push(&self, value: T) -> Rid {
        let rid = self.next_rid.load(Ordering::Relaxed);
        let new_rid = rid + 1;
        let rid = Rid::from_index(rid);

        self.table.lock().unwrap().insert(rid, value);
        self.next_rid.store(new_rid, Ordering::Relaxed);

        rid
    }

    /// Remove a resource by Rid.
    pub fn remove(&self, rid: Rid) -> Option<T> {
        self.table.lock().unwrap().remove(&rid)
    }
}
