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

// Stream states based on the Streams specification
#[derive(Clone, Debug, PartialEq)]
enum StreamState {
    Readable,
    Closed,
    Errored,
}

#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq)]
enum WritableStreamState {
    Writable,
    Closing,
    Closed,
    Erroring,
    Errored,
}

// Enhanced stream data structure with proper state management
#[derive(Clone)]
struct StreamData {
    chunks: Vec<Vec<u8>>,
    readable_state: StreamState,
    writable_state: WritableStreamState,
    error_message: Option<String>,
    // For backpressure handling
    desired_size: i32,
    high_water_mark: usize,
    // For tracking readers/writers
    locked: bool,
}

#[derive(Default)]
pub struct StreamsExt;

impl StreamsExt {
    pub fn new_extension() -> Extension {
        Extension {
            name: "streams",
            ops: vec![
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
                ExtensionOp::new(
                    "internal_stream_get_state",
                    Self::internal_stream_get_state,
                    1,
                    false,
                ),
                ExtensionOp::new(
                    "internal_readable_stream_error",
                    Self::internal_readable_stream_error,
                    2,
                    false,
                ),
                ExtensionOp::new(
                    "internal_readable_stream_lock",
                    Self::internal_readable_stream_lock,
                    1,
                    false,
                ),
                ExtensionOp::new(
                    "internal_readable_stream_unlock",
                    Self::internal_readable_stream_unlock,
                    1,
                    false,
                ),
                ExtensionOp::new(
                    "internal_readable_stream_tee",
                    Self::internal_readable_stream_tee,
                    1,
                    false,
                ),
                ExtensionOp::new(
                    "internal_writable_stream_error",
                    Self::internal_writable_stream_error,
                    2,
                    false,
                ),
                ExtensionOp::new(
                    "internal_writable_stream_lock",
                    Self::internal_writable_stream_lock,
                    1,
                    false,
                ),
                ExtensionOp::new(
                    "internal_writable_stream_unlock",
                    Self::internal_writable_stream_unlock,
                    1,
                    false,
                ),
                ExtensionOp::new(
                    "internal_stream_set_desired_size",
                    Self::internal_stream_set_desired_size,
                    2,
                    false,
                ),
                ExtensionOp::new(
                    "internal_stream_get_desired_size",
                    Self::internal_stream_get_desired_size,
                    1,
                    false,
                ),
                ExtensionOp::new(
                    "internal_stream_get_chunk_count",
                    Self::internal_stream_get_chunk_count,
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
            readable_state: StreamState::Readable,
            writable_state: WritableStreamState::Closed,
            error_message: None,
            desired_size: 1,
            high_water_mark: 1,
            locked: false,
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
            if stream_data.readable_state == StreamState::Errored {
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

            if stream_data.readable_state == StreamState::Closed && stream_data.chunks.is_empty() {
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
            stream_data.readable_state = StreamState::Closed;
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
            stream_data.readable_state = StreamState::Closed;
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
            if stream_data.readable_state == StreamState::Closed {
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
            readable_state: StreamState::Closed,
            writable_state: WritableStreamState::Writable,
            error_message: None,
            desired_size: 1,
            high_water_mark: 1,
            locked: false,
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
            if stream_data.writable_state != WritableStreamState::Writable {
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
            stream_data.writable_state = WritableStreamState::Closed;
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
            stream_data.writable_state = WritableStreamState::Errored;
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
            // Return state as a formatted string: "readable_state:writable_state:locked:chunk_count"
            let state = format!(
                "{}:{}:{}:{}",
                match stream_data.readable_state {
                    StreamState::Readable => "readable",
                    StreamState::Closed => "closed",
                    StreamState::Errored => "errored",
                },
                match stream_data.writable_state {
                    WritableStreamState::Writable => "writable",
                    WritableStreamState::Closing => "closing",
                    WritableStreamState::Closed => "closed",
                    WritableStreamState::Erroring => "erroring",
                    WritableStreamState::Errored => "errored",
                },
                stream_data.locked,
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

    pub fn internal_readable_stream_error<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let stream_id_arg = args.get(0);
        let error_message_arg = args.get(1);

        let stream_id_str = stream_id_arg.to_string(agent, gc.reborrow()).unbind()?;
        let stream_id = stream_id_str
            .as_str(agent)
            .expect("String is not valid UTF-8");

        let error_message_str = error_message_arg.to_string(agent, gc.reborrow()).unbind()?;
        let error_message = error_message_str
            .as_str(agent)
            .expect("String is not valid UTF-8");

        let storage = Self::get_stream_storage();
        let mut storage_lock = storage.lock().unwrap();

        if let Some(stream_data) = storage_lock.get_mut(stream_id) {
            stream_data.readable_state = StreamState::Errored;
            stream_data.error_message = Some(error_message.to_string());
            stream_data.chunks.clear();
            Ok(Value::from_string(agent, "errored".to_string(), gc.nogc()).unbind())
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

    pub fn internal_readable_stream_lock<'gc>(
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
            if stream_data.locked {
                return Err(agent
                    .throw_exception(
                        ExceptionType::Error,
                        "Stream is already locked".to_string(),
                        gc.nogc(),
                    )
                    .unbind());
            }
            stream_data.locked = true;
            Ok(Value::from_string(agent, "locked".to_string(), gc.nogc()).unbind())
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

    pub fn internal_readable_stream_unlock<'gc>(
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
            stream_data.locked = false;
            Ok(Value::from_string(agent, "unlocked".to_string(), gc.nogc()).unbind())
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

    pub fn internal_readable_stream_tee<'gc>(
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

        if let Some(original_stream) = storage_lock.get(stream_id).cloned() {
            // Create two new streams
            let stream1_id = Uuid::new_v4().to_string();
            let stream2_id = Uuid::new_v4().to_string();

            // Clone the original stream data for both new streams
            let stream1_data = StreamData {
                chunks: original_stream.chunks.clone(),
                readable_state: original_stream.readable_state.clone(),
                writable_state: WritableStreamState::Closed,
                error_message: original_stream.error_message.clone(),
                desired_size: original_stream.desired_size,
                high_water_mark: original_stream.high_water_mark,
                locked: false,
            };

            let stream2_data = stream1_data.clone();

            storage_lock.insert(stream1_id.clone(), stream1_data);
            storage_lock.insert(stream2_id.clone(), stream2_data);

            // Return both stream IDs as a comma-separated string
            let result = format!("{stream1_id},{stream2_id}");
            Ok(Value::from_string(agent, result, gc.nogc()).unbind())
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

    pub fn internal_writable_stream_error<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let stream_id_arg = args.get(0);
        let error_message_arg = args.get(1);

        let stream_id_str = stream_id_arg.to_string(agent, gc.reborrow()).unbind()?;
        let stream_id = stream_id_str
            .as_str(agent)
            .expect("String is not valid UTF-8");

        let error_message_str = error_message_arg.to_string(agent, gc.reborrow()).unbind()?;
        let error_message = error_message_str
            .as_str(agent)
            .expect("String is not valid UTF-8");

        let storage = Self::get_stream_storage();
        let mut storage_lock = storage.lock().unwrap();

        if let Some(stream_data) = storage_lock.get_mut(stream_id) {
            stream_data.writable_state = WritableStreamState::Errored;
            stream_data.error_message = Some(error_message.to_string());
            stream_data.chunks.clear();
            Ok(Value::from_string(agent, "errored".to_string(), gc.nogc()).unbind())
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

    pub fn internal_writable_stream_lock<'gc>(
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
            if stream_data.locked {
                return Err(agent
                    .throw_exception(
                        ExceptionType::Error,
                        "Stream is already locked".to_string(),
                        gc.nogc(),
                    )
                    .unbind());
            }
            stream_data.locked = true;
            Ok(Value::from_string(agent, "locked".to_string(), gc.nogc()).unbind())
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

    pub fn internal_writable_stream_unlock<'gc>(
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
            stream_data.locked = false;
            Ok(Value::from_string(agent, "unlocked".to_string(), gc.nogc()).unbind())
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

    pub fn internal_stream_set_desired_size<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let stream_id_arg = args.get(0);
        let desired_size_arg = args.get(1);

        let stream_id_str = stream_id_arg.to_string(agent, gc.reborrow()).unbind()?;
        let stream_id = stream_id_str
            .as_str(agent)
            .expect("String is not valid UTF-8");

        // Parse desired size as integer
        let desired_size_str = desired_size_arg.to_string(agent, gc.reborrow()).unbind()?;
        let desired_size_string = desired_size_str
            .as_str(agent)
            .expect("String is not valid UTF-8");

        let desired_size: i32 = desired_size_string.parse().unwrap_or(1);

        let storage = Self::get_stream_storage();
        let mut storage_lock = storage.lock().unwrap();

        if let Some(stream_data) = storage_lock.get_mut(stream_id) {
            stream_data.desired_size = desired_size;
            Ok(Value::from_string(agent, "set".to_string(), gc.nogc()).unbind())
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

    pub fn internal_stream_get_desired_size<'gc>(
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
            Ok(Value::from_string(agent, stream_data.desired_size.to_string(), gc.nogc()).unbind())
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

    pub fn internal_stream_get_chunk_count<'gc>(
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
            Ok(Value::from_string(agent, stream_data.chunks.len().to_string(), gc.nogc()).unbind())
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
