use std::{borrow::BorrowMut, fs::File};

use nova_vm::{
    ecmascript::{
        builtins::ArgumentsList,
        execution::{Agent, JsResult},
        types::Value,
    },
    SmallInteger,
};

use andromeda_core::{Extension, ExtensionOp, HostData, OpsStorage, ResourceTable};

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
    pub fn internal_read_text_file(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
    ) -> JsResult<Value> {
        let binding = args.get(0).to_string(agent)?;
        let path = binding.as_str(agent);
        let content = match std::fs::read_to_string(path) {
            Ok(content) => content,
            Err(e) => {
                return Ok(Value::from_string(agent, format!("Error: {}", e)));
            }
        };
        Ok(Value::from_string(agent, content))
    }

    /// Write a text file with the content of the second argument.
    pub fn internal_write_text_file(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
    ) -> JsResult<Value> {
        let binding = args.get(0).to_string(agent)?;
        let content = args.get(1).to_string(agent.borrow_mut())?;
        match std::fs::write(binding.as_str(agent), content.as_str(agent)) {
            Ok(_) => Ok(Value::from_string(agent, "Success".to_string())),
            Err(e) => Ok(Value::from_string(agent, format!("Error: {}", e))),
        }
    }

    /// Open a file and return a Rid.
    pub fn internal_open_file(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
    ) -> JsResult<Value> {
        let binding = args.get(0).to_string(agent)?;
        let path = binding.as_str(agent);
        let file = File::open(path).unwrap(); // TODO: Handle errors

        let host_data = agent.get_host_data();
        let host_data: &HostData<RuntimeMacroTask> = host_data.downcast_ref().unwrap();
        let storage = host_data.storage.borrow();
        let resources: &FsExtResources = storage.get().unwrap();
        let rid = resources.files.push(file);

        Ok(Value::Integer(SmallInteger::from(rid.index())))
    }
}
