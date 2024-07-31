use nova_vm::ecmascript::types::{Global, Value};

use crate::IntervalId;

/// Collection of tasks dispatched and handled by the Runtime.
#[derive(Debug)]
pub enum MacroTask {
    // TODO: This should include some kind of resolved value?
    ResolvePromise(Global<Value>),
    /// Run an interval.
    RunInterval(IntervalId),
    /// Stop an interval from running no further.
    ClearInterval(IntervalId),
}
