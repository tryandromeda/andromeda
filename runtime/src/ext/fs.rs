// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::{
    borrow::BorrowMut,
    fs::{File, Metadata},
    path::Path,
    time::SystemTime,
};

use nova_vm::{
    SmallInteger,
    ecmascript::{
        builtins::{
            ArgumentsList,
            promise_objects::promise_abstract_operations::promise_capability_records::PromiseCapability,
        },
        execution::{Agent, JsResult},
        types::{IntoValue, Value},
    },
    engine::{
        Global,
        context::{Bindable, GcScope},
    },
};

use andromeda_core::{
    AndromedaError, ErrorReporter, Extension, ExtensionOp, HostData, MacroTask, OpsStorage,
    ResourceTable,
};

use crate::RuntimeMacroTask;

struct FsExtResources {
    files: ResourceTable<File>,
}

#[derive(Default)]
pub struct FsExt;

/// Helper function to resolve paths relative to current working directory
/// This ensures behavior consistent with Deno - all paths are resolved from
/// where the user invokes the command, not from the binary location
fn resolve_path(path: &str) -> std::io::Result<std::path::PathBuf> {
    let path_buf = std::path::Path::new(path);

    // If it's an absolute path, use it as-is
    if path_buf.is_absolute() {
        Ok(path_buf.to_path_buf())
    } else {
        // Resolve relative paths against current working directory
        let current_dir = std::env::current_dir()?;
        Ok(current_dir.join(path))
    }
}

#[cfg_attr(feature = "hotpath", hotpath::measure_all)]
impl FsExt {
    pub fn new_extension() -> Extension {
        Extension {
            name: "fs",
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
                storage.insert(FsExtResources {
                    files: ResourceTable::<File>::new(),
                });
            })),
            files: vec![],
        }
    }
    /// Read a text file and return the content as a string.
    pub fn internal_read_text_file<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let binding = args.get(0).to_string(agent, gc.reborrow()).unbind()?;
        let path = binding.as_str(agent).expect("String is not valid UTF-8");

        let resolved_path = match resolve_path(path) {
            Ok(p) => p,
            Err(e) => {
                let error = AndromedaError::fs_error(e, "read_text_file", path);
                let error_msg = ErrorReporter::format_error(&error);
                return Ok(
                    Value::from_string(agent, format!("Error: {error_msg}"), gc.nogc()).unbind(),
                );
            }
        };

        match std::fs::read_to_string(&resolved_path) {
            Ok(content) => Ok(Value::from_string(agent, content, gc.nogc()).unbind()),
            Err(e) => {
                let error = AndromedaError::fs_error(e, "read_text_file", path);
                let error_msg = ErrorReporter::format_error(&error);
                Ok(Value::from_string(agent, format!("Error: {error_msg}"), gc.nogc()).unbind())
            }
        }
    }
    /// Write a text file with the content of the second argument.
    pub fn internal_write_text_file<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let binding = args
            .get(0)
            .to_string(agent, gc.borrow_mut().reborrow())
            .unbind()?;
        let content = args
            .get(1)
            .to_string(agent.borrow_mut(), gc.reborrow())
            .unbind()?;

        let path = binding.as_str(agent).expect("String is not valid UTF-8");
        let resolved_path = match resolve_path(path) {
            Ok(p) => p,
            Err(e) => {
                let error = AndromedaError::fs_error(e, "write_text_file", path);
                let error_msg = ErrorReporter::format_error(&error);
                return Ok(
                    Value::from_string(agent, format!("Error: {error_msg}"), gc.nogc()).unbind(),
                );
            }
        };

        match std::fs::write(
            &resolved_path,
            content.as_str(agent).expect("String is not valid UTF-8"),
        ) {
            Ok(_) => Ok(Value::from_string(agent, "Success".to_string(), gc.nogc()).unbind()),
            Err(e) => {
                let error = AndromedaError::fs_error(
                    e,
                    "write_text_file",
                    binding.as_str(agent).expect("String is not valid UTF-8"),
                );
                let error_msg = ErrorReporter::format_error(&error);
                Ok(Value::from_string(agent, format!("Error: {error_msg}"), gc.nogc()).unbind())
            }
        }
    }
    /// Create a file and return a Rid.
    pub fn internal_create_file<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let binding = args.get(0).to_string(agent, gc.reborrow()).unbind()?;
        let path = binding.as_str(agent).expect("String is not valid UTF-8");

        let resolved_path = match resolve_path(path) {
            Ok(p) => p,
            Err(e) => {
                let error = AndromedaError::fs_error(e, "create_file", path);
                let error_msg = ErrorReporter::format_error(&error);
                return Ok(
                    Value::from_string(agent, format!("Error: {error_msg}"), gc.nogc()).unbind(),
                );
            }
        };

        let file = match File::create(&resolved_path) {
            Ok(file) => file,
            Err(e) => {
                let error = AndromedaError::fs_error(e, "create_file", path);
                let error_msg = ErrorReporter::format_error(&error);
                return Ok(
                    Value::from_string(agent, format!("Error: {error_msg}"), gc.nogc()).unbind(),
                );
            }
        };

        let host_data = agent.get_host_data();
        let host_data: &HostData<RuntimeMacroTask> = host_data.downcast_ref().unwrap();
        let storage = host_data.storage.borrow();
        let resources: &FsExtResources = storage.get().unwrap();
        let rid = resources.files.push(file);

        Ok(Value::Integer(SmallInteger::from(rid.index())))
    }
    /// Copy a file from the first argument to the second argument.
    pub fn internal_copy_file<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let from = args.get(0).to_string(agent, gc.reborrow()).unbind()?;
        let to = args
            .get(1)
            .to_string(agent, gc.borrow_mut().reborrow())
            .unbind()?;

        let from_path = from.as_str(agent).expect("String is not valid UTF-8");
        let to_path = to.as_str(agent).expect("String is not valid UTF-8");

        let resolved_from = match resolve_path(from_path) {
            Ok(p) => p,
            Err(e) => {
                let error = AndromedaError::fs_error(e, "copy_file", from_path);
                let error_msg = ErrorReporter::format_error(&error);
                return Ok(
                    Value::from_string(agent, format!("Error: {error_msg}"), gc.nogc()).unbind(),
                );
            }
        };

        let resolved_to = match resolve_path(to_path) {
            Ok(p) => p,
            Err(e) => {
                let error = AndromedaError::fs_error(e, "copy_file", to_path);
                let error_msg = ErrorReporter::format_error(&error);
                return Ok(
                    Value::from_string(agent, format!("Error: {error_msg}"), gc.nogc()).unbind(),
                );
            }
        };

        match std::fs::copy(&resolved_from, &resolved_to) {
            Ok(_) => Ok(Value::from_string(agent, "Success".to_string(), gc.nogc()).unbind()),
            Err(e) => {
                let error = AndromedaError::fs_error(
                    e,
                    "copy_file",
                    format!(
                        "{} -> {}",
                        from.as_str(agent).expect("String is not valid UTF-8"),
                        to.as_str(agent).expect("String is not valid UTF-8")
                    ),
                );
                let error_msg = ErrorReporter::format_error(&error);
                Ok(Value::from_string(agent, format!("Error: {error_msg}"), gc.nogc()).unbind())
            }
        }
    }
    /// Create a directory.
    pub fn internal_mk_dir<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let binding = args.get(0).to_string(agent, gc.reborrow()).unbind()?;
        let path = binding.as_str(agent).expect("String is not valid UTF-8");

        let resolved_path = match resolve_path(path) {
            Ok(p) => p,
            Err(e) => {
                let error = AndromedaError::fs_error(e, "create_directory", path);
                let error_msg = ErrorReporter::format_error(&error);
                return Ok(
                    Value::from_string(agent, format!("Error: {error_msg}"), gc.nogc()).unbind(),
                );
            }
        };

        match std::fs::create_dir(&resolved_path) {
            Ok(_) => Ok(Value::from_string(agent, "Success".to_string(), gc.nogc()).unbind()),
            Err(e) => {
                let error = AndromedaError::fs_error(e, "create_directory", path);
                let error_msg = ErrorReporter::format_error(&error);
                Ok(Value::from_string(agent, format!("Error: {error_msg}"), gc.nogc()).unbind())
            }
        }
    }
    /// Create a directory recursively (mkdir -p equivalent).
    pub fn internal_mk_dir_all<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let binding = args.get(0).to_string(agent, gc.reborrow()).unbind()?;
        let path = binding.as_str(agent).expect("String is not valid UTF-8");

        let resolved_path = match resolve_path(path) {
            Ok(p) => p,
            Err(e) => {
                let error = AndromedaError::fs_error(e, "create_dir_all", path);
                let error_msg = ErrorReporter::format_error(&error);
                return Ok(
                    Value::from_string(agent, format!("Error: {error_msg}"), gc.nogc()).unbind(),
                );
            }
        };

        match std::fs::create_dir_all(&resolved_path) {
            Ok(_) => Ok(Value::from_string(agent, "Success".to_string(), gc.nogc()).unbind()),
            Err(e) => {
                let error = AndromedaError::fs_error(e, "create_dir_all", path);
                let error_msg = ErrorReporter::format_error(&error);
                Ok(Value::from_string(agent, format!("Error: {error_msg}"), gc.nogc()).unbind())
            }
        }
    }

    /// Read a file as bytes and return as a Uint8Array-like structure.
    pub fn internal_read_file<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let binding = args.get(0).to_string(agent, gc.reborrow()).unbind()?;
        let path = binding.as_str(agent).expect("String is not valid UTF-8");

        let resolved_path = match resolve_path(path) {
            Ok(p) => p,
            Err(e) => {
                let error = AndromedaError::fs_error(e, "read_file", path);
                let error_msg = ErrorReporter::format_error(&error);
                return Ok(
                    Value::from_string(agent, format!("Error: {error_msg}"), gc.nogc()).unbind(),
                );
            }
        };

        match std::fs::read(&resolved_path) {
            Ok(content) => {
                // For now, return the content as a hex encoded string
                // In a full implementation, you'd want to return an actual Uint8Array
                let hex_content = content.iter().fold(String::new(), |mut acc, b| {
                    use std::fmt::Write;
                    write!(&mut acc, "{b:02x}").unwrap();
                    acc
                });
                Ok(Value::from_string(agent, hex_content, gc.nogc()).unbind())
            }
            Err(e) => {
                let error = AndromedaError::fs_error(e, "read_file", path);
                let error_msg = ErrorReporter::format_error(&error);
                Ok(Value::from_string(agent, format!("Error: {error_msg}"), gc.nogc()).unbind())
            }
        }
    }

    /// Write bytes to a file.
    pub fn internal_write_file<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let path_binding = args
            .get(0)
            .to_string(agent, gc.borrow_mut().reborrow())
            .unbind()?;
        let content_binding = args
            .get(1)
            .to_string(agent.borrow_mut(), gc.reborrow())
            .unbind()?;

        let path = path_binding
            .as_str(agent)
            .expect("String is not valid UTF-8");
        let content_str = content_binding
            .as_str(agent)
            .expect("String is not valid UTF-8");

        let resolved_path = match resolve_path(path) {
            Ok(p) => p,
            Err(e) => {
                let error = AndromedaError::fs_error(e, "write_file", path);
                let error_msg = ErrorReporter::format_error(&error);
                return Ok(
                    Value::from_string(agent, format!("Error: {error_msg}"), gc.nogc()).unbind(),
                );
            }
        };

        // For now, just write the string as bytes
        // TODO: handle Uint8Array directly
        let content = content_str.as_bytes();

        match std::fs::write(&resolved_path, content) {
            Ok(_) => Ok(Value::from_string(agent, "Success".to_string(), gc.nogc()).unbind()),
            Err(e) => {
                let error = AndromedaError::fs_error(e, "write_file", path);
                let error_msg = ErrorReporter::format_error(&error);
                Ok(Value::from_string(agent, format!("Error: {error_msg}"), gc.nogc()).unbind())
            }
        }
    }

    /// Get file/directory statistics.
    pub fn internal_stat<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let binding = args.get(0).to_string(agent, gc.reborrow()).unbind()?;
        let path = binding.as_str(agent).expect("String is not valid UTF-8");

        let resolved_path = match resolve_path(path) {
            Ok(p) => p,
            Err(e) => {
                let error = AndromedaError::fs_error(e, "stat", path);
                let error_msg = ErrorReporter::format_error(&error);
                return Ok(
                    Value::from_string(agent, format!("Error: {error_msg}"), gc.nogc()).unbind(),
                );
            }
        };

        match std::fs::metadata(&resolved_path) {
            Ok(metadata) => {
                let stat_info = Self::format_file_info(&metadata);
                Ok(Value::from_string(agent, stat_info, gc.nogc()).unbind())
            }
            Err(e) => {
                let error = AndromedaError::fs_error(e, "stat", path);
                let error_msg = ErrorReporter::format_error(&error);
                Ok(Value::from_string(agent, format!("Error: {error_msg}"), gc.nogc()).unbind())
            }
        }
    }

    /// Get file/directory statistics without following symlinks.
    pub fn internal_lstat<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let binding = args.get(0).to_string(agent, gc.reborrow()).unbind()?;
        let path = binding.as_str(agent).expect("String is not valid UTF-8");

        let resolved_path = match resolve_path(path) {
            Ok(p) => p,
            Err(e) => {
                let error = AndromedaError::fs_error(e, "lstat", path);
                let error_msg = ErrorReporter::format_error(&error);
                return Ok(
                    Value::from_string(agent, format!("Error: {error_msg}"), gc.nogc()).unbind(),
                );
            }
        };

        match std::fs::symlink_metadata(&resolved_path) {
            Ok(metadata) => {
                let stat_info = Self::format_file_info(&metadata);
                Ok(Value::from_string(agent, stat_info, gc.nogc()).unbind())
            }
            Err(e) => {
                let error = AndromedaError::fs_error(e, "lstat", path);
                let error_msg = ErrorReporter::format_error(&error);
                Ok(Value::from_string(agent, format!("Error: {error_msg}"), gc.nogc()).unbind())
            }
        }
    }

    /// Read directory contents.
    pub fn internal_read_dir<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let binding = args.get(0).to_string(agent, gc.reborrow()).unbind()?;
        let path = binding.as_str(agent).expect("String is not valid UTF-8");

        let resolved_path = match resolve_path(path) {
            Ok(p) => p,
            Err(e) => {
                let error = AndromedaError::fs_error(e, "read_dir", path);
                let error_msg = ErrorReporter::format_error(&error);
                return Ok(
                    Value::from_string(agent, format!("Error: {error_msg}"), gc.nogc()).unbind(),
                );
            }
        };

        match std::fs::read_dir(&resolved_path) {
            Ok(entries) => {
                let mut result = String::from("[");
                let mut first = true;

                for entry in entries {
                    match entry {
                        Ok(entry) => {
                            if !first {
                                result.push(',');
                            }
                            first = false;

                            let file_name = entry.file_name();
                            let name = file_name.to_string_lossy();
                            let is_dir = entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false);
                            let is_file = entry.file_type().map(|ft| ft.is_file()).unwrap_or(false);
                            let is_symlink =
                                entry.file_type().map(|ft| ft.is_symlink()).unwrap_or(false);

                            result.push_str(&format!(
                                "{{\"name\":\"{name}\",\"isFile\":{is_file},\"isDirectory\":{is_dir},\"isSymlink\":{is_symlink}}}"
                            ));
                        }
                        Err(_) => continue,
                    }
                }
                result.push(']');

                Ok(Value::from_string(agent, result, gc.nogc()).unbind())
            }
            Err(e) => {
                let error = AndromedaError::fs_error(e, "read_dir", path);
                let error_msg = ErrorReporter::format_error(&error);
                Ok(Value::from_string(agent, format!("Error: {error_msg}"), gc.nogc()).unbind())
            }
        }
    }

    /// Remove a file or empty directory.
    pub fn internal_remove<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let binding = args.get(0).to_string(agent, gc.reborrow()).unbind()?;
        let path = binding.as_str(agent).expect("String is not valid UTF-8");

        let resolved_path = match resolve_path(path) {
            Ok(p) => p,
            Err(e) => {
                let error = AndromedaError::fs_error(e, "remove", path);
                let error_msg = ErrorReporter::format_error(&error);
                return Ok(
                    Value::from_string(agent, format!("Error: {error_msg}"), gc.nogc()).unbind(),
                );
            }
        };

        let result = if resolved_path.is_dir() {
            std::fs::remove_dir(&resolved_path)
        } else {
            std::fs::remove_file(&resolved_path)
        };

        match result {
            Ok(_) => Ok(Value::from_string(agent, "Success".to_string(), gc.nogc()).unbind()),
            Err(e) => {
                let error = AndromedaError::fs_error(e, "remove", path);
                let error_msg = ErrorReporter::format_error(&error);
                Ok(Value::from_string(agent, format!("Error: {error_msg}"), gc.nogc()).unbind())
            }
        }
    }

    /// Remove a file or directory recursively.
    pub fn internal_remove_all<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let binding = args.get(0).to_string(agent, gc.reborrow()).unbind()?;
        let path = binding.as_str(agent).expect("String is not valid UTF-8");

        let resolved_path = match resolve_path(path) {
            Ok(p) => p,
            Err(e) => {
                let error = AndromedaError::fs_error(e, "remove_all", path);
                let error_msg = ErrorReporter::format_error(&error);
                return Ok(
                    Value::from_string(agent, format!("Error: {error_msg}"), gc.nogc()).unbind(),
                );
            }
        };

        let result = if resolved_path.is_dir() {
            std::fs::remove_dir_all(&resolved_path)
        } else {
            std::fs::remove_file(&resolved_path)
        };

        match result {
            Ok(_) => Ok(Value::from_string(agent, "Success".to_string(), gc.nogc()).unbind()),
            Err(e) => {
                let error = AndromedaError::fs_error(e, "remove_all", path);
                let error_msg = ErrorReporter::format_error(&error);
                Ok(Value::from_string(agent, format!("Error: {error_msg}"), gc.nogc()).unbind())
            }
        }
    }

    /// Rename/move a file or directory.
    pub fn internal_rename<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let from = args.get(0).to_string(agent, gc.reborrow()).unbind()?;
        let to = args.get(1).to_string(agent, gc.reborrow()).unbind()?;

        let from_path = from.as_str(agent).expect("String is not valid UTF-8");
        let to_path = to.as_str(agent).expect("String is not valid UTF-8");

        let resolved_from = match resolve_path(from_path) {
            Ok(p) => p,
            Err(e) => {
                let error = AndromedaError::fs_error(e, "rename", from_path);
                let error_msg = ErrorReporter::format_error(&error);
                return Ok(
                    Value::from_string(agent, format!("Error: {error_msg}"), gc.nogc()).unbind(),
                );
            }
        };

        let resolved_to = match resolve_path(to_path) {
            Ok(p) => p,
            Err(e) => {
                let error = AndromedaError::fs_error(e, "rename", to_path);
                let error_msg = ErrorReporter::format_error(&error);
                return Ok(
                    Value::from_string(agent, format!("Error: {error_msg}"), gc.nogc()).unbind(),
                );
            }
        };

        match std::fs::rename(&resolved_from, &resolved_to) {
            Ok(_) => Ok(Value::from_string(agent, "Success".to_string(), gc.nogc()).unbind()),
            Err(e) => {
                let error = AndromedaError::fs_error(
                    e,
                    "rename",
                    format!(
                        "{} -> {}",
                        from.as_str(agent).expect("String is not valid UTF-8"),
                        to.as_str(agent).expect("String is not valid UTF-8")
                    ),
                );
                let error_msg = ErrorReporter::format_error(&error);
                Ok(Value::from_string(agent, format!("Error: {error_msg}"), gc.nogc()).unbind())
            }
        }
    }

    /// Check if a file or directory exists.
    pub fn internal_exists<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let binding = args.get(0).to_string(agent, gc.reborrow()).unbind()?;
        let path = binding.as_str(agent).expect("String is not valid UTF-8");

        let resolved_path = match resolve_path(path) {
            Ok(p) => p,
            Err(_) => {
                // If we can't resolve the path (e.g., cwd doesn't exist), it doesn't exist
                return Ok(Value::from_string(agent, "false".to_string(), gc.nogc()).unbind());
            }
        };

        let exists = resolved_path.exists();
        Ok(Value::from_string(agent, exists.to_string(), gc.nogc()).unbind())
    }
    /// Truncate a file to a specific length.
    pub fn internal_truncate<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let path_binding = args
            .get(0)
            .to_string(agent, gc.borrow_mut().reborrow())
            .unbind()?;
        let len_binding = args
            .get(1)
            .to_string(agent.borrow_mut(), gc.reborrow())
            .unbind()?;

        let path = path_binding
            .as_str(agent)
            .expect("String is not valid UTF-8");
        let len_str = len_binding
            .as_str(agent)
            .expect("String is not valid UTF-8");

        let len: u64 = match len_str.parse() {
            Ok(l) => l,
            Err(_) => {
                return Ok(Value::from_string(
                    agent,
                    "Error: Invalid length parameter".to_string(),
                    gc.nogc(),
                )
                .unbind());
            }
        };

        let resolved_path = match resolve_path(path) {
            Ok(p) => p,
            Err(e) => {
                let error = AndromedaError::fs_error(e, "truncate", path);
                let error_msg = ErrorReporter::format_error(&error);
                return Ok(
                    Value::from_string(agent, format!("Error: {error_msg}"), gc.nogc()).unbind(),
                );
            }
        };

        let file = std::fs::OpenOptions::new().write(true).open(&resolved_path);
        match file {
            Ok(f) => match f.set_len(len) {
                Ok(_) => Ok(Value::from_string(agent, "Success".to_string(), gc.nogc()).unbind()),
                Err(e) => {
                    let error = AndromedaError::fs_error(e, "truncate", path);
                    let error_msg = ErrorReporter::format_error(&error);
                    Ok(
                        Value::from_string(agent, format!("Error: {error_msg}"), gc.nogc())
                            .unbind(),
                    )
                }
            },
            Err(e) => {
                let error = AndromedaError::fs_error(e, "truncate", path);
                let error_msg = ErrorReporter::format_error(&error);
                Ok(Value::from_string(agent, format!("Error: {error_msg}"), gc.nogc()).unbind())
            }
        }
    }
    /// Change file permissions (chmod).
    pub fn internal_chmod<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let path_binding = args
            .get(0)
            .to_string(agent, gc.borrow_mut().reborrow())
            .unbind()?;
        let mode_binding = args
            .get(1)
            .to_string(agent.borrow_mut(), gc.reborrow())
            .unbind()?;

        let path = path_binding
            .as_str(agent)
            .expect("String is not valid UTF-8");
        let mode_str = mode_binding
            .as_str(agent)
            .expect("String is not valid UTF-8");

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            let mode: u32 = match u32::from_str_radix(mode_str, 8) {
                Ok(m) => m,
                Err(_) => {
                    return Ok(Value::from_string(
                        agent,
                        "Error: Invalid mode parameter".to_string(),
                        gc.nogc(),
                    )
                    .unbind());
                }
            };

            let resolved_path = match resolve_path(path) {
                Ok(p) => p,
                Err(e) => {
                    let error = AndromedaError::fs_error(e, "chmod", path);
                    let error_msg = ErrorReporter::format_error(&error);
                    return Ok(
                        Value::from_string(agent, format!("Error: {error_msg}"), gc.nogc())
                            .unbind(),
                    );
                }
            };

            let permissions = std::fs::Permissions::from_mode(mode);
            match std::fs::set_permissions(&resolved_path, permissions) {
                Ok(_) => Ok(Value::from_string(agent, "Success".to_string(), gc.nogc()).unbind()),
                Err(e) => {
                    let error = AndromedaError::fs_error(e, "chmod", path);
                    let error_msg = ErrorReporter::format_error(&error);
                    Ok(
                        Value::from_string(agent, format!("Error: {error_msg}"), gc.nogc())
                            .unbind(),
                    )
                }
            }
        }

        #[cfg(not(unix))]
        {
            let _ = path; // Use variables to avoid warnings
            let _ = mode_str;
            Ok(Value::from_string(
                agent,
                "Error: chmod not supported on this platform".to_string(),
                gc.nogc(),
            )
            .unbind())
        }
    }

    /// Create a symbolic link.
    pub fn internal_symlink<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let target = args.get(0).to_string(agent, gc.reborrow()).unbind()?;
        let link = args.get(1).to_string(agent, gc.reborrow()).unbind()?;

        let target_path = target.as_str(agent).expect("String is not valid UTF-8");
        let link_path = link.as_str(agent).expect("String is not valid UTF-8");

        let resolved_target = match resolve_path(target_path) {
            Ok(p) => p,
            Err(e) => {
                let error = AndromedaError::fs_error(e, "symlink", target_path);
                let error_msg = ErrorReporter::format_error(&error);
                return Ok(
                    Value::from_string(agent, format!("Error: {error_msg}"), gc.nogc()).unbind(),
                );
            }
        };

        let resolved_link = match resolve_path(link_path) {
            Ok(p) => p,
            Err(e) => {
                let error = AndromedaError::fs_error(e, "symlink", link_path);
                let error_msg = ErrorReporter::format_error(&error);
                return Ok(
                    Value::from_string(agent, format!("Error: {error_msg}"), gc.nogc()).unbind(),
                );
            }
        };

        #[cfg(unix)]
        {
            match std::os::unix::fs::symlink(&resolved_target, &resolved_link) {
                Ok(_) => Ok(Value::from_string(agent, "Success".to_string(), gc.nogc()).unbind()),
                Err(e) => {
                    let error = AndromedaError::fs_error(
                        e,
                        "symlink",
                        format!(
                            "{} -> {}",
                            target.as_str(agent).expect("String is not valid UTF-8"),
                            link.as_str(agent).expect("String is not valid UTF-8")
                        ),
                    );
                    let error_msg = ErrorReporter::format_error(&error);
                    Ok(
                        Value::from_string(agent, format!("Error: {error_msg}"), gc.nogc())
                            .unbind(),
                    )
                }
            }
        }

        #[cfg(windows)]
        {
            use std::os::windows::fs;
            let result = if resolved_target.is_dir() {
                fs::symlink_dir(&resolved_target, &resolved_link)
            } else {
                fs::symlink_file(&resolved_target, &resolved_link)
            };

            match result {
                Ok(_) => Ok(Value::from_string(agent, "Success".to_string(), gc.nogc()).unbind()),
                Err(e) => {
                    let error = AndromedaError::fs_error(
                        e,
                        "symlink",
                        format!(
                            "{} -> {}",
                            target.as_str(agent).expect("String is not valid UTF-8"),
                            link.as_str(agent).expect("String is not valid UTF-8")
                        ),
                    );
                    let error_msg = ErrorReporter::format_error(&error);
                    Ok(
                        Value::from_string(agent, format!("Error: {error_msg}"), gc.nogc())
                            .unbind(),
                    )
                }
            }
        }
    }

    /// Read the target of a symbolic link.
    pub fn internal_read_link<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let binding = args.get(0).to_string(agent, gc.reborrow()).unbind()?;
        let path = binding.as_str(agent).expect("String is not valid UTF-8");

        let resolved_path = match resolve_path(path) {
            Ok(p) => p,
            Err(e) => {
                let error = AndromedaError::fs_error(e, "read_link", path);
                let error_msg = ErrorReporter::format_error(&error);
                return Ok(
                    Value::from_string(agent, format!("Error: {error_msg}"), gc.nogc()).unbind(),
                );
            }
        };

        match std::fs::read_link(&resolved_path) {
            Ok(target) => {
                let target_str = target.to_string_lossy().to_string();
                Ok(Value::from_string(agent, target_str, gc.nogc()).unbind())
            }
            Err(e) => {
                let error = AndromedaError::fs_error(e, "read_link", path);
                let error_msg = ErrorReporter::format_error(&error);
                Ok(Value::from_string(agent, format!("Error: {error_msg}"), gc.nogc()).unbind())
            }
        }
    }

    /// Get the real (canonical) path of a file/directory.
    pub fn internal_real_path<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let binding = args.get(0).to_string(agent, gc.reborrow()).unbind()?;
        let path = binding.as_str(agent).expect("String is not valid UTF-8");

        let resolved_path = match resolve_path(path) {
            Ok(p) => p,
            Err(e) => {
                let error = AndromedaError::fs_error(e, "real_path", path);
                let error_msg = ErrorReporter::format_error(&error);
                return Ok(
                    Value::from_string(agent, format!("Error: {error_msg}"), gc.nogc()).unbind(),
                );
            }
        };

        match std::fs::canonicalize(&resolved_path) {
            Ok(real_path) => {
                let path_str = real_path.to_string_lossy().to_string();
                Ok(Value::from_string(agent, path_str, gc.nogc()).unbind())
            }
            Err(e) => {
                let error = AndromedaError::fs_error(e, "real_path", path);
                let error_msg = ErrorReporter::format_error(&error);
                Ok(Value::from_string(agent, format!("Error: {error_msg}"), gc.nogc()).unbind())
            }
        }
    }

    /// Open a file and return a Rid.
    pub fn internal_open_file<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let binding = args.get(0).to_string(agent, gc.reborrow()).unbind()?;
        let path = binding.as_str(agent).expect("String is not valid UTF-8");

        let resolved_path = match resolve_path(path) {
            Ok(p) => p,
            Err(e) => {
                let error = AndromedaError::fs_error(e, "open_file", path);
                let error_msg = ErrorReporter::format_error(&error);
                return Ok(
                    Value::from_string(agent, format!("Error: {error_msg}"), gc.nogc()).unbind(),
                );
            }
        };

        let file = match File::open(&resolved_path) {
            Ok(file) => file,
            Err(e) => {
                let error = AndromedaError::fs_error(e, "open_file", path);
                let error_msg = ErrorReporter::format_error(&error);
                return Ok(
                    Value::from_string(agent, format!("Error: {error_msg}"), gc.nogc()).unbind(),
                );
            }
        };

        let host_data = agent.get_host_data();
        let host_data: &HostData<RuntimeMacroTask> = host_data.downcast_ref().unwrap();
        let storage = host_data.storage.borrow();
        let resources: &FsExtResources = storage.get().unwrap();
        let rid = resources.files.push(file);

        Ok(Value::Integer(SmallInteger::from(rid.index())))
    }

    /// Helper function to format file metadata as JSON string.
    fn format_file_info(metadata: &Metadata) -> String {
        let size = metadata.len();
        let is_file = metadata.is_file();
        let is_dir = metadata.is_dir();
        let is_symlink = metadata.is_symlink();

        let modified = metadata
            .modified()
            .map(|time| {
                time.duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs()
            })
            .unwrap_or(0);

        let accessed = metadata
            .accessed()
            .map(|time| {
                time.duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs()
            })
            .unwrap_or(0);

        let created = metadata
            .created()
            .map(|time| {
                time.duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs()
            })
            .unwrap_or(0);

        format!(
            "{{\"size\":{size},\"isFile\":{is_file},\"isDirectory\":{is_dir},\"isSymlink\":{is_symlink},\"modified\":{modified},\"accessed\":{accessed},\"created\":{created}}}"
        )
    }

    // Async file operations

    /// Read a text file asynchronously and return the content as a string.
    pub fn internal_read_text_file_async<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let binding = args.get(0).to_string(agent, gc.reborrow()).unbind()?;
        let path = binding.as_str(agent).expect("String is not valid UTF-8");

        let resolved_path = match resolve_path(path) {
            Ok(p) => p,
            Err(e) => {
                let error = AndromedaError::fs_error(e, "read_text_file_async", path);
                let error_msg = ErrorReporter::format_error(&error);
                let promise_capability = PromiseCapability::new(agent, gc.nogc());
                let root_value =
                    Global::new(agent, promise_capability.promise().into_value().unbind());
                let host_data = agent.get_host_data();
                let host_data: &HostData<RuntimeMacroTask> = host_data.downcast_ref().unwrap();
                let macro_task_tx = host_data.macro_task_tx();
                macro_task_tx
                    .send(MacroTask::User(RuntimeMacroTask::RejectPromise(
                        root_value,
                        format!("Error: {error_msg}"),
                    )))
                    .unwrap();
                return Ok(Value::Promise(promise_capability.promise()).unbind());
            }
        };

        let path_string = resolved_path.to_string_lossy().to_string();

        let promise_capability = PromiseCapability::new(agent, gc.nogc());
        let root_value = Global::new(agent, promise_capability.promise().into_value().unbind());
        let host_data = agent.get_host_data();
        let host_data: &HostData<RuntimeMacroTask> = host_data.downcast_ref().unwrap();
        let macro_task_tx = host_data.macro_task_tx();

        host_data.spawn_macro_task(async move {
            let result = tokio::fs::read_to_string(&path_string).await;
            match result {
                Ok(content) => {
                    macro_task_tx
                        .send(MacroTask::User(RuntimeMacroTask::ResolvePromiseWithString(
                            root_value, content,
                        )))
                        .unwrap();
                }
                Err(e) => {
                    let error = AndromedaError::fs_error(e, "read_text_file_async", &path_string);
                    let error_msg = ErrorReporter::format_error(&error);
                    macro_task_tx
                        .send(MacroTask::User(RuntimeMacroTask::RejectPromise(
                            root_value,
                            format!("Error: {error_msg}"),
                        )))
                        .unwrap();
                }
            }
        });

        Ok(Value::Promise(promise_capability.promise()).unbind())
    }

    /// Write a text file asynchronously with the content of the second argument.
    pub fn internal_write_text_file_async<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let path_binding = args
            .get(0)
            .to_string(agent, gc.borrow_mut().reborrow())
            .unbind()?;
        let content_binding = args
            .get(1)
            .to_string(agent.borrow_mut(), gc.reborrow())
            .unbind()?;

        let path = path_binding
            .as_str(agent)
            .expect("String is not valid UTF-8");
        let content_string = content_binding
            .as_str(agent)
            .expect("String is not valid UTF-8")
            .to_string();

        let resolved_path = match resolve_path(path) {
            Ok(p) => p,
            Err(e) => {
                let error = AndromedaError::fs_error(e, "write_text_file_async", path);
                let error_msg = ErrorReporter::format_error(&error);
                let promise_capability = PromiseCapability::new(agent, gc.nogc());
                let root_value =
                    Global::new(agent, promise_capability.promise().into_value().unbind());
                let host_data = agent.get_host_data();
                let host_data: &HostData<RuntimeMacroTask> = host_data.downcast_ref().unwrap();
                let macro_task_tx = host_data.macro_task_tx();
                macro_task_tx
                    .send(MacroTask::User(RuntimeMacroTask::RejectPromise(
                        root_value,
                        format!("Error: {error_msg}"),
                    )))
                    .unwrap();
                return Ok(Value::Promise(promise_capability.promise()).unbind());
            }
        };

        let path_string = resolved_path.to_string_lossy().to_string();

        let promise_capability = PromiseCapability::new(agent, gc.nogc());
        let root_value = Global::new(agent, promise_capability.promise().into_value().unbind());
        let host_data = agent.get_host_data();
        let host_data: &HostData<RuntimeMacroTask> = host_data.downcast_ref().unwrap();
        let macro_task_tx = host_data.macro_task_tx();

        host_data.spawn_macro_task(async move {
            let result = tokio::fs::write(&path_string, &content_string).await;
            match result {
                Ok(_) => {
                    macro_task_tx
                        .send(MacroTask::User(RuntimeMacroTask::ResolvePromiseWithString(
                            root_value,
                            "Success".to_string(),
                        )))
                        .unwrap();
                }
                Err(e) => {
                    let error = AndromedaError::fs_error(e, "write_text_file_async", &path_string);
                    let error_msg = ErrorReporter::format_error(&error);
                    macro_task_tx
                        .send(MacroTask::User(RuntimeMacroTask::RejectPromise(
                            root_value,
                            format!("Error: {error_msg}"),
                        )))
                        .unwrap();
                }
            }
        });

        Ok(Value::Promise(promise_capability.promise()).unbind())
    }

    /// Read a file asynchronously as bytes and return as a Uint8Array-like structure.
    pub fn internal_read_file_async<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let binding = args.get(0).to_string(agent, gc.reborrow()).unbind()?;
        let path_string = binding
            .as_str(agent)
            .expect("String is not valid UTF-8")
            .to_string();

        let promise_capability = PromiseCapability::new(agent, gc.nogc());
        let root_value = Global::new(agent, promise_capability.promise().into_value().unbind());
        let host_data = agent.get_host_data();
        let host_data: &HostData<RuntimeMacroTask> = host_data.downcast_ref().unwrap();
        let macro_task_tx = host_data.macro_task_tx();

        host_data.spawn_macro_task(async move {
            let result = tokio::fs::read(&path_string).await;
            match result {
                Ok(content) => {
                    macro_task_tx
                        .send(MacroTask::User(RuntimeMacroTask::ResolvePromiseWithBytes(
                            root_value, content,
                        )))
                        .unwrap();
                }
                Err(e) => {
                    let error = AndromedaError::fs_error(e, "read_file_async", &path_string);
                    let error_msg = ErrorReporter::format_error(&error);
                    macro_task_tx
                        .send(MacroTask::User(RuntimeMacroTask::RejectPromise(
                            root_value,
                            format!("Error: {error_msg}"),
                        )))
                        .unwrap();
                }
            }
        });

        Ok(Value::Promise(promise_capability.promise()).unbind())
    }

    /// Write bytes to a file asynchronously.
    pub fn internal_write_file_async<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let path_binding = args
            .get(0)
            .to_string(agent, gc.borrow_mut().reborrow())
            .unbind()?;
        let content_binding = args
            .get(1)
            .to_string(agent.borrow_mut(), gc.reborrow())
            .unbind()?;

        let path_string = path_binding
            .as_str(agent)
            .expect("String is not valid UTF-8")
            .to_string();
        let content_string = content_binding
            .as_str(agent)
            .expect("String is not valid UTF-8")
            .to_string();

        let promise_capability = PromiseCapability::new(agent, gc.nogc());
        let root_value = Global::new(agent, promise_capability.promise().into_value().unbind());
        let host_data = agent.get_host_data();
        let host_data: &HostData<RuntimeMacroTask> = host_data.downcast_ref().unwrap();
        let macro_task_tx = host_data.macro_task_tx();

        host_data.spawn_macro_task(async move {
            // For now, just write the string as bytes
            // TODO: In a full implementation, you'd want to handle Uint8Array directly
            let content = content_string.as_bytes();
            let result = tokio::fs::write(&path_string, content).await;
            match result {
                Ok(_) => {
                    macro_task_tx
                        .send(MacroTask::User(RuntimeMacroTask::ResolvePromiseWithString(
                            root_value,
                            "Success".to_string(),
                        )))
                        .unwrap();
                }
                Err(e) => {
                    let error = AndromedaError::fs_error(e, "write_file_async", &path_string);
                    let error_msg = ErrorReporter::format_error(&error);
                    macro_task_tx
                        .send(MacroTask::User(RuntimeMacroTask::RejectPromise(
                            root_value,
                            format!("Error: {error_msg}"),
                        )))
                        .unwrap();
                }
            }
        });

        Ok(Value::Promise(promise_capability.promise()).unbind())
    }

    /// Copy a file asynchronously from the first argument to the second argument.
    pub fn internal_copy_file_async<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let from_binding = args.get(0).to_string(agent, gc.reborrow()).unbind()?;
        let to_binding = args
            .get(1)
            .to_string(agent, gc.borrow_mut().reborrow())
            .unbind()?;

        let from_string = from_binding
            .as_str(agent)
            .expect("String is not valid UTF-8")
            .to_string();
        let to_string = to_binding
            .as_str(agent)
            .expect("String is not valid UTF-8")
            .to_string();

        let promise_capability = PromiseCapability::new(agent, gc.nogc());
        let root_value = Global::new(agent, promise_capability.promise().into_value().unbind());
        let host_data = agent.get_host_data();
        let host_data: &HostData<RuntimeMacroTask> = host_data.downcast_ref().unwrap();
        let macro_task_tx = host_data.macro_task_tx();

        host_data.spawn_macro_task(async move {
            let result = tokio::fs::copy(&from_string, &to_string).await;
            match result {
                Ok(_) => {
                    macro_task_tx
                        .send(MacroTask::User(RuntimeMacroTask::ResolvePromiseWithString(
                            root_value,
                            "Success".to_string(),
                        )))
                        .unwrap();
                }
                Err(e) => {
                    let error = AndromedaError::fs_error(
                        e,
                        "copy_file_async",
                        format!("{from_string} -> {to_string}"),
                    );
                    let error_msg = ErrorReporter::format_error(&error);
                    macro_task_tx
                        .send(MacroTask::User(RuntimeMacroTask::RejectPromise(
                            root_value,
                            format!("Error: {error_msg}"),
                        )))
                        .unwrap();
                }
            }
        });

        Ok(Value::Promise(promise_capability.promise()).unbind())
    }

    /// Remove a file or directory asynchronously.
    pub fn internal_remove_async<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let binding = args.get(0).to_string(agent, gc.reborrow()).unbind()?;
        let path_string = binding
            .as_str(agent)
            .expect("String is not valid UTF-8")
            .to_string();

        let promise_capability = PromiseCapability::new(agent, gc.nogc());
        let root_value = Global::new(agent, promise_capability.promise().into_value().unbind());
        let host_data = agent.get_host_data();
        let host_data: &HostData<RuntimeMacroTask> = host_data.downcast_ref().unwrap();
        let macro_task_tx = host_data.macro_task_tx();

        host_data.spawn_macro_task(async move {
            let path = Path::new(&path_string);
            let result = if path.is_dir() {
                tokio::fs::remove_dir(&path_string).await
            } else {
                tokio::fs::remove_file(&path_string).await
            };

            match result {
                Ok(_) => {
                    macro_task_tx
                        .send(MacroTask::User(RuntimeMacroTask::ResolvePromiseWithString(
                            root_value,
                            "Success".to_string(),
                        )))
                        .unwrap();
                }
                Err(e) => {
                    let error = AndromedaError::fs_error(e, "remove_async", &path_string);
                    let error_msg = ErrorReporter::format_error(&error);
                    macro_task_tx
                        .send(MacroTask::User(RuntimeMacroTask::RejectPromise(
                            root_value,
                            format!("Error: {error_msg}"),
                        )))
                        .unwrap();
                }
            }
        });

        Ok(Value::Promise(promise_capability.promise()).unbind())
    }

    /// Create a file asynchronously.
    pub fn internal_create_file_async<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let binding = args.get(0).to_string(agent, gc.reborrow()).unbind()?;
        let path_string = binding
            .as_str(agent)
            .expect("String is not valid UTF-8")
            .to_string();

        let promise_capability = PromiseCapability::new(agent, gc.nogc());
        let root_value = Global::new(agent, promise_capability.promise().into_value().unbind());
        let host_data = agent.get_host_data();
        let host_data: &HostData<RuntimeMacroTask> = host_data.downcast_ref().unwrap();
        let macro_task_tx = host_data.macro_task_tx();

        host_data.spawn_macro_task(async move {
            let result = tokio::fs::File::create(&path_string).await;
            match result {
                Ok(_) => {
                    macro_task_tx
                        .send(MacroTask::User(RuntimeMacroTask::ResolvePromiseWithString(
                            root_value,
                            "Success".to_string(),
                        )))
                        .unwrap();
                }
                Err(e) => {
                    let error = AndromedaError::fs_error(e, "create_file_async", &path_string);
                    let error_msg = ErrorReporter::format_error(&error);
                    macro_task_tx
                        .send(MacroTask::User(RuntimeMacroTask::RejectPromise(
                            root_value,
                            format!("Error: {error_msg}"),
                        )))
                        .unwrap();
                }
            }
        });

        Ok(Value::Promise(promise_capability.promise()).unbind())
    }

    /// Create a directory asynchronously.
    pub fn internal_mk_dir_async<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let binding = args.get(0).to_string(agent, gc.reborrow()).unbind()?;
        let path_string = binding
            .as_str(agent)
            .expect("String is not valid UTF-8")
            .to_string();

        let promise_capability = PromiseCapability::new(agent, gc.nogc());
        let root_value = Global::new(agent, promise_capability.promise().into_value().unbind());
        let host_data = agent.get_host_data();
        let host_data: &HostData<RuntimeMacroTask> = host_data.downcast_ref().unwrap();
        let macro_task_tx = host_data.macro_task_tx();

        host_data.spawn_macro_task(async move {
            let result = tokio::fs::create_dir(&path_string).await;
            match result {
                Ok(_) => {
                    macro_task_tx
                        .send(MacroTask::ResolvePromise(root_value))
                        .unwrap();
                }
                Err(e) => {
                    let error = AndromedaError::fs_error(e, "mk_dir_async", &path_string);
                    let error_msg = ErrorReporter::format_error(&error);
                    macro_task_tx
                        .send(MacroTask::User(RuntimeMacroTask::RejectPromise(
                            root_value,
                            format!("Error: {error_msg}"),
                        )))
                        .unwrap();
                }
            }
        });

        Ok(Value::Promise(promise_capability.promise()).unbind())
    }

    /// Create a directory recursively asynchronously (mkdir -p equivalent).
    pub fn internal_mk_dir_all_async<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let binding = args.get(0).to_string(agent, gc.reborrow()).unbind()?;
        let path_string = binding
            .as_str(agent)
            .expect("String is not valid UTF-8")
            .to_string();

        let promise_capability = PromiseCapability::new(agent, gc.nogc());
        let root_value = Global::new(agent, promise_capability.promise().into_value().unbind());
        let host_data = agent.get_host_data();
        let host_data: &HostData<RuntimeMacroTask> = host_data.downcast_ref().unwrap();
        let macro_task_tx = host_data.macro_task_tx();

        host_data.spawn_macro_task(async move {
            let result = tokio::fs::create_dir_all(&path_string).await;
            match result {
                Ok(_) => {
                    macro_task_tx
                        .send(MacroTask::ResolvePromise(root_value))
                        .unwrap();
                }
                Err(e) => {
                    let error = AndromedaError::fs_error(e, "mk_dir_all_async", &path_string);
                    let error_msg = ErrorReporter::format_error(&error);
                    macro_task_tx
                        .send(MacroTask::User(RuntimeMacroTask::RejectPromise(
                            root_value,
                            format!("Error: {error_msg}"),
                        )))
                        .unwrap();
                }
            }
        });

        Ok(Value::Promise(promise_capability.promise()).unbind())
    }

    /// Check if a file or directory exists asynchronously.
    pub fn internal_exists_async<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let binding = args.get(0).to_string(agent, gc.reborrow()).unbind()?;
        let path_string = binding
            .as_str(agent)
            .expect("String is not valid UTF-8")
            .to_string();

        let promise_capability = PromiseCapability::new(agent, gc.nogc());
        let root_value = Global::new(agent, promise_capability.promise().into_value().unbind());
        let host_data = agent.get_host_data();
        let host_data: &HostData<RuntimeMacroTask> = host_data.downcast_ref().unwrap();
        let macro_task_tx = host_data.macro_task_tx();

        host_data.spawn_macro_task(async move {
            let exists = tokio::fs::try_exists(&path_string).await.unwrap_or(false);
            macro_task_tx
                .send(MacroTask::User(RuntimeMacroTask::ResolvePromiseWithString(
                    root_value,
                    exists.to_string(),
                )))
                .unwrap();
        });

        Ok(Value::Promise(promise_capability.promise()).unbind())
    }

    /// Rename/move a file or directory asynchronously.
    pub fn internal_rename_async<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let from_binding = args.get(0).to_string(agent, gc.reborrow()).unbind()?;
        let to_binding = args.get(1).to_string(agent, gc.reborrow()).unbind()?;

        let from_string = from_binding
            .as_str(agent)
            .expect("String is not valid UTF-8")
            .to_string();
        let to_string = to_binding
            .as_str(agent)
            .expect("String is not valid UTF-8")
            .to_string();

        let promise_capability = PromiseCapability::new(agent, gc.nogc());
        let root_value = Global::new(agent, promise_capability.promise().into_value().unbind());
        let host_data = agent.get_host_data();
        let host_data: &HostData<RuntimeMacroTask> = host_data.downcast_ref().unwrap();
        let macro_task_tx = host_data.macro_task_tx();

        host_data.spawn_macro_task(async move {
            let result = tokio::fs::rename(&from_string, &to_string).await;
            match result {
                Ok(_) => {
                    macro_task_tx
                        .send(MacroTask::ResolvePromise(root_value))
                        .unwrap();
                }
                Err(e) => {
                    let error = AndromedaError::fs_error(e, "rename_async", &from_string);
                    let error_msg = ErrorReporter::format_error(&error);
                    macro_task_tx
                        .send(MacroTask::User(RuntimeMacroTask::RejectPromise(
                            root_value,
                            format!("Error: {error_msg}"),
                        )))
                        .unwrap();
                }
            }
        });

        Ok(Value::Promise(promise_capability.promise()).unbind())
    }

    /// Remove a file or directory recursively asynchronously.
    pub fn internal_remove_all_async<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let binding = args.get(0).to_string(agent, gc.reborrow()).unbind()?;
        let path_string = binding
            .as_str(agent)
            .expect("String is not valid UTF-8")
            .to_string();

        let promise_capability = PromiseCapability::new(agent, gc.nogc());
        let root_value = Global::new(agent, promise_capability.promise().into_value().unbind());
        let host_data = agent.get_host_data();
        let host_data: &HostData<RuntimeMacroTask> = host_data.downcast_ref().unwrap();
        let macro_task_tx = host_data.macro_task_tx();

        host_data.spawn_macro_task(async move {
            // Check if it's a directory first
            let result = if tokio::fs::metadata(&path_string)
                .await
                .map(|m| m.is_dir())
                .unwrap_or(false)
            {
                tokio::fs::remove_dir_all(&path_string).await
            } else {
                tokio::fs::remove_file(&path_string).await
            };

            match result {
                Ok(_) => {
                    macro_task_tx
                        .send(MacroTask::ResolvePromise(root_value))
                        .unwrap();
                }
                Err(e) => {
                    let error = AndromedaError::fs_error(e, "remove_all_async", &path_string);
                    let error_msg = ErrorReporter::format_error(&error);
                    macro_task_tx
                        .send(MacroTask::User(RuntimeMacroTask::RejectPromise(
                            root_value,
                            format!("Error: {error_msg}"),
                        )))
                        .unwrap();
                }
            }
        });

        Ok(Value::Promise(promise_capability.promise()).unbind())
    }
}
