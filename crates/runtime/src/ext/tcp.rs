// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use andromeda_core::Rid;
use andromeda_core::{Extension, ExtensionOp, HostData, MacroTask, OpsStorage, ResourceTable};
use nova_vm::{
    ecmascript::{Agent, ArgumentsList, ExceptionType, JsResult, Value},
    engine::{Bindable, GcScope, Global},
};

use std::sync::Arc as StdArc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::Mutex as TokioMutex;

use crate::RuntimeMacroTask;

#[derive(Clone)]
pub(crate) enum TcpResource {
    Client(StdArc<TokioMutex<TcpStream>>),
}

pub(crate) struct TcpResources {
    pub(crate) streams: ResourceTable<TcpResource>,
}

#[derive(Default)]
pub struct TcpExt;

impl TcpExt {
    pub fn new_extension() -> Extension {
        Extension {
            name: "tcp_fetch",
            ops: vec![
                ExtensionOp::new("internal_tcp_connect", Self::internal_tcp_connect, 2, false),
                ExtensionOp::new("internal_tcp_close", Self::internal_tcp_close, 1, false),
                ExtensionOp::new("internal_tcp_read", Self::internal_tcp_read, 2, false),
                ExtensionOp::new(
                    "internal_tcp_write_bytes",
                    Self::internal_tcp_write_bytes,
                    3,
                    false,
                ),
            ],
            storage: Some(Box::new(|storage: &mut OpsStorage| {
                storage.insert(TcpResources {
                    streams: ResourceTable::new(),
                });
            })),
            files: vec![],
        }
    }

    fn internal_tcp_connect<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let host_binding = args.get(0).to_string(agent, gc.reborrow()).unbind()?;
        let port_binding = args.get(1).to_string(agent, gc.reborrow()).unbind()?;

        let host = host_binding
            .as_str(agent)
            .expect("String is not valid UTF-8")
            .to_string();
        let port = port_binding
            .as_str(agent)
            .expect("String is not valid UTF-8")
            .to_string();

        let promise_capability = nova_vm::ecmascript::PromiseCapability::new(agent, gc.nogc());
        let root_value = Global::new(agent, Value::from(promise_capability.promise()).unbind());

        let host_data = agent.get_host_data();
        let host_data: &HostData<RuntimeMacroTask> = host_data.downcast_ref().unwrap();
        let macro_task_tx = host_data.macro_task_tx();

        host_data.spawn_macro_task(async move {
            let addr = format!("{host}:{port}");
            match TcpStream::connect(&addr).await {
                Ok(stream) => {
                    macro_task_tx
                        .send(MacroTask::User(RuntimeMacroTask::RegisterTcpStream(
                            root_value,
                            Box::new(stream),
                        )))
                        .unwrap();
                }
                Err(e) => {
                    macro_task_tx
                        .send(MacroTask::User(RuntimeMacroTask::RejectPromise(
                            root_value,
                            format!("TCP connect failed: {e}"),
                        )))
                        .unwrap();
                }
            }
        });

        Ok(Value::Promise(promise_capability.promise()).unbind())
    }

    fn internal_tcp_close<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let rid_binding = args.get(0).to_string(agent, gc.reborrow()).unbind()?;
        let rid_str = rid_binding
            .as_str(agent)
            .expect("String is not valid UTF-8");

        let rid_val: u32 = match rid_str.parse() {
            Ok(v) => v,
            Err(_) => {
                return Err(agent
                    .throw_exception_with_static_message(
                        ExceptionType::Error,
                        "Invalid RID",
                        gc.nogc(),
                    )
                    .unbind());
            }
        };

        let host_data = agent.get_host_data();
        let host_data: &HostData<RuntimeMacroTask> = host_data.downcast_ref().unwrap();
        let storage = host_data.storage.borrow();
        let resources: &TcpResources = storage.get().unwrap();
        resources.streams.remove(Rid::from_index(rid_val));
        drop(storage);

        Ok(Value::from_string(agent, "Success".to_string(), gc.nogc()).unbind())
    }

    fn internal_tcp_read<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let rid_binding = args.get(0).to_string(agent, gc.reborrow()).unbind()?;
        let len_binding = args.get(1).to_string(agent, gc.reborrow()).unbind()?;

        let rid_str = rid_binding
            .as_str(agent)
            .expect("String is not valid UTF-8")
            .to_string();
        let len_str = len_binding
            .as_str(agent)
            .expect("String is not valid UTF-8")
            .to_string();
        let len: usize = match len_str.parse() {
            Ok(v) => v,
            Err(_) => {
                return Err(agent
                    .throw_exception_with_static_message(
                        ExceptionType::Error,
                        "Invalid length",
                        gc.nogc(),
                    )
                    .unbind());
            }
        };

        let rid_val: u32 = match rid_str.parse() {
            Ok(v) => v,
            Err(_) => {
                return Err(agent
                    .throw_exception_with_static_message(
                        ExceptionType::Error,
                        "Invalid RID",
                        gc.nogc(),
                    )
                    .unbind());
            }
        };
        let rid = Rid::from_index(rid_val);

        let promise_capability = nova_vm::ecmascript::PromiseCapability::new(agent, gc.nogc());
        let root_value = Global::new(agent, Value::from(promise_capability.promise()).unbind());

        let host_data = agent.get_host_data();
        let host_data: &HostData<RuntimeMacroTask> = host_data.downcast_ref().unwrap();
        let macro_task_tx = host_data.macro_task_tx();

        let storage = host_data.storage.borrow();
        let resources: &TcpResources = storage.get().unwrap();
        let stream_arc_opt = resources.streams.get_mut(rid).map(|r| {
            let TcpResource::Client(ref tcp_stream_arc) = *r;
            tcp_stream_arc.clone()
        });
        drop(storage);

        match stream_arc_opt {
            Some(stream_arc) => {
                host_data.spawn_macro_task(async move {
                    let mut buffer = vec![0u8; len];
                    let mut guard = stream_arc.lock().await;
                    match guard.read(&mut buffer).await {
                        Ok(n) => {
                            buffer.truncate(n);
                            macro_task_tx
                                .send(MacroTask::User(RuntimeMacroTask::ResolvePromiseWithBytes(
                                    root_value, buffer,
                                )))
                                .unwrap();
                        }
                        Err(e) => {
                            macro_task_tx
                                .send(MacroTask::User(RuntimeMacroTask::RejectPromise(
                                    root_value,
                                    format!("TCP read error: {e}"),
                                )))
                                .unwrap();
                        }
                    }
                });
            }
            None => {
                macro_task_tx
                    .send(MacroTask::User(RuntimeMacroTask::RejectPromise(
                        root_value,
                        "Resource not found or invalid type".to_string(),
                    )))
                    .unwrap();
            }
        }

        Ok(Value::Promise(promise_capability.promise()).unbind())
    }

    fn internal_tcp_write_bytes<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let rid_binding = args.get(0).to_string(agent, gc.reborrow()).unbind()?;
        let header_binding = args.get(1).to_string(agent, gc.reborrow()).unbind()?;
        let body_value = args.get(2);

        let rid_str = rid_binding
            .as_str(agent)
            .expect("String is not valid UTF-8")
            .to_string();
        let header_str = header_binding
            .as_str(agent)
            .expect("String is not valid UTF-8")
            .to_string();

        let rid_val: u32 = match rid_str.parse() {
            Ok(v) => v,
            Err(_) => {
                return Err(agent
                    .throw_exception_with_static_message(
                        ExceptionType::Error,
                        "Invalid RID",
                        gc.nogc(),
                    )
                    .unbind());
            }
        };
        let rid = Rid::from_index(rid_val);

        let body_bytes: Vec<u8> = {
            let is_empty_string = match body_value {
                Value::String(s) => s.as_str(agent).is_some_and(|s| s.is_empty()),
                Value::SmallString(s) => s.as_str().is_some_and(|s| s.is_empty()),
                _ => false,
            };
            if matches!(body_value, Value::Undefined | Value::Null) || is_empty_string {
                Vec::new()
            } else {
                match crate::ext::web::bytes::bytes_from_value(agent, body_value, gc.reborrow()) {
                    Ok(b) => b,
                    Err(msg) => {
                        return Err(agent
                            .throw_exception(
                                ExceptionType::Error,
                                format!("internal_tcp_write_bytes: body argument: {msg}"),
                                gc.nogc(),
                            )
                            .unbind());
                    }
                }
            }
        };

        let promise_capability = nova_vm::ecmascript::PromiseCapability::new(agent, gc.nogc());
        let root_value = Global::new(agent, Value::from(promise_capability.promise()).unbind());

        let host_data = agent.get_host_data();
        let host_data: &HostData<RuntimeMacroTask> = host_data.downcast_ref().unwrap();
        let macro_task_tx = host_data.macro_task_tx();

        let storage = host_data.storage.borrow();
        let resources: &TcpResources = storage.get().unwrap();
        let stream_arc_opt = resources.streams.get_mut(rid).map(|r| {
            let TcpResource::Client(ref tcp_stream_arc) = *r;
            tcp_stream_arc.clone()
        });
        drop(storage);

        match stream_arc_opt {
            Some(stream_arc) => {
                let macro_task_tx = macro_task_tx.clone();
                host_data.spawn_macro_task(async move {
                    let mut guard = stream_arc.lock().await;
                    let write_res: std::io::Result<()> = async {
                        guard.write_all(header_str.as_bytes()).await?;
                        if !body_bytes.is_empty() {
                            guard.write_all(&body_bytes).await?;
                        }
                        Ok(())
                    }
                    .await;
                    match write_res {
                        Ok(_) => {
                            macro_task_tx
                                .send(MacroTask::User(RuntimeMacroTask::ResolvePromiseWithString(
                                    root_value,
                                    "Success".to_string(),
                                )))
                                .unwrap();
                        }
                        Err(e) => {
                            macro_task_tx
                                .send(MacroTask::User(RuntimeMacroTask::RejectPromise(
                                    root_value,
                                    format!("TCP write error: {e}"),
                                )))
                                .unwrap();
                        }
                    }
                });
            }
            None => {
                macro_task_tx
                    .send(MacroTask::User(RuntimeMacroTask::RejectPromise(
                        root_value,
                        "Resource not found or invalid type".to_string(),
                    )))
                    .unwrap();
            }
        }

        Ok(Value::Promise(promise_capability.promise()).unbind())
    }
}
