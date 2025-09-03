// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use andromeda_core::{Extension, ExtensionOp};
use nova_vm::{
    ecmascript::{
        builtins::ArgumentsList,
        execution::{Agent, JsResult, agent::ExceptionType},
        types::Value,
    },
    engine::context::{Bindable, GcScope},
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

// Resource table for managing stream data
static STREAM_STORAGE: std::sync::OnceLock<Arc<Mutex<HashMap<String, StreamData>>>> =
    std::sync::OnceLock::new();

// Internal stream data structure
#[derive(Clone)]
struct StreamData {
    chunks: Vec<Vec<u8>>,
    readable: bool,
    writable: bool,
    closed: bool,
    errored: bool,
    error_message: Option<String>,
}

#[derive(Default)]
pub struct StreamsExt;

impl StreamsExt {
    pub fn new_extension() -> Extension {
        Extension {
            name: "streams",
            ops: vec![
                // ReadableStream operations
                ExtensionOp::new(
                    "internal_readable_stream_create",
                    Self::internal_readable_stream_create,
                    0,
                    false,
                ),
                ExtensionOp::new(
                    "internal_readable_stream_read",
                    Self::internal_readable_stream_read,
                    1,
                    false,
                ),
                ExtensionOp::new(
                    "internal_readable_stream_cancel",
                    Self::internal_readable_stream_cancel,
                    1,
                    false,
                ),
                ExtensionOp::new(
                    "internal_readable_stream_close",
                    Self::internal_readable_stream_close,
                    1,
                    false,
                ),
                ExtensionOp::new(
                    "internal_readable_stream_enqueue",
                    Self::internal_readable_stream_enqueue,
                    2,
                    false,
                ),
                // WritableStream operations
                ExtensionOp::new(
                    "internal_writable_stream_create",
                    Self::internal_writable_stream_create,
                    0,
                    false,
                ),
                ExtensionOp::new(
                    "internal_writable_stream_write",
                    Self::internal_writable_stream_write,
                    2,
                    false,
                ),
                ExtensionOp::new(
                    "internal_writable_stream_close",
                    Self::internal_writable_stream_close,
                    1,
                    false,
                ),
                ExtensionOp::new(
                    "internal_writable_stream_abort",
                    Self::internal_writable_stream_abort,
                    1,
                    false,
                ),
                // Stream utility operations
                ExtensionOp::new(
                    "internal_stream_get_state",
                    Self::internal_stream_get_state,
                    1,
                    false,
                ),
            ],
            storage: None,
            files: vec![
                include_str!("./readable_stream.ts"),
                include_str!("./writable_stream.ts"),
                include_str!("./transform_stream.ts"),
                include_str!("./queuing_strategy.ts"),
            ],
        }
    }

    fn get_stream_storage() -> Arc<Mutex<HashMap<String, StreamData>>> {
        STREAM_STORAGE
            .get_or_init(|| Arc::new(Mutex::new(HashMap::new())))
            .clone()
    }

    // ReadableStream operations
    pub fn internal_readable_stream_create<'gc>(
        agent: &mut Agent,
        _this: Value,
        _args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let stream_id = Uuid::new_v4().to_string();

        let stream_data = StreamData {
            chunks: Vec::new(),
            readable: true,
            writable: false,
            closed: false,
            errored: false,
            error_message: None,
        };

        let storage = Self::get_stream_storage();
        storage
            .lock()
            .unwrap()
            .insert(stream_id.clone(), stream_data);

        Ok(Value::from_string(agent, stream_id, gc.nogc()).unbind())
    }

    pub fn internal_readable_stream_read<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let stream_id_arg = args.get(0);
        let stream_id_str = stream_id_arg.to_string(agent, gc.reborrow()).unbind()?;
        let stream_id = stream_id_str
            .as_str(agent)
            .expect("String is not valid UTF-8");

        let storage = Self::get_stream_storage();
        let mut storage_lock = storage.lock().unwrap();

        if let Some(stream_data) = storage_lock.get_mut(stream_id) {
            if stream_data.errored {
                return Err(agent
                    .throw_exception(
                        ExceptionType::Error,
                        stream_data
                            .error_message
                            .clone()
                            .unwrap_or_else(|| "Stream errored".to_string()),
                        gc.nogc(),
                    )
                    .unbind());
            }

            if stream_data.closed && stream_data.chunks.is_empty() {
                // Return done: true, value: undefined
                return Ok(Value::from_string(agent, "done".to_string(), gc.nogc()).unbind());
            }

            if !stream_data.chunks.is_empty() {
                let chunk = stream_data.chunks.remove(0);
                // Convert chunk to comma-separated bytes string
                let bytes_str = chunk
                    .iter()
                    .map(|b| b.to_string())
                    .collect::<Vec<_>>()
                    .join(",");
                Ok(Value::from_string(agent, bytes_str, gc.nogc()).unbind())
            } else {
                // No chunks available, return empty string (will be handled as pending)
                Ok(Value::from_string(agent, "".to_string(), gc.nogc()).unbind())
            }
        } else {
            Err(agent
                .throw_exception(
                    ExceptionType::Error,
                    "Invalid stream ID".to_string(),
                    gc.nogc(),
                )
                .unbind())
        }
    }

    pub fn internal_readable_stream_cancel<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let stream_id_arg = args.get(0);
        let stream_id_str = stream_id_arg.to_string(agent, gc.reborrow()).unbind()?;
        let stream_id = stream_id_str
            .as_str(agent)
            .expect("String is not valid UTF-8");

        let storage = Self::get_stream_storage();
        let mut storage_lock = storage.lock().unwrap();

        if let Some(stream_data) = storage_lock.get_mut(stream_id) {
            stream_data.readable = false;
            stream_data.closed = true;
            stream_data.chunks.clear();
            Ok(Value::from_string(agent, "cancelled".to_string(), gc.nogc()).unbind())
        } else {
            Err(agent
                .throw_exception(
                    ExceptionType::Error,
                    "Invalid stream ID".to_string(),
                    gc.nogc(),
                )
                .unbind())
        }
    }

    pub fn internal_readable_stream_close<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let stream_id_arg = args.get(0);
        let stream_id_str = stream_id_arg.to_string(agent, gc.reborrow()).unbind()?;
        let stream_id = stream_id_str
            .as_str(agent)
            .expect("String is not valid UTF-8");

        let storage = Self::get_stream_storage();
        let mut storage_lock = storage.lock().unwrap();

        if let Some(stream_data) = storage_lock.get_mut(stream_id) {
            stream_data.closed = true;
            Ok(Value::from_string(agent, "closed".to_string(), gc.nogc()).unbind())
        } else {
            Err(agent
                .throw_exception(
                    ExceptionType::Error,
                    "Invalid stream ID".to_string(),
                    gc.nogc(),
                )
                .unbind())
        }
    }

    pub fn internal_readable_stream_enqueue<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let stream_id_arg = args.get(0);
        let chunk_arg = args.get(1);

        let stream_id_str = stream_id_arg.to_string(agent, gc.reborrow()).unbind()?;
        let stream_id = stream_id_str
            .as_str(agent)
            .expect("String is not valid UTF-8");

        let chunk_str = chunk_arg.to_string(agent, gc.reborrow()).unbind()?;
        let chunk_string = chunk_str.as_str(agent).expect("String is not valid UTF-8");

        // Parse chunk as comma-separated bytes
        let chunk_bytes: Vec<u8> = if chunk_string.is_empty() {
            Vec::new()
        } else {
            chunk_string
                .split(',')
                .filter_map(|s| s.trim().parse::<u8>().ok())
                .collect()
        };

        let storage = Self::get_stream_storage();
        let mut storage_lock = storage.lock().unwrap();

        if let Some(stream_data) = storage_lock.get_mut(stream_id) {
            if stream_data.closed {
                return Err(agent
                    .throw_exception(
                        ExceptionType::Error,
                        "Cannot enqueue to closed stream".to_string(),
                        gc.nogc(),
                    )
                    .unbind());
            }

            stream_data.chunks.push(chunk_bytes);
            Ok(Value::from_string(agent, "enqueued".to_string(), gc.nogc()).unbind())
        } else {
            Err(agent
                .throw_exception(
                    ExceptionType::Error,
                    "Invalid stream ID".to_string(),
                    gc.nogc(),
                )
                .unbind())
        }
    }

    // WritableStream operations
    pub fn internal_writable_stream_create<'gc>(
        agent: &mut Agent,
        _this: Value,
        _args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let stream_id = Uuid::new_v4().to_string();

        let stream_data = StreamData {
            chunks: Vec::new(),
            readable: false,
            writable: true,
            closed: false,
            errored: false,
            error_message: None,
        };

        let storage = Self::get_stream_storage();
        storage
            .lock()
            .unwrap()
            .insert(stream_id.clone(), stream_data);

        Ok(Value::from_string(agent, stream_id, gc.nogc()).unbind())
    }

    pub fn internal_writable_stream_write<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let stream_id_arg = args.get(0);
        let chunk_arg = args.get(1);

        let stream_id_str = stream_id_arg.to_string(agent, gc.reborrow()).unbind()?;
        let stream_id = stream_id_str
            .as_str(agent)
            .expect("String is not valid UTF-8");

        let chunk_str = chunk_arg.to_string(agent, gc.reborrow()).unbind()?;
        let chunk_string = chunk_str.as_str(agent).expect("String is not valid UTF-8");

        // Parse chunk as comma-separated bytes
        let chunk_bytes: Vec<u8> = if chunk_string.is_empty() {
            Vec::new()
        } else {
            chunk_string
                .split(',')
                .filter_map(|s| s.trim().parse::<u8>().ok())
                .collect()
        };

        let storage = Self::get_stream_storage();
        let mut storage_lock = storage.lock().unwrap();

        if let Some(stream_data) = storage_lock.get_mut(stream_id) {
            if !stream_data.writable || stream_data.closed {
                return Err(agent
                    .throw_exception(
                        ExceptionType::Error,
                        "Cannot write to closed or non-writable stream".to_string(),
                        gc.nogc(),
                    )
                    .unbind());
            }

            stream_data.chunks.push(chunk_bytes);
            Ok(Value::from_string(agent, "written".to_string(), gc.nogc()).unbind())
        } else {
            Err(agent
                .throw_exception(
                    ExceptionType::Error,
                    "Invalid stream ID".to_string(),
                    gc.nogc(),
                )
                .unbind())
        }
    }

    pub fn internal_writable_stream_close<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let stream_id_arg = args.get(0);
        let stream_id_str = stream_id_arg.to_string(agent, gc.reborrow()).unbind()?;
        let stream_id = stream_id_str
            .as_str(agent)
            .expect("String is not valid UTF-8");

        let storage = Self::get_stream_storage();
        let mut storage_lock = storage.lock().unwrap();

        if let Some(stream_data) = storage_lock.get_mut(stream_id) {
            stream_data.writable = false;
            stream_data.closed = true;
            Ok(Value::from_string(agent, "closed".to_string(), gc.nogc()).unbind())
        } else {
            Err(agent
                .throw_exception(
                    ExceptionType::Error,
                    "Invalid stream ID".to_string(),
                    gc.nogc(),
                )
                .unbind())
        }
    }

    pub fn internal_writable_stream_abort<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let stream_id_arg = args.get(0);
        let stream_id_str = stream_id_arg.to_string(agent, gc.reborrow()).unbind()?;
        let stream_id = stream_id_str
            .as_str(agent)
            .expect("String is not valid UTF-8");

        let storage = Self::get_stream_storage();
        let mut storage_lock = storage.lock().unwrap();

        if let Some(stream_data) = storage_lock.get_mut(stream_id) {
            stream_data.writable = false;
            stream_data.closed = true;
            stream_data.errored = true;
            stream_data.error_message = Some("Stream aborted".to_string());
            stream_data.chunks.clear();
            Ok(Value::from_string(agent, "aborted".to_string(), gc.nogc()).unbind())
        } else {
            Err(agent
                .throw_exception(
                    ExceptionType::Error,
                    "Invalid stream ID".to_string(),
                    gc.nogc(),
                )
                .unbind())
        }
    }

    // Utility operations
    pub fn internal_stream_get_state<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let stream_id_arg = args.get(0);
        let stream_id_str = stream_id_arg.to_string(agent, gc.reborrow()).unbind()?;
        let stream_id = stream_id_str
            .as_str(agent)
            .expect("String is not valid UTF-8");

        let storage = Self::get_stream_storage();
        let storage_lock = storage.lock().unwrap();

        if let Some(stream_data) = storage_lock.get(stream_id) {
            // Return state as a formatted string: "readable:writable:closed:errored:chunk_count"
            let state = format!(
                "{}:{}:{}:{}:{}",
                stream_data.readable,
                stream_data.writable,
                stream_data.closed,
                stream_data.errored,
                stream_data.chunks.len()
            );
            Ok(Value::from_string(agent, state, gc.nogc()).unbind())
        } else {
            Err(agent
                .throw_exception(
                    ExceptionType::Error,
                    "Invalid stream ID".to_string(),
                    gc.nogc(),
                )
                .unbind())
        }
    }
}
