// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::ext::{LockMode, cron::CronId, interval::IntervalId, timeout::TimeoutId};
use nova_vm::{ecmascript::Value, engine::Global};
use tokio::net::TcpStream;
use tokio_rustls::client::TlsStream;

pub enum RuntimeMacroTask {
    /// Run an interval.
    RunInterval(IntervalId),
    /// Stop an interval from running no further.
    ClearInterval(IntervalId),
    /// Run and clear a timeout.
    RunAndClearTimeout(TimeoutId),
    /// Stop a timeout from running no further.
    ClearTimeout(TimeoutId),
    /// Run a cron job.
    RunCron(CronId),
    /// Clear a cron job.
    ClearCron(CronId),
    /// Resolve a promise with a pre-created Value.
    ResolvePromiseWithValue(Global<Value<'static>>, Global<Value<'static>>),
    /// Resolve a promise with a string value.
    ResolvePromiseWithString(Global<Value<'static>>, String),
    /// Resolve a promise with bytes as ArrayBuffer (JS wraps with `new Uint8Array(buf)`).
    ResolvePromiseWithBytes(Global<Value<'static>>, Vec<u8>),
    /// Resolve a promise with bytes hex-encoded into a string. Kept for
    /// callers (e.g. `Andromeda.readFile`) whose public API still returns hex.
    ResolvePromiseWithHexBytes(Global<Value<'static>>, Vec<u8>),
    /// Reject a promise with an error message.
    RejectPromise(Global<Value<'static>>, String),
    /// Register a TLS stream into the runtime resource table and resolve a promise with its rid.
    RegisterTlsStream(Global<Value<'static>>, Box<TlsStream<TcpStream>>),
    /// Register a plaintext TCP stream into the runtime resource table and resolve a promise with its rid.
    RegisterTcpStream(Global<Value<'static>>, Box<TcpStream>),
    /// Acquire a lock and resolve the promise with the lock result.
    AcquireLock {
        promise: Global<Value<'static>>,
        lock_id: u64,
        name: String,
        mode: LockMode,
    },
    /// Release a lock and process any pending requests.
    ReleaseLock { name: String, lock_id: u64 },
    /// Abort a pending lock request.
    AbortLockRequest { name: String, lock_id: u64 },
    /// Deliver a structured-clone serialized message from a worker thread to
    /// the parent's `Worker` instance. JS-side dispatches `message` event.
    WorkerDeliverMessage { worker_id: u32, payload: String },
    /// Deliver a `messageerror` event to the parent's `Worker` instance.
    WorkerDeliverMessageError { worker_id: u32, reason: String },
    /// Deliver an `error` (ErrorEvent) to the parent's `Worker` instance.
    WorkerDeliverError {
        worker_id: u32,
        message: String,
        filename: String,
        lineno: u32,
        colno: u32,
    },
    /// Deliver a parent-posted message into the worker realm; dispatches
    /// `message` event on `self` (DedicatedWorkerGlobalScope).
    WorkerSelfDeliverMessage { payload: String },
    /// Worker-side close request: drives the runtime's event loop to exit.
    WorkerSelfClose,
    /// Posted by the parent-side forwarder thread when the worker has
    /// fully exited. Wakes the parent runtime loop so it can re-check
    /// `macro_task_count` (now decremented) and also lets the dispatcher
    /// prune the worker from the parent's registry / JS-side map.
    WorkerForwarderClosed { worker_id: u32 },
}
