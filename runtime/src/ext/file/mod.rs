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

// Resource table for managing blob data
static BLOB_STORAGE: std::sync::OnceLock<Arc<Mutex<HashMap<String, BlobData>>>> =
    std::sync::OnceLock::new();

// Internal blob data structure
#[derive(Clone)]
struct BlobData {
    data: Vec<u8>,
    content_type: String,
    size: usize,
}

#[derive(Default)]
pub struct FileExt;

impl FileExt {
    pub fn new_extension() -> Extension {
        Extension {
            name: "file",
            ops: vec![
                // Blob operations
                ExtensionOp::new("internal_blob_create", Self::internal_blob_create, 2),
                ExtensionOp::new("internal_blob_slice", Self::internal_blob_slice, 4),
                ExtensionOp::new("internal_blob_get_data", Self::internal_blob_get_data, 1),
                ExtensionOp::new("internal_blob_get_size", Self::internal_blob_get_size, 1),
                ExtensionOp::new("internal_blob_get_type", Self::internal_blob_get_type, 1),
                ExtensionOp::new("internal_blob_stream", Self::internal_blob_stream, 1),
                ExtensionOp::new(
                    "internal_blob_array_buffer",
                    Self::internal_blob_array_buffer,
                    1,
                ),
                ExtensionOp::new("internal_blob_text", Self::internal_blob_text, 1),
                // File operations
                ExtensionOp::new("internal_file_create", Self::internal_file_create, 4),
                // FormData operations
                ExtensionOp::new(
                    "internal_formdata_create",
                    Self::internal_formdata_create,
                    0,
                ),
                ExtensionOp::new(
                    "internal_formdata_append",
                    Self::internal_formdata_append,
                    3,
                ),
                ExtensionOp::new(
                    "internal_formdata_delete",
                    Self::internal_formdata_delete,
                    2,
                ),
                ExtensionOp::new("internal_formdata_get", Self::internal_formdata_get, 2),
                ExtensionOp::new(
                    "internal_formdata_get_all",
                    Self::internal_formdata_get_all,
                    2,
                ),
                ExtensionOp::new("internal_formdata_has", Self::internal_formdata_has, 2),
                ExtensionOp::new("internal_formdata_set", Self::internal_formdata_set, 3),
                ExtensionOp::new("internal_formdata_keys", Self::internal_formdata_keys, 1),
                ExtensionOp::new(
                    "internal_formdata_values",
                    Self::internal_formdata_values,
                    1,
                ),
                ExtensionOp::new(
                    "internal_formdata_entries",
                    Self::internal_formdata_entries,
                    1,
                ),
            ],
            storage: None,
            files: vec![
                include_str!("./blob.ts"),
                include_str!("./file.ts"),
                include_str!("./form_data.ts"),
            ],
        }
    }

    fn get_blob_storage() -> Arc<Mutex<HashMap<String, BlobData>>> {
        BLOB_STORAGE
            .get_or_init(|| Arc::new(Mutex::new(HashMap::new())))
            .clone()
    }

    pub fn internal_blob_create<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        // args: [parts_array, options]
        let parts_arg = args.get(0);
        let options_arg = args.get(1);

        // Parse options
        let content_type = if options_arg.is_undefined() || options_arg.is_null() {
            String::new()
        } else {
            // For now, expect a string representation of type
            let type_str = options_arg.to_string(agent, gc.reborrow()).unbind()?;
            type_str
                .as_str(agent)
                .expect("String is not valid UTF-8")
                .to_string()
        };

        // Parse parts - expect comma-separated byte values
        let mut blob_data = Vec::new();
        if !parts_arg.is_undefined() && !parts_arg.is_null() {
            let parts_str = parts_arg.to_string(agent, gc.reborrow()).unbind()?;
            let parts_string = parts_str.as_str(agent).expect("String is not valid UTF-8");

            if !parts_string.is_empty() {
                blob_data = parts_string
                    .split(',')
                    .filter_map(|s| s.trim().parse::<u8>().ok())
                    .collect();
            }
        }

        let size = blob_data.len();
        let blob_id = Uuid::new_v4().to_string();

        let blob = BlobData {
            data: blob_data,
            content_type: content_type.clone(),
            size,
        };

        let storage = Self::get_blob_storage();
        storage.lock().unwrap().insert(blob_id.clone(), blob);

        let gc_no = gc.into_nogc();
        Ok(Value::from_string(agent, blob_id, gc_no).unbind())
    }

    pub fn internal_blob_slice<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        // args: [blob_id, start, end, content_type]
        let blob_id_arg = args.get(0);
        let start_arg = args.get(1);
        let end_arg = args.get(2);
        let content_type_arg = args.get(3);

        let blob_id_str = blob_id_arg.to_string(agent, gc.reborrow()).unbind()?;
        let blob_id = blob_id_str
            .as_str(agent)
            .expect("String is not valid UTF-8");

        let start = if start_arg.is_undefined() {
            0
        } else {
            start_arg
                .to_number(agent, gc.reborrow())
                .unbind()?
                .into_f64(agent) as usize
        };

        let content_type = if content_type_arg.is_undefined() {
            String::new()
        } else {
            let type_str = content_type_arg.to_string(agent, gc.reborrow()).unbind()?;
            type_str
                .as_str(agent)
                .expect("String is not valid UTF-8")
                .to_string()
        };

        let storage = Self::get_blob_storage();
        let storage_lock = storage.lock().unwrap();

        if let Some(blob) = storage_lock.get(blob_id) {
            let end = if end_arg.is_undefined() {
                blob.size
            } else {
                (end_arg
                    .to_number(agent, gc.reborrow())
                    .unbind()?
                    .into_f64(agent) as usize)
                    .min(blob.size)
            };

            let start = start.min(blob.size);
            let end = end.max(start);

            let sliced_data = blob.data[start..end].to_vec();
            let new_blob_id = Uuid::new_v4().to_string();

            let new_blob = BlobData {
                data: sliced_data,
                content_type,
                size: end - start,
            };

            drop(storage_lock);
            storage
                .lock()
                .unwrap()
                .insert(new_blob_id.clone(), new_blob);

            Ok(Value::from_string(agent, new_blob_id, gc.nogc()).unbind())
        } else {
            Err(agent
                .throw_exception(
                    ExceptionType::Error,
                    "Invalid blob ID".to_string(),
                    gc.nogc(),
                )
                .unbind())
        }
    }

    pub fn internal_blob_get_data<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let blob_id_arg = args.get(0);

        let blob_id_str = blob_id_arg.to_string(agent, gc.reborrow()).unbind()?;
        let blob_id = blob_id_str
            .as_str(agent)
            .expect("String is not valid UTF-8");

        let storage = Self::get_blob_storage();
        let storage_lock = storage.lock().unwrap();

        if let Some(blob) = storage_lock.get(blob_id) {
            let bytes_str = blob
                .data
                .iter()
                .map(|b| b.to_string())
                .collect::<Vec<_>>()
                .join(",");
            Ok(Value::from_string(agent, bytes_str, gc.nogc()).unbind())
        } else {
            Err(agent
                .throw_exception(
                    ExceptionType::Error,
                    "Invalid blob ID".to_string(),
                    gc.nogc(),
                )
                .unbind())
        }
    }

    pub fn internal_blob_get_size<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let blob_id_arg = args.get(0);

        let blob_id_str = blob_id_arg.to_string(agent, gc.reborrow()).unbind()?;
        let blob_id = blob_id_str
            .as_str(agent)
            .expect("String is not valid UTF-8");

        let storage = Self::get_blob_storage();
        let storage_lock = storage.lock().unwrap();

        if let Some(blob) = storage_lock.get(blob_id) {
            Ok(Value::from_f64(agent, blob.size as f64, gc.nogc()).unbind())
        } else {
            Err(agent
                .throw_exception(
                    ExceptionType::Error,
                    "Invalid blob ID".to_string(),
                    gc.nogc(),
                )
                .unbind())
        }
    }

    pub fn internal_blob_get_type<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let blob_id_arg = args.get(0);

        let blob_id_str = blob_id_arg.to_string(agent, gc.reborrow()).unbind()?;
        let blob_id = blob_id_str
            .as_str(agent)
            .expect("String is not valid UTF-8");

        let storage = Self::get_blob_storage();
        let storage_lock = storage.lock().unwrap();

        if let Some(blob) = storage_lock.get(blob_id) {
            Ok(Value::from_string(agent, blob.content_type.clone(), gc.nogc()).unbind())
        } else {
            Err(agent
                .throw_exception(
                    ExceptionType::Error,
                    "Invalid blob ID".to_string(),
                    gc.nogc(),
                )
                .unbind())
        }
    }

    pub fn internal_blob_stream<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let blob_id_arg = args.get(0);

        let blob_id_str = blob_id_arg.to_string(agent, gc.reborrow()).unbind()?;
        let blob_id = blob_id_str
            .as_str(agent)
            .expect("String is not valid UTF-8");

        let storage = Self::get_blob_storage();
        let storage_lock = storage.lock().unwrap();

        if let Some(blob) = storage_lock.get(blob_id) {
            // Return the blob data as comma-separated bytes for now
            // TODO: return a ReadableStream
            let bytes_str = blob
                .data
                .iter()
                .map(|b| b.to_string())
                .collect::<Vec<_>>()
                .join(",");
            Ok(Value::from_string(agent, bytes_str, gc.nogc()).unbind())
        } else {
            Err(agent
                .throw_exception(
                    ExceptionType::Error,
                    "Invalid blob ID".to_string(),
                    gc.nogc(),
                )
                .unbind())
        }
    }

    pub fn internal_blob_array_buffer<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let blob_id_arg = args.get(0);

        let blob_id_str = blob_id_arg.to_string(agent, gc.reborrow()).unbind()?;
        let blob_id = blob_id_str
            .as_str(agent)
            .expect("String is not valid UTF-8");

        let storage = Self::get_blob_storage();
        let storage_lock = storage.lock().unwrap();

        if let Some(blob) = storage_lock.get(blob_id) {
            // Return the blob data as comma-separated bytes
            let bytes_str = blob
                .data
                .iter()
                .map(|b| b.to_string())
                .collect::<Vec<_>>()
                .join(",");
            Ok(Value::from_string(agent, bytes_str, gc.nogc()).unbind())
        } else {
            Err(agent
                .throw_exception(
                    ExceptionType::Error,
                    "Invalid blob ID".to_string(),
                    gc.nogc(),
                )
                .unbind())
        }
    }

    pub fn internal_blob_text<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let blob_id_arg = args.get(0);

        let blob_id_str = blob_id_arg.to_string(agent, gc.reborrow()).unbind()?;
        let blob_id = blob_id_str
            .as_str(agent)
            .expect("String is not valid UTF-8");

        let storage = Self::get_blob_storage();
        let storage_lock = storage.lock().unwrap();

        if let Some(blob) = storage_lock.get(blob_id) {
            let text = String::from_utf8_lossy(&blob.data).to_string();
            Ok(Value::from_string(agent, text, gc.nogc()).unbind())
        } else {
            Err(agent
                .throw_exception(
                    ExceptionType::Error,
                    "Invalid blob ID".to_string(),
                    gc.nogc(),
                )
                .unbind())
        }
    }

    pub fn internal_file_create<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        // args: [parts_array, name, options, last_modified]
        let parts_arg = args.get(0);
        let name_arg = args.get(1);
        let options_arg = args.get(2);
        let last_modified_arg = args.get(3);

        let name_str = name_arg.to_string(agent, gc.reborrow()).unbind()?;
        let name = name_str
            .as_str(agent)
            .expect("String is not valid UTF-8")
            .to_string();

        let last_modified = if last_modified_arg.is_undefined() {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as f64
        } else {
            last_modified_arg
                .to_number(agent, gc.reborrow())
                .unbind()?
                .into_f64(agent)
        };

        // Parse options for content type
        let content_type = if options_arg.is_undefined() || options_arg.is_null() {
            String::new()
        } else {
            let type_str = options_arg.to_string(agent, gc.reborrow()).unbind()?;
            type_str
                .as_str(agent)
                .expect("String is not valid UTF-8")
                .to_string()
        };

        // Parse parts - expect comma-separated byte values
        let mut blob_data = Vec::new();
        if !parts_arg.is_undefined() && !parts_arg.is_null() {
            let parts_str = parts_arg.to_string(agent, gc.reborrow()).unbind()?;
            let parts_string = parts_str.as_str(agent).expect("String is not valid UTF-8");

            if !parts_string.is_empty() {
                blob_data = parts_string
                    .split(',')
                    .filter_map(|s| s.trim().parse::<u8>().ok())
                    .collect();
            }
        }

        let size = blob_data.len();
        let blob_id = Uuid::new_v4().to_string();

        let blob = BlobData {
            data: blob_data,
            content_type: content_type.clone(),
            size,
        };

        let storage = Self::get_blob_storage();
        storage.lock().unwrap().insert(blob_id.clone(), blob);

        // Return combined data: blob_id:name:last_modified
        let result = format!("{blob_id}:{name}:{last_modified}");
        Ok(Value::from_string(agent, result, gc.nogc()).unbind())
    }

    // FormData operations
    pub fn internal_formdata_create<'gc>(
        agent: &mut Agent,
        _this: Value,
        _args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let formdata_id = Uuid::new_v4().to_string();
        Ok(Value::from_string(agent, formdata_id, gc.nogc()).unbind())
    }

    pub fn internal_formdata_append<'gc>(
        agent: &mut Agent,
        _this: Value,
        _args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        // For now, return success - full implementation would store in a FormData resource table
        Ok(Value::from_string(agent, "success".to_string(), gc.nogc()).unbind())
    }

    pub fn internal_formdata_delete<'gc>(
        agent: &mut Agent,
        _this: Value,
        _args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        Ok(Value::from_string(agent, "success".to_string(), gc.nogc()).unbind())
    }

    pub fn internal_formdata_get<'gc>(
        agent: &mut Agent,
        _this: Value,
        _args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        Ok(Value::from_string(agent, String::new(), gc.nogc()).unbind())
    }

    pub fn internal_formdata_get_all<'gc>(
        agent: &mut Agent,
        _this: Value,
        _args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        Ok(Value::from_string(agent, "[]".to_string(), gc.nogc()).unbind())
    }

    pub fn internal_formdata_has<'gc>(
        agent: &mut Agent,
        _this: Value,
        _args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        Ok(Value::from_f64(agent, 0.0, gc.nogc()).unbind()) // false
    }

    pub fn internal_formdata_set<'gc>(
        agent: &mut Agent,
        _this: Value,
        _args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        Ok(Value::from_string(agent, "success".to_string(), gc.nogc()).unbind())
    }

    pub fn internal_formdata_keys<'gc>(
        agent: &mut Agent,
        _this: Value,
        _args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        Ok(Value::from_string(agent, "[]".to_string(), gc.nogc()).unbind())
    }

    pub fn internal_formdata_values<'gc>(
        agent: &mut Agent,
        _this: Value,
        _args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        Ok(Value::from_string(agent, "[]".to_string(), gc.nogc()).unbind())
    }

    pub fn internal_formdata_entries<'gc>(
        agent: &mut Agent,
        _this: Value,
        _args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        Ok(Value::from_string(agent, "[]".to_string(), gc.nogc()).unbind())
    }
}
