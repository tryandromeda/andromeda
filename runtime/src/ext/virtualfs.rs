// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::{
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

use nova_vm::{
    SmallInteger,
    ecmascript::{
        builtins::ArgumentsList,
        execution::{Agent, JsResult},
        types::Value,
    },
    engine::context::{Bindable, GcScope},
};

use andromeda_core::{Extension, ExtensionOp, HostData, OpsStorage, ResourceTable};

use crate::RuntimeMacroTask;
use base64_simd as base64;
use rusqlite::{Connection, OptionalExtension, params};

const CHUNK_SIZE: usize = 64 * 1024; // 64KB chunks for file content
const MAX_VFS_SIZE: usize = 100 * 1024 * 1024; // 100MB virtual filesystem limit

#[derive(Debug, thiserror::Error)]
pub enum VirtualFsError {
    #[error("Virtual filesystem not supported in this context")]
    ContextNotSupported,
    #[error(transparent)]
    Sqlite(#[from] rusqlite::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("Virtual filesystem size exceeded")]
    FilesystemSizeExceeded,
    #[error("Path not found: {0}")]
    PathNotFound(String),
    #[error("Path already exists: {0}")]
    PathAlreadyExists(String),
    #[error("Not a directory: {0}")]
    NotADirectory(String),
    #[error("Not a file: {0}")]
    NotAFile(String),
    #[error("Directory not empty: {0}")]
    DirectoryNotEmpty(String),
    #[error("Invalid path: {0}")]
    InvalidPath(String),
}

struct VirtualFs(Connection);

type NodeInfo = (i64, String, String, i64, i64, i64, i64, i32, Option<String>);

// Dummy file handle for ResourceTable compatibility
#[derive(Debug)]
struct VirtualFileHandle {
    #[allow(dead_code)]
    path: String,
    _node_id: i64,
}

struct VirtualFsExtResources {
    files: ResourceTable<VirtualFileHandle>,
}

fn normalize_path(path: &str) -> String {
    let path = if path.starts_with('/') {
        path.to_string()
    } else {
        format!("/{}", path)
    };

    // Simple path normalization (not comprehensive)
    let parts: Vec<&str> = path
        .split('/')
        .filter(|s| !s.is_empty() && *s != ".")
        .collect();
    let mut normalized = Vec::new();

    for part in parts {
        if part == ".." {
            if !normalized.is_empty() {
                normalized.pop();
            }
        } else {
            normalized.push(part);
        }
    }

    if normalized.is_empty() {
        "/".to_string()
    } else {
        format!("/{}", normalized.join("/"))
    }
}

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

fn current_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

fn with_virtualfs<T, F>(
    host_data: &HostData<crate::RuntimeMacroTask>,
    operation: F,
) -> Result<T, VirtualFsError>
where
    F: FnOnce(&Connection) -> Result<T, VirtualFsError>,
{
    if host_data.storage.borrow().get::<VirtualFs>().is_none() {
        let storage_dir = std::env::temp_dir().join("andromeda_vfs");
        std::fs::create_dir_all(&storage_dir)?;

        let conn = Connection::open(storage_dir.join("virtual_fs.db"))?;

        let initial_pragmas = r#"
            PRAGMA journal_mode=WAL;
            PRAGMA synchronous=NORMAL;
            PRAGMA temp_store=memory;
            PRAGMA page_size=4096;
            PRAGMA mmap_size=8000000;
            PRAGMA foreign_keys=ON;
            PRAGMA optimize;
        "#;

        conn.execute_batch(initial_pragmas)?;
        conn.set_prepared_statement_cache_capacity(256);

        // Create tables if they don't exist
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS vfs_nodes (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                path TEXT NOT NULL UNIQUE,
                name TEXT NOT NULL,
                parent_path TEXT,
                node_type TEXT NOT NULL CHECK(node_type IN ('file', 'directory', 'symlink')),
                size INTEGER DEFAULT 0,
                created_at INTEGER NOT NULL,
                modified_at INTEGER NOT NULL,
                accessed_at INTEGER NOT NULL,
                mode INTEGER DEFAULT 644,
                symlink_target TEXT
            )
            "#,
            params![],
        )?;

        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS vfs_file_content (
                node_id INTEGER NOT NULL,
                chunk_index INTEGER NOT NULL,
                content BLOB NOT NULL,
                PRIMARY KEY (node_id, chunk_index),
                FOREIGN KEY (node_id) REFERENCES vfs_nodes(id) ON DELETE CASCADE
            )
            "#,
            params![],
        )?;

        // Create indexes
        let _ = conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_vfs_nodes_path ON vfs_nodes(path)",
            params![],
        );
        let _ = conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_vfs_nodes_parent ON vfs_nodes(parent_path)",
            params![],
        );

        // Create root directory if it doesn't exist
        let current_time = current_timestamp();
        let _ = conn.execute(
            "INSERT OR IGNORE INTO vfs_nodes (path, name, parent_path, node_type, created_at, modified_at, accessed_at, mode) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
            params!["/", "", None::<String>, "directory", current_time, current_time, current_time, 755],
        );

        host_data.storage.borrow_mut().insert(VirtualFs(conn));
    }

    let storage = host_data.storage.borrow();
    let vfs = storage.get::<VirtualFs>().unwrap();
    operation(&vfs.0)
}

#[derive(Default)]
pub struct VirtualFsExt;

impl VirtualFsExt {
    #[cfg_attr(feature = "hotpath", hotpath::measure)]
    pub fn new_extension() -> Extension {
        Extension {
            name: "vfs",
            ops: vec![
                ExtensionOp::new(
                    "internal_read_text_file",
                    Self::internal_read_text_file,
                    1,
                    false,
                ),
                ExtensionOp::new(
                    "internal_write_text_file",
                    Self::internal_write_text_file,
                    2,
                    false,
                ),
                ExtensionOp::new("internal_create_file", Self::internal_create_file, 1, false),
                ExtensionOp::new("internal_copy_file", Self::internal_copy_file, 2, false),
                ExtensionOp::new("internal_mk_dir", Self::internal_mk_dir, 1, false),
                ExtensionOp::new("internal_mk_dir_all", Self::internal_mk_dir_all, 1, false),
                ExtensionOp::new("internal_open_file", Self::internal_open_file, 1, false),
                ExtensionOp::new("internal_read_file", Self::internal_read_file, 1, false),
                ExtensionOp::new("internal_write_file", Self::internal_write_file, 2, false),
                ExtensionOp::new("internal_stat", Self::internal_stat, 1, false),
                ExtensionOp::new("internal_lstat", Self::internal_lstat, 1, false),
                ExtensionOp::new("internal_read_dir", Self::internal_read_dir, 1, false),
                ExtensionOp::new("internal_remove", Self::internal_remove, 1, false),
                ExtensionOp::new("internal_remove_all", Self::internal_remove_all, 1, false),
                ExtensionOp::new("internal_rename", Self::internal_rename, 2, false),
                ExtensionOp::new("internal_exists", Self::internal_exists, 1, false),
                ExtensionOp::new("internal_truncate", Self::internal_truncate, 2, false),
                ExtensionOp::new("internal_chmod", Self::internal_chmod, 2, false),
                ExtensionOp::new("internal_symlink", Self::internal_symlink, 2, false),
                ExtensionOp::new("internal_read_link", Self::internal_read_link, 1, false),
                ExtensionOp::new("internal_real_path", Self::internal_real_path, 1, false),
                // Async methods
                ExtensionOp::new(
                    "internal_read_text_file_async",
                    Self::internal_read_text_file_async,
                    1,
                    false,
                ),
                ExtensionOp::new(
                    "internal_write_text_file_async",
                    Self::internal_write_text_file_async,
                    2,
                    false,
                ),
                ExtensionOp::new(
                    "internal_read_file_async",
                    Self::internal_read_file_async,
                    1,
                    false,
                ),
                ExtensionOp::new(
                    "internal_write_file_async",
                    Self::internal_write_file_async,
                    2,
                    false,
                ),
                ExtensionOp::new(
                    "internal_copy_file_async",
                    Self::internal_copy_file_async,
                    2,
                    false,
                ),
                ExtensionOp::new(
                    "internal_remove_async",
                    Self::internal_remove_async,
                    1,
                    false,
                ),
                ExtensionOp::new(
                    "internal_create_file_async",
                    Self::internal_create_file_async,
                    1,
                    false,
                ),
                ExtensionOp::new(
                    "internal_mk_dir_async",
                    Self::internal_mk_dir_async,
                    1,
                    false,
                ),
                ExtensionOp::new(
                    "internal_mk_dir_all_async",
                    Self::internal_mk_dir_all_async,
                    1,
                    false,
                ),
                ExtensionOp::new(
                    "internal_exists_async",
                    Self::internal_exists_async,
                    1,
                    false,
                ),
                ExtensionOp::new(
                    "internal_rename_async",
                    Self::internal_rename_async,
                    2,
                    false,
                ),
                ExtensionOp::new(
                    "internal_remove_all_async",
                    Self::internal_remove_all_async,
                    1,
                    false,
                ),
            ],
            storage: Some(Box::new(|storage: &mut OpsStorage| {
                storage.insert(VirtualFsExtResources {
                    files: ResourceTable::<VirtualFileHandle>::new(),
                });
            })),
            files: vec![],
        }
    }

    // Helper methods for virtual filesystem operations

    fn get_node_id_by_path(conn: &Connection, path: &str) -> Result<Option<i64>, VirtualFsError> {
        let path = normalize_path(path);
        let mut stmt = conn.prepare_cached("SELECT id FROM vfs_nodes WHERE path = ?")?;
        let node_id: Option<i64> = stmt.query_row(params![path], |row| row.get(0)).optional()?;
        Ok(node_id)
    }

    fn get_node_info(conn: &Connection, path: &str) -> Result<Option<NodeInfo>, VirtualFsError> {
        let path = normalize_path(path);
        let mut stmt = conn.prepare_cached(
            "SELECT id, node_type, name, size, created_at, modified_at, accessed_at, mode, symlink_target FROM vfs_nodes WHERE path = ?"
        )?;
        let info = stmt
            .query_row(params![path], |row| {
                Ok((
                    row.get::<_, i64>(0)?,            // id
                    row.get::<_, String>(1)?,         // node_type
                    row.get::<_, String>(2)?,         // name
                    row.get::<_, i64>(3)?,            // size
                    row.get::<_, i64>(4)?,            // created_at
                    row.get::<_, i64>(5)?,            // modified_at
                    row.get::<_, i64>(6)?,            // accessed_at
                    row.get::<_, i32>(7)?,            // mode
                    row.get::<_, Option<String>>(8)?, // symlink_target
                ))
            })
            .optional()?;
        Ok(info)
    }

    fn create_parent_directories(conn: &Connection, path: &str) -> Result<(), VirtualFsError> {
        let path = normalize_path(path);
        let parent_path = match Path::new(&path).parent() {
            Some(p) => p.to_str().unwrap_or("/").to_string(),
            None => return Ok(()),
        };

        if parent_path == "/" || Self::get_node_id_by_path(conn, &parent_path)?.is_some() {
            return Ok(());
        }

        Self::create_parent_directories(conn, &parent_path)?;

        let parent_name = Path::new(&parent_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();
        let grandparent_path = Path::new(&parent_path)
            .parent()
            .and_then(|p| p.to_str())
            .map(normalize_path)
            .filter(|p| p != &parent_path);

        let current_time = current_timestamp();
        conn.execute(
            "INSERT INTO vfs_nodes (path, name, parent_path, node_type, created_at, modified_at, accessed_at, mode) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
            params![parent_path, parent_name, grandparent_path, "directory", current_time, current_time, current_time, 755],
        )?;

        Ok(())
    }

    fn read_file_content(conn: &Connection, node_id: i64) -> Result<Vec<u8>, VirtualFsError> {
        let mut stmt = conn.prepare_cached(
            "SELECT content FROM vfs_file_content WHERE node_id = ? ORDER BY chunk_index",
        )?;
        let chunks: Result<Vec<Vec<u8>>, _> = stmt
            .query_map(params![node_id], |row| row.get::<_, Vec<u8>>(0))?
            .collect();

        let mut content = Vec::new();
        for chunk in chunks? {
            content.extend(chunk);
        }
        Ok(content)
    }

    fn write_file_content(
        conn: &Connection,
        node_id: i64,
        content: &[u8],
    ) -> Result<(), VirtualFsError> {
        // Delete existing content
        conn.execute(
            "DELETE FROM vfs_file_content WHERE node_id = ?",
            params![node_id],
        )?;

        // Write content in chunks
        for (chunk_index, chunk) in content.chunks(CHUNK_SIZE).enumerate() {
            conn.execute(
                "INSERT INTO vfs_file_content (node_id, chunk_index, content) VALUES (?, ?, ?)",
                params![node_id, chunk_index as i64, chunk],
            )?;
        }

        // Update file size and modified time
        let current_time = current_timestamp();
        conn.execute(
            "UPDATE vfs_nodes SET size = ?, modified_at = ? WHERE id = ?",
            params![content.len() as i64, current_time, node_id],
        )?;

        Ok(())
    }

    fn check_filesystem_size(
        conn: &Connection,
        additional_size: usize,
    ) -> Result<(), VirtualFsError> {
        let mut stmt =
            conn.prepare_cached("SELECT COALESCE(SUM(LENGTH(content)), 0) FROM vfs_file_content")?;
        let current_size: i64 = stmt.query_row(params![], |row| row.get(0))?;

        if current_size as usize + additional_size > MAX_VFS_SIZE {
            return Err(VirtualFsError::FilesystemSizeExceeded);
        }
        Ok(())
    }

    /// Read a text file and return the content as a string.
    #[cfg_attr(feature = "hotpath", hotpath::measure)]
    pub fn internal_read_text_file<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let path = match extract_string(agent, args.get(0)) {
            Some(p) => normalize_path(&p),
            None => {
                return Err(agent
                    .throw_exception_with_static_message(
                        nova_vm::ecmascript::execution::agent::ExceptionType::TypeError,
                        "Path must be a string",
                        gc.nogc(),
                    )
                    .unbind());
            }
        };

        let host_data = agent.get_host_data();
        let host_data: &HostData<RuntimeMacroTask> = host_data.downcast_ref().unwrap();

        let result = with_virtualfs(host_data, |conn| match Self::get_node_info(conn, &path)? {
            Some((node_id, node_type, _, _, _, _, _, _, _)) => {
                if node_type != "file" {
                    return Err(VirtualFsError::NotAFile(path.clone()));
                }

                let content = Self::read_file_content(conn, node_id)?;
                match String::from_utf8(content) {
                    Ok(text) => Ok(text),
                    Err(_) => Err(VirtualFsError::InvalidPath(
                        "File contains invalid UTF-8".to_string(),
                    )),
                }
            }
            None => Err(VirtualFsError::PathNotFound(path.clone())),
        });

        match result {
            Ok(content) => Ok(Value::from_string(agent, content, gc.nogc()).unbind()),
            Err(e) => Ok(Value::from_string(agent, format!("Error: {e}"), gc.nogc()).unbind()),
        }
    }

    /// Write a text file with the content of the second argument.
    #[cfg_attr(feature = "hotpath", hotpath::measure)]
    pub fn internal_write_text_file<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let path = match extract_string(agent, args.get(0)) {
            Some(p) => normalize_path(&p),
            None => {
                return Err(agent
                    .throw_exception_with_static_message(
                        nova_vm::ecmascript::execution::agent::ExceptionType::TypeError,
                        "Path must be a string",
                        gc.nogc(),
                    )
                    .unbind());
            }
        };

        let content = match extract_string(agent, args.get(1)) {
            Some(c) => c,
            None => {
                return Err(agent
                    .throw_exception_with_static_message(
                        nova_vm::ecmascript::execution::agent::ExceptionType::TypeError,
                        "Content must be a string",
                        gc.nogc(),
                    )
                    .unbind());
            }
        };

        let host_data = agent.get_host_data();
        let host_data: &HostData<RuntimeMacroTask> = host_data.downcast_ref().unwrap();

        let result = with_virtualfs(host_data, |conn| {
            let content_bytes = content.as_bytes();
            Self::check_filesystem_size(conn, content_bytes.len())?;

            Self::create_parent_directories(conn, &path)?;

            let current_time = current_timestamp();
            let file_name = Path::new(&path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();
            let parent_path = Path::new(&path)
                .parent()
                .and_then(|p| p.to_str())
                .map(normalize_path)
                .filter(|p| p != &path);

            // Check if file exists
            match Self::get_node_id_by_path(conn, &path)? {
                Some(node_id) => {
                    // Update existing file
                    Self::write_file_content(conn, node_id, content_bytes)?;
                }
                None => {
                    // Create new file
                    conn.execute(
                        "INSERT INTO vfs_nodes (path, name, parent_path, node_type, size, created_at, modified_at, accessed_at, mode) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
                        params![path, file_name, parent_path, "file", content_bytes.len() as i64, current_time, current_time, current_time, 644],
                    )?;

                    let node_id = conn.last_insert_rowid();
                    Self::write_file_content(conn, node_id, content_bytes)?;
                }
            }

            Ok(())
        });

        match result {
            Ok(()) => Ok(Value::from_string(agent, "Success".to_string(), gc.nogc()).unbind()),
            Err(e) => Ok(Value::from_string(agent, format!("Error: {e}"), gc.nogc()).unbind()),
        }
    }

    /// Create a file and return a Rid.
    pub fn internal_create_file<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let path = match extract_string(agent, args.get(0)) {
            Some(p) => normalize_path(&p),
            None => {
                return Err(agent
                    .throw_exception_with_static_message(
                        nova_vm::ecmascript::execution::agent::ExceptionType::TypeError,
                        "Path must be a string",
                        gc.nogc(),
                    )
                    .unbind());
            }
        };

        let host_data = agent.get_host_data();
        let host_data: &HostData<RuntimeMacroTask> = host_data.downcast_ref().unwrap();

        let result = with_virtualfs(host_data, |conn| {
            if Self::get_node_id_by_path(conn, &path)?.is_some() {
                return Err(VirtualFsError::PathAlreadyExists(path.clone()));
            }

            Self::create_parent_directories(conn, &path)?;

            let current_time = current_timestamp();
            let file_name = Path::new(&path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();
            let parent_path = Path::new(&path)
                .parent()
                .and_then(|p| p.to_str())
                .map(normalize_path)
                .filter(|p| p != &path);

            conn.execute(
                "INSERT INTO vfs_nodes (path, name, parent_path, node_type, size, created_at, modified_at, accessed_at, mode) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
                params![path, file_name, parent_path, "file", 0i64, current_time, current_time, current_time, 644],
            )?;

            let node_id = conn.last_insert_rowid();

            // Create a virtual file handle
            let file_handle = VirtualFileHandle {
                path: path.clone(),
                _node_id: node_id,
            };

            Ok((node_id, file_handle))
        });

        match result {
            Ok((_node_id, file_handle)) => {
                let storage = host_data.storage.borrow();
                let resources: &VirtualFsExtResources = storage.get().unwrap();
                let rid = resources.files.push(file_handle);
                Ok(Value::Integer(SmallInteger::from(rid.index())))
            }
            Err(e) => Ok(Value::from_string(agent, format!("Error: {e}"), gc.nogc()).unbind()),
        }
    }

    /// Check if a file or directory exists.
    #[cfg_attr(feature = "hotpath", hotpath::measure)]
    pub fn internal_exists<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let path = match extract_string(agent, args.get(0)) {
            Some(p) => normalize_path(&p),
            None => {
                return Ok(Value::from_string(agent, "false".to_string(), gc.nogc()).unbind());
            }
        };

        let host_data = agent.get_host_data();
        let host_data: &HostData<RuntimeMacroTask> = host_data.downcast_ref().unwrap();

        let exists = with_virtualfs(host_data, |conn| {
            Ok(Self::get_node_id_by_path(conn, &path)?.is_some())
        })
        .unwrap_or(false);

        Ok(Value::from_string(agent, exists.to_string(), gc.nogc()).unbind())
    }

    /// Create a directory.
    pub fn internal_mk_dir<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let path = match extract_string(agent, args.get(0)) {
            Some(p) => normalize_path(&p),
            None => {
                return Err(agent
                    .throw_exception_with_static_message(
                        nova_vm::ecmascript::execution::agent::ExceptionType::TypeError,
                        "Path must be a string",
                        gc.nogc(),
                    )
                    .unbind());
            }
        };

        let host_data = agent.get_host_data();
        let host_data: &HostData<RuntimeMacroTask> = host_data.downcast_ref().unwrap();

        let result = with_virtualfs(host_data, |conn| {
            if Self::get_node_id_by_path(conn, &path)?.is_some() {
                return Err(VirtualFsError::PathAlreadyExists(path.clone()));
            }

            // Check that parent directory exists
            let parent_path = Path::new(&path)
                .parent()
                .and_then(|p| p.to_str())
                .map(normalize_path);

            if let Some(parent) = &parent_path
                && parent != "/"
                && Self::get_node_id_by_path(conn, parent)?.is_none()
            {
                return Err(VirtualFsError::PathNotFound(parent.clone()));
            }

            let current_time = current_timestamp();
            let dir_name = Path::new(&path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();

            conn.execute(
                "INSERT INTO vfs_nodes (path, name, parent_path, node_type, created_at, modified_at, accessed_at, mode) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
                params![path, dir_name, parent_path, "directory", current_time, current_time, current_time, 755],
            )?;

            Ok(())
        });

        match result {
            Ok(()) => Ok(Value::from_string(agent, "Success".to_string(), gc.nogc()).unbind()),
            Err(e) => Ok(Value::from_string(agent, format!("Error: {e}"), gc.nogc()).unbind()),
        }
    }

    /// Create a directory recursively (mkdir -p equivalent).
    #[cfg_attr(feature = "hotpath", hotpath::measure)]
    pub fn internal_mk_dir_all<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let path = match extract_string(agent, args.get(0)) {
            Some(p) => normalize_path(&p),
            None => {
                return Err(agent
                    .throw_exception_with_static_message(
                        nova_vm::ecmascript::execution::agent::ExceptionType::TypeError,
                        "Path must be a string",
                        gc.nogc(),
                    )
                    .unbind());
            }
        };

        let host_data = agent.get_host_data();
        let host_data: &HostData<RuntimeMacroTask> = host_data.downcast_ref().unwrap();

        let result = with_virtualfs(host_data, |conn| {
            Self::create_parent_directories(conn, &path)?;

            // Create the target directory if it doesn't exist
            if Self::get_node_id_by_path(conn, &path)?.is_none() {
                let current_time = current_timestamp();
                let dir_name = Path::new(&path)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string();
                let parent_path = Path::new(&path)
                    .parent()
                    .and_then(|p| p.to_str())
                    .map(normalize_path)
                    .filter(|p| p != &path);

                conn.execute(
                    "INSERT INTO vfs_nodes (path, name, parent_path, node_type, created_at, modified_at, accessed_at, mode) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
                    params![path, dir_name, parent_path, "directory", current_time, current_time, current_time, 755],
                )?;
            }

            Ok(())
        });

        match result {
            Ok(()) => Ok(Value::from_string(agent, "Success".to_string(), gc.nogc()).unbind()),
            Err(e) => Ok(Value::from_string(agent, format!("Error: {e}"), gc.nogc()).unbind()),
        }
    }

    /// Remove a file or empty directory.
    #[cfg_attr(feature = "hotpath", hotpath::measure)]
    pub fn internal_remove<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let path = match extract_string(agent, args.get(0)) {
            Some(p) => normalize_path(&p),
            None => {
                return Err(agent
                    .throw_exception_with_static_message(
                        nova_vm::ecmascript::execution::agent::ExceptionType::TypeError,
                        "Path must be a string",
                        gc.nogc(),
                    )
                    .unbind());
            }
        };

        let host_data = agent.get_host_data();
        let host_data: &HostData<RuntimeMacroTask> = host_data.downcast_ref().unwrap();

        let result = with_virtualfs(host_data, |conn| {
            match Self::get_node_info(conn, &path)? {
                Some((node_id, node_type, _, _, _, _, _, _, _)) => {
                    if node_type == "directory" {
                        // Check if directory is empty
                        let mut stmt = conn.prepare_cached(
                            "SELECT COUNT(*) FROM vfs_nodes WHERE parent_path = ?",
                        )?;
                        let child_count: i64 = stmt.query_row(params![path], |row| row.get(0))?;

                        if child_count > 0 {
                            return Err(VirtualFsError::DirectoryNotEmpty(path));
                        }
                    }

                    // Delete the node (CASCADE will handle file content)
                    conn.execute("DELETE FROM vfs_nodes WHERE id = ?", params![node_id])?;
                    Ok(())
                }
                None => Err(VirtualFsError::PathNotFound(path)),
            }
        });

        match result {
            Ok(()) => Ok(Value::from_string(agent, "Success".to_string(), gc.nogc()).unbind()),
            Err(e) => Ok(Value::from_string(agent, format!("Error: {e}"), gc.nogc()).unbind()),
        }
    }

    /// Copy a file from source to destination path.
    #[cfg_attr(feature = "hotpath", hotpath::measure)]
    pub fn internal_copy_file<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let source = match extract_string(agent, args.get(0)) {
            Some(p) => normalize_path(&p),
            None => {
                return Err(agent
                    .throw_exception_with_static_message(
                        nova_vm::ecmascript::execution::agent::ExceptionType::TypeError,
                        "Source path must be a string",
                        gc.nogc(),
                    )
                    .unbind());
            }
        };

        let dest = match extract_string(agent, args.get(1)) {
            Some(p) => normalize_path(&p),
            None => {
                return Err(agent
                    .throw_exception_with_static_message(
                        nova_vm::ecmascript::execution::agent::ExceptionType::TypeError,
                        "Destination path must be a string",
                        gc.nogc(),
                    )
                    .unbind());
            }
        };

        let host_data = agent.get_host_data();
        let host_data: &HostData<RuntimeMacroTask> = host_data.downcast_ref().unwrap();

        let result = with_virtualfs(host_data, |conn| {
            // Check if source exists and is a file
            match Self::get_node_info(conn, &source)? {
                Some((src_id, node_type, _, _, _, _, _, _, _)) => {
                    if node_type != "file" {
                        return Err(VirtualFsError::NotAFile(source.clone()));
                    }

                    // Check if destination already exists
                    if Self::get_node_id_by_path(conn, &dest)?.is_some() {
                        return Err(VirtualFsError::PathAlreadyExists(dest.clone()));
                    }

                    // Read source content
                    let content = Self::read_file_content(conn, src_id)?;

                    // Check filesystem size limit
                    Self::check_filesystem_size(conn, content.len())?;

                    // Create parent directories for destination
                    Self::create_parent_directories(conn, &dest)?;

                    // Create destination file
                    let current_time = current_timestamp();
                    let dest_name = Path::new(&dest)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("")
                        .to_string();
                    let dest_parent = Path::new(&dest)
                        .parent()
                        .and_then(|p| p.to_str())
                        .map(normalize_path)
                        .filter(|p| p != &dest);

                    conn.execute(
                        "INSERT INTO vfs_nodes (path, name, parent_path, node_type, size, created_at, modified_at, accessed_at, mode) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
                        params![dest, dest_name, dest_parent, "file", content.len() as i64, current_time, current_time, current_time, 644],
                    )?;

                    let dest_id = conn.last_insert_rowid();
                    Self::write_file_content(conn, dest_id, &content)?;

                    Ok(())
                }
                None => Err(VirtualFsError::PathNotFound(source)),
            }
        });

        match result {
            Ok(()) => Ok(Value::from_string(agent, "Success".to_string(), gc.nogc()).unbind()),
            Err(e) => Ok(Value::from_string(agent, format!("Error: {e}"), gc.nogc()).unbind()),
        }
    }

    /// Open a file and return a resource ID (RID).
    #[cfg_attr(feature = "hotpath", hotpath::measure)]
    pub fn internal_open_file<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let path = match extract_string(agent, args.get(0)) {
            Some(p) => normalize_path(&p),
            None => {
                return Err(agent
                    .throw_exception_with_static_message(
                        nova_vm::ecmascript::execution::agent::ExceptionType::TypeError,
                        "Path must be a string",
                        gc.nogc(),
                    )
                    .unbind());
            }
        };

        let host_data = agent.get_host_data();
        let host_data: &HostData<RuntimeMacroTask> = host_data.downcast_ref().unwrap();

        let result = with_virtualfs(host_data, |conn| match Self::get_node_info(conn, &path)? {
            Some((node_id, node_type, _, _, _, _, _, _, _)) => {
                if node_type != "file" {
                    return Err(VirtualFsError::NotAFile(path.clone()));
                }

                let file_handle = VirtualFileHandle {
                    path: path.clone(),
                    _node_id: node_id,
                };

                Ok(file_handle)
            }
            None => Err(VirtualFsError::PathNotFound(path)),
        });

        match result {
            Ok(file_handle) => {
                let storage = host_data.storage.borrow();
                let resources: &VirtualFsExtResources = storage.get().unwrap();
                let rid = resources.files.push(file_handle);
                Ok(Value::Integer(SmallInteger::from(rid.index())))
            }
            Err(e) => Ok(Value::from_string(agent, format!("Error: {e}"), gc.nogc()).unbind()),
        }
    }

    /// Read binary file content and return as a string representation of bytes.
    #[cfg_attr(feature = "hotpath", hotpath::measure)]
    pub fn internal_read_file<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let path = match extract_string(agent, args.get(0)) {
            Some(p) => normalize_path(&p),
            None => {
                return Err(agent
                    .throw_exception_with_static_message(
                        nova_vm::ecmascript::execution::agent::ExceptionType::TypeError,
                        "Path must be a string",
                        gc.nogc(),
                    )
                    .unbind());
            }
        };

        let host_data = agent.get_host_data();
        let host_data: &HostData<RuntimeMacroTask> = host_data.downcast_ref().unwrap();

        let result = with_virtualfs(host_data, |conn| match Self::get_node_info(conn, &path)? {
            Some((node_id, node_type, _, _, _, _, _, _, _)) => {
                if node_type != "file" {
                    return Err(VirtualFsError::NotAFile(path.clone()));
                }

                let content = Self::read_file_content(conn, node_id)?;
                Ok(content)
            }
            None => Err(VirtualFsError::PathNotFound(path)),
        });

        match result {
            Ok(content) => {
                let content_str = base64::STANDARD.encode_to_string(&content);
                Ok(Value::from_string(agent, content_str, gc.nogc()).unbind())
            }
            Err(e) => Ok(Value::from_string(agent, format!("Error: {e}"), gc.nogc()).unbind()),
        }
    }

    /// Write binary data to a file (expects base64 encoded data).
    #[cfg_attr(feature = "hotpath", hotpath::measure)]
    pub fn internal_write_file<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let path = match extract_string(agent, args.get(0)) {
            Some(p) => normalize_path(&p),
            None => {
                return Err(agent
                    .throw_exception_with_static_message(
                        nova_vm::ecmascript::execution::agent::ExceptionType::TypeError,
                        "Path must be a string",
                        gc.nogc(),
                    )
                    .unbind());
            }
        };

        let data = match extract_string(agent, args.get(1)) {
            Some(d) => d,
            None => {
                return Err(agent
                    .throw_exception_with_static_message(
                        nova_vm::ecmascript::execution::agent::ExceptionType::TypeError,
                        "Data must be a string (base64 encoded)",
                        gc.nogc(),
                    )
                    .unbind());
            }
        };

        let host_data = agent.get_host_data();
        let host_data: &HostData<RuntimeMacroTask> = host_data.downcast_ref().unwrap();

        let result = with_virtualfs(host_data, |conn| {
            // Decode base64 data
            let content_bytes = match base64::STANDARD.decode_to_vec(&data) {
                Ok(bytes) => bytes,
                Err(_) => {
                    return Err(VirtualFsError::InvalidPath(
                        "Invalid base64 data".to_string(),
                    ));
                }
            };

            Self::check_filesystem_size(conn, content_bytes.len())?;
            Self::create_parent_directories(conn, &path)?;

            let current_time = current_timestamp();
            let file_name = Path::new(&path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();
            let parent_path = Path::new(&path)
                .parent()
                .and_then(|p| p.to_str())
                .map(normalize_path)
                .filter(|p| p != &path);

            // Check if file exists
            match Self::get_node_id_by_path(conn, &path)? {
                Some(node_id) => {
                    // Update existing file
                    Self::write_file_content(conn, node_id, &content_bytes)?;
                }
                None => {
                    // Create new file
                    conn.execute(
                        "INSERT INTO vfs_nodes (path, name, parent_path, node_type, size, created_at, modified_at, accessed_at, mode) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
                        params![path, file_name, parent_path, "file", content_bytes.len() as i64, current_time, current_time, current_time, 644],
                    )?;

                    let node_id = conn.last_insert_rowid();
                    Self::write_file_content(conn, node_id, &content_bytes)?;
                }
            }

            Ok(())
        });

        match result {
            Ok(()) => Ok(Value::from_string(agent, "Success".to_string(), gc.nogc()).unbind()),
            Err(e) => Ok(Value::from_string(agent, format!("Error: {e}"), gc.nogc()).unbind()),
        }
    }

    /// Get file/directory statistics.
    #[cfg_attr(feature = "hotpath", hotpath::measure)]
    pub fn internal_stat<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let path = match extract_string(agent, args.get(0)) {
            Some(p) => normalize_path(&p),
            None => {
                return Err(agent
                    .throw_exception_with_static_message(
                        nova_vm::ecmascript::execution::agent::ExceptionType::TypeError,
                        "Path must be a string",
                        gc.nogc(),
                    )
                    .unbind());
            }
        };

        let host_data = agent.get_host_data();
        let host_data: &HostData<RuntimeMacroTask> = host_data.downcast_ref().unwrap();

        let result = with_virtualfs(host_data, |conn| match Self::get_node_info(conn, &path)? {
            Some((
                _node_id,
                node_type,
                name,
                size,
                created_at,
                modified_at,
                accessed_at,
                mode,
                _symlink_target,
            )) => {
                let stat_info = format!(
                    "{{\"type\":\"{}\",\"name\":\"{}\",\"size\":{},\"created_at\":{},\"modified_at\":{},\"accessed_at\":{},\"mode\":{}}}",
                    node_type, name, size, created_at, modified_at, accessed_at, mode
                );
                Ok(stat_info)
            }
            None => Err(VirtualFsError::PathNotFound(path)),
        });

        match result {
            Ok(stat_info) => Ok(Value::from_string(agent, stat_info, gc.nogc()).unbind()),
            Err(e) => Ok(Value::from_string(agent, format!("Error: {e}"), gc.nogc()).unbind()),
        }
    }

    /// Get link statistics (like stat but doesn't follow symlinks).
    #[cfg_attr(feature = "hotpath", hotpath::measure)]
    pub fn internal_lstat<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        // For now, lstat behaves the same as stat since we handle symlinks in the database
        Self::internal_stat(agent, _this, args, gc)
    }

    /// List directory contents.
    #[cfg_attr(feature = "hotpath", hotpath::measure)]
    pub fn internal_read_dir<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let path = match extract_string(agent, args.get(0)) {
            Some(p) => normalize_path(&p),
            None => {
                return Err(agent
                    .throw_exception_with_static_message(
                        nova_vm::ecmascript::execution::agent::ExceptionType::TypeError,
                        "Path must be a string",
                        gc.nogc(),
                    )
                    .unbind());
            }
        };

        let host_data = agent.get_host_data();
        let host_data: &HostData<RuntimeMacroTask> = host_data.downcast_ref().unwrap();

        let result = with_virtualfs(host_data, |conn| match Self::get_node_info(conn, &path)? {
            Some((_node_id, node_type, _, _, _, _, _, _, _)) => {
                if node_type != "directory" {
                    return Err(VirtualFsError::NotADirectory(path.clone()));
                }

                let mut stmt = conn.prepare_cached(
                    "SELECT name, node_type FROM vfs_nodes WHERE parent_path = ? ORDER BY name",
                )?;
                let entries: Result<Vec<String>, _> = stmt
                    .query_map(params![path], |row| {
                        let name: String = row.get(0)?;
                        let node_type: String = row.get(1)?;
                        Ok(format!(
                            "{{\"name\":\"{}\",\"type\":\"{}\"}}",
                            name, node_type
                        ))
                    })?
                    .collect();

                let entries = entries?;
                let entries_json = format!("[{}]", entries.join(","));
                Ok(entries_json)
            }
            None => Err(VirtualFsError::PathNotFound(path)),
        });

        match result {
            Ok(entries) => Ok(Value::from_string(agent, entries, gc.nogc()).unbind()),
            Err(e) => Ok(Value::from_string(agent, format!("Error: {e}"), gc.nogc()).unbind()),
        }
    }

    /// Remove a directory and all its contents recursively.
    #[cfg_attr(feature = "hotpath", hotpath::measure)]
    pub fn internal_remove_all<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let path = match extract_string(agent, args.get(0)) {
            Some(p) => normalize_path(&p),
            None => {
                return Err(agent
                    .throw_exception_with_static_message(
                        nova_vm::ecmascript::execution::agent::ExceptionType::TypeError,
                        "Path must be a string",
                        gc.nogc(),
                    )
                    .unbind());
            }
        };

        let host_data = agent.get_host_data();
        let host_data: &HostData<RuntimeMacroTask> = host_data.downcast_ref().unwrap();

        let result = with_virtualfs(host_data, |conn| {
            fn remove_recursive(conn: &Connection, path: &str) -> Result<(), VirtualFsError> {
                let mut stmt = conn.prepare_cached(
                    "SELECT path, node_type FROM vfs_nodes WHERE parent_path = ?",
                )?;
                let children: Result<Vec<(String, String)>, _> = stmt
                    .query_map(params![path], |row| {
                        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
                    })?
                    .collect();

                let children = children?;

                for (child_path, child_type) in children {
                    if child_type == "directory" {
                        remove_recursive(conn, &child_path)?;
                    }
                    // Remove child node (CASCADE will handle file content)
                    conn.execute("DELETE FROM vfs_nodes WHERE path = ?", params![child_path])?;
                }

                Ok(())
            }

            match Self::get_node_info(conn, &path)? {
                Some((node_id, _node_type, _, _, _, _, _, _, _)) => {
                    // Remove all children recursively
                    remove_recursive(conn, &path)?;

                    // Remove the target node itself
                    conn.execute("DELETE FROM vfs_nodes WHERE id = ?", params![node_id])?;
                    Ok(())
                }
                None => Err(VirtualFsError::PathNotFound(path)),
            }
        });

        match result {
            Ok(()) => Ok(Value::from_string(agent, "Success".to_string(), gc.nogc()).unbind()),
            Err(e) => Ok(Value::from_string(agent, format!("Error: {e}"), gc.nogc()).unbind()),
        }
    }

    /// Rename/move a file or directory.
    #[cfg_attr(feature = "hotpath", hotpath::measure)]
    pub fn internal_rename<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let old_path = match extract_string(agent, args.get(0)) {
            Some(p) => normalize_path(&p),
            None => {
                return Err(agent
                    .throw_exception_with_static_message(
                        nova_vm::ecmascript::execution::agent::ExceptionType::TypeError,
                        "Old path must be a string",
                        gc.nogc(),
                    )
                    .unbind());
            }
        };

        let new_path = match extract_string(agent, args.get(1)) {
            Some(p) => normalize_path(&p),
            None => {
                return Err(agent
                    .throw_exception_with_static_message(
                        nova_vm::ecmascript::execution::agent::ExceptionType::TypeError,
                        "New path must be a string",
                        gc.nogc(),
                    )
                    .unbind());
            }
        };

        let host_data = agent.get_host_data();
        let host_data: &HostData<RuntimeMacroTask> = host_data.downcast_ref().unwrap();

        let result = with_virtualfs(host_data, |conn| {
            // Check if old path exists
            match Self::get_node_info(conn, &old_path)? {
                Some((node_id, node_type, _, _, _, _, _, _, _)) => {
                    // Check if new path already exists
                    if Self::get_node_id_by_path(conn, &new_path)?.is_some() {
                        return Err(VirtualFsError::PathAlreadyExists(new_path.clone()));
                    }

                    // Create parent directories for new path
                    Self::create_parent_directories(conn, &new_path)?;

                    let new_name = Path::new(&new_path)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("")
                        .to_string();
                    let new_parent = Path::new(&new_path)
                        .parent()
                        .and_then(|p| p.to_str())
                        .map(normalize_path)
                        .filter(|p| p != &new_path);

                    let current_time = current_timestamp();

                    // Update the node with new path, name, and parent
                    conn.execute(
                        "UPDATE vfs_nodes SET path = ?, name = ?, parent_path = ?, modified_at = ? WHERE id = ?",
                        params![new_path, new_name, new_parent, current_time, node_id],
                    )?;

                    // If it's a directory, update all children's parent_path references
                    if node_type == "directory" {
                        fn update_children_paths(
                            conn: &Connection,
                            old_parent: &str,
                            new_parent: &str,
                        ) -> Result<(), VirtualFsError> {
                            let mut stmt = conn.prepare_cached(
                                "SELECT id, path FROM vfs_nodes WHERE parent_path = ?",
                            )?;
                            let children: Result<Vec<(i64, String)>, _> = stmt
                                .query_map(params![old_parent], |row| {
                                    Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
                                })?
                                .collect();

                            let children = children?;
                            for (child_id, child_path) in children {
                                let child_name = Path::new(&child_path)
                                    .file_name()
                                    .and_then(|n| n.to_str())
                                    .unwrap_or("");
                                let new_child_path = format!("{}/{}", new_parent, child_name);

                                conn.execute(
                                    "UPDATE vfs_nodes SET path = ?, parent_path = ? WHERE id = ?",
                                    params![new_child_path, new_parent, child_id],
                                )?;

                                // Check if this child is also a directory
                                let child_type: String = conn.query_row(
                                    "SELECT node_type FROM vfs_nodes WHERE id = ?",
                                    params![child_id],
                                    |row| row.get(0),
                                )?;

                                if child_type == "directory" {
                                    update_children_paths(conn, &child_path, &new_child_path)?;
                                }
                            }
                            Ok(())
                        }

                        update_children_paths(conn, &old_path, &new_path)?;
                    }

                    Ok(())
                }
                None => Err(VirtualFsError::PathNotFound(old_path)),
            }
        });

        match result {
            Ok(()) => Ok(Value::from_string(agent, "Success".to_string(), gc.nogc()).unbind()),
            Err(e) => Ok(Value::from_string(agent, format!("Error: {e}"), gc.nogc()).unbind()),
        }
    }

    /// Truncate a file to a specific size.
    #[cfg_attr(feature = "hotpath", hotpath::measure)]
    pub fn internal_truncate<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let path = match extract_string(agent, args.get(0)) {
            Some(p) => normalize_path(&p),
            None => {
                return Err(agent
                    .throw_exception_with_static_message(
                        nova_vm::ecmascript::execution::agent::ExceptionType::TypeError,
                        "Path must be a string",
                        gc.nogc(),
                    )
                    .unbind());
            }
        };

        // Extract size parameter
        let size_value = args.get(1);
        let size = match size_value.to_number(agent, gc.reborrow()) {
            Ok(num) => num.unbind().into_f64(agent) as i64,
            Err(_) => {
                return Err(agent
                    .throw_exception_with_static_message(
                        nova_vm::ecmascript::execution::agent::ExceptionType::TypeError,
                        "Size must be a number",
                        gc.nogc(),
                    )
                    .unbind());
            }
        };

        if size < 0 {
            return Err(agent
                .throw_exception_with_static_message(
                    nova_vm::ecmascript::execution::agent::ExceptionType::RangeError,
                    "Size must be non-negative",
                    gc.nogc(),
                )
                .unbind());
        }

        let host_data = agent.get_host_data();
        let host_data: &HostData<RuntimeMacroTask> = host_data.downcast_ref().unwrap();

        let result = with_virtualfs(host_data, |conn| {
            match Self::get_node_info(conn, &path)? {
                Some((node_id, node_type, _, current_size, _, _, _, _, _)) => {
                    if node_type != "file" {
                        return Err(VirtualFsError::NotAFile(path.clone()));
                    }

                    let new_size = size as usize;

                    if new_size as i64 == current_size {
                        return Ok(());
                    }

                    if new_size == 0 {
                        // Truncate to zero - delete all content
                        conn.execute(
                            "DELETE FROM vfs_file_content WHERE node_id = ?",
                            params![node_id],
                        )?;
                    } else {
                        let mut content = Self::read_file_content(conn, node_id)?;

                        if content.len() > new_size {
                            // Truncate content
                            content.truncate(new_size);
                        } else if content.len() < new_size {
                            // Extend with zeros
                            content.resize(new_size, 0);
                        }

                        Self::write_file_content(conn, node_id, &content)?;
                    }

                    // Update file size and modified time
                    let current_time = current_timestamp();
                    conn.execute(
                        "UPDATE vfs_nodes SET size = ?, modified_at = ? WHERE id = ?",
                        params![size, current_time, node_id],
                    )?;

                    Ok(())
                }
                None => Err(VirtualFsError::PathNotFound(path)),
            }
        });

        match result {
            Ok(()) => Ok(Value::from_string(agent, "Success".to_string(), gc.nogc()).unbind()),
            Err(e) => Ok(Value::from_string(agent, format!("Error: {e}"), gc.nogc()).unbind()),
        }
    }

    /// Change file/directory permissions.
    #[cfg_attr(feature = "hotpath", hotpath::measure)]
    pub fn internal_chmod<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let path = match extract_string(agent, args.get(0)) {
            Some(p) => normalize_path(&p),
            None => {
                return Err(agent
                    .throw_exception_with_static_message(
                        nova_vm::ecmascript::execution::agent::ExceptionType::TypeError,
                        "Path must be a string",
                        gc.nogc(),
                    )
                    .unbind());
            }
        };

        let mode_value = args.get(1);
        let mode = match mode_value.to_number(agent, gc.reborrow()) {
            Ok(num) => num.unbind().into_f64(agent) as i32,
            Err(_) => {
                return Err(agent
                    .throw_exception_with_static_message(
                        nova_vm::ecmascript::execution::agent::ExceptionType::TypeError,
                        "Mode must be a number",
                        gc.nogc(),
                    )
                    .unbind());
            }
        };

        let host_data = agent.get_host_data();
        let host_data: &HostData<RuntimeMacroTask> = host_data.downcast_ref().unwrap();

        let result = with_virtualfs(host_data, |conn| match Self::get_node_info(conn, &path)? {
            Some((node_id, _, _, _, _, _, _, _, _)) => {
                let current_time = current_timestamp();
                conn.execute(
                    "UPDATE vfs_nodes SET mode = ?, modified_at = ? WHERE id = ?",
                    params![mode, current_time, node_id],
                )?;
                Ok(())
            }
            None => Err(VirtualFsError::PathNotFound(path)),
        });

        match result {
            Ok(()) => Ok(Value::from_string(agent, "Success".to_string(), gc.nogc()).unbind()),
            Err(e) => Ok(Value::from_string(agent, format!("Error: {e}"), gc.nogc()).unbind()),
        }
    }

    /// Create a symbolic link.
    #[cfg_attr(feature = "hotpath", hotpath::measure)]
    pub fn internal_symlink<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let target = match extract_string(agent, args.get(0)) {
            Some(t) => t,
            None => {
                return Err(agent
                    .throw_exception_with_static_message(
                        nova_vm::ecmascript::execution::agent::ExceptionType::TypeError,
                        "Target must be a string",
                        gc.nogc(),
                    )
                    .unbind());
            }
        };

        let link_path = match extract_string(agent, args.get(1)) {
            Some(p) => normalize_path(&p),
            None => {
                return Err(agent
                    .throw_exception_with_static_message(
                        nova_vm::ecmascript::execution::agent::ExceptionType::TypeError,
                        "Link path must be a string",
                        gc.nogc(),
                    )
                    .unbind());
            }
        };

        let host_data = agent.get_host_data();
        let host_data: &HostData<RuntimeMacroTask> = host_data.downcast_ref().unwrap();

        let result = with_virtualfs(host_data, |conn| {
            if Self::get_node_id_by_path(conn, &link_path)?.is_some() {
                return Err(VirtualFsError::PathAlreadyExists(link_path.clone()));
            }

            Self::create_parent_directories(conn, &link_path)?;

            let current_time = current_timestamp();
            let link_name = Path::new(&link_path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();
            let parent_path = Path::new(&link_path)
                .parent()
                .and_then(|p| p.to_str())
                .map(normalize_path)
                .filter(|p| p != &link_path);

            conn.execute(
                "INSERT INTO vfs_nodes (path, name, parent_path, node_type, size, created_at, modified_at, accessed_at, mode, symlink_target) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                params![link_path, link_name, parent_path, "symlink", 0i64, current_time, current_time, current_time, 777, Some(target)],
            )?;

            Ok(())
        });

        match result {
            Ok(()) => Ok(Value::from_string(agent, "Success".to_string(), gc.nogc()).unbind()),
            Err(e) => Ok(Value::from_string(agent, format!("Error: {e}"), gc.nogc()).unbind()),
        }
    }

    /// Read the target of a symbolic link.
    #[cfg_attr(feature = "hotpath", hotpath::measure)]
    pub fn internal_read_link<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let path = match extract_string(agent, args.get(0)) {
            Some(p) => normalize_path(&p),
            None => {
                return Err(agent
                    .throw_exception_with_static_message(
                        nova_vm::ecmascript::execution::agent::ExceptionType::TypeError,
                        "Path must be a string",
                        gc.nogc(),
                    )
                    .unbind());
            }
        };

        let host_data = agent.get_host_data();
        let host_data: &HostData<RuntimeMacroTask> = host_data.downcast_ref().unwrap();

        let result = with_virtualfs(host_data, |conn| match Self::get_node_info(conn, &path)? {
            Some((_node_id, node_type, _, _, _, _, _, _, symlink_target)) => {
                if node_type != "symlink" {
                    return Err(VirtualFsError::InvalidPath(
                        "Path is not a symbolic link".to_string(),
                    ));
                }

                match symlink_target {
                    Some(target) => Ok(target),
                    None => Err(VirtualFsError::InvalidPath(
                        "Symbolic link has no target".to_string(),
                    )),
                }
            }
            None => Err(VirtualFsError::PathNotFound(path)),
        });

        match result {
            Ok(target) => Ok(Value::from_string(agent, target, gc.nogc()).unbind()),
            Err(e) => Ok(Value::from_string(agent, format!("Error: {e}"), gc.nogc()).unbind()),
        }
    }

    /// Resolve path to canonical absolute path, following symlinks.
    #[cfg_attr(feature = "hotpath", hotpath::measure)]
    pub fn internal_real_path<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let path = match extract_string(agent, args.get(0)) {
            Some(p) => normalize_path(&p),
            None => {
                return Err(agent
                    .throw_exception_with_static_message(
                        nova_vm::ecmascript::execution::agent::ExceptionType::TypeError,
                        "Path must be a string",
                        gc.nogc(),
                    )
                    .unbind());
            }
        };

        let host_data = agent.get_host_data();
        let host_data: &HostData<RuntimeMacroTask> = host_data.downcast_ref().unwrap();

        let result = with_virtualfs(host_data, |conn| {
            fn resolve_path(
                conn: &Connection,
                current_path: &str,
                visited: &mut std::collections::HashSet<String>,
            ) -> Result<String, VirtualFsError> {
                let normalized = normalize_path(current_path);

                // Prevent infinite loops
                if visited.contains(&normalized) {
                    return Err(VirtualFsError::InvalidPath(
                        "Circular symlink detected".to_string(),
                    ));
                }
                visited.insert(normalized.clone());

                match VirtualFsExt::get_node_info(conn, &normalized)? {
                    Some((_node_id, node_type, _, _, _, _, _, _, symlink_target)) => {
                        if node_type == "symlink" {
                            match symlink_target {
                                Some(target) => {
                                    let target_path = if target.starts_with('/') {
                                        target
                                    } else {
                                        // Relative path - resolve relative to link's parent
                                        let link_parent = Path::new(&normalized)
                                            .parent()
                                            .and_then(|p| p.to_str())
                                            .unwrap_or("/");
                                        let resolved = Path::new(link_parent).join(&target);
                                        resolved.to_str().unwrap_or("/").to_string()
                                    };
                                    resolve_path(conn, &target_path, visited)
                                }
                                None => {
                                    Err(VirtualFsError::InvalidPath("Broken symlink".to_string()))
                                }
                            }
                        } else {
                            // Not a symlink, return the normalized path
                            Ok(normalized)
                        }
                    }
                    None => Err(VirtualFsError::PathNotFound(normalized)),
                }
            }

            let mut visited = std::collections::HashSet::new();
            resolve_path(conn, &path, &mut visited)
        });

        match result {
            Ok(real_path) => Ok(Value::from_string(agent, real_path, gc.nogc()).unbind()),
            Err(e) => Ok(Value::from_string(agent, format!("Error: {e}"), gc.nogc()).unbind()),
        }
    }

    pub fn internal_read_text_file_async<'gc>(
        agent: &mut Agent,
        _this: Value,
        _args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        Ok(Value::from_string(
            agent,
            "Async read text file not yet implemented in VFS".to_string(),
            gc.nogc(),
        )
        .unbind())
    }

    pub fn internal_write_text_file_async<'gc>(
        agent: &mut Agent,
        _this: Value,
        _args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        Ok(Value::from_string(
            agent,
            "Async write text file not yet implemented in VFS".to_string(),
            gc.nogc(),
        )
        .unbind())
    }

    pub fn internal_read_file_async<'gc>(
        agent: &mut Agent,
        _this: Value,
        _args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        Ok(Value::from_string(
            agent,
            "Async read file not yet implemented in VFS".to_string(),
            gc.nogc(),
        )
        .unbind())
    }

    pub fn internal_write_file_async<'gc>(
        agent: &mut Agent,
        _this: Value,
        _args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        Ok(Value::from_string(
            agent,
            "Async write file not yet implemented in VFS".to_string(),
            gc.nogc(),
        )
        .unbind())
    }

    pub fn internal_copy_file_async<'gc>(
        agent: &mut Agent,
        _this: Value,
        _args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        Ok(Value::from_string(
            agent,
            "Async copy file not yet implemented in VFS".to_string(),
            gc.nogc(),
        )
        .unbind())
    }

    pub fn internal_remove_async<'gc>(
        agent: &mut Agent,
        _this: Value,
        _args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        Ok(Value::from_string(
            agent,
            "Async remove not yet implemented in VFS".to_string(),
            gc.nogc(),
        )
        .unbind())
    }

    pub fn internal_create_file_async<'gc>(
        agent: &mut Agent,
        _this: Value,
        _args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        Ok(Value::from_string(
            agent,
            "Async create file not yet implemented in VFS".to_string(),
            gc.nogc(),
        )
        .unbind())
    }

    pub fn internal_mk_dir_async<'gc>(
        agent: &mut Agent,
        _this: Value,
        _args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        Ok(Value::from_string(
            agent,
            "Async mk dir not yet implemented in VFS".to_string(),
            gc.nogc(),
        )
        .unbind())
    }

    pub fn internal_mk_dir_all_async<'gc>(
        agent: &mut Agent,
        _this: Value,
        _args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        Ok(Value::from_string(
            agent,
            "Async mk dir all not yet implemented in VFS".to_string(),
            gc.nogc(),
        )
        .unbind())
    }

    pub fn internal_exists_async<'gc>(
        agent: &mut Agent,
        _this: Value,
        _args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        Ok(Value::from_string(
            agent,
            "Async exists not yet implemented in VFS".to_string(),
            gc.nogc(),
        )
        .unbind())
    }

    pub fn internal_rename_async<'gc>(
        agent: &mut Agent,
        _this: Value,
        _args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        Ok(Value::from_string(
            agent,
            "Async rename not yet implemented in VFS".to_string(),
            gc.nogc(),
        )
        .unbind())
    }

    pub fn internal_remove_all_async<'gc>(
        agent: &mut Agent,
        _this: Value,
        _args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        Ok(Value::from_string(
            agent,
            "Async remove all not yet implemented in VFS".to_string(),
            gc.nogc(),
        )
        .unbind())
    }
}
