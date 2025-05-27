// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::{cell::RefCell, collections::HashMap, hash::Hash};

#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq)]
pub struct Rid(u32);

impl Rid {
    pub fn index(&self) -> u32 {
        self.0
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

    pub fn push(&self, value: T) -> Rid {
        let rid = *self.next_rid.borrow();
        let new_rid = Rid(rid.index() + 1);

        self.table.borrow_mut().insert(rid, value);
        *self.next_rid.borrow_mut() = new_rid;

        rid
    }
}
