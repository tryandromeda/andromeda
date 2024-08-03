use nova_vm::ecmascript::types::{Global, Value};

/// Collection of tasks dispatched and handled by the Runtime.
#[derive(Debug)]
pub enum MacroTask<UserMacroTask> {
    /// Resolve a promise.
    ResolvePromise(Global<Value>),
    /// User-defined macro task.
    User(UserMacroTask),
}
