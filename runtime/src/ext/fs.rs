// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::{borrow::BorrowMut, fs::File};

use nova_vm::{
    SmallInteger,
    ecmascript::{
        builtins::ArgumentsList,
        execution::{Agent, JsResult},
        types::Value,
    },
    engine::context::{Bindable, GcScope},
};

use andromeda_core::{
    AndromedaError, ErrorReporter, Extension, ExtensionOp, HostData, OpsStorage, ResourceTable,
};

use crate::RuntimeMacroTask;

struct FsExtResources {
    files: ResourceTable<File>,
}

#[derive(Default)]
pub struct FsExt;

impl FsExt {
    pub fn new_extension() -> Extension {
        Extension {
            name: "fs",
            ops: vec![
                ExtensionOp::new("internal_read_text_file", Self::internal_read_text_file, 1),
                ExtensionOp::new(
                    "internal_write_text_file",
                    Self::internal_write_text_file,
                    2,
                ),
                ExtensionOp::new("internal_create_file", Self::internal_create_file, 1),
                ExtensionOp::new("internal_copy_file", Self::internal_copy_file, 2),
                ExtensionOp::new("internal_mk_dir", Self::internal_mk_dir, 1),
                ExtensionOp::new("internal_open_file", Self::internal_open_file, 1),
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
        let path = binding.as_str(agent);

        match std::fs::read_to_string(path) {
            Ok(content) => Ok(Value::from_string(agent, content, gc.nogc()).unbind()),
            Err(e) => {
                let error = AndromedaError::fs_error(e, "read_text_file", path);
                let error_msg = ErrorReporter::format_error(&error);
                Ok(Value::from_string(agent, format!("Error: {}", error_msg), gc.nogc()).unbind())
            }
        }
    } // /// Write a text file with the content of the second argument.
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

        match std::fs::write(binding.as_str(agent), content.as_str(agent)) {
            Ok(_) => Ok(Value::from_string(agent, "Success".to_string(), gc.nogc()).unbind()),
            Err(e) => {
                let error = AndromedaError::fs_error(e, "write_text_file", binding.as_str(agent));
                let error_msg = ErrorReporter::format_error(&error);
                Ok(Value::from_string(agent, format!("Error: {}", error_msg), gc.nogc()).unbind())
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
        let path = binding.as_str(agent);

        let file = match File::create(path) {
            Ok(file) => file,
            Err(e) => {
                let error = AndromedaError::fs_error(e, "create_file", path);
                let error_msg = ErrorReporter::format_error(&error);
                return Ok(
                    Value::from_string(agent, format!("Error: {}", error_msg), gc.nogc()).unbind(),
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

        match std::fs::copy(from.as_str(agent), to.as_str(agent)) {
            Ok(_) => Ok(Value::from_string(agent, "Success".to_string(), gc.nogc()).unbind()),
            Err(e) => {
                let error = AndromedaError::fs_error(
                    e,
                    "copy_file",
                    format!("{} -> {}", from.as_str(agent), to.as_str(agent)),
                );
                let error_msg = ErrorReporter::format_error(&error);
                Ok(Value::from_string(agent, format!("Error: {}", error_msg), gc.nogc()).unbind())
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
        let path = binding.as_str(agent);

        match std::fs::create_dir(path) {
            Ok(_) => Ok(Value::from_string(agent, "Success".to_string(), gc.nogc()).unbind()),
            Err(e) => {
                let error = AndromedaError::fs_error(e, "create_directory", path);
                let error_msg = ErrorReporter::format_error(&error);
                Ok(Value::from_string(agent, format!("Error: {}", error_msg), gc.nogc()).unbind())
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
        let path = binding.as_str(agent);

        let file = match File::open(path) {
            Ok(file) => file,
            Err(e) => {
                let error = AndromedaError::fs_error(e, "open_file", path);
                let error_msg = ErrorReporter::format_error(&error);
                return Ok(
                    Value::from_string(agent, format!("Error: {}", error_msg), gc.nogc()).unbind(),
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
}
