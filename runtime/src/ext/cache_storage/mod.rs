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
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(dead_code)]
const MAX_CACHE_SIZE: usize = 100 * 1024 * 1024; // 100MB per cache
#[allow(dead_code)]
const MAX_TOTAL_CACHE_SIZE: usize = 500 * 1024 * 1024; // 500MB total

#[derive(Debug, thiserror::Error)]
pub enum CacheStorageError {
    #[error("CacheStorage is not supported in this context")]
    ContextNotSupported,
    #[error(transparent)]
    Sqlite(#[from] rusqlite::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("Exceeded maximum cache size")]
    CacheExceeded,
    #[error("Cache not found")]
    CacheNotFound,
    #[error("Failed to serialize response")]
    SerializationError,
}

#[derive(Debug, Serialize, Deserialize)]
struct CachedResponse {
    status: u16,
    status_text: String,
    headers: HashMap<String, String>,
    body: Vec<u8>,
    url: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct CachedRequest {
    url: String,
    method: String,
    headers: HashMap<String, String>,
}

struct CacheStorageManager(Connection);

fn extract_string(agent: &Agent, value: Value) -> Option<String> {
    match value {
        Value::String(s) => Some(
            s.as_str(agent)
                .expect("String is not valid UTF-8")
                .to_string(),
        ),
        Value::SmallString(s) => Some(s.as_str().expect("String is not valid UTF-8").to_string()),
        _ => None,
    }
}

fn with_cache_storage<T, F>(
    host_data: &HostData<crate::RuntimeMacroTask>,
    operation: F,
) -> Result<T, CacheStorageError>
where
    F: FnOnce(&Connection) -> Result<T, CacheStorageError>,
{
    if host_data
        .storage
        .borrow()
        .get::<CacheStorageManager>()
        .is_none()
    {
        let storage_dir = std::env::temp_dir().join("andromeda_cache_storage");
        std::fs::create_dir_all(&storage_dir)?;

        let conn = Connection::open(storage_dir.join("cache_storage.db"))?;

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

        // Create tables if they don't exist
        conn.execute(
            r#"CREATE TABLE IF NOT EXISTS caches (
                name TEXT PRIMARY KEY,
                created_at INTEGER DEFAULT (strftime('%s', 'now'))
            )"#,
            params![],
        )?;

        conn.execute(
            r#"CREATE TABLE IF NOT EXISTS cache_entries (
                cache_name TEXT,
                request_key TEXT,
                request_data TEXT,
                response_data TEXT,
                created_at INTEGER DEFAULT (strftime('%s', 'now')),
                PRIMARY KEY (cache_name, request_key),
                FOREIGN KEY (cache_name) REFERENCES caches(name) ON DELETE CASCADE
            )"#,
            params![],
        )?;

        // Create indexes for performance
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_cache_entries_cache_name ON cache_entries(cache_name)",
            params![],
        )?;

        host_data
            .storage
            .borrow_mut()
            .insert(CacheStorageManager(conn));
    }

    let storage = host_data.storage.borrow();
    let cache_storage = storage.get::<CacheStorageManager>().unwrap();
    operation(&cache_storage.0)
}

#[allow(dead_code)]
fn size_check(total_size: usize, cache_size: usize) -> Result<(), CacheStorageError> {
    if total_size >= MAX_TOTAL_CACHE_SIZE || cache_size >= MAX_CACHE_SIZE {
        return Err(CacheStorageError::CacheExceeded);
    }
    Ok(())
}

#[allow(dead_code)]
fn serialize_request(request: &CachedRequest) -> Result<String, CacheStorageError> {
    serde_json::to_string(request).map_err(|_| CacheStorageError::SerializationError)
}

#[allow(dead_code)]
fn serialize_response(response: &CachedResponse) -> Result<String, CacheStorageError> {
    serde_json::to_string(response).map_err(|_| CacheStorageError::SerializationError)
}

#[allow(dead_code)]
fn deserialize_request(data: &str) -> Result<CachedRequest, CacheStorageError> {
    serde_json::from_str(data).map_err(|_| CacheStorageError::SerializationError)
}

#[allow(dead_code)]
fn deserialize_response(data: &str) -> Result<CachedResponse, CacheStorageError> {
    serde_json::from_str(data).map_err(|_| CacheStorageError::SerializationError)
}

#[allow(dead_code)]
fn generate_request_key(request: &CachedRequest) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    request.url.hash(&mut hasher);
    request.method.hash(&mut hasher);
    format!("{}:{}:{:x}", request.method, request.url, hasher.finish())
}

#[derive(Default)]
pub struct CacheStorageExt;

impl CacheStorageExt {
    #[hotpath::measure]
    pub fn new_extension() -> Extension {
        Extension {
            name: "cacheStorage",
            ops: vec![
                // CacheStorage methods
                ExtensionOp::new("cacheStorage_open", Self::cache_storage_open, 1, false),
                ExtensionOp::new("cacheStorage_has", Self::cache_storage_has, 1, false),
                ExtensionOp::new("cacheStorage_delete", Self::cache_storage_delete, 1, false),
                ExtensionOp::new("cacheStorage_keys", Self::cache_storage_keys, 0, false),
                ExtensionOp::new("cacheStorage_match", Self::cache_storage_match, 1, false),
                // Cache methods
                ExtensionOp::new("cache_match", Self::cache_match, 2, false),
                ExtensionOp::new("cache_matchAll", Self::cache_match_all, 2, false),
                ExtensionOp::new("cache_add", Self::cache_add, 2, false),
                ExtensionOp::new("cache_addAll", Self::cache_add_all, 2, false),
                ExtensionOp::new("cache_put", Self::cache_put, 3, false),
                ExtensionOp::new("cache_delete", Self::cache_delete, 2, false),
                ExtensionOp::new("cache_keys", Self::cache_keys, 1, false),
            ],
            storage: None,
            files: vec![include_str!("cache_storage.ts")],
        }
    }

    /// Open a cache with the given name
    fn cache_storage_open<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let cache_name = match extract_string(agent, args.get(0)) {
            Some(name) => name,
            None => return Ok(Value::Undefined),
        };

        let host_data = agent.get_host_data();
        let host_data: &HostData<crate::RuntimeMacroTask> = host_data.downcast_ref().unwrap();

        let result = with_cache_storage(host_data, |conn| {
            // Create cache entry if it doesn't exist
            let mut stmt = conn.prepare_cached("INSERT OR IGNORE INTO caches (name) VALUES (?)")?;
            stmt.execute(params![cache_name])?;
            Ok(())
        });

        match result {
            Ok(_) => {
                // For now, return undefined directly - in a full implementation,
                // you would return a proper Cache object
                Ok(Value::Undefined)
            }
            Err(_) => Err(agent
                .throw_exception_with_static_message(
                    nova_vm::ecmascript::execution::agent::ExceptionType::Error,
                    "Failed to open cache",
                    gc.nogc(),
                )
                .unbind()),
        }
    }

    /// Check if a cache with the given name exists
    fn cache_storage_has<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        _gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let cache_name = match extract_string(agent, args.get(0)) {
            Some(name) => name,
            None => return Ok(Value::Boolean(false)),
        };

        let host_data = agent.get_host_data();
        let host_data: &HostData<crate::RuntimeMacroTask> = host_data.downcast_ref().unwrap();

        let result = with_cache_storage(host_data, |conn| {
            let mut stmt = conn.prepare_cached("SELECT 1 FROM caches WHERE name = ?")?;
            let exists = stmt.query_row(params![cache_name], |_| Ok(())).is_ok();
            Ok(exists)
        });

        let exists = result.unwrap_or(false);
        Ok(Value::Boolean(exists))
    }

    /// Delete a cache with the given name
    fn cache_storage_delete<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        _gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let cache_name = match extract_string(agent, args.get(0)) {
            Some(name) => name,
            None => return Ok(Value::Boolean(false)),
        };

        let host_data = agent.get_host_data();
        let host_data: &HostData<crate::RuntimeMacroTask> = host_data.downcast_ref().unwrap();

        let result = with_cache_storage(host_data, |conn| {
            let mut stmt = conn.prepare_cached("DELETE FROM caches WHERE name = ?")?;
            let affected = stmt.execute(params![cache_name])?;
            Ok(affected > 0)
        });

        let deleted = result.unwrap_or(false);
        Ok(Value::Boolean(deleted))
    }

    /// Get all cache names
    fn cache_storage_keys<'gc>(
        agent: &mut Agent,
        _this: Value,
        _args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let host_data = agent.get_host_data();
        let host_data: &HostData<crate::RuntimeMacroTask> = host_data.downcast_ref().unwrap();

        let result = with_cache_storage(host_data, |conn| {
            let mut stmt = conn.prepare_cached("SELECT name FROM caches ORDER BY name")?;
            let cache_names: Result<Vec<String>, _> = stmt
                .query_map(params![], |row| row.get::<_, String>(0))?
                .collect();
            Ok(cache_names.unwrap_or_default())
        });

        let cache_names = result.unwrap_or_default();
        let name_values: Vec<Value> = cache_names
            .into_iter()
            .map(|name| Value::from_string(agent, name, gc.nogc()).unbind())
            .collect();

        let array = Array::from_slice(agent, name_values.as_slice(), gc.nogc())
            .unbind()
            .into();

        Ok(array)
    }

    /// Match a request across all caches
    fn cache_storage_match<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        _gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let request_url = match extract_string(agent, args.get(0)) {
            Some(url) => url,
            None => return Ok(Value::Undefined),
        };

        let host_data = agent.get_host_data();
        let host_data: &HostData<crate::RuntimeMacroTask> = host_data.downcast_ref().unwrap();

        let result = with_cache_storage(host_data, |conn| {
            // Create a request to generate the same key
            let cached_request = CachedRequest {
                url: request_url,
                method: "GET".to_string(),
                headers: HashMap::new(),
            };

            let request_key = generate_request_key(&cached_request);

            let mut stmt = conn.prepare_cached(
                "SELECT response_data FROM cache_entries WHERE request_key = ? LIMIT 1",
            )?;

            let response_data: Result<String, _> =
                stmt.query_row(params![request_key], |row| row.get(0));

            match response_data {
                Ok(data) => {
                    // For now, return a simple indication that we found something
                    // In a full implementation, you'd deserialize and create a proper Response object
                    Ok(Some(data))
                }
                Err(_) => Ok(None),
            }
        });

        match result {
            Ok(Some(_)) => {
                // Create a simple Response object representation
                // In a full implementation, you'd create a proper Response object
                Ok(Value::Boolean(true)) // Placeholder indicating match found
            }
            _ => Ok(Value::Undefined), // No match found
        }
    }

    /// Match a request in a specific cache
    fn cache_match<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        _gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let cache_name = match extract_string(agent, args.get(0)) {
            Some(name) => name,
            None => return Ok(Value::Undefined),
        };

        let request_url = match extract_string(agent, args.get(1)) {
            Some(url) => url,
            None => return Ok(Value::Undefined),
        };

        let host_data = agent.get_host_data();
        let host_data: &HostData<crate::RuntimeMacroTask> = host_data.downcast_ref().unwrap();

        let result = with_cache_storage(host_data, |conn| {
            // Create a request to generate the same key
            let cached_request = CachedRequest {
                url: request_url.clone(),
                method: "GET".to_string(),
                headers: HashMap::new(),
            };

            let request_key = generate_request_key(&cached_request);

            let mut stmt = conn.prepare_cached(
                "SELECT response_data FROM cache_entries WHERE cache_name = ? AND request_key = ?",
            )?;

            let response_result = stmt.query_row(params![cache_name, request_key], |row| {
                let response_data: String = row.get(0)?;
                Ok(response_data)
            });

            match response_result {
                Ok(response_data) => {
                    // Deserialize the response data back to CachedResponse
                    let cached_response: CachedResponse = serde_json::from_str(&response_data)
                        .map_err(|e| {
                            rusqlite::Error::InvalidColumnType(
                                0,
                                format!("JSON deserialization error: {e}"),
                                rusqlite::types::Type::Text,
                            )
                        })?;

                    Ok(Some(cached_response))
                }
                Err(_) => Ok(None),
            }
        });

        match result {
            Ok(Some(_cached_response)) => {
                // For now, return true indicating that we found cached data
                // In a full implementation, you'd create a proper Response object
                Ok(Value::Boolean(true)) // Placeholder indicating match found with actual data
            }
            _ => Ok(Value::Undefined), // No match found
        }
    }

    /// Match all requests in a cache
    fn cache_match_all<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let cache_name = match extract_string(agent, args.get(0)) {
            Some(name) => name,
            None => {
                let empty_array = Array::from_slice(agent, &[], gc.nogc()).unbind().into();
                return Ok(empty_array);
            }
        };

        let request_url = extract_string(agent, args.get(1));

        let host_data = agent.get_host_data();
        let host_data: &HostData<crate::RuntimeMacroTask> = host_data.downcast_ref().unwrap();

        let result = with_cache_storage(host_data, |conn| {
            let (query, params): (String, Vec<String>) = if let Some(url) = request_url {
                // Match specific request
                let cached_request = CachedRequest {
                    url: url.clone(),
                    method: "GET".to_string(),
                    headers: HashMap::new(),
                };
                let request_key = generate_request_key(&cached_request);
                (
                    "SELECT response_data FROM cache_entries WHERE cache_name = ? AND request_key = ?".to_string(),
                    vec![cache_name, request_key]
                )
            } else {
                // Match all requests in cache
                (
                    "SELECT response_data FROM cache_entries WHERE cache_name = ? ORDER BY created_at".to_string(),
                    vec![cache_name]
                )
            };

            let mut stmt = conn.prepare_cached(&query)?;
            let response_data_results: Result<Vec<String>, _> = stmt
                .query_map(rusqlite::params_from_iter(params.iter()), |row| {
                    row.get::<_, String>(0)
                })?
                .collect();

            match response_data_results {
                Ok(response_data_list) => {
                    let mut responses = Vec::new();
                    for data in response_data_list {
                        if let Ok(_cached_response) = deserialize_response(&data) {
                            // In a full implementation, you'd create proper Response objects
                            // For now, we'll just indicate that responses were found
                            responses.push("response".to_string());
                        }
                    }
                    Ok(responses)
                }
                Err(_) => Ok(Vec::new()),
            }
        });

        let responses = result.unwrap_or_default();
        let response_values: Vec<Value> = responses
            .into_iter()
            .map(|_| Value::Boolean(true)) // Placeholder for actual Response objects
            .collect();

        let array = Array::from_slice(agent, response_values.as_slice(), gc.nogc())
            .unbind()
            .into();

        Ok(array)
    }

    /// Add a URL to the cache (fetch and cache)
    fn cache_add<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        _gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let cache_name = match extract_string(agent, args.get(0)) {
            Some(name) => name,
            None => return Ok(Value::Undefined),
        };

        let request_url = match extract_string(agent, args.get(1)) {
            Some(url) => url,
            None => return Ok(Value::Undefined),
        };

        let host_data = agent.get_host_data();
        let host_data: &HostData<crate::RuntimeMacroTask> = host_data.downcast_ref().unwrap();

        let result = with_cache_storage(host_data, |conn| {
            // Simulate fetching the URL and create a cached entry
            // In a full implementation, you'd actually fetch the URL
            let cached_request = CachedRequest {
                url: request_url.clone(),
                method: "GET".to_string(),
                headers: HashMap::new(),
            };

            let cached_response = CachedResponse {
                status: 200,
                status_text: "OK".to_string(),
                headers: HashMap::new(),
                body: format!("Fetched content from {request_url}").into_bytes(),
                url: request_url.clone(),
            };

            let request_key = generate_request_key(&cached_request);
            let request_data = serialize_request(&cached_request)?;
            let response_data = serialize_response(&cached_response)?;

            let mut stmt = conn.prepare_cached(
                "INSERT OR REPLACE INTO cache_entries (cache_name, request_key, request_data, response_data) VALUES (?, ?, ?, ?)"
            )?;
            stmt.execute(params![
                cache_name,
                request_key,
                request_data,
                response_data
            ])?;

            Ok(())
        });

        match result {
            Ok(_) => Ok(Value::Undefined),
            Err(_) => Ok(Value::Undefined), // Silent failure for now
        }
    }

    /// Add multiple URLs to the cache
    fn cache_add_all<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        _gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let cache_name = match extract_string(agent, args.get(0)) {
            Some(name) => name,
            None => return Ok(Value::Undefined),
        };

        // Extract URLs from array argument
        let urls = match args.get(1) {
            Value::Object(_array_obj) => {
                // Simplified array handling - in a full implementation you'd properly iterate
                // For now, we'll just process one URL as a demo
                vec!["https://example.com/multiple".to_string()]
            }
            _ => return Ok(Value::Undefined),
        };

        let host_data = agent.get_host_data();
        let host_data: &HostData<crate::RuntimeMacroTask> = host_data.downcast_ref().unwrap();

        let result = with_cache_storage(host_data, |conn| {
            for request_url in urls {
                // Simulate fetching each URL and create cached entries
                let cached_request = CachedRequest {
                    url: request_url.clone(),
                    method: "GET".to_string(),
                    headers: HashMap::new(),
                };

                let cached_response = CachedResponse {
                    status: 200,
                    status_text: "OK".to_string(),
                    headers: HashMap::new(),
                    body: format!("Fetched content from {request_url}").into_bytes(),
                    url: request_url.clone(),
                };

                let request_key = generate_request_key(&cached_request);
                let request_data = serialize_request(&cached_request)?;
                let response_data = serialize_response(&cached_response)?;

                let mut stmt = conn.prepare_cached(
                    "INSERT OR REPLACE INTO cache_entries (cache_name, request_key, request_data, response_data) VALUES (?, ?, ?, ?)"
                )?;
                stmt.execute(params![
                    cache_name,
                    request_key,
                    request_data,
                    response_data
                ])?;
            }

            Ok(())
        });

        match result {
            Ok(_) => Ok(Value::Undefined),
            Err(_) => Ok(Value::Undefined), // Silent failure for now
        }
    }

    /// Put a request/response pair in the cache
    fn cache_put<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        _gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let cache_name = match extract_string(agent, args.get(0)) {
            Some(name) => name,
            None => return Ok(Value::Undefined),
        };

        // For now, we'll create a simple request key from the request argument
        // In a full implementation, you'd properly extract Request and Response objects
        let request_url = match extract_string(agent, args.get(1)) {
            Some(url) => url,
            None => "https://example.com/default".to_string(), // Default URL for testing
        };

        let host_data = agent.get_host_data();
        let host_data: &HostData<crate::RuntimeMacroTask> = host_data.downcast_ref().unwrap();

        let result = with_cache_storage(host_data, |conn| {
            // Create a simple cached request and response
            let cached_request = CachedRequest {
                url: request_url.clone(),
                method: "GET".to_string(),
                headers: HashMap::new(),
            };

            let cached_response = CachedResponse {
                status: 200,
                status_text: "OK".to_string(),
                headers: HashMap::new(),
                body: b"cached data".to_vec(),
                url: request_url.clone(),
            };

            let request_key = generate_request_key(&cached_request);
            let request_data = serialize_request(&cached_request)?;
            let response_data = serialize_response(&cached_response)?;

            let mut stmt = conn.prepare_cached(
                "INSERT OR REPLACE INTO cache_entries (cache_name, request_key, request_data, response_data) VALUES (?, ?, ?, ?)"
            )?;
            stmt.execute(params![
                cache_name,
                request_key,
                request_data,
                response_data
            ])?;

            Ok(())
        });

        match result {
            Ok(_) => Ok(Value::Undefined),
            Err(_) => Ok(Value::Undefined), // Silent failure for now
        }
    }

    /// Delete a request from the cache
    fn cache_delete<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        _gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let cache_name = match extract_string(agent, args.get(0)) {
            Some(name) => name,
            None => return Ok(Value::Boolean(false)),
        };

        let request_url = match extract_string(agent, args.get(1)) {
            Some(url) => url,
            None => return Ok(Value::Boolean(false)),
        };

        let host_data = agent.get_host_data();
        let host_data: &HostData<crate::RuntimeMacroTask> = host_data.downcast_ref().unwrap();

        let result = with_cache_storage(host_data, |conn| {
            // Create a request to generate the same key
            let cached_request = CachedRequest {
                url: request_url,
                method: "GET".to_string(),
                headers: HashMap::new(),
            };

            let request_key = generate_request_key(&cached_request);

            let mut stmt = conn.prepare_cached(
                "DELETE FROM cache_entries WHERE cache_name = ? AND request_key = ?",
            )?;

            let affected = stmt.execute(params![cache_name, request_key])?;
            Ok(affected > 0)
        });

        let deleted = result.unwrap_or(false);
        Ok(Value::Boolean(deleted))
    }

    /// Get all cache keys (requests)
    fn cache_keys<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let cache_name = match extract_string(agent, args.get(0)) {
            Some(name) => name,
            None => {
                let empty_array = Array::from_slice(agent, &[], gc.nogc()).unbind().into();
                return Ok(empty_array);
            }
        };

        let host_data = agent.get_host_data();
        let host_data: &HostData<crate::RuntimeMacroTask> = host_data.downcast_ref().unwrap();

        let result = with_cache_storage(host_data, |conn| {
            let mut stmt = conn.prepare_cached(
                "SELECT request_data FROM cache_entries WHERE cache_name = ? ORDER BY created_at",
            )?;

            let request_data_results: Result<Vec<String>, _> = stmt
                .query_map(params![cache_name], |row| row.get::<_, String>(0))?
                .collect();

            match request_data_results {
                Ok(request_data_list) => {
                    let mut request_urls = Vec::new();
                    for data in request_data_list {
                        if let Ok(cached_request) = deserialize_request(&data) {
                            request_urls.push(cached_request.url);
                        }
                    }
                    Ok(request_urls)
                }
                Err(_) => Ok(Vec::new()),
            }
        });

        let request_urls = result.unwrap_or_default();
        let url_values: Vec<Value> = request_urls
            .into_iter()
            .map(|url| Value::from_string(agent, url, gc.nogc()).unbind())
            .collect();

        let array = Array::from_slice(agent, url_values.as_slice(), gc.nogc())
            .unbind()
            .into();

        Ok(array)
    }
}
