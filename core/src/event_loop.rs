use nova_vm::{ecmascript::types::Value, engine::Global};

/// Collection of tasks dispatched and handled by the Runtime.
#[derive(Debug)]
pub enum MacroTask<UserMacroTask> {
    /// Resolve a promise.
    ResolvePromise(Global<Value<'static>>),
    /// User-defined macro task.
    User(UserMacroTask),
}
