// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use andromeda_core::{Extension, ExtensionOp, HostData};
use nova_vm::{
    ecmascript::{
        builtins::{ArgumentsList, Array},
        execution::{Agent, JsResult},
        types::Value,
    },
    engine::context::{Bindable, GcScope},
};
use rusqlite::{Connection, OptionalExtension, params};

const MAX_STORAGE_BYTES: usize = 10 * 1024 * 1024; // 10MB

#[derive(Debug, thiserror::Error)]
pub enum WebStorageError {
    #[error("LocalStorage is not supported in this context")]
    ContextNotSupported,
    #[error(transparent)]
    Sqlite(#[from] rusqlite::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("Exceeded maximum storage size")]
    StorageExceeded,
}

struct LocalStorage(Connection);
struct SessionStorage(Connection);

fn extract_string(agent: &Agent, value: Value) -> Option<String> {
    match value {
        Value::String(s) => Some(s.as_str(agent).to_string()),
        Value::SmallString(s) => Some(s.as_str().to_string()),
        _ => None,
    }
}

fn with_webstorage<T, F>(
    host_data: &HostData<crate::RuntimeMacroTask>,
    persistent: bool,
    operation: F,
) -> Result<T, WebStorageError>
where
    F: FnOnce(&Connection) -> Result<T, WebStorageError>,
{
    if persistent {
        if host_data.storage.borrow().get::<LocalStorage>().is_none() {
            let storage_dir = std::env::temp_dir().join("andromeda_storage");
            std::fs::create_dir_all(&storage_dir)?;

            let conn = Connection::open(storage_dir.join("local_storage"))?;

            let initial_pragmas = r#"
                PRAGMA journal_mode=WAL;
                PRAGMA synchronous=NORMAL;
                PRAGMA temp_store=memory;
                PRAGMA page_size=4096;
                PRAGMA mmap_size=6000000;
                PRAGMA optimize;
            "#;

            conn.execute_batch(initial_pragmas)?;
            conn.set_prepared_statement_cache_capacity(128);

            // Create table if it doesn't exist
            conn.execute(
                "CREATE TABLE IF NOT EXISTS data (key VARCHAR UNIQUE, value VARCHAR)",
                params![],
            )?;

            host_data.storage.borrow_mut().insert(LocalStorage(conn));
        }

        let storage = host_data.storage.borrow();
        let local_storage = storage.get::<LocalStorage>().unwrap();
        operation(&local_storage.0)
    } else {
        // sessionStorage - in-memory SQLite database
        if host_data.storage.borrow().get::<SessionStorage>().is_none() {
            let conn = Connection::open_in_memory()?;
            conn.execute(
                "CREATE TABLE data (key VARCHAR UNIQUE, value VARCHAR)",
                params![],
            )?;

            host_data.storage.borrow_mut().insert(SessionStorage(conn));
        }

        let storage = host_data.storage.borrow();
        let session_storage = storage.get::<SessionStorage>().unwrap();
        operation(&session_storage.0)
    }
}

fn size_check(input: usize) -> Result<(), WebStorageError> {
    if input >= MAX_STORAGE_BYTES {
        return Err(WebStorageError::StorageExceeded);
    }
    Ok(())
}

#[derive(Default)]
pub struct LocalStorageExt;

impl LocalStorageExt {
    pub fn new_extension() -> Extension {
        Extension {
            name: "localStorage",
            ops: vec![
                ExtensionOp::new("storage_new", Self::storage_new, 1),
                ExtensionOp::new("storage_length", Self::storage_length, 1),
                ExtensionOp::new("storage_key", Self::storage_key, 2),
                ExtensionOp::new("storage_getItem", Self::storage_get_item, 2),
                ExtensionOp::new("storage_setItem", Self::storage_set_item, 3),
                ExtensionOp::new("storage_removeItem", Self::storage_remove_item, 2),
                ExtensionOp::new("storage_clear", Self::storage_clear, 1),
                ExtensionOp::new("storage_iterate_keys", Self::storage_iterate_keys, 1),
            ],
            storage: None,
            files: vec![include_str!("local_storage.ts")],
        }
    }

    /// Create a new Storage instance (persistent=true for localStorage, false for sessionStorage)
    fn storage_new<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let persistent = if args.len() > 0 {
            match args.get(0) {
                Value::Boolean(b) => b,
                _ => true, // default to localStorage
            }
        } else {
            true
        };
        let host_data = agent.get_host_data();
        let host_data: &HostData<crate::RuntimeMacroTask> = host_data.downcast_ref().unwrap();

        match with_webstorage(host_data, persistent, |_conn| Ok(())) {
            Ok(_) => Ok(Value::Boolean(persistent)),
            Err(_) => Err(agent
                .throw_exception_with_static_message(
                    nova_vm::ecmascript::execution::agent::ExceptionType::Error,
                    "LocalStorage is not supported in this context",
                    gc.nogc(),
                )
                .unbind()),
        }
    }

    /// Get the number of items in storage
    fn storage_length<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let persistent = match args.get(0) {
            Value::Boolean(b) => b,
            _ => true,
        };

        let host_data = agent.get_host_data();
        let host_data: &HostData<crate::RuntimeMacroTask> = host_data.downcast_ref().unwrap();
        let length = with_webstorage(host_data, persistent, |conn| {
            let mut stmt = conn.prepare_cached("SELECT COUNT(*) FROM data")?;
            let count: u32 = stmt.query_row(params![], |row| row.get(0))?;
            Ok(count as f64)
        })
        .unwrap_or(0.0);

        Ok(Value::from_f64(agent, length, gc.nogc()).unbind())
    }

    /// Get a key by index
    fn storage_key<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let persistent = match args.get(0) {
            Value::Boolean(b) => b,
            _ => true,
        };
        let index = match args.get(1).to_number(agent, gc.reborrow()) {
            Ok(num) => {
                let f = num.unbind().into_f64(agent);
                if f.is_finite() && f >= 0.0 && f <= u32::MAX as f64 {
                    f as u32
                } else {
                    return Ok(Value::Null);
                }
            }
            Err(_) => return Ok(Value::Null),
        };

        let host_data = agent.get_host_data();
        let host_data: &HostData<crate::RuntimeMacroTask> = host_data.downcast_ref().unwrap();
        let key_result = with_webstorage(host_data, persistent, |conn| {
            let mut stmt = conn.prepare_cached("SELECT key FROM data LIMIT 1 OFFSET ?")?;
            let key: Option<String> = stmt
                .query_row(params![index], |row| row.get(0))
                .optional()?;
            Ok(key)
        });

        match key_result {
            Ok(Some(key)) => Ok(Value::from_string(agent, key, gc.nogc()).unbind()),
            _ => Ok(Value::Null),
        }
    }

    /// Get an item by key
    fn storage_get_item<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let persistent = match args.get(0) {
            Value::Boolean(b) => b,
            _ => true,
        };
        let key = match extract_string(agent, args.get(1)) {
            Some(k) => k,
            None => return Ok(Value::Null),
        };

        let host_data = agent.get_host_data();
        let host_data: &HostData<crate::RuntimeMacroTask> = host_data.downcast_ref().unwrap();
        let result = with_webstorage(host_data, persistent, |conn| {
            let mut stmt = conn.prepare_cached("SELECT value FROM data WHERE key = ?")?;
            let value: Option<String> =
                stmt.query_row(params![key], |row| row.get(0)).optional()?;
            Ok(value)
        });

        match result {
            Ok(Some(value)) => Ok(Value::from_string(agent, value, gc.nogc()).unbind()),
            _ => Ok(Value::Null),
        }
    }

    /// Set an item
    fn storage_set_item<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let persistent = match args.get(0) {
            Value::Boolean(b) => b,
            _ => true,
        };
        let key = match extract_string(agent, args.get(1)) {
            Some(k) => k,
            None => return Ok(Value::Undefined),
        };

        let value = match extract_string(agent, args.get(2)) {
            Some(v) => v,
            None => return Ok(Value::Undefined),
        };
        if let Err(_) = size_check(key.len() + value.len()) {
            return Err(agent
                .throw_exception_with_static_message(
                    nova_vm::ecmascript::execution::agent::ExceptionType::Error,
                    "Exceeded maximum storage size",
                    gc.nogc(),
                )
                .unbind());
        }

        let host_data = agent.get_host_data();
        let host_data: &HostData<crate::RuntimeMacroTask> = host_data.downcast_ref().unwrap();
        let result = with_webstorage(host_data, persistent, |conn| {
            let mut stmt = conn.prepare_cached("SELECT COUNT(*) FROM data")?;
            let count: u32 = stmt.query_row(params![], |row| row.get(0))?;

            if count > 10000 {
                return Err(WebStorageError::StorageExceeded);
            }

            let mut stmt =
                conn.prepare_cached("INSERT OR REPLACE INTO data (key, value) VALUES (?, ?)")?;
            stmt.execute(params![key, value])?;
            Ok(())
        });

        match result {
            Ok(_) => Ok(Value::Undefined),
            Err(WebStorageError::StorageExceeded) => Err(agent
                .throw_exception_with_static_message(
                    nova_vm::ecmascript::execution::agent::ExceptionType::Error,
                    "Exceeded maximum storage size",
                    gc.nogc(),
                )
                .unbind()),
            Err(_) => Err(agent
                .throw_exception_with_static_message(
                    nova_vm::ecmascript::execution::agent::ExceptionType::Error,
                    "Failed to store item",
                    gc.nogc(),
                )
                .unbind()),
        }
    }

    /// Remove an item
    fn storage_remove_item<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        _gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let persistent = match args.get(0) {
            Value::Boolean(b) => b,
            _ => true,
        };
        let key = match extract_string(agent, args.get(1)) {
            Some(k) => k,
            None => return Ok(Value::Undefined),
        };
        let host_data = agent.get_host_data();
        let host_data: &HostData<crate::RuntimeMacroTask> = host_data.downcast_ref().unwrap();

        let _ = with_webstorage(host_data, persistent, |conn| {
            let mut stmt = conn.prepare_cached("DELETE FROM data WHERE key = ?")?;
            stmt.execute(params![key])?;
            Ok(())
        });

        Ok(Value::Undefined)
    }

    /// Clear all items
    fn storage_clear<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        _gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let persistent = match args.get(0) {
            Value::Boolean(b) => b,
            _ => true,
        };
        let host_data = agent.get_host_data();
        let host_data: &HostData<crate::RuntimeMacroTask> = host_data.downcast_ref().unwrap();

        let _ = with_webstorage(host_data, persistent, |conn| {
            let mut stmt = conn.prepare_cached("DELETE FROM data")?;
            stmt.execute(params![])?;
            Ok(())
        });

        Ok(Value::Undefined)
    }

    /// Iterate over keys (for implementing ownKeys in proxy)
    fn storage_iterate_keys<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let persistent = match args.get(0) {
            Value::Boolean(b) => b,
            _ => true,
        };
        let host_data = agent.get_host_data();
        let host_data: &HostData<crate::RuntimeMacroTask> = host_data.downcast_ref().unwrap();

        let keys_result = with_webstorage(host_data, persistent, |conn| {
            let mut stmt = conn.prepare_cached("SELECT key FROM data")?;
            let keys: Result<Vec<String>, _> = stmt
                .query_map(params![], |row| row.get::<_, String>(0))?
                .collect();
            Ok(keys.unwrap_or_default())
        });

        match keys_result {
            Ok(keys) => {
                let key_values: Vec<Value> = keys
                    .into_iter()
                    .map(|k| Value::from_string(agent, k, gc.nogc()).unbind())
                    .collect();

                let array = Array::from_slice(agent, key_values.as_slice(), gc.nogc())
                    .unbind()
                    .into();
                Ok(array)
            }
            Err(_) => {
                let empty_array = Array::from_slice(agent, &[], gc.nogc()).unbind().into();
                Ok(empty_array)
            }
        }
    }
}
