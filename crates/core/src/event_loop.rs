// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use nova_vm::{ecmascript::types::Value, engine::Global};

/// Collection of tasks dispatched and handled by the Runtime.
#[derive(Debug)]
pub enum MacroTask<UserMacroTask> {
    /// Resolve a promise.
    ResolvePromise(Global<Value<'static>>),
    /// User-defined macro task.
    User(UserMacroTask),
}
