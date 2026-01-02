// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::ext::{LockMode, cron::CronId, interval::IntervalId, timeout::TimeoutId};
use nova_vm::{ecmascript::types::Value, engine::Global};
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
    /// Resolve a promise with bytes as Uint8Array.
    ResolvePromiseWithBytes(Global<Value<'static>>, Vec<u8>),
    /// Reject a promise with an error message.
    RejectPromise(Global<Value<'static>>, String),
    /// Register a TLS stream into the runtime resource table and resolve a promise with its rid.
    RegisterTlsStream(Global<Value<'static>>, Box<TlsStream<TcpStream>>),
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
}
