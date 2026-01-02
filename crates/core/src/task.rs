// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

/// An Id representing a Task.
#[derive(Debug, PartialEq, Hash, Eq, Clone, Copy)]
pub struct TaskId(u32);

impl TaskId {
    pub fn index(&self) -> u32 {
        self.0
    }

    pub fn from_index(index: u32) -> Self {
        Self(index)
    }
}
