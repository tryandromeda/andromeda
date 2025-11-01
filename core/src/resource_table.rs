// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::{
    cell::{RefCell, RefMut},
    collections::HashMap,
    hash::Hash,
};

use crate::{AndromedaError, AndromedaResult};

#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq)]
pub struct Rid(u32);

impl Rid {
    pub fn index(&self) -> u32 {
        self.0
    }
    /// Create a Rid from its numeric index.
    pub fn from_index(index: u32) -> Rid {
        Rid(index)
    }
}

// Allow retrieving resources; requires T: Clone
impl<T: Clone> ResourceTable<T> {
    /// Get a clone of the resource by Rid.
    pub fn get(&self, rid: Rid) -> Option<T> {
        self.table.borrow().get(&rid).cloned()
    }

    /// Get a clone of the resource by Rid with proper error handling.
    pub fn get_or_error(&self, rid: Rid, operation: &str) -> AndromedaResult<T> {
        self.table.borrow().get(&rid).cloned().ok_or_else(|| {
            Box::new(AndromedaError::resource_error(
                rid.index(),
                operation,
                "Resource not found",
            ))
        })
    }
}

pub struct ResourceTable<T> {
    table: RefCell<HashMap<Rid, T>>,
    next_rid: RefCell<Rid>,
}

impl<T> Default for ResourceTable<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> ResourceTable<T> {
    pub fn new() -> Self {
        Self {
            table: RefCell::default(),
            next_rid: RefCell::new(Rid(0)),
        }
    }

    /// Returns true if the table contains the given Rid.
    pub fn contains(&self, rid: Rid) -> bool {
        self.table.borrow().contains_key(&rid)
    }

    /// Returns the number of resources currently stored.
    pub fn len(&self) -> usize {
        self.table.borrow().len()
    }

    /// Returns true if the resource table has no entries.
    pub fn is_empty(&self) -> bool {
        self.table.borrow().is_empty()
    }

    pub fn push(&self, value: T) -> Rid {
        let rid = *self.next_rid.borrow();
        let new_rid = Rid(rid.index() + 1);

        self.table.borrow_mut().insert(rid, value);
        *self.next_rid.borrow_mut() = new_rid;

        rid
    }

    /// Remove a resource by Rid.
    pub fn remove(&self, rid: Rid) -> Option<T> {
        self.table.borrow_mut().remove(&rid)
    }

    /// Get a mutable reference to the resource by Rid.
    pub fn get_mut(&self, rid: Rid) -> Option<RefMut<'_, T>> {
        let borrow = self.table.borrow_mut();
        if borrow.contains_key(&rid) {
            // SAFETY: key exists, unwrap is safe
            Some(RefMut::map(borrow, move |m| m.get_mut(&rid).unwrap()))
        } else {
            None
        }
    }
}
