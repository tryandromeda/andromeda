use nova_vm::ecmascript::types::GlobalValue;

/// Collection of tasks dispatched and handled by the Runtime.
pub enum MacroTask {
    // TODO: This should include some kind of resolved value?
    ResolvePromise(GlobalValue),
}
