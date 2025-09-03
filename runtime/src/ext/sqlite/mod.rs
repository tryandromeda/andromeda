// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};

use andromeda_core::{Extension, ExtensionOp, HostData};
use nova_vm::{
    ecmascript::{
        builtins::{ArgumentsList, Array},
        execution::{Agent, JsResult},
        types::Value,
    },
    engine::context::{Bindable, GcScope},
};
use rusqlite::{Connection, OpenFlags, types::ToSql, types::ValueRef};

static DATABASE_ID_COUNTER: AtomicU32 = AtomicU32::new(1);
static STATEMENT_ID_COUNTER: AtomicU32 = AtomicU32::new(1);

#[derive(Debug)]
struct DatabaseConnection {
    connection: Arc<Mutex<Connection>>,
    #[allow(dead_code)]
    id: u32,
}

#[derive(Debug)]
struct PreparedStatement {
    sql: String,
    #[allow(dead_code)]
    id: u32,
    #[allow(dead_code)]
    db_id: u32,
    allow_bare_named_params: bool,
    read_bigints: bool,
}

type DatabaseMap = HashMap<u32, DatabaseConnection>;
type StatementMap = HashMap<u32, PreparedStatement>;

#[derive(Default)]
pub struct SqliteExt;

impl SqliteExt {
    pub fn new_extension() -> Extension {
        Extension {
            name: "sqlite",
            ops: vec![
                ExtensionOp::new(
                    "sqlite_database_sync_constructor",
                    Self::sqlite_database_sync_constructor,
                    2,
                    false,
                ),
                ExtensionOp::new(
                    "sqlite_database_sync_exec",
                    Self::sqlite_database_sync_exec,
                    2,
                    false,
                ),
                ExtensionOp::new(
                    "sqlite_database_sync_prepare",
                    Self::sqlite_database_sync_prepare,
                    2,
                    false,
                ),
                ExtensionOp::new(
                    "sqlite_database_sync_close",
                    Self::sqlite_database_sync_close,
                    1,
                    false,
                ),
                ExtensionOp::new(
                    "sqlite_statement_sync_all",
                    Self::sqlite_statement_sync_all,
                    3,
                    false,
                ),
                ExtensionOp::new(
                    "sqlite_statement_sync_get",
                    Self::sqlite_statement_sync_get,
                    3,
                    false,
                ),
                ExtensionOp::new(
                    "sqlite_statement_sync_run",
                    Self::sqlite_statement_sync_run,
                    3,
                    false,
                ),
                ExtensionOp::new(
                    "sqlite_statement_sync_expanded_sql",
                    Self::sqlite_statement_sync_expanded_sql,
                    1,
                    false,
                ),
                ExtensionOp::new(
                    "sqlite_statement_sync_source_sql",
                    Self::sqlite_statement_sync_source_sql,
                    1,
                    false,
                ),
                ExtensionOp::new(
                    "sqlite_statement_sync_iterate",
                    Self::sqlite_statement_sync_iterate,
                    3,
                    false,
                ),
                ExtensionOp::new(
                    "sqlite_statement_sync_set_allow_bare_named_parameters",
                    Self::sqlite_statement_sync_set_allow_bare_named_parameters,
                    2,
                    false,
                ),
                ExtensionOp::new(
                    "sqlite_statement_sync_set_read_bigints",
                    Self::sqlite_statement_sync_set_read_bigints,
                    2,
                    false,
                ),
                ExtensionOp::new(
                    "sqlite_statement_sync_finalize",
                    Self::sqlite_statement_sync_finalize,
                    1,
                    false,
                ),
                ExtensionOp::new(
                    "sqlite_database_sync_enable_load_extension",
                    Self::sqlite_database_sync_enable_load_extension,
                    2,
                    false,
                ),
                ExtensionOp::new(
                    "sqlite_database_sync_load_extension",
                    Self::sqlite_database_sync_load_extension,
                    3,
                    false,
                ),
                ExtensionOp::new(
                    "sqlite_database_sync_function",
                    Self::sqlite_database_sync_function,
                    4,
                    false,
                ),
                ExtensionOp::new(
                    "sqlite_database_sync_open",
                    Self::sqlite_database_sync_open,
                    3,
                    false,
                ),
            ],
            storage: None,
            files: vec![include_str!("sqlite.ts")],
        }
    }

    fn with_database<T, F>(agent: &mut Agent, db_id: u32, operation: F) -> Result<T, Value>
    where
        F: FnOnce(&Connection) -> Result<T, rusqlite::Error>,
    {
        let host_data = agent.get_host_data();
        let host_data: &HostData<crate::RuntimeMacroTask> = host_data.downcast_ref().unwrap();

        let storage = host_data.storage.borrow();
        let databases = storage.get::<DatabaseMap>().ok_or(Value::Undefined)?;
        let db = databases.get(&db_id).ok_or(Value::Undefined)?;

        let connection = db.connection.lock().unwrap();
        operation(&connection).map_err(|_| Value::Undefined)
    }

    // Convert SQLite value to JavaScript value
    #[allow(dead_code)]
    fn convert_sqlite_value_to_js<'gc>(
        agent: &mut Agent,
        value: rusqlite::types::Value,
        gc: GcScope<'gc, '_>,
    ) -> Value<'gc> {
        match value {
            rusqlite::types::Value::Null => Value::Null,
            rusqlite::types::Value::Integer(i) => {
                Value::from_f64(agent, i as f64, gc.nogc()).unbind()
            }
            rusqlite::types::Value::Real(f) => Value::from_f64(agent, f, gc.nogc()).unbind(),
            rusqlite::types::Value::Text(s) => Value::from_string(agent, s, gc.nogc()).unbind(),
            rusqlite::types::Value::Blob(b) => {
                let byte_values: Vec<Value> = b
                    .into_iter()
                    .map(|byte| Value::from_f64(agent, byte as f64, gc.nogc()).unbind())
                    .collect();
                let array = Array::from_slice(agent, &byte_values, gc.nogc()).unbind();
                array.into()
            }
        }
    }

    // Convert SQLite ValueRef to JavaScript value
    #[allow(dead_code)]
    fn convert_sqlite_valueref_to_js<'gc>(
        agent: &mut Agent,
        value: ValueRef,
        gc: GcScope<'gc, '_>,
    ) -> Value<'gc> {
        match value {
            ValueRef::Null => Value::Null,
            ValueRef::Integer(i) => Value::from_f64(agent, i as f64, gc.nogc()).unbind(),
            ValueRef::Real(f) => Value::from_f64(agent, f, gc.nogc()).unbind(),
            ValueRef::Text(s) => {
                let text = std::str::from_utf8(s).unwrap_or("");
                Value::from_string(agent, text.to_string(), gc.nogc()).unbind()
            }
            ValueRef::Blob(b) => {
                let byte_values: Vec<Value> = b
                    .iter()
                    .map(|byte| Value::from_f64(agent, *byte as f64, gc.nogc()).unbind())
                    .collect();
                let array = Array::from_slice(agent, &byte_values, gc.nogc()).unbind();
                array.into()
            }
        }
    }

    // Convert JavaScript values to SQLite parameters
    fn convert_js_params_to_sqlite<'gc>(
        agent: &mut Agent,
        params: &[Value<'gc>],
        mut gc: GcScope<'gc, '_>,
    ) -> Vec<Box<dyn ToSql>> {
        params
            .iter()
            .map(|value| -> Box<dyn ToSql> {
                match value {
                    Value::Null | Value::Undefined => Box::new(rusqlite::types::Null),
                    Value::Boolean(b) => Box::new(*b),
                    Value::Number(_n) => match value.to_number(agent, gc.reborrow()) {
                        Ok(num) => {
                            let f = num.unbind().into_f64(agent);
                            if f.fract() == 0.0 && f >= i64::MIN as f64 && f <= i64::MAX as f64 {
                                Box::new(f as i64)
                            } else {
                                Box::new(f)
                            }
                        }
                        Err(_) => Box::new(0i64),
                    },
                    Value::String(s) => Box::new(
                        s.as_str(agent)
                            .expect("String is not valid UTF-8")
                            .to_string(),
                    ),
                    Value::SmallString(s) => {
                        Box::new(s.as_str().expect("String is not valid UTF-8").to_string())
                    }
                    Value::BigInt(_b) => match value.to_string(agent, gc.reborrow()) {
                        Ok(s) => {
                            let str_val = s.as_str(agent).expect("String is not valid UTF-8");
                            match str_val.parse::<i64>() {
                                Ok(i) => Box::new(i),
                                Err(_) => Box::new(str_val.to_string()),
                            }
                        }
                        Err(_) => Box::new(String::from("0")),
                    },
                    _ => match value.to_string(agent, gc.reborrow()) {
                        Ok(s) => Box::new(
                            s.as_str(agent)
                                .expect("String is not valid UTF-8")
                                .to_string(),
                        ),
                        Err(_) => Box::new(String::from("[object Object]")),
                    },
                }
            })
            .collect()
    }

    pub fn sqlite_database_sync_constructor<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let filename = match args.get(0) {
            Value::String(s) => s
                .as_str(agent)
                .expect("String is not valid UTF-8")
                .to_string(),
            Value::SmallString(s) => s.as_str().expect("String is not valid UTF-8").to_string(),
            _ => {
                return Err(agent
                    .throw_exception_with_static_message(
                        nova_vm::ecmascript::execution::agent::ExceptionType::TypeError,
                        "Filename must be a string",
                        gc.nogc(),
                    )
                    .unbind());
            }
        };

        let flags = OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE;

        let connection = match Connection::open_with_flags(&filename, flags) {
            Ok(conn) => conn,
            Err(_e) => {
                return Err(agent
                    .throw_exception_with_static_message(
                        nova_vm::ecmascript::execution::agent::ExceptionType::Error,
                        "Failed to open database",
                        gc.nogc(),
                    )
                    .unbind());
            }
        };

        let db_id = DATABASE_ID_COUNTER.fetch_add(1, Ordering::SeqCst);
        let db = DatabaseConnection {
            connection: Arc::new(Mutex::new(connection)),
            id: db_id,
        };

        let host_data = agent.get_host_data();
        let host_data: &HostData<crate::RuntimeMacroTask> = host_data.downcast_ref().unwrap();

        if host_data.storage.borrow().get::<DatabaseMap>().is_none() {
            host_data.storage.borrow_mut().insert(DatabaseMap::new());
        }
        if host_data.storage.borrow().get::<StatementMap>().is_none() {
            host_data.storage.borrow_mut().insert(StatementMap::new());
        }

        host_data
            .storage
            .borrow_mut()
            .get_mut::<DatabaseMap>()
            .unwrap()
            .insert(db_id, db);

        Ok(Value::from_f64(agent, db_id as f64, gc.nogc()).unbind())
    }

    pub fn sqlite_database_sync_exec<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let db_id = match args.get(0).to_number(agent, gc.reborrow()) {
            Ok(num) => {
                let f = num.unbind().into_f64(agent);
                if f.is_finite() && f >= 0.0 {
                    f as u32
                } else {
                    return Err(agent
                        .throw_exception_with_static_message(
                            nova_vm::ecmascript::execution::agent::ExceptionType::TypeError,
                            "Invalid database ID",
                            gc.nogc(),
                        )
                        .unbind());
                }
            }
            Err(_) => {
                return Err(agent
                    .throw_exception_with_static_message(
                        nova_vm::ecmascript::execution::agent::ExceptionType::TypeError,
                        "Database ID must be a number",
                        gc.nogc(),
                    )
                    .unbind());
            }
        };

        let sql = match args.get(1) {
            Value::String(s) => s
                .as_str(agent)
                .expect("String is not valid UTF-8")
                .to_string(),
            Value::SmallString(s) => s.as_str().expect("String is not valid UTF-8").to_string(),
            _ => {
                return Err(agent
                    .throw_exception_with_static_message(
                        nova_vm::ecmascript::execution::agent::ExceptionType::TypeError,
                        "SQL must be a string",
                        gc.nogc(),
                    )
                    .unbind());
            }
        };

        match Self::with_database(agent, db_id, |conn| conn.execute_batch(&sql)) {
            Ok(_) => Ok(Value::Undefined.unbind()),
            Err(_) => Err(agent
                .throw_exception_with_static_message(
                    nova_vm::ecmascript::execution::agent::ExceptionType::Error,
                    "Failed to execute SQL",
                    gc.nogc(),
                )
                .unbind()),
        }
    }

    pub fn sqlite_database_sync_prepare<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let db_id = match args.get(0).to_number(agent, gc.reborrow()) {
            Ok(num) => num.unbind().into_f64(agent) as u32,
            Err(_) => {
                return Err(agent
                    .throw_exception_with_static_message(
                        nova_vm::ecmascript::execution::agent::ExceptionType::TypeError,
                        "Database ID must be a number",
                        gc.nogc(),
                    )
                    .unbind());
            }
        };

        let sql = match args.get(1) {
            Value::String(s) => s
                .as_str(agent)
                .expect("String is not valid UTF-8")
                .to_string(),
            Value::SmallString(s) => s.as_str().expect("String is not valid UTF-8").to_string(),
            _ => {
                return Err(agent
                    .throw_exception_with_static_message(
                        nova_vm::ecmascript::execution::agent::ExceptionType::TypeError,
                        "SQL must be a string",
                        gc.nogc(),
                    )
                    .unbind());
            }
        };

        let stmt_id = STATEMENT_ID_COUNTER.fetch_add(1, Ordering::SeqCst);
        let stmt = PreparedStatement {
            sql: sql.clone(),
            id: stmt_id,
            db_id,
            allow_bare_named_params: false,
            read_bigints: false,
        };

        let host_data = agent.get_host_data();
        let host_data: &HostData<crate::RuntimeMacroTask> = host_data.downcast_ref().unwrap();
        host_data
            .storage
            .borrow_mut()
            .get_mut::<StatementMap>()
            .unwrap()
            .insert(stmt_id, stmt);

        Ok(Value::from_f64(agent, stmt_id as f64, gc.nogc()).unbind())
    }

    pub fn sqlite_database_sync_close<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let db_id = match args.get(0).to_number(agent, gc.reborrow()) {
            Ok(num) => num.unbind().into_f64(agent) as u32,
            Err(_) => {
                return Err(agent
                    .throw_exception_with_static_message(
                        nova_vm::ecmascript::execution::agent::ExceptionType::TypeError,
                        "Database ID must be a number",
                        gc.nogc(),
                    )
                    .unbind());
            }
        };

        let host_data = agent.get_host_data();
        let host_data: &HostData<crate::RuntimeMacroTask> = host_data.downcast_ref().unwrap();
        host_data
            .storage
            .borrow_mut()
            .get_mut::<DatabaseMap>()
            .unwrap()
            .remove(&db_id);

        Ok(Value::Undefined.unbind())
    }

    pub fn sqlite_statement_sync_all<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let db_id = match args.get(0).to_number(agent, gc.reborrow()) {
            Ok(num) => num.unbind().into_f64(agent) as u32,
            Err(_) => {
                return Err(agent
                    .throw_exception_with_static_message(
                        nova_vm::ecmascript::execution::agent::ExceptionType::TypeError,
                        "Database ID must be a number",
                        gc.nogc(),
                    )
                    .unbind());
            }
        };

        let stmt_id = match args.get(1).to_number(agent, gc.reborrow()) {
            Ok(num) => num.unbind().into_f64(agent) as u32,
            Err(_) => {
                return Err(agent
                    .throw_exception_with_static_message(
                        nova_vm::ecmascript::execution::agent::ExceptionType::TypeError,
                        "Statement ID must be a number",
                        gc.nogc(),
                    )
                    .unbind());
            }
        };

        // Extract parameters from the third argument
        //TODO: add more sophisticated array handling later.
        let mut params = Vec::new();
        for i in 2..args.len() {
            params.push(args.get(i));
        }

        let sql = {
            let host_data = agent.get_host_data();
            let host_data: &HostData<crate::RuntimeMacroTask> = host_data.downcast_ref().unwrap();
            let storage = host_data.storage.borrow();
            let statements = storage.get::<StatementMap>().unwrap();
            statements.get(&stmt_id).map(|stmt| stmt.sql.clone())
        };

        let sql = match sql {
            Some(s) => s,
            None => {
                let array = Array::from_slice(agent, &[], gc.nogc()).unbind();
                return Ok(array.into());
            }
        };

        let sqlite_params = Self::convert_js_params_to_sqlite(agent, &params, gc.reborrow());

        match Self::with_database(agent, db_id, |conn| {
            let mut stmt = conn.prepare(&sql)?;
            let column_names: Vec<String> =
                stmt.column_names().iter().map(|s| s.to_string()).collect();
            let param_refs: Vec<&dyn ToSql> = sqlite_params.iter().map(|p| p.as_ref()).collect();

            let rows = stmt.query_map(&param_refs[..], |row| {
                let mut result = std::collections::HashMap::new();
                for (i, column_name) in column_names.iter().enumerate() {
                    let value = row.get_ref(i)?;
                    let json_value = match value {
                        ValueRef::Null => "null".to_string(),
                        ValueRef::Integer(i) => i.to_string(),
                        ValueRef::Real(f) => f.to_string(),
                        ValueRef::Text(s) => {
                            format!("\"{}\"", String::from_utf8_lossy(s).replace('"', "\\\""))
                        }
                        ValueRef::Blob(b) => format!("\"[Blob: {} bytes]\"", b.len()),
                    };
                    result.insert(column_name.clone(), json_value);
                }
                Ok(result)
            })?;

            let mut results = Vec::new();
            for row_result in rows {
                match row_result {
                    Ok(row_map) => {
                        let json_string = format!(
                            "{{{}}}",
                            row_map
                                .iter()
                                .map(|(k, v)| format!("\"{k}\": {v}"))
                                .collect::<Vec<_>>()
                                .join(", ")
                        );
                        results.push(json_string);
                    }
                    Err(_) => {
                        continue;
                    }
                }
            }
            Ok(results)
        }) {
            Ok(rows) => {
                let js_values: Vec<Value> = rows
                    .into_iter()
                    .map(|json_str| Value::from_string(agent, json_str, gc.nogc()).unbind())
                    .collect();

                let array = Array::from_slice(agent, &js_values, gc.nogc()).unbind();
                Ok(array.into())
            }
            Err(_) => {
                let array = Array::from_slice(agent, &[], gc.nogc()).unbind();
                Ok(array.into())
            }
        }
    }

    pub fn sqlite_statement_sync_get<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let db_id = match args.get(0).to_number(agent, gc.reborrow()) {
            Ok(num) => num.unbind().into_f64(agent) as u32,
            Err(_) => {
                return Err(agent
                    .throw_exception_with_static_message(
                        nova_vm::ecmascript::execution::agent::ExceptionType::TypeError,
                        "Database ID must be a number",
                        gc.nogc(),
                    )
                    .unbind());
            }
        };

        let stmt_id = match args.get(1).to_number(agent, gc.reborrow()) {
            Ok(num) => num.unbind().into_f64(agent) as u32,
            Err(_) => {
                return Err(agent
                    .throw_exception_with_static_message(
                        nova_vm::ecmascript::execution::agent::ExceptionType::TypeError,
                        "Statement ID must be a number",
                        gc.nogc(),
                    )
                    .unbind());
            }
        };

        let mut params = Vec::new();
        for i in 2..args.len() {
            params.push(args.get(i));
        }

        let sql = {
            let host_data = agent.get_host_data();
            let host_data: &HostData<crate::RuntimeMacroTask> = host_data.downcast_ref().unwrap();
            let storage = host_data.storage.borrow();
            let statements = storage.get::<StatementMap>().unwrap();

            if let Some(stmt) = statements.get(&stmt_id) {
                stmt.sql.clone()
            } else {
                return Ok(Value::Undefined.unbind());
            }
        };

        let sqlite_params = Self::convert_js_params_to_sqlite(agent, &params, gc.reborrow());

        match Self::with_database(agent, db_id, |conn| {
            let mut stmt = conn.prepare(&sql)?;
            let column_names: Vec<String> =
                stmt.column_names().iter().map(|s| s.to_string()).collect();

            let param_refs: Vec<&dyn ToSql> = sqlite_params.iter().map(|p| p.as_ref()).collect();
            let mut rows = stmt.query(&param_refs[..])?;

            if let Some(row) = rows.next()? {
                let mut result = std::collections::HashMap::new();
                for (i, column_name) in column_names.iter().enumerate() {
                    let value = row.get_ref(i)?;
                    let json_value = match value {
                        ValueRef::Null => "null".to_string(),
                        ValueRef::Integer(i) => i.to_string(),
                        ValueRef::Real(f) => f.to_string(),
                        ValueRef::Text(s) => {
                            format!("\"{}\"", String::from_utf8_lossy(s).replace('"', "\\\""))
                        }
                        ValueRef::Blob(b) => format!("\"[Blob: {} bytes]\"", b.len()),
                    };
                    result.insert(column_name.clone(), json_value);
                }

                let json_string = format!(
                    "{{{}}}",
                    result
                        .iter()
                        .map(|(k, v)| format!("\"{k}\": {v}"))
                        .collect::<Vec<_>>()
                        .join(", ")
                );
                Ok(Some(json_string))
            } else {
                Ok(None)
            }
        }) {
            Ok(Some(json_str)) => Ok(Value::from_string(agent, json_str, gc.nogc()).unbind()),
            Ok(None) => Ok(Value::Undefined.unbind()),
            Err(_) => Ok(Value::Undefined.unbind()),
        }
    }

    pub fn sqlite_statement_sync_run<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let db_id = match args.get(0).to_number(agent, gc.reborrow()) {
            Ok(num) => num.unbind().into_f64(agent) as u32,
            Err(_) => {
                return Err(agent
                    .throw_exception_with_static_message(
                        nova_vm::ecmascript::execution::agent::ExceptionType::TypeError,
                        "Database ID must be a number",
                        gc.nogc(),
                    )
                    .unbind());
            }
        };

        let stmt_id = match args.get(1).to_number(agent, gc.reborrow()) {
            Ok(num) => num.unbind().into_f64(agent) as u32,
            Err(_) => {
                return Err(agent
                    .throw_exception_with_static_message(
                        nova_vm::ecmascript::execution::agent::ExceptionType::TypeError,
                        "Statement ID must be a number",
                        gc.nogc(),
                    )
                    .unbind());
            }
        };

        let mut params = Vec::new();
        for i in 2..args.len() {
            params.push(args.get(i));
        }

        let sql = {
            let host_data = agent.get_host_data();
            let host_data: &HostData<crate::RuntimeMacroTask> = host_data.downcast_ref().unwrap();
            let storage = host_data.storage.borrow();
            let statements = storage.get::<StatementMap>().unwrap();
            statements.get(&stmt_id).map(|stmt| stmt.sql.clone())
        };

        let sql = match sql {
            Some(s) => s,
            None => {
                let result_json = r#"{"changes": 0, "lastInsertRowid": 0}"#.to_string();
                return Ok(Value::from_string(agent, result_json, gc.nogc()).unbind());
            }
        };

        let sqlite_params = Self::convert_js_params_to_sqlite(agent, &params, gc.reborrow());

        match Self::with_database(agent, db_id, |conn| {
            let param_refs: Vec<&dyn ToSql> = sqlite_params.iter().map(|p| p.as_ref()).collect();
            let affected_rows = conn.execute(&sql, &param_refs[..])?;
            let last_insert_rowid = conn.last_insert_rowid();
            Ok((affected_rows, last_insert_rowid))
        }) {
            Ok((changes, last_insert_rowid)) => {
                let result_json =
                    format!(r#"{{"changes": {changes}, "lastInsertRowid": {last_insert_rowid}}}"#);
                Ok(Value::from_string(agent, result_json, gc.nogc()).unbind())
            }
            Err(_) => {
                let result_json = r#"{"changes": 0, "lastInsertRowid": 0}"#.to_string();
                Ok(Value::from_string(agent, result_json, gc.nogc()).unbind())
            }
        }
    }

    pub fn sqlite_statement_sync_expanded_sql<'gc>(
        agent: &mut Agent,
        _this: Value,
        _args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        Ok(Value::from_string(agent, String::new(), gc.nogc()).unbind())
    }

    pub fn sqlite_statement_sync_source_sql<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let stmt_id = match args.get(0).to_number(agent, gc.reborrow()) {
            Ok(num) => num.unbind().into_f64(agent) as u32,
            Err(_) => {
                return Err(agent
                    .throw_exception_with_static_message(
                        nova_vm::ecmascript::execution::agent::ExceptionType::TypeError,
                        "Statement ID must be a number",
                        gc.nogc(),
                    )
                    .unbind());
            }
        };

        let host_data = agent.get_host_data();
        let host_data: &HostData<crate::RuntimeMacroTask> = host_data.downcast_ref().unwrap();
        let storage = host_data.storage.borrow();
        let statements = storage.get::<StatementMap>().unwrap();

        let sql = if let Some(stmt) = statements.get(&stmt_id) {
            stmt.sql.clone()
        } else {
            String::new()
        };

        drop(storage);

        Ok(Value::from_string(agent, sql, gc.nogc()).unbind())
    }

    pub fn sqlite_statement_sync_iterate<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let db_id = match args.get(0).to_number(agent, gc.reborrow()) {
            Ok(num) => num.unbind().into_f64(agent) as u32,
            Err(_) => {
                return Err(agent
                    .throw_exception_with_static_message(
                        nova_vm::ecmascript::execution::agent::ExceptionType::TypeError,
                        "Database ID must be a number",
                        gc.nogc(),
                    )
                    .unbind());
            }
        };

        let stmt_id = match args.get(1).to_number(agent, gc.reborrow()) {
            Ok(num) => num.unbind().into_f64(agent) as u32,
            Err(_) => {
                return Err(agent
                    .throw_exception_with_static_message(
                        nova_vm::ecmascript::execution::agent::ExceptionType::TypeError,
                        "Statement ID must be a number",
                        gc.nogc(),
                    )
                    .unbind());
            }
        };

        let mut params = Vec::new();
        for i in 2..args.len() {
            params.push(args.get(i));
        }

        let sql = {
            let host_data = agent.get_host_data();
            let host_data: &HostData<crate::RuntimeMacroTask> = host_data.downcast_ref().unwrap();
            let storage = host_data.storage.borrow();
            let statements = storage.get::<StatementMap>().unwrap();
            statements.get(&stmt_id).map(|stmt| stmt.sql.clone())
        };

        let sql = match sql {
            Some(s) => s,
            None => {
                let array = Array::from_slice(agent, &[], gc.nogc()).unbind();
                return Ok(array.into());
            }
        };

        let sqlite_params = Self::convert_js_params_to_sqlite(agent, &params, gc.reborrow());

        match Self::with_database(agent, db_id, |conn| {
            let mut stmt = conn.prepare(&sql)?;
            let column_names: Vec<String> =
                stmt.column_names().iter().map(|s| s.to_string()).collect();

            let param_refs: Vec<&dyn ToSql> = sqlite_params.iter().map(|p| p.as_ref()).collect();
            let rows = stmt.query_map(&param_refs[..], |row| {
                let mut result = std::collections::HashMap::new();
                for (i, column_name) in column_names.iter().enumerate() {
                    let value = row.get_ref(i)?;
                    let json_value = match value {
                        ValueRef::Null => "null".to_string(),
                        ValueRef::Integer(i) => i.to_string(),
                        ValueRef::Real(f) => f.to_string(),
                        ValueRef::Text(s) => {
                            format!("\"{}\"", String::from_utf8_lossy(s).replace('"', "\\\""))
                        }
                        ValueRef::Blob(b) => format!("\"[Blob: {} bytes]\"", b.len()),
                    };
                    result.insert(column_name.clone(), json_value);
                }
                Ok(result)
            })?;

            let mut results = Vec::new();
            for row_result in rows {
                match row_result {
                    Ok(row_map) => {
                        let json_string = format!(
                            "{{{}}}",
                            row_map
                                .iter()
                                .map(|(k, v)| format!("\"{k}\": {v}"))
                                .collect::<Vec<_>>()
                                .join(", ")
                        );
                        results.push(json_string);
                    }
                    Err(_) => continue,
                }
            }
            Ok(results)
        }) {
            Ok(rows) => {
                let js_values: Vec<Value> = rows
                    .into_iter()
                    .map(|json_str| Value::from_string(agent, json_str, gc.nogc()).unbind())
                    .collect();

                let array = Array::from_slice(agent, &js_values, gc.nogc()).unbind();
                Ok(array.into())
            }
            Err(_) => {
                let array = Array::from_slice(agent, &[], gc.nogc()).unbind();
                Ok(array.into())
            }
        }
    }

    pub fn sqlite_statement_sync_set_allow_bare_named_parameters<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let stmt_id = match args.get(0).to_number(agent, gc.reborrow()) {
            Ok(num) => num.unbind().into_f64(agent) as u32,
            Err(_) => {
                return Err(agent
                    .throw_exception_with_static_message(
                        nova_vm::ecmascript::execution::agent::ExceptionType::TypeError,
                        "Statement ID must be a number",
                        gc.nogc(),
                    )
                    .unbind());
            }
        };

        let allow_bare = match args.get(1) {
            Value::Boolean(b) => b,
            _ => false,
        };

        let host_data = agent.get_host_data();
        let host_data: &HostData<crate::RuntimeMacroTask> = host_data.downcast_ref().unwrap();
        if let Some(stmt) = host_data
            .storage
            .borrow_mut()
            .get_mut::<StatementMap>()
            .unwrap()
            .get_mut(&stmt_id)
        {
            stmt.allow_bare_named_params = allow_bare;
        }

        Ok(Value::Undefined.unbind())
    }

    pub fn sqlite_statement_sync_set_read_bigints<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let stmt_id = match args.get(0).to_number(agent, gc.reborrow()) {
            Ok(num) => num.unbind().into_f64(agent) as u32,
            Err(_) => {
                return Err(agent
                    .throw_exception_with_static_message(
                        nova_vm::ecmascript::execution::agent::ExceptionType::TypeError,
                        "Statement ID must be a number",
                        gc.nogc(),
                    )
                    .unbind());
            }
        };

        let read_bigints = match args.get(1) {
            Value::Boolean(b) => b,
            _ => false,
        };

        let host_data = agent.get_host_data();
        let host_data: &HostData<crate::RuntimeMacroTask> = host_data.downcast_ref().unwrap();
        if let Some(stmt) = host_data
            .storage
            .borrow_mut()
            .get_mut::<StatementMap>()
            .unwrap()
            .get_mut(&stmt_id)
        {
            stmt.read_bigints = read_bigints;
        }

        Ok(Value::Undefined.unbind())
    }

    pub fn sqlite_statement_sync_finalize<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let stmt_id = match args.get(0).to_number(agent, gc.reborrow()) {
            Ok(num) => num.unbind().into_f64(agent) as u32,
            Err(_) => {
                return Err(agent
                    .throw_exception_with_static_message(
                        nova_vm::ecmascript::execution::agent::ExceptionType::TypeError,
                        "Statement ID must be a number",
                        gc.nogc(),
                    )
                    .unbind());
            }
        };

        let host_data = agent.get_host_data();
        let host_data: &HostData<crate::RuntimeMacroTask> = host_data.downcast_ref().unwrap();
        host_data
            .storage
            .borrow_mut()
            .get_mut::<StatementMap>()
            .unwrap()
            .remove(&stmt_id);

        Ok(Value::Undefined.unbind())
    }

    pub fn sqlite_database_sync_enable_load_extension<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let db_id = match args.get(0).to_number(agent, gc.reborrow()) {
            Ok(num) => num.unbind().into_f64(agent) as u32,
            Err(_) => {
                return Err(agent
                    .throw_exception_with_static_message(
                        nova_vm::ecmascript::execution::agent::ExceptionType::TypeError,
                        "Database ID must be a number",
                        gc.nogc(),
                    )
                    .unbind());
            }
        };

        let enabled = match args.get(1) {
            Value::Boolean(b) => b,
            _ => false,
        };

        match Self::with_database(agent, db_id, |conn| {
            if enabled {
                unsafe { conn.load_extension_enable() }
            } else {
                conn.load_extension_disable()
            }
        }) {
            Ok(_) => Ok(Value::Undefined.unbind()),
            Err(_) => Err(agent
                .throw_exception_with_static_message(
                    nova_vm::ecmascript::execution::agent::ExceptionType::Error,
                    "Failed to enable/disable extension loading",
                    gc.nogc(),
                )
                .unbind()),
        }
    }

    pub fn sqlite_database_sync_load_extension<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let db_id = match args.get(0).to_number(agent, gc.reborrow()) {
            Ok(num) => num.unbind().into_f64(agent) as u32,
            Err(_) => {
                return Err(agent
                    .throw_exception_with_static_message(
                        nova_vm::ecmascript::execution::agent::ExceptionType::TypeError,
                        "Database ID must be a number",
                        gc.nogc(),
                    )
                    .unbind());
            }
        };

        let path = match args.get(1) {
            Value::String(s) => s
                .as_str(agent)
                .expect("String is not valid UTF-8")
                .to_string(),
            Value::SmallString(s) => s.as_str().expect("String is not valid UTF-8").to_string(),
            _ => {
                return Err(agent
                    .throw_exception_with_static_message(
                        nova_vm::ecmascript::execution::agent::ExceptionType::TypeError,
                        "Extension path must be a string",
                        gc.nogc(),
                    )
                    .unbind());
            }
        };

        let entry_point = match args.get(2) {
            Value::String(s) => Some(
                s.as_str(agent)
                    .expect("String is not valid UTF-8")
                    .to_string(),
            ),
            Value::SmallString(s) => {
                Some(s.as_str().expect("String is not valid UTF-8").to_string())
            }
            Value::Undefined | Value::Null => None,
            _ => {
                return Err(agent
                    .throw_exception_with_static_message(
                        nova_vm::ecmascript::execution::agent::ExceptionType::TypeError,
                        "Entry point must be a string or undefined",
                        gc.nogc(),
                    )
                    .unbind());
            }
        };

        match Self::with_database(agent, db_id, |conn| unsafe {
            conn.load_extension(&path, entry_point.as_deref())
        }) {
            Ok(_) => Ok(Value::Undefined.unbind()),
            Err(_) => Err(agent
                .throw_exception_with_static_message(
                    nova_vm::ecmascript::execution::agent::ExceptionType::Error,
                    "Failed to load extension",
                    gc.nogc(),
                )
                .unbind()),
        }
    }

    pub fn sqlite_database_sync_function<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let _db_id = match args.get(0).to_number(agent, gc.reborrow()) {
            Ok(num) => num.unbind().into_f64(agent) as u32,
            Err(_) => {
                return Err(agent
                    .throw_exception_with_static_message(
                        nova_vm::ecmascript::execution::agent::ExceptionType::TypeError,
                        "Database ID must be a number",
                        gc.nogc(),
                    )
                    .unbind());
            }
        };

        let _name = match args.get(1) {
            Value::String(s) => s
                .as_str(agent)
                .expect("String is not valid UTF-8")
                .to_string(),
            Value::SmallString(s) => s.as_str().expect("String is not valid UTF-8").to_string(),
            _ => {
                return Err(agent
                    .throw_exception_with_static_message(
                        nova_vm::ecmascript::execution::agent::ExceptionType::TypeError,
                        "Function name must be a string",
                        gc.nogc(),
                    )
                    .unbind());
            }
        };

        // For now, we'll just return undefined as implementing custom functions
        // would require a complex setup to store JS functions and call them from Rust
        // This would need a callback mechanism that's quite involved
        Ok(Value::Undefined.unbind())
    }

    pub fn sqlite_database_sync_open<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let db_id = match args.get(0).to_number(agent, gc.reborrow()) {
            Ok(num) => num.unbind().into_f64(agent) as u32,
            Err(_) => {
                return Err(agent
                    .throw_exception_with_static_message(
                        nova_vm::ecmascript::execution::agent::ExceptionType::TypeError,
                        "Database ID must be a number",
                        gc.nogc(),
                    )
                    .unbind());
            }
        };

        let filename = match args.get(1) {
            Value::String(s) => s
                .as_str(agent)
                .expect("String is not valid UTF-8")
                .to_string(),
            Value::SmallString(s) => s.as_str().expect("String is not valid UTF-8").to_string(),
            _ => {
                return Err(agent
                    .throw_exception_with_static_message(
                        nova_vm::ecmascript::execution::agent::ExceptionType::TypeError,
                        "Filename must be a string",
                        gc.nogc(),
                    )
                    .unbind());
            }
        };

        // Parse options (3rd argument)
        let read_only = false;
        if args.len() > 2 {
            if let Value::Object(_) = args.get(2) {
                // For now, we'll just assume read_only is false
                // A full implementation would parse the options object
            }
        }

        let flags = if read_only {
            OpenFlags::SQLITE_OPEN_READ_ONLY
        } else {
            OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE
        };

        let connection = match Connection::open_with_flags(&filename, flags) {
            Ok(conn) => conn,
            Err(_e) => {
                return Err(agent
                    .throw_exception_with_static_message(
                        nova_vm::ecmascript::execution::agent::ExceptionType::Error,
                        "Failed to open database",
                        gc.nogc(),
                    )
                    .unbind());
            }
        };

        // Replace the existing connection
        let host_data = agent.get_host_data();
        let host_data: &HostData<crate::RuntimeMacroTask> = host_data.downcast_ref().unwrap();
        if let Some(db) = host_data
            .storage
            .borrow_mut()
            .get_mut::<DatabaseMap>()
            .unwrap()
            .get_mut(&db_id)
        {
            db.connection = Arc::new(Mutex::new(connection));
        }

        Ok(Value::Undefined.unbind())
    }
}
