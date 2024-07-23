use std::{borrow::BorrowMut, cell::RefCell, fs::File};

use anymap::AnyMap;
use nova_vm::{
    ecmascript::{
        builtins::ArgumentsList,
        execution::{Agent, JsResult},
        types::Value,
    },
    SmallInteger,
};

use crate::{
    ext_interface::{Ext, ExtLoader},
    resource_table::ResourceTable,
};

struct FsExtResources {
    files: ResourceTable<File>,
}

#[derive(Default)]
pub struct FsExt;

impl Ext for FsExt {
    fn load(&self, mut loader: ExtLoader) {
        loader.init_storage(FsExtResources {
            files: ResourceTable::<File>::new(),
        });

        loader.load_op("internal_read_text_file", Self::internal_read_text_file, 1);
        loader.load_op(
            "internal_write_text_file",
            Self::internal_write_text_file,
            2,
        );
        loader.load_op("internal_open_file", Self::internal_open_file, 1);
    }
}

impl FsExt {
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
                return Ok(Value::from_string(
                    agent,
                    format!("Error: {}", e.to_string()),
                ));
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
            Err(e) => Ok(Value::from_string(
                agent,
                format!("Error: {}", e.to_string()),
            )),
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
        let storage: &RefCell<AnyMap> = host_data.downcast_ref().unwrap();
        let storage = storage.borrow();
        let resources: &FsExtResources = storage.get().unwrap();
        let rid = resources.files.push(file);

        Ok(Value::Integer(SmallInteger::from(rid.index())))
    }
}
