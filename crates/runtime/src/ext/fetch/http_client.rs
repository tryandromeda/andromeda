// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::sync::Arc;

use andromeda_core::{HostData, MacroTask, Rid, ResourceTable};
use nova_vm::{
    ecmascript::{Agent, ArgumentsList, ExceptionType, JsResult, Value},
    engine::{Bindable, GcScope, Global},
};
use reqwest::Response;
use tokio::sync::Mutex as TokioMutex;

use crate::RuntimeMacroTask;

pub(crate) struct HttpClientResources {
    pub(crate) client: reqwest::Client,
    pub(crate) bodies: ResourceTable<Arc<TokioMutex<Response>>>,
}

impl HttpClientResources {
    pub(crate) fn new() -> Self {
        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .expect("Failed to build reqwest client");
        Self {
            client,
            bodies: ResourceTable::new(),
        }
    }
}

pub(crate) fn internal_http_request<'gc>(
    agent: &mut Agent,
    _this: Value,
    args: ArgumentsList,
    mut gc: GcScope<'gc, '_>,
) -> JsResult<'gc, Value<'gc>> {
    let method_binding = args.get(0).to_string(agent, gc.reborrow()).unbind()?;
    let url_binding = args.get(1).to_string(agent, gc.reborrow()).unbind()?;
    let headers_binding = args.get(2).to_string(agent, gc.reborrow()).unbind()?;

    let method_str = method_binding
        .as_str(agent)
        .expect("non-UTF-8")
        .to_string();
    let url_str = url_binding.as_str(agent).expect("non-UTF-8").to_string();
    let headers_str = headers_binding
        .as_str(agent)
        .expect("non-UTF-8")
        .to_string();

    // Body: undefined/null/empty-string → no body; otherwise bytes.
    let body_value = args.get(3);
    let body_bytes: Option<Vec<u8>> = {
        let is_empty = match body_value {
            Value::String(s) => s.as_str(agent).is_some_and(|s| s.is_empty()),
            Value::SmallString(s) => s.as_str().is_some_and(|s| s.is_empty()),
            _ => false,
        };
        if matches!(body_value, Value::Undefined | Value::Null) || is_empty {
            None
        } else {
            match crate::ext::web::bytes::bytes_from_value(agent, body_value, gc.reborrow()) {
                Ok(b) => Some(b),
                Err(msg) => {
                    return Err(agent
                        .throw_exception(
                            ExceptionType::Error,
                            format!("internal_http_request: body: {msg}"),
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
    let resources: &HttpClientResources = storage.get().unwrap();
    let client = resources.client.clone();
    drop(storage);

    host_data.spawn_macro_task(async move {
        let method = match method_str.to_uppercase().as_str() {
            "GET" => reqwest::Method::GET,
            "POST" => reqwest::Method::POST,
            "PUT" => reqwest::Method::PUT,
            "DELETE" => reqwest::Method::DELETE,
            "HEAD" => reqwest::Method::HEAD,
            "OPTIONS" => reqwest::Method::OPTIONS,
            "PATCH" => reqwest::Method::PATCH,
            "CONNECT" => reqwest::Method::CONNECT,
            "TRACE" => reqwest::Method::TRACE,
            other => match reqwest::Method::from_bytes(other.as_bytes()) {
                Ok(m) => m,
                Err(_) => {
                    let _ = macro_task_tx.send(MacroTask::User(
                        RuntimeMacroTask::RejectPromise(
                            root_value,
                            format!("Invalid HTTP method: {other}"),
                        ),
                    ));
                    return;
                }
            },
        };

        // Parse and set request headers.
        let headers_parsed: Vec<[String; 2]> =
            match serde_json::from_str::<Vec<[String; 2]>>(&headers_str) {
                Ok(h) => h,
                Err(e) => {
                    let _ = macro_task_tx.send(MacroTask::User(
                        RuntimeMacroTask::RejectPromise(
                            root_value,
                            format!("Invalid headers JSON: {e}"),
                        ),
                    ));
                    return;
                }
            };

        let mut req_builder = client.request(method, &url_str);
        for [name, value] in &headers_parsed {
            req_builder = req_builder.header(name.as_str(), value.as_str());
        }

        if let Some(body) = body_bytes {
            req_builder = req_builder.body(body);
        }

        match req_builder.send().await {
            Ok(response) => {
                let _ = macro_task_tx.send(MacroTask::User(
                    RuntimeMacroTask::RegisterHttpResponse(root_value, Box::new(response)),
                ));
            }
            Err(e) => {
                let _ = macro_task_tx.send(MacroTask::User(RuntimeMacroTask::RejectPromise(
                    root_value,
                    format!("HTTP request failed: {e}"),
                )));
            }
        }
    });

    Ok(Value::Promise(promise_capability.promise()).unbind())
}

pub(crate) fn internal_http_response_read<'gc>(
    agent: &mut Agent,
    _this: Value,
    args: ArgumentsList,
    mut gc: GcScope<'gc, '_>,
) -> JsResult<'gc, Value<'gc>> {
    let rid_binding = args.get(0).to_string(agent, gc.reborrow()).unbind()?;
    let rid_str = rid_binding
        .as_str(agent)
        .expect("non-UTF-8")
        .to_string();
    let rid_val: u32 = match rid_str.parse() {
        Ok(v) => v,
        Err(_) => {
            return Err(agent
                .throw_exception_with_static_message(ExceptionType::Error, "Invalid RID", gc.nogc())
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
    let resources: &HttpClientResources = storage.get().unwrap();
    let arc_opt = resources.bodies.get(rid);
    drop(storage);

    match arc_opt {
        Some(arc) => {
            host_data.spawn_macro_task(async move {
                let mut guard = arc.lock().await;
                match guard.chunk().await {
                    Ok(Some(bytes)) => {
                        let _ = macro_task_tx.send(MacroTask::User(
                            RuntimeMacroTask::ResolvePromiseWithBytes(
                                root_value,
                                bytes.to_vec(),
                            ),
                        ));
                    }
                    Ok(None) => {
                        // EOF — empty vec signals end-of-body to JS.
                        let _ = macro_task_tx.send(MacroTask::User(
                            RuntimeMacroTask::ResolvePromiseWithBytes(root_value, Vec::new()),
                        ));
                    }
                    Err(e) => {
                        let _ = macro_task_tx.send(MacroTask::User(
                            RuntimeMacroTask::RejectPromise(
                                root_value,
                                format!("HTTP response read error: {e}"),
                            ),
                        ));
                    }
                }
            });
        }
        None => {
            let _ = macro_task_tx.send(MacroTask::User(RuntimeMacroTask::RejectPromise(
                root_value,
                "HTTP response resource not found".to_string(),
            )));
        }
    }

    Ok(Value::Promise(promise_capability.promise()).unbind())
}

pub(crate) fn internal_http_response_close<'gc>(
    agent: &mut Agent,
    _this: Value,
    args: ArgumentsList,
    mut gc: GcScope<'gc, '_>,
) -> JsResult<'gc, Value<'gc>> {
    let rid_binding = args.get(0).to_string(agent, gc.reborrow()).unbind()?;
    let rid_str = rid_binding.as_str(agent).expect("non-UTF-8");
    let rid_val: u32 = match rid_str.parse() {
        Ok(v) => v,
        Err(_) => {
            return Err(agent
                .throw_exception_with_static_message(ExceptionType::Error, "Invalid RID", gc.nogc())
                .unbind());
        }
    };
    let rid = Rid::from_index(rid_val);

    let host_data = agent.get_host_data();
    let host_data: &HostData<RuntimeMacroTask> = host_data.downcast_ref().unwrap();
    let storage = host_data.storage.borrow();
    let resources: &HttpClientResources = storage.get().unwrap();
    resources.bodies.remove(rid);
    drop(storage);

    Ok(Value::Undefined)
}
