// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Bytes-from-Value helper.
//!
//! Extracts an owned `Vec<u8>` from any bytes-like `Value` — `ArrayBuffer`,
//! `DataView`, or any normal `TypedArray` variant — using Nova VM's public
//! buffer-access API. Reads the underlying `ArrayBuffer` slice once and
//! copies out so the result is decoupled from Nova's GC lifetime (the slice
//! borrow `&'ab [u8]` is invalidated by JS re-entry).

use nova_vm::{
    ecmascript::{Agent, Value},
    engine::GcScope,
};

/// Extract bytes from any bytes-like `Value`.
///
/// Supports `ArrayBuffer`, `DataView`, and every normal `TypedArray` variant.
/// Rejects `SharedArrayBuffer`-backed values, detached buffers, and anything
/// not bytes-shaped.
pub fn bytes_from_value<'gc>(
    agent: &mut Agent,
    value: Value<'gc>,
    _gc: GcScope<'_, '_>,
) -> Result<Vec<u8>, String> {
    match value {
        Value::ArrayBuffer(ab) => {
            if ab.is_detached(agent) {
                return Err("ArrayBuffer is detached".to_string());
            }
            Ok(ab.as_slice(agent).to_vec())
        }

        Value::DataView(dv) => {
            let buf = dv.viewed_array_buffer(agent);
            if buf.is_detached(agent) {
                return Err("DataView's ArrayBuffer is detached".to_string());
            }
            let off = dv.byte_offset(agent);
            let buf_len = buf.byte_length(agent);
            let len = dv.byte_length(agent).unwrap_or_else(|| buf_len.saturating_sub(off));
            let end = off.checked_add(len).ok_or_else(|| "DataView range overflows".to_string())?;
            if end > buf_len {
                return Err("DataView range exceeds ArrayBuffer".to_string());
            }
            Ok(buf.as_slice(agent)[off..end].to_vec())
        }

        Value::Int8Array(_)
        | Value::Uint8Array(_)
        | Value::Uint8ClampedArray(_)
        | Value::Int16Array(_)
        | Value::Uint16Array(_)
        | Value::Int32Array(_)
        | Value::Uint32Array(_)
        | Value::BigInt64Array(_)
        | Value::BigUint64Array(_)
        | Value::Float32Array(_)
        | Value::Float64Array(_) => typed_array_bytes(agent, value),

        // SharedArrayBuffer-backed values cross thread boundaries and need
        // their own design; rejected for v1.
        Value::SharedArrayBuffer(_) | Value::SharedDataView(_) => {
            Err("SharedArrayBuffer is not supported".to_string())
        }

        _ => Err("Value is not bytes-like".to_string()),
    }
}

fn typed_array_bytes<'gc>(agent: &mut Agent, value: Value<'gc>) -> Result<Vec<u8>, String> {
    use nova_vm::ecmascript::TypedArray;

    let ta = TypedArray::try_from(value)
        .map_err(|_| "Value is not a TypedArray".to_string())?;
    let element_size = match ta {
        TypedArray::Int8Array(_)
        | TypedArray::Uint8Array(_)
        | TypedArray::Uint8ClampedArray(_) => 1usize,
        TypedArray::Int16Array(_) | TypedArray::Uint16Array(_) => 2,
        TypedArray::Int32Array(_) | TypedArray::Uint32Array(_) | TypedArray::Float32Array(_) => 4,
        TypedArray::BigInt64Array(_)
        | TypedArray::BigUint64Array(_)
        | TypedArray::Float64Array(_) => 8,
        #[cfg(feature = "proposals")]
        TypedArray::Float16Array(_) => 2,
    };

    let buf = ta.get_viewed_array_buffer(agent);
    if buf.is_detached(agent) {
        return Err("TypedArray's ArrayBuffer is detached".to_string());
    }

    let buf_len = buf.byte_length(agent);
    let off = ta.byte_offset(agent);
    // array_length is element count; when auto-length (None), the view runs
    // to the end of the buffer.
    let byte_len: usize = match ta.array_length(agent) {
        Some(n) => n.checked_mul(element_size)
            .ok_or_else(|| "TypedArray length overflows".to_string())?,
        None => buf_len.saturating_sub(off),
    };
    let end = off.checked_add(byte_len)
        .ok_or_else(|| "TypedArray range overflows".to_string())?;
    if end > buf_len {
        return Err("TypedArray range exceeds ArrayBuffer".to_string());
    }
    Ok(buf.as_slice(agent)[off..end].to_vec())
}
