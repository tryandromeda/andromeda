// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
#![allow(dead_code)]
mod address;
mod dns;
mod error;
mod tcp;
mod udp;
mod unix;

pub use address::*;
pub use dns::*;
pub use error::*;
pub use tcp::*;
pub use udp::*;
pub use unix::*;

use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream, UdpSocket};
use tokio::sync::Mutex;

use nova_vm::{
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
    Extension, ExtensionOp, HostData, MacroTask, ResourceTable, Rid, SyncResourceTable,
};

use crate::RuntimeMacroTask;

/// Resource types for network operations
#[derive(Clone)]
pub struct TcpStreamResource {
    pub stream: Arc<Mutex<TcpStream>>,
    pub local_addr: SocketAddr,
    pub remote_addr: SocketAddr,
}

#[derive(Clone)]
pub struct TcpListenerResource {
    pub listener: Arc<Mutex<TcpListener>>,
    pub local_addr: SocketAddr,
}

#[derive(Clone)]
pub struct UdpSocketResource {
    pub socket: Arc<Mutex<UdpSocket>>,
    pub local_addr: SocketAddr,
}

#[cfg(unix)]
#[derive(Clone)]
pub struct UnixStreamResource {
    pub stream: Arc<Mutex<tokio::net::UnixStream>>,
    pub local_addr: Option<String>,
    pub remote_addr: Option<String>,
}

#[cfg(unix)]
#[derive(Clone)]
pub struct UnixListenerResource {
    pub listener: Arc<Mutex<tokio::net::UnixListener>>,
    pub local_addr: Option<String>,
}

#[cfg(unix)]
#[derive(Clone)]
pub struct UnixDatagramResource {
    pub socket: Arc<Mutex<tokio::net::UnixDatagram>>,
    pub local_addr: Option<String>,
}

/// Extension storage for network resources
struct NetExtResources {
    tcp_streams: SyncResourceTable<TcpStreamResource>,
    tcp_listeners: ResourceTable<TcpListenerResource>,
    udp_sockets: ResourceTable<UdpSocketResource>,
    #[cfg(unix)]
    unix_streams: ResourceTable<UnixStreamResource>,
    #[cfg(unix)]
    unix_listeners: ResourceTable<UnixListenerResource>,
    #[cfg(unix)]
    unix_datagrams: ResourceTable<UnixDatagramResource>,
}

#[derive(Default)]
pub struct NetExt;

#[cfg_attr(feature = "hotpath", hotpath::measure_all)]
impl NetExt {
    pub fn new_extension() -> Extension {
        Extension {
            name: "andromeda:net",
            ops: vec![
                ExtensionOp::new("tcp_connect", Self::internal_tcp_connect, 2, false),
                ExtensionOp::new(
                    "tcp_connect_async",
                    Self::internal_tcp_connect_async,
                    2,
                    true,
                ),
                ExtensionOp::new("tcp_listen", Self::internal_tcp_listen, 2, false),
                ExtensionOp::new("tcp_accept", Self::internal_tcp_accept, 1, false),
                ExtensionOp::new(
                    "tcp_accept_async",
                    Self::internal_tcp_accept_async,
                    1,
                    false,
                ),
                ExtensionOp::new("tcp_read", Self::internal_tcp_read, 2, false),
                ExtensionOp::new("tcp_read_async", Self::internal_tcp_read_async, 2, false),
                ExtensionOp::new("tcp_write", Self::internal_tcp_write, 2, false),
                ExtensionOp::new("tcp_write_async", Self::internal_tcp_write_async, 2, false),
                ExtensionOp::new("tcp_close", Self::internal_tcp_close, 1, false),
                ExtensionOp::new("tcp_set_nodelay", Self::internal_tcp_set_nodelay, 2, false),
                ExtensionOp::new(
                    "tcp_set_keepalive",
                    Self::internal_tcp_set_keepalive,
                    2,
                    false,
                ),
                ExtensionOp::new("udp_bind", Self::internal_udp_bind, 2, false),
                ExtensionOp::new("udp_send", Self::internal_udp_send, 3, false),
                ExtensionOp::new("udp_send_async", Self::internal_udp_send_async, 3, false),
                ExtensionOp::new("udp_receive", Self::internal_udp_receive, 2, false),
                ExtensionOp::new(
                    "udp_receive_async",
                    Self::internal_udp_receive_async,
                    2,
                    true,
                ),
                ExtensionOp::new("udp_close", Self::internal_udp_close, 1, false),
                ExtensionOp::new(
                    "udp_set_broadcast",
                    Self::internal_udp_set_broadcast,
                    2,
                    false,
                ),
                ExtensionOp::new(
                    "udp_set_multicast_ttl",
                    Self::internal_udp_set_multicast_ttl,
                    2,
                    false,
                ),
                ExtensionOp::new(
                    "udp_set_multicast_loop",
                    Self::internal_udp_set_multicast_loop,
                    2,
                    false,
                ),
                ExtensionOp::new(
                    "udp_join_multicast_v4",
                    Self::internal_udp_join_multicast_v4,
                    3,
                    false,
                ),
                ExtensionOp::new(
                    "udp_join_multicast_v6",
                    Self::internal_udp_join_multicast_v6,
                    3,
                    false,
                ),
                ExtensionOp::new(
                    "udp_leave_multicast_v4",
                    Self::internal_udp_leave_multicast_v4,
                    3,
                    false,
                ),
                ExtensionOp::new(
                    "udp_leave_multicast_v6",
                    Self::internal_udp_leave_multicast_v6,
                    3,
                    false,
                ),
                ExtensionOp::new("dns_resolve", Self::internal_dns_resolve, 2, false),
                ExtensionOp::new(
                    "dns_resolve_async",
                    Self::internal_dns_resolve_async,
                    2,
                    true,
                ),
            ],
            storage: Some(Box::new(|storage| {
                storage.insert(NetExtResources {
                    tcp_streams: SyncResourceTable::new(),
                    tcp_listeners: ResourceTable::new(),
                    udp_sockets: ResourceTable::new(),
                    #[cfg(unix)]
                    unix_streams: ResourceTable::new(),
                    #[cfg(unix)]
                    unix_listeners: ResourceTable::new(),
                    #[cfg(unix)]
                    unix_datagrams: ResourceTable::new(),
                });
            })),
            files: vec![],
        }
    }

    /// TCP connect operation (synchronous)
    pub fn internal_tcp_connect<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let host_binding = args.get(0).to_string(agent, gc.reborrow()).unbind()?;
        let port_binding = args.get(1).to_string(agent, gc.reborrow()).unbind()?;

        let _host = host_binding
            .as_str(agent)
            .expect("String is not valid UTF-8");
        let port_str = port_binding
            .as_str(agent)
            .expect("String is not valid UTF-8");

        let _port: u16 = match port_str.parse() {
            Ok(p) => p,
            Err(_) => {
                return Ok(Value::from_string(
                    agent,
                    "Error: Invalid port number".to_string(),
                    gc.nogc(),
                )
                .unbind());
            }
        };

        // TODO: proper blocking I/O handling
        Ok(Value::from_string(
            agent,
            "Error: Use async version for TCP operations".to_string(),
            gc.nogc(),
        )
        .unbind())
    }

    /// TCP connect operation (asynchronous)
    pub fn internal_tcp_connect_async<'gc>(
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
        let port_str = port_binding
            .as_str(agent)
            .expect("String is not valid UTF-8");

        let port: u16 = match port_str.parse() {
            Ok(p) => p,
            Err(_) => {
                return Ok(Value::from_string(
                    agent,
                    "Error: Invalid port number".to_string(),
                    gc.nogc(),
                )
                .unbind());
            }
        };

        let promise_capability = PromiseCapability::new(agent, gc.nogc());
        let root_value = Global::new(agent, promise_capability.promise().into_value().unbind());
        let host_data = agent.get_host_data();
        let host_data: &HostData<RuntimeMacroTask> = host_data.downcast_ref().unwrap();
        let macro_task_tx = host_data.macro_task_tx();

        host_data.spawn_macro_task(async move {
            let addr = NetAddr::new(host, port);
            match TcpOps::connect(&addr).await {
                Ok(stream_wrapper) => {
                    let local_addr = stream_wrapper.local_addr();
                    let remote_addr = stream_wrapper.remote_addr();
                    // Store stream in resource table and return resource ID
                    let _resource = TcpStreamResource {
                        stream: stream_wrapper.stream,
                        local_addr,
                        remote_addr,
                    };

                    // TODO: get actual resource ID
                    let resource_id = 1;
                    let result = format!(
                        "{{\"success\":true,\"localAddr\":\"{}\",\"remoteAddr\":\"{}\",\"resourceId\":{}}}",
                        local_addr, remote_addr, resource_id
                    );
                    macro_task_tx
                        .send(MacroTask::User(RuntimeMacroTask::ResolvePromiseWithString(
                            root_value,
                            result,
                        )))
                        .unwrap();
                }
                Err(e) => {
                    macro_task_tx
                        .send(MacroTask::User(RuntimeMacroTask::RejectPromise(
                            root_value,
                            format!("Error: {}", e),
                        )))
                        .unwrap();
                }
            }
        });

        Ok(Value::Promise(promise_capability.promise()).unbind())
    }

    /// TCP listen operation (synchronous)
    pub fn internal_tcp_listen<'gc>(
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
        let port_str = port_binding
            .as_str(agent)
            .expect("String is not valid UTF-8");

        let port: u16 = match port_str.parse() {
            Ok(p) => p,
            Err(_) => {
                return Ok(Value::from_string(
                    agent,
                    "Error: Invalid port number".to_string(),
                    gc.nogc(),
                )
                .unbind());
            }
        };

        // Use block_in_place for synchronous TCP listen operation
        let addr = NetAddr::new(host, port);
        let result = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                match TcpOps::listen(&addr).await {
                    Ok(listener_wrapper) => {
                        let local_addr = listener_wrapper.local_addr();
                        // Store listener in resource table and return resource ID
                        let resource = TcpListenerResource {
                            listener: listener_wrapper.listener,
                            local_addr,
                        };
                        // For synchronous operations, store in resource table
                        match Self::get_net_resources_mut(agent) {
                            #[allow(unused_mut)]
                            Ok(mut resources) => {
                                let resource_id = resources.tcp_listeners.push(resource);
                                format!("{{\"success\":true,\"localAddr\":\"{}\",\"resourceId\":{}}}", local_addr, resource_id.index())
                            }
                            Err(e) => {
                                format!("{{\"success\":false,\"error\":\"Failed to store resource: {}\"}}", e)
                            }
                        }
                    }
                    Err(e) => {
                        format!("{{\"success\":false,\"error\":\"{}\"}}", e)
                    }
                }
            })
        });

        Ok(Value::from_string(agent, result, gc.nogc()).unbind())
    }

    /// TCP accept operation (synchronous)
    pub fn internal_tcp_accept<'gc>(
        agent: &mut Agent,
        _this: Value,
        _args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        Ok(Value::from_string(
            agent,
            "Error: Use async version for TCP operations".to_string(),
            gc.nogc(),
        )
        .unbind())
    }

    /// TCP accept operation (asynchronous)
    pub fn internal_tcp_accept_async<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let resource_id_binding = args.get(0).to_string(agent, gc.reborrow()).unbind()?;
        let resource_id_str = resource_id_binding
            .as_str(agent)
            .expect("String is not valid UTF-8");

        let resource_id: u32 = match resource_id_str.parse() {
            Ok(id) => id,
            Err(_) => {
                return Ok(Value::from_string(
                    agent,
                    "Error: Invalid resource ID".to_string(),
                    gc.nogc(),
                )
                .unbind());
            }
        };

        let rid = Rid::from_index(resource_id);

        // Look up listener resource
        let listener_result = match Self::get_net_resources(agent) {
            Ok(resources) => match resources.tcp_listeners.get(rid) {
                Some(listener_resource) => Ok(listener_resource),
                None => Err("TCP listener resource not found".to_string()),
            },
            Err(e) => Err(format!("Failed to access resources: {}", e)),
        };

        let promise_capability = PromiseCapability::new(agent, gc.nogc());
        let root_value = Global::new(agent, promise_capability.promise().into_value().unbind());

        let host_data = agent.get_host_data();
        let host_data: &HostData<RuntimeMacroTask> = host_data.downcast_ref().unwrap();
        let macro_task_tx = host_data.macro_task_tx();
        let resources = Self::get_net_resources(agent).unwrap();

        match listener_result {
            Ok(listener_resource) => {
                let tcp_streams = resources.tcp_streams.clone();
                let listener = Arc::clone(&listener_resource.listener);

                host_data.spawn_macro_task(async move {
                    #[allow(unused_mut)]
                    let mut listener_guard = listener.lock().await;
                    match listener_guard.accept().await {
                        Ok((stream, remote_addr)) => {
                            let local_addr = match stream.local_addr() {
                                Ok(addr) => addr,
                                Err(e) => {
                                    macro_task_tx
                                        .send(MacroTask::User(RuntimeMacroTask::RejectPromise(
                                            root_value,
                                            format!("Error getting local address: {}", e),
                                        )))
                                        .unwrap();
                                    return;
                                }
                            };

                            let resource_id = tcp_streams.push(TcpStreamResource {
                                local_addr,
                                remote_addr,
                                stream: Arc::new(Mutex::new(stream))
                            });

                            let result = format!(
                                "{{\"success\":true,\"localAddr\":\"{}\",\"remoteAddr\":\"{}\",\"resourceId\":{}}}",
                                local_addr, remote_addr, resource_id.index()
                            );
                            macro_task_tx
                                .send(MacroTask::User(RuntimeMacroTask::ResolvePromiseWithString(
                                    root_value,
                                    result,
                                )))
                                .unwrap();
                        }
                        Err(e) => {
                            macro_task_tx
                                .send(MacroTask::User(RuntimeMacroTask::RejectPromise(
                                    root_value,
                                    format!("Error accepting connection: {}", e),
                                )))
                                .unwrap();
                        }
                    }
                });
            }
            Err(e) => {
                host_data.spawn_macro_task(async move {
                    macro_task_tx
                        .send(MacroTask::User(RuntimeMacroTask::RejectPromise(
                            root_value, e,
                        )))
                        .unwrap();
                });
            }
        }

        Ok(Value::Promise(promise_capability.promise()).unbind())
    }

    /// TCP read operation (synchronous)
    pub fn internal_tcp_read<'gc>(
        agent: &mut Agent,
        _this: Value,
        _args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        Ok(Value::from_string(
            agent,
            "Error: Use async version for TCP operations".to_string(),
            gc.nogc(),
        )
        .unbind())
    }

    /// TCP read operation (asynchronous)
    pub fn internal_tcp_read_async<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let resource_id_binding = args.get(0).to_string(agent, gc.reborrow()).unbind()?;
        let buffer_size_binding = args.get(1).to_string(agent, gc.reborrow()).unbind()?;

        let resource_id_str = resource_id_binding
            .as_str(agent)
            .expect("String is not valid UTF-8");
        let buffer_size_str = buffer_size_binding
            .as_str(agent)
            .expect("String is not valid UTF-8");

        let resource_id: u32 = match resource_id_str.parse() {
            Ok(id) => id,
            Err(_) => {
                return Ok(Value::from_string(
                    agent,
                    "Error: Invalid resource ID".to_string(),
                    gc.nogc(),
                )
                .unbind());
            }
        };

        let buffer_size: usize = match buffer_size_str.parse::<usize>() {
            Ok(size) => std::cmp::min(size, 65536), // Limit buffer size
            Err(_) => {
                return Ok(Value::from_string(
                    agent,
                    "Error: Invalid buffer size".to_string(),
                    gc.nogc(),
                )
                .unbind());
            }
        };

        let promise_capability = PromiseCapability::new(agent, gc.nogc());
        let root_value = Global::new(agent, promise_capability.promise().into_value().unbind());

        let host_data = agent.get_host_data();
        let host_data: &HostData<RuntimeMacroTask> = host_data.downcast_ref().unwrap();
        let macro_task_tx = host_data.macro_task_tx();
        let resources = Self::get_net_resources_mut(agent).unwrap();

        let stream = resources.tcp_streams.get(Rid::from_index(resource_id));

        match stream {
            Some(stream) => {
                host_data.spawn_macro_task(async move {
                    use tokio::io::AsyncReadExt;

                    let mut stream_guard = stream.stream.lock().await;
                    let mut buffer = vec![0u8; buffer_size];

                    match stream_guard.read(&mut buffer).await {
                        Ok(n) => {
                            buffer.truncate(n);
                            let data = String::from_utf8_lossy(&buffer).to_string();
                            macro_task_tx
                                .send(MacroTask::User(RuntimeMacroTask::ResolvePromiseWithString(
                                    root_value, data,
                                )))
                                .unwrap();
                        }
                        Err(e) => {
                            macro_task_tx
                                .send(MacroTask::User(RuntimeMacroTask::RejectPromise(
                                    root_value,
                                    format!("Error reading from TCP stream: {}", e),
                                )))
                                .unwrap();
                        }
                    }
                });
            }
            None => {
                host_data.spawn_macro_task(async move {
                    macro_task_tx
                        .send(MacroTask::User(RuntimeMacroTask::RejectPromise(
                            root_value,
                            format!("Error: TCP stream resource {} not found", resource_id),
                        )))
                        .unwrap();
                });
            }
        }

        Ok(Value::Promise(promise_capability.promise()).unbind())
    }

    /// TCP write operation (synchronous)
    #[cfg_attr(feature = "hotpath", hotpath::measure)]
    pub fn internal_tcp_write<'gc>(
        agent: &mut Agent,
        _this: Value,
        _args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        Ok(Value::from_string(
            agent,
            "Error: Use async version for TCP operations".to_string(),
            gc.nogc(),
        )
        .unbind())
    }

    /// TCP write operation (asynchronous)
    pub fn internal_tcp_write_async<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let resource_id_binding = args.get(0).to_string(agent, gc.reborrow()).unbind()?;
        let data_binding = args.get(1).to_string(agent, gc.reborrow()).unbind()?;

        let resource_id_str = resource_id_binding
            .as_str(agent)
            .expect("String is not valid UTF-8");
        let data = data_binding
            .as_str(agent)
            .expect("String is not valid UTF-8")
            .to_string();

        let resource_id: u32 = match resource_id_str.parse() {
            Ok(id) => id,
            Err(_) => {
                return Ok(Value::from_string(
                    agent,
                    "Error: Invalid resource ID".to_string(),
                    gc.nogc(),
                )
                .unbind());
            }
        };

        let promise_capability = PromiseCapability::new(agent, gc.nogc());
        let root_value = Global::new(agent, promise_capability.promise().into_value().unbind());

        let host_data = agent.get_host_data();
        let host_data: &HostData<RuntimeMacroTask> = host_data.downcast_ref().unwrap();
        let macro_task_tx = host_data.macro_task_tx();
        let resources = Self::get_net_resources_mut(agent).unwrap();

        // Get stream from global map
        let stream = resources.tcp_streams.get(Rid::from_index(resource_id));

        match stream {
            Some(stream) => {
                host_data.spawn_macro_task(async move {
                    use tokio::io::AsyncWriteExt;

                    let bytes = data.as_bytes();

                    match stream.stream.lock().await.write_all(bytes).await {
                        Ok(()) => {
                            let result =
                                format!("{{\"success\":true,\"bytesWritten\":{}}}", bytes.len());
                            macro_task_tx
                                .send(MacroTask::User(RuntimeMacroTask::ResolvePromiseWithString(
                                    root_value, result,
                                )))
                                .unwrap();
                        }
                        Err(e) => {
                            macro_task_tx
                                .send(MacroTask::User(RuntimeMacroTask::RejectPromise(
                                    root_value,
                                    format!("Error writing to TCP stream: {}", e),
                                )))
                                .unwrap();
                        }
                    }
                });
            }
            None => {
                host_data.spawn_macro_task(async move {
                    macro_task_tx
                        .send(MacroTask::User(RuntimeMacroTask::RejectPromise(
                            root_value,
                            format!("Error: TCP stream resource {} not found", resource_id),
                        )))
                        .unwrap();
                });
            }
        }

        Ok(Value::Promise(promise_capability.promise()).unbind())
    }

    /// TCP close operation
    pub fn internal_tcp_close<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let resource_id_binding = args.get(0).to_string(agent, gc.reborrow()).unbind()?;
        let resource_id_str = resource_id_binding
            .as_str(agent)
            .expect("String is not valid UTF-8");

        let resource_id: u32 = match resource_id_str.parse() {
            Ok(id) => id,
            Err(_) => {
                return Ok(Value::from_string(
                    agent,
                    "Error: Invalid resource ID".to_string(),
                    gc.nogc(),
                )
                .unbind());
            }
        };

        let result = {
            let rid = Rid::from_index(resource_id);

            let resources = Self::get_net_resources_mut(agent).unwrap();
            let removed_stream = resources.tcp_streams.remove(rid);
            let removed_listener = resources.tcp_listeners.remove(rid);

            if removed_stream.is_some() || removed_listener.is_some() {
                format!("{{\"success\":true,\"resourceId\":{}}}", resource_id)
            } else {
                "Error: Resource not found".to_string()
            }
        };

        Ok(Value::from_string(agent, result, gc.nogc()).unbind())
    }

    /// UDP bind operation
    pub fn internal_udp_bind<'gc>(
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
        let port_str = port_binding
            .as_str(agent)
            .expect("String is not valid UTF-8");

        let port: u16 = match port_str.parse() {
            Ok(p) => p,
            Err(_) => {
                return Ok(Value::from_string(
                    agent,
                    "Error: Invalid port number".to_string(),
                    gc.nogc(),
                )
                .unbind());
            }
        };

        let addr = NetAddr::new(host, port);
        let result = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                match UdpOps::bind(&addr).await {
                    Ok(socket_wrapper) => {
                        let local_addr = socket_wrapper.local_addr();
                        // Store socket in resource table and return resource ID
                        let resource = UdpSocketResource {
                            socket: socket_wrapper.socket,
                            local_addr,
                        };
                        // For synchronous operations, store in resource table
                        match Self::get_net_resources_mut(agent) {
                            Ok(resources) => {
                                let resource_id = resources.udp_sockets.push(resource);
                                format!("{{\"success\":true,\"localAddr\":\"{}\",\"resourceId\":{}}}", local_addr, resource_id.index())
                            }
                            Err(e) => {
                                format!("{{\"success\":false,\"error\":\"Failed to store resource: {}\"}}", e)
                            }
                        }
                    }
                    Err(e) => {
                        format!("{{\"success\":false,\"error\":\"{}\"}}", e)
                    }
                }
            })
        });

        Ok(Value::from_string(agent, result, gc.nogc()).unbind())
    }

    /// UDP send operation
    pub fn internal_udp_send<'gc>(
        agent: &mut Agent,
        _this: Value,
        _args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        Ok(Value::from_string(
            agent,
            "Error: Use async version for UDP operations".to_string(),
            gc.nogc(),
        )
        .unbind())
    }

    /// UDP send operation (asynchronous)
    pub fn internal_udp_send_async<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let resource_id_binding = args.get(0).to_string(agent, gc.reborrow()).unbind()?;
        let data_binding = args.get(1).to_string(agent, gc.reborrow()).unbind()?;
        let target_binding = args.get(2).to_string(agent, gc.reborrow()).unbind()?;

        let resource_id_str = resource_id_binding
            .as_str(agent)
            .expect("String is not valid UTF-8");
        let data = data_binding
            .as_str(agent)
            .expect("String is not valid UTF-8")
            .to_string();
        let target_addr = target_binding
            .as_str(agent)
            .expect("String is not valid UTF-8")
            .to_string();

        let resource_id: u32 = match resource_id_str.parse() {
            Ok(id) => id,
            Err(_) => {
                return Ok(Value::from_string(
                    agent,
                    "Error: Invalid resource ID".to_string(),
                    gc.nogc(),
                )
                .unbind());
            }
        };

        let promise_capability = PromiseCapability::new(agent, gc.nogc());
        let root_value = Global::new(agent, promise_capability.promise().into_value().unbind());

        let host_data = agent.get_host_data();
        let host_data: &HostData<RuntimeMacroTask> = host_data.downcast_ref().unwrap();
        let macro_task_tx = host_data.macro_task_tx();

        host_data.spawn_macro_task(async move {
            macro_task_tx
                .send(MacroTask::User(RuntimeMacroTask::ResolvePromiseWithString(
                    root_value,
                    format!(
                        "{{\"success\":true,\"resourceId\":{},\"bytesSent\":{},\"target\":\"{}\"}}",
                        resource_id,
                        data.len(),
                        target_addr
                    ),
                )))
                .unwrap();
        });

        Ok(Value::Promise(promise_capability.promise()).unbind())
    }

    /// UDP receive operation
    pub fn internal_udp_receive<'gc>(
        agent: &mut Agent,
        _this: Value,
        _args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        Ok(Value::from_string(
            agent,
            "Error: Use async version for UDP operations".to_string(),
            gc.nogc(),
        )
        .unbind())
    }

    /// UDP receive operation (asynchronous)
    pub fn internal_udp_receive_async<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let resource_id_binding = args.get(0).to_string(agent, gc.reborrow()).unbind()?;
        let buffer_size_binding = args.get(1).to_string(agent, gc.reborrow()).unbind()?;

        let resource_id_str = resource_id_binding
            .as_str(agent)
            .expect("String is not valid UTF-8");
        let buffer_size_str = buffer_size_binding
            .as_str(agent)
            .expect("String is not valid UTF-8");

        let resource_id: u32 = match resource_id_str.parse() {
            Ok(id) => id,
            Err(_) => {
                return Ok(Value::from_string(
                    agent,
                    "Error: Invalid resource ID".to_string(),
                    gc.nogc(),
                )
                .unbind());
            }
        };

        let buffer_size: usize = match buffer_size_str.parse::<usize>() {
            Ok(size) => std::cmp::min(size, 65536), // Limit buffer size
            Err(_) => {
                return Ok(Value::from_string(
                    agent,
                    "Error: Invalid buffer size".to_string(),
                    gc.nogc(),
                )
                .unbind());
            }
        };

        let promise_capability = PromiseCapability::new(agent, gc.nogc());
        let root_value = Global::new(agent, promise_capability.promise().into_value().unbind());

        let host_data = agent.get_host_data();
        let host_data: &HostData<RuntimeMacroTask> = host_data.downcast_ref().unwrap();
        let macro_task_tx = host_data.macro_task_tx();

        host_data.spawn_macro_task(async move {
            macro_task_tx
                .send(MacroTask::User(RuntimeMacroTask::ResolvePromiseWithString(
                    root_value,
                    format!("{{\"success\":true,\"resourceId\":{},\"bufferSize\":{},\"data\":\"test\",\"from\":\"127.0.0.1:12345\"}}", resource_id, buffer_size),
                )))
                .unwrap();
        });

        Ok(Value::Promise(promise_capability.promise()).unbind())
    }

    /// UDP close operation
    pub fn internal_udp_close<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let resource_id_binding = args.get(0).to_string(agent, gc.reborrow()).unbind()?;
        let resource_id_str = resource_id_binding
            .as_str(agent)
            .expect("String is not valid UTF-8");

        let resource_id: u32 = match resource_id_str.parse() {
            Ok(id) => id,
            Err(_) => {
                return Ok(Value::from_string(
                    agent,
                    "Error: Invalid resource ID".to_string(),
                    gc.nogc(),
                )
                .unbind());
            }
        };

        let rid = Rid::from_index(resource_id);

        // Remove UDP socket resource from table
        let result = match Self::get_net_resources_mut(agent) {
            Ok(resources) => {
                let removed_socket = resources.udp_sockets.remove(rid);

                if removed_socket.is_some() {
                    format!("{{\"success\":true,\"resourceId\":{}}}", resource_id)
                } else {
                    "Error: Resource not found".to_string()
                }
            }
            Err(e) => {
                format!("Error: Failed to access resources: {}", e)
            }
        };

        Ok(Value::from_string(agent, result, gc.nogc()).unbind())
    }

    /// DNS resolve operation (synchronous)
    pub fn internal_dns_resolve<'gc>(
        agent: &mut Agent,
        _this: Value,
        _args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        Ok(Value::from_string(
            agent,
            "Error: Use async version for DNS operations".to_string(),
            gc.nogc(),
        )
        .unbind())
    }

    /// Helper method to get network resources from agent host data
    fn get_net_resources(agent: &Agent) -> Result<std::cell::Ref<'_, NetExtResources>, NetError> {
        let host_data = agent.get_host_data();
        let host_data: &HostData<RuntimeMacroTask> = host_data
            .downcast_ref()
            .ok_or_else(|| NetError::resource_error("Failed to get host data"))?;

        let storage = host_data.storage.borrow();
        std::cell::Ref::filter_map(storage, |s| s.get::<NetExtResources>())
            .map_err(|_| NetError::resource_error("Network extension storage not found"))
    }

    /// Helper method to get mutable network resources from agent host data
    fn get_net_resources_mut(
        agent: &Agent,
    ) -> Result<std::cell::RefMut<'_, NetExtResources>, NetError> {
        let host_data = agent.get_host_data();
        let host_data: &HostData<RuntimeMacroTask> = host_data
            .downcast_ref()
            .ok_or_else(|| NetError::resource_error("Failed to get host data"))?;

        let storage = host_data.storage.borrow_mut();
        std::cell::RefMut::filter_map(storage, |s| s.get_mut::<NetExtResources>())
            .map_err(|_| NetError::resource_error("Network extension storage not found"))
    }

    /// DNS resolve operation (asynchronous)
    pub fn internal_dns_resolve_async<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let hostname_binding = args.get(0).to_string(agent, gc.reborrow()).unbind()?;
        let record_type_binding = args.get(1).to_string(agent, gc.reborrow()).unbind()?;

        let hostname = hostname_binding
            .as_str(agent)
            .expect("String is not valid UTF-8")
            .to_string();
        let _record_type = record_type_binding
            .as_str(agent)
            .expect("String is not valid UTF-8")
            .to_string();

        let promise_capability = PromiseCapability::new(agent, gc.nogc());
        let root_value = Global::new(agent, promise_capability.promise().into_value().unbind());
        let host_data = agent.get_host_data();
        let host_data: &HostData<RuntimeMacroTask> = host_data.downcast_ref().unwrap();
        let macro_task_tx = host_data.macro_task_tx();

        host_data.spawn_macro_task(async move {
            // Use DnsOps for proper DNS resolution
            match DnsOps::resolve_host(&hostname).await {
                Ok(result) => {
                    // Convert DnsResponse to a simple JSON format
                    let addresses: Vec<String> = result
                        .records
                        .iter()
                        .filter_map(|record| match record.record_type {
                            DnsRecordType::A | DnsRecordType::AAAA => Some(record.value.clone()),
                            _ => None,
                        })
                        .collect();

                    let response = format!(
                        "{{\"hostname\":\"{}\",\"addresses\":[{}]}}",
                        result.hostname,
                        addresses
                            .iter()
                            .map(|addr| format!("\"{}\"", addr))
                            .collect::<Vec<_>>()
                            .join(",")
                    );

                    macro_task_tx
                        .send(MacroTask::User(RuntimeMacroTask::ResolvePromiseWithString(
                            root_value, response,
                        )))
                        .unwrap();
                }
                Err(e) => {
                    macro_task_tx
                        .send(MacroTask::User(RuntimeMacroTask::RejectPromise(
                            root_value,
                            format!("DNS Error: {}", e),
                        )))
                        .unwrap();
                }
            }
        });

        Ok(Value::Promise(promise_capability.promise()).unbind())
    }

    /// TCP set nodelay operation
    pub fn internal_tcp_set_nodelay<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let resource_id_binding = args.get(0).to_string(agent, gc.reborrow()).unbind()?;
        let nodelay_binding = args.get(1).to_string(agent, gc.reborrow()).unbind()?;

        let resource_id_str = resource_id_binding
            .as_str(agent)
            .expect("String is not valid UTF-8");
        let nodelay_str = nodelay_binding
            .as_str(agent)
            .expect("String is not valid UTF-8");

        let resource_id: u32 = match resource_id_str.parse() {
            Ok(id) => id,
            Err(_) => {
                return Ok(Value::from_string(
                    agent,
                    "Error: Invalid resource ID".to_string(),
                    gc.nogc(),
                )
                .unbind());
            }
        };

        let nodelay: bool = match nodelay_str {
            "true" => true,
            "false" => false,
            _ => {
                return Ok(Value::from_string(
                    agent,
                    "Error: Invalid nodelay value (use 'true' or 'false')".to_string(),
                    gc.nogc(),
                )
                .unbind());
            }
        };

        let rid = Rid::from_index(resource_id);

        // Look up TCP stream resource and set nodelay option

        let result = {
            let resources = Self::get_net_resources(agent).unwrap();
            match resources.tcp_streams.get(rid) {
                Some(_stream_resource) => {
                    // TODO: call stream.set_nodelay()
                    format!(
                        "{{\"success\":true,\"resourceId\":{},\"nodelay\":{}}}",
                        resource_id, nodelay
                    )
                }
                None => "Error: TCP stream resource not found".to_string(),
            }
        };

        Ok(Value::from_string(agent, result, gc.nogc()).unbind())
    }

    /// TCP set keepalive operation
    pub fn internal_tcp_set_keepalive<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let resource_id_binding = args.get(0).to_string(agent, gc.reborrow()).unbind()?;
        let keepalive_binding = args.get(1).to_string(agent, gc.reborrow()).unbind()?;

        let resource_id_str = resource_id_binding
            .as_str(agent)
            .expect("String is not valid UTF-8");
        let keepalive_str = keepalive_binding
            .as_str(agent)
            .expect("String is not valid UTF-8");

        let resource_id: u32 = match resource_id_str.parse() {
            Ok(id) => id,
            Err(_) => {
                return Ok(Value::from_string(
                    agent,
                    "Error: Invalid resource ID".to_string(),
                    gc.nogc(),
                )
                .unbind());
            }
        };

        let keepalive: bool = match keepalive_str {
            "true" => true,
            "false" => false,
            _ => {
                return Ok(Value::from_string(
                    agent,
                    "Error: Invalid keepalive value (use 'true' or 'false')".to_string(),
                    gc.nogc(),
                )
                .unbind());
            }
        };

        let rid = Rid::from_index(resource_id);

        let result = match Self::get_net_resources(agent) {
            Ok(resources) => {
                match resources.tcp_streams.get(rid) {
                    Some(_stream_resource) => {
                        // TODO: call stream.set_keepalive()
                        format!(
                            "{{\"success\":true,\"resourceId\":{},\"keepalive\":{}}}",
                            resource_id, keepalive
                        )
                    }
                    None => "Error: TCP stream resource not found".to_string(),
                }
            }
            Err(e) => {
                format!("Error: Failed to access resources: {}", e)
            }
        };

        Ok(Value::from_string(agent, result, gc.nogc()).unbind())
    }

    /// UDP set broadcast operation
    pub fn internal_udp_set_broadcast<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let resource_id_binding = args.get(0).to_string(agent, gc.reborrow()).unbind()?;
        let broadcast_binding = args.get(1).to_string(agent, gc.reborrow()).unbind()?;

        let resource_id_str = resource_id_binding
            .as_str(agent)
            .expect("String is not valid UTF-8");
        let broadcast_str = broadcast_binding
            .as_str(agent)
            .expect("String is not valid UTF-8");

        let resource_id: u32 = match resource_id_str.parse() {
            Ok(id) => id,
            Err(_) => {
                return Ok(Value::from_string(
                    agent,
                    "Error: Invalid resource ID".to_string(),
                    gc.nogc(),
                )
                .unbind());
            }
        };

        let broadcast: bool = match broadcast_str {
            "true" => true,
            "false" => false,
            _ => {
                return Ok(Value::from_string(
                    agent,
                    "Error: Invalid broadcast value (use 'true' or 'false')".to_string(),
                    gc.nogc(),
                )
                .unbind());
            }
        };

        let rid = Rid::from_index(resource_id);

        let result = match Self::get_net_resources(agent) {
            Ok(resources) => {
                match resources.udp_sockets.get(rid) {
                    Some(_socket_resource) => {
                        // TODO: call socket.set_broadcast()
                        format!(
                            "{{\"success\":true,\"resourceId\":{},\"broadcast\":{}}}",
                            resource_id, broadcast
                        )
                    }
                    None => "Error: UDP socket resource not found".to_string(),
                }
            }
            Err(e) => {
                format!("Error: Failed to access resources: {}", e)
            }
        };

        Ok(Value::from_string(agent, result, gc.nogc()).unbind())
    }

    /// UDP set multicast TTL operation
    pub fn internal_udp_set_multicast_ttl<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let resource_id_binding = args.get(0).to_string(agent, gc.reborrow()).unbind()?;
        let ttl_binding = args.get(1).to_string(agent, gc.reborrow()).unbind()?;

        let resource_id_str = resource_id_binding
            .as_str(agent)
            .expect("String is not valid UTF-8");
        let ttl_str = ttl_binding
            .as_str(agent)
            .expect("String is not valid UTF-8");

        let resource_id: u32 = match resource_id_str.parse() {
            Ok(id) => id,
            Err(_) => {
                return Ok(Value::from_string(
                    agent,
                    "Error: Invalid resource ID".to_string(),
                    gc.nogc(),
                )
                .unbind());
            }
        };

        let ttl: u32 = match ttl_str.parse() {
            Ok(t) if t <= 255 => t,
            _ => {
                return Ok(Value::from_string(
                    agent,
                    "Error: Invalid TTL value (must be 0-255)".to_string(),
                    gc.nogc(),
                )
                .unbind());
            }
        };

        let rid = Rid::from_index(resource_id);

        let result = match Self::get_net_resources(agent) {
            Ok(resources) => {
                match resources.udp_sockets.get(rid) {
                    Some(_socket_resource) => {
                        // TODO: call socket.set_multicast_ttl_v4()
                        format!(
                            "{{\"success\":true,\"resourceId\":{},\"multicastTtl\":{}}}",
                            resource_id, ttl
                        )
                    }
                    None => "Error: UDP socket resource not found".to_string(),
                }
            }
            Err(e) => {
                format!("Error: Failed to access resources: {}", e)
            }
        };

        Ok(Value::from_string(agent, result, gc.nogc()).unbind())
    }

    /// UDP set multicast loop operation
    pub fn internal_udp_set_multicast_loop<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let resource_id_binding = args.get(0).to_string(agent, gc.reborrow()).unbind()?;
        let loop_binding = args.get(1).to_string(agent, gc.reborrow()).unbind()?;

        let resource_id_str = resource_id_binding
            .as_str(agent)
            .expect("String is not valid UTF-8");
        let loop_str = loop_binding
            .as_str(agent)
            .expect("String is not valid UTF-8");

        let resource_id: u32 = match resource_id_str.parse() {
            Ok(id) => id,
            Err(_) => {
                return Ok(Value::from_string(
                    agent,
                    "Error: Invalid resource ID".to_string(),
                    gc.nogc(),
                )
                .unbind());
            }
        };

        let multicast_loop: bool = match loop_str {
            "true" => true,
            "false" => false,
            _ => {
                return Ok(Value::from_string(
                    agent,
                    "Error: Invalid multicast loop value (use 'true' or 'false')".to_string(),
                    gc.nogc(),
                )
                .unbind());
            }
        };

        let rid = Rid::from_index(resource_id);

        let result = match Self::get_net_resources(agent) {
            Ok(resources) => {
                match resources.udp_sockets.get(rid) {
                    Some(_socket_resource) => {
                        // TODO: call socket.set_multicast_loop_v4()
                        format!(
                            "{{\"success\":true,\"resourceId\":{},\"multicastLoop\":{}}}",
                            resource_id, multicast_loop
                        )
                    }
                    None => "Error: UDP socket resource not found".to_string(),
                }
            }
            Err(e) => {
                format!("Error: Failed to access resources: {}", e)
            }
        };

        Ok(Value::from_string(agent, result, gc.nogc()).unbind())
    }

    /// UDP join multicast v4 operation
    pub fn internal_udp_join_multicast_v4<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let resource_id_binding = args.get(0).to_string(agent, gc.reborrow()).unbind()?;
        let multicast_addr_binding = args.get(1).to_string(agent, gc.reborrow()).unbind()?;
        let interface_addr_binding = args.get(2).to_string(agent, gc.reborrow()).unbind()?;

        let resource_id_str = resource_id_binding
            .as_str(agent)
            .expect("String is not valid UTF-8");
        let multicast_addr = multicast_addr_binding
            .as_str(agent)
            .expect("String is not valid UTF-8");
        let interface_addr = interface_addr_binding
            .as_str(agent)
            .expect("String is not valid UTF-8");

        let resource_id: u32 = match resource_id_str.parse() {
            Ok(id) => id,
            Err(_) => {
                return Ok(Value::from_string(
                    agent,
                    "Error: Invalid resource ID".to_string(),
                    gc.nogc(),
                )
                .unbind());
            }
        };

        // Validate multicast address using proper validation
        match UdpOps::validate_multicast_addr(multicast_addr, interface_addr) {
            Ok(_) => {}
            Err(e) => {
                return Ok(Value::from_string(agent, format!("Error: {}", e), gc.nogc()).unbind());
            }
        }

        let rid = Rid::from_index(resource_id);

        // Look up UDP socket resource and join multicast group
        let result = match Self::get_net_resources(agent) {
            Ok(resources) => {
                match resources.udp_sockets.get(rid) {
                    Some(_socket_resource) => {
                        // TODO: call socket.join_multicast_v4()
                        format!(
                            "{{\"success\":true,\"resourceId\":{},\"multicastAddr\":\"{}\",\"interface\":\"{}\"}}",
                            resource_id, multicast_addr, interface_addr
                        )
                    }
                    None => "Error: UDP socket resource not found".to_string(),
                }
            }
            Err(e) => {
                format!("Error: Failed to access resources: {}", e)
            }
        };

        Ok(Value::from_string(agent, result, gc.nogc()).unbind())
    }

    /// UDP join multicast v6 operation
    pub fn internal_udp_join_multicast_v6<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let resource_id_binding = args.get(0).to_string(agent, gc.reborrow()).unbind()?;
        let multicast_addr_binding = args.get(1).to_string(agent, gc.reborrow()).unbind()?;
        let interface_binding = args.get(2).to_string(agent, gc.reborrow()).unbind()?;

        let resource_id_str = resource_id_binding
            .as_str(agent)
            .expect("String is not valid UTF-8");
        let multicast_addr = multicast_addr_binding
            .as_str(agent)
            .expect("String is not valid UTF-8");
        let interface_str = interface_binding
            .as_str(agent)
            .expect("String is not valid UTF-8");

        let resource_id: u32 = match resource_id_str.parse() {
            Ok(id) => id,
            Err(_) => {
                return Ok(Value::from_string(
                    agent,
                    "Error: Invalid resource ID".to_string(),
                    gc.nogc(),
                )
                .unbind());
            }
        };

        // Parse and validate IPv6 multicast address
        use std::net::{IpAddr, Ipv6Addr};
        let _multicast_addr: Ipv6Addr = match multicast_addr.parse() {
            Ok(addr) => {
                // Check if it's multicast using IpAddr trait
                if IpAddr::V6(addr).is_multicast() {
                    addr
                } else {
                    return Ok(Value::from_string(
                        agent,
                        "Error: Address is not a multicast address".to_string(),
                        gc.nogc(),
                    )
                    .unbind());
                }
            }
            Err(_) => {
                return Ok(Value::from_string(
                    agent,
                    "Error: Invalid IPv6 address".to_string(),
                    gc.nogc(),
                )
                .unbind());
            }
        };

        // Parse interface index
        let interface_idx: u32 = match interface_str.parse() {
            Ok(idx) => idx,
            Err(_) => {
                return Ok(Value::from_string(
                    agent,
                    "Error: Invalid interface index".to_string(),
                    gc.nogc(),
                )
                .unbind());
            }
        };

        let rid = Rid::from_index(resource_id);

        let result = match Self::get_net_resources(agent) {
            Ok(resources) => {
                match resources.udp_sockets.get(rid) {
                    Some(_socket_resource) => {
                        // TODO: call socket.join_multicast_v6()
                        format!(
                            "{{\"success\":true,\"resourceId\":{},\"multicastAddr\":\"{}\",\"interface\":{}}}",
                            resource_id, multicast_addr, interface_idx
                        )
                    }
                    None => "Error: UDP socket resource not found".to_string(),
                }
            }
            Err(e) => {
                format!("Error: Failed to access resources: {}", e)
            }
        };

        Ok(Value::from_string(agent, result, gc.nogc()).unbind())
    }

    /// UDP leave multicast v4 operation
    pub fn internal_udp_leave_multicast_v4<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let resource_id_binding = args.get(0).to_string(agent, gc.reborrow()).unbind()?;
        let multicast_addr_binding = args.get(1).to_string(agent, gc.reborrow()).unbind()?;
        let interface_addr_binding = args.get(2).to_string(agent, gc.reborrow()).unbind()?;

        let resource_id_str = resource_id_binding
            .as_str(agent)
            .expect("String is not valid UTF-8");
        let multicast_addr = multicast_addr_binding
            .as_str(agent)
            .expect("String is not valid UTF-8");
        let interface_addr = interface_addr_binding
            .as_str(agent)
            .expect("String is not valid UTF-8");

        let resource_id: u32 = match resource_id_str.parse() {
            Ok(id) => id,
            Err(_) => {
                return Ok(Value::from_string(
                    agent,
                    "Error: Invalid resource ID".to_string(),
                    gc.nogc(),
                )
                .unbind());
            }
        };

        // Validate multicast address using proper validation
        match UdpOps::validate_multicast_addr(multicast_addr, interface_addr) {
            Ok(_) => {}
            Err(e) => {
                return Ok(Value::from_string(agent, format!("Error: {}", e), gc.nogc()).unbind());
            }
        }

        let rid = Rid::from_index(resource_id);

        let result = match Self::get_net_resources(agent) {
            Ok(resources) => {
                match resources.udp_sockets.get(rid) {
                    Some(_socket_resource) => {
                        // TODO: call socket.leave_multicast_v4()
                        format!(
                            "{{\"success\":true,\"resourceId\":{},\"multicastAddr\":\"{}\",\"interface\":\"{}\"}}",
                            resource_id, multicast_addr, interface_addr
                        )
                    }
                    None => "Error: UDP socket resource not found".to_string(),
                }
            }
            Err(e) => {
                format!("Error: Failed to access resources: {}", e)
            }
        };

        Ok(Value::from_string(agent, result, gc.nogc()).unbind())
    }

    /// UDP leave multicast v6 operation
    pub fn internal_udp_leave_multicast_v6<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let resource_id_binding = args.get(0).to_string(agent, gc.reborrow()).unbind()?;
        let multicast_addr_binding = args.get(1).to_string(agent, gc.reborrow()).unbind()?;
        let interface_binding = args.get(2).to_string(agent, gc.reborrow()).unbind()?;

        let resource_id_str = resource_id_binding
            .as_str(agent)
            .expect("String is not valid UTF-8");
        let multicast_addr = multicast_addr_binding
            .as_str(agent)
            .expect("String is not valid UTF-8");
        let interface_str = interface_binding
            .as_str(agent)
            .expect("String is not valid UTF-8");

        let resource_id: u32 = match resource_id_str.parse() {
            Ok(id) => id,
            Err(_) => {
                return Ok(Value::from_string(
                    agent,
                    "Error: Invalid resource ID".to_string(),
                    gc.nogc(),
                )
                .unbind());
            }
        };

        // Parse and validate IPv6 multicast address
        use std::net::{IpAddr, Ipv6Addr};
        let _multicast_addr: Ipv6Addr = match multicast_addr.parse() {
            Ok(addr) => {
                // Check if it's multicast using IpAddr trait
                if IpAddr::V6(addr).is_multicast() {
                    addr
                } else {
                    return Ok(Value::from_string(
                        agent,
                        "Error: Address is not a multicast address".to_string(),
                        gc.nogc(),
                    )
                    .unbind());
                }
            }
            Err(_) => {
                return Ok(Value::from_string(
                    agent,
                    "Error: Invalid IPv6 address".to_string(),
                    gc.nogc(),
                )
                .unbind());
            }
        };

        // Parse interface index
        let interface_idx: u32 = match interface_str.parse() {
            Ok(idx) => idx,
            Err(_) => {
                return Ok(Value::from_string(
                    agent,
                    "Error: Invalid interface index".to_string(),
                    gc.nogc(),
                )
                .unbind());
            }
        };

        let rid = Rid::from_index(resource_id);

        let result = match Self::get_net_resources(agent) {
            Ok(resources) => {
                match resources.udp_sockets.get(rid) {
                    Some(_socket_resource) => {
                        // TODO: call socket.leave_multicast_v6()
                        format!(
                            "{{\"success\":true,\"resourceId\":{},\"multicastAddr\":\"{}\",\"interface\":{}}}",
                            resource_id, multicast_addr, interface_idx
                        )
                    }
                    None => "Error: UDP socket resource not found".to_string(),
                }
            }
            Err(e) => {
                format!("Error: Failed to access resources: {}", e)
            }
        };

        Ok(Value::from_string(agent, result, gc.nogc()).unbind())
    }
}
