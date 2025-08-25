// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::sync::Arc;

use andromeda_core::Rid;
use andromeda_core::{Extension, ExtensionOp, HostData, MacroTask, OpsStorage, ResourceTable};
use nova_vm::{
    ecmascript::{
        builtins::ArgumentsList,
        execution::{Agent, JsResult, agent::ExceptionType},
        types::IntoValue,
        types::Value,
    },
    engine::context::Bindable,
    engine::{Global, context::GcScope},
};

use rustls;
use rustls_pki_types::ServerName;
use std::convert::TryInto;
use std::sync::Arc as StdArc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::Mutex as TokioMutex;
use tokio_rustls::TlsConnector;
use tokio_rustls::client::TlsStream;

use crate::RuntimeMacroTask;

#[derive(Clone)]
pub(crate) enum TlsResource {
    Client(StdArc<TokioMutex<TlsStream<TcpStream>>>),
}

pub(crate) struct TlsResources {
    pub(crate) streams: ResourceTable<TlsResource>,
}

#[derive(Default)]
pub struct TlsExt;

impl TlsExt {
    pub fn new_extension() -> Extension {
        Extension {
            name: "tls",
            ops: vec![
                ExtensionOp::new("internal_tls_connect", Self::internal_tls_connect, 2),
                ExtensionOp::new("internal_tls_close", Self::internal_tls_close, 1),
                ExtensionOp::new("internal_tls_read", Self::internal_tls_read, 2),
                ExtensionOp::new("internal_tls_write", Self::internal_tls_write, 2),
                ExtensionOp::new(
                    "internal_tls_get_peer_certificate",
                    Self::internal_tls_get_peer_certificate,
                    1,
                ),
            ],
            storage: Some(Box::new(|storage: &mut OpsStorage| {
                storage.insert(TlsResources {
                    streams: ResourceTable::new(),
                });
            })),
            files: vec![],
        }
    }

    fn internal_tls_connect<'gc>(
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

        let promise_capability = nova_vm::ecmascript::builtins::promise_objects::promise_abstract_operations::promise_capability_records::PromiseCapability::new(agent, gc.nogc());
        let root_value = Global::new(agent, promise_capability.promise().into_value().unbind());

        let host_data = agent.get_host_data();
        let host_data: &HostData<RuntimeMacroTask> = host_data.downcast_ref().unwrap();
        let macro_task_tx = host_data.macro_task_tx();

        let host_clone = host.clone();
        let port_clone = port.clone();

        let domain: ServerName = match host_clone.clone().try_into() {
            Ok(d) => d,
            Err(_) => {
                macro_task_tx
                    .send(MacroTask::User(RuntimeMacroTask::RejectPromise(
                        root_value,
                        "Invalid DNS name".to_string(),
                    )))
                    .unwrap();
                return Ok(Value::Promise(promise_capability.promise()).unbind());
            }
        };

        host_data.spawn_macro_task(async move {
            let addr = format!("{host_clone}:{port_clone}");
            let connect_res = TcpStream::connect(&addr).await;
            match connect_res {
                Ok(tcp_stream) => {
                    let root_store = rustls::RootCertStore::from_iter(
                        webpki_roots::TLS_SERVER_ROOTS.iter().cloned(),
                    );

                    let config = rustls::ClientConfig::builder()
                        .with_root_certificates(root_store)
                        .with_no_client_auth();

                    let connector = TlsConnector::from(Arc::new(config));
                    let tls_res = connector.connect(domain, tcp_stream).await;
                    match tls_res {
                        Ok(tls_stream) => {
                            macro_task_tx
                                .send(MacroTask::User(RuntimeMacroTask::RegisterTlsStream(
                                    root_value,
                                    Box::new(tls_stream),
                                )))
                                .unwrap();
                        }
                        Err(e) => {
                            macro_task_tx
                                .send(MacroTask::User(RuntimeMacroTask::RejectPromise(
                                    root_value,
                                    format!("TLS handshake failed: {e}"),
                                )))
                                .unwrap();
                        }
                    }
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

    fn internal_tls_close<'gc>(
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
        let resources: &TlsResources = storage.get().unwrap();

        let rid = Rid::from_index(rid_val);
        resources.streams.remove(rid);

        drop(storage);

        Ok(Value::from_string(agent, "Success".to_string(), gc.nogc()).unbind())
    }

    fn internal_tls_read<'gc>(
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

        let promise_capability = nova_vm::ecmascript::builtins::promise_objects::promise_abstract_operations::promise_capability_records::PromiseCapability::new(agent, gc.nogc());
        let root_value = Global::new(agent, promise_capability.promise().into_value().unbind());

        let host_data = agent.get_host_data();
        let host_data: &HostData<RuntimeMacroTask> = host_data.downcast_ref().unwrap();
        let macro_task_tx = host_data.macro_task_tx();

        let storage = host_data.storage.borrow();
        let resources: &TlsResources = storage.get().unwrap();
        let stream_arc_opt = resources.streams.get_mut(rid).map(|r| {
            let TlsResource::Client(ref tls_stream_arc) = *r;
            tls_stream_arc.clone()
        });

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
                                    format!("TLS read error: {e}"),
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

    fn internal_tls_write<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let rid_binding = args.get(0).to_string(agent, gc.reborrow()).unbind()?;
        let data_binding = args.get(1).to_string(agent, gc.reborrow()).unbind()?;

        let rid_str = rid_binding
            .as_str(agent)
            .expect("String is not valid UTF-8")
            .to_string();
        let data_str = data_binding
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

        let promise_capability = nova_vm::ecmascript::builtins::promise_objects::promise_abstract_operations::promise_capability_records::PromiseCapability::new(agent, gc.nogc());
        let root_value = Global::new(agent, promise_capability.promise().into_value().unbind());

        let host_data = agent.get_host_data();
        let host_data: &HostData<RuntimeMacroTask> = host_data.downcast_ref().unwrap();
        let macro_task_tx = host_data.macro_task_tx();

        let storage = host_data.storage.borrow();
        let resources: &TlsResources = storage.get().unwrap();
        let stream_arc_opt = resources.streams.get_mut(rid).map(|r| {
            let TlsResource::Client(ref tls_stream_arc) = *r;
            tls_stream_arc.clone()
        });
        drop(storage);

        match stream_arc_opt {
            Some(stream_arc) => {
                let macro_task_tx = macro_task_tx.clone();
                host_data.spawn_macro_task(async move {
                    let mut guard = stream_arc.lock().await;
                    match guard.write_all(data_str.as_bytes()).await {
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
                                    format!("TLS write error: {e}"),
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

    fn internal_tls_get_peer_certificate<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let rid_binding = args.get(0).to_string(agent, gc.reborrow()).unbind()?;
        let rid_str = rid_binding
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

        let promise_capability = nova_vm::ecmascript::builtins::promise_objects::promise_abstract_operations::promise_capability_records::PromiseCapability::new(agent, gc.nogc());
        let root_value = Global::new(agent, promise_capability.promise().into_value().unbind());

        let host_data = agent.get_host_data();
        let host_data: &HostData<RuntimeMacroTask> = host_data.downcast_ref().unwrap();
        let macro_task_tx = host_data.macro_task_tx();

        let storage = host_data.storage.borrow();
        let resources: &TlsResources = storage.get().unwrap();
        let stream_arc_opt = resources.streams.get_mut(rid).map(|r| {
            let TlsResource::Client(ref tls_stream_arc) = *r;
            tls_stream_arc.clone()
        });
        drop(storage);

        match stream_arc_opt {
            Some(stream_arc) => {
                let macro_task_tx = macro_task_tx.clone();
                host_data.spawn_macro_task(async move {
                    let guard = stream_arc.lock().await;
                    let peer_certs = guard.get_ref().1.peer_certificates();
                    if let Some(certs) = peer_certs {
                        if !certs.is_empty() {
                            let cert_bytes: &[u8] = certs[0].as_ref();
                            let hex = cert_bytes.iter().fold(String::new(), |mut acc, &b: &u8| {
                                use std::fmt::Write;
                                write!(&mut acc, "{b:02x}").unwrap();
                                acc
                            });
                            macro_task_tx
                                .send(MacroTask::User(RuntimeMacroTask::ResolvePromiseWithString(
                                    root_value, hex,
                                )))
                                .unwrap();
                            return;
                        }
                    }
                    macro_task_tx
                        .send(MacroTask::User(RuntimeMacroTask::RejectPromise(
                            root_value,
                            "No peer certificate".to_string(),
                        )))
                        .unwrap();
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
