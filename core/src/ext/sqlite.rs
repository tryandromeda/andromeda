use nova_vm::ecmascript::{
    builtins::ArgumentsList,
    execution::{Agent, JsResult},
    types::Value,
};
use sqlite::Connection;

use crate::{
    ext_interface::{Ext, ExtLoader},
    HostData,
};

struct SQliteExtResources {
    connection: Connection,
}
pub struct SQliteExt {
    pub connection: Connection,
}

impl Ext for SQliteExt {
    fn load(&self, mut loader: ExtLoader) {
        loader.init_storage(SQliteExtResources {
            connection: Connection::open(":memory:").unwrap(),
        });
        loader.load_op("internal_sqlite_execute", Self::internal_sqlite_execute, 1);
    }
}

impl SQliteExt {
    pub fn internal_sqlite_execute(
        agent: &mut Agent,
        _this: Value,
        _args: ArgumentsList,
    ) -> JsResult<Value> {
        let host_data: &HostData = agent.get_host_data().downcast_ref::<HostData>().unwrap();
        let resources = host_data.storage.borrow();
        let resources: &SQliteExtResources = resources.get::<SQliteExtResources>().unwrap();
        let _connection = &resources.connection;
        todo!("sqlite execute")
    }
}
