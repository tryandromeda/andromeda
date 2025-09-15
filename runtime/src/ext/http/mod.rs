use crate::RuntimeMacroTask;
use crate::ext::net::{NetAddr, TcpOps, TcpStreamWrapper};
use andromeda_core::{Extension, ExtensionOp, HostData, OpsStorage, ResourceTable};
use nova_vm::{
    ecmascript::{
        builtins::ArgumentsList,
        execution::{Agent, JsResult},
        types::Value,
    },
    engine::context::{Bindable, GcScope},
};
use std::collections::HashMap;

const DEFAULT_HOSTNAME: &str = "127.0.0.1";
const DEFAULT_PORT: u16 = 8080;

// TODO
pub struct HttpServer {}
pub struct HttpResources {
    servers: ResourceTable<HttpServer>,
}

#[derive(Default)]
pub struct ServeExt;

impl ServeExt {
    #[cfg_attr(feature = "hotpath", hotpath::measure)]
    pub fn new_extension() -> Extension {
        Extension {
            name: "http",
            ops: vec![
                ExtensionOp::new("internal_http_listen", Self::internal_http_listen, 2, false),
                ExtensionOp::new("internal_http_close", Self::internal_http_close, 1, false),
            ],
            storage: Some(Box::new(|storage: &mut OpsStorage| {
                storage.insert(HttpResources {
                    servers: ResourceTable::new(),
                });
            })),
            files: vec![include_str!("./mod.ts")],
        }
    }

    fn internal_http_listen<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let hostname = if !args.is_empty() {
            let binding = args.get(0).to_string(agent, gc.reborrow()).ok();
            binding
                .and_then(|s| s.as_str(agent).map(|str| str.to_string()))
                .unwrap_or_else(|| DEFAULT_HOSTNAME.to_string())
        } else {
            DEFAULT_HOSTNAME.to_string()
        };

        let port = if args.len() > 1 {
            let binding = args.get(1).to_string(agent, gc.reborrow()).ok();
            binding
                .and_then(|s| s.as_str(agent).and_then(|str| str.parse::<u16>().ok()))
                .unwrap_or(DEFAULT_PORT)
        } else {
            DEFAULT_PORT
        };

        let host_data = agent.get_host_data();
        let host_data: &HostData<RuntimeMacroTask> = host_data.downcast_ref().unwrap();

        let server_id = host_data
            .storage
            .borrow()
            .get::<HttpResources>()
            .unwrap()
            .servers
            .len() as u32;

        let hostname_clone = hostname.clone();
        let port_clone = port;

        host_data.spawn_macro_task(async move {
            let addr = NetAddr::new(hostname_clone.clone(), port_clone);
            match TcpOps::listen(&addr).await {
                Ok(listener) => {
                    println!(
                        "Server listening on http://{}:{}",
                        hostname_clone, port_clone
                    );

                    loop {
                        match listener.accept().await {
                            Ok(stream) => {
                                let sid = server_id;
                                tokio::spawn(async move {
                                    let _ = handle_connection(stream, sid).await;
                                });
                            }
                            Err(e) => {
                                eprintln!("Failed to accept connection: {}", e);
                                break;
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!(
                        "Failed to bind to {}:{} - {}",
                        hostname_clone, port_clone, e
                    );
                }
            }
        });

        Ok(Value::from_f64(agent, server_id as f64, gc.nogc()).unbind())
    }

    fn internal_http_close<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let server_id = if !args.is_empty() {
            let binding = args.get(0).to_string(agent, gc.reborrow()).ok();
            binding
                .and_then(|s| s.as_str(agent).and_then(|str| str.parse::<u32>().ok()))
                .unwrap_or(0)
        } else {
            0
        };

        // TODO: Implement server close
        println!("Closing server {}", server_id);

        Ok(Value::Undefined.unbind())
    }
}

async fn handle_connection(
    stream: TcpStreamWrapper,
    server_id: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut buffer = vec![0u8; 4096];
    let mut total_read = 0;

    loop {
        let n = stream.read(&mut buffer[total_read..]).await?;
        if n == 0 {
            break;
        }
        total_read += n;

        if buffer[..total_read].windows(4).any(|w| w == b"\r\n\r\n") {
            break;
        }

        if total_read == buffer.len() {
            buffer.resize(buffer.len() * 2, 0);
        }
    }

    let request_str = String::from_utf8_lossy(&buffer[..total_read]);
    let lines: Vec<&str> = request_str.lines().collect();

    if lines.is_empty() {
        return send_error_response(&stream, 400, "Bad Request").await;
    }

    let request_parts: Vec<&str> = lines[0].split_whitespace().collect();
    if request_parts.len() < 3 {
        return send_error_response(&stream, 400, "Bad Request").await;
    }

    let method = request_parts[0].to_string();
    let path = request_parts[1].to_string();
    let _http_version = request_parts[2];

    let mut headers = HashMap::new();
    let mut i = 1;
    while i < lines.len() && !lines[i].is_empty() {
        if let Some((key, value)) = lines[i].split_once(':') {
            headers.insert(key.trim().to_lowercase(), value.trim().to_string());
        }
        i += 1;
    }

    let body_start = request_str.find("\r\n\r\n").map(|pos| pos + 4);
    let _body = if let Some(start) = body_start {
        if start < total_read {
            Some(&buffer[start..total_read])
        } else {
            None
        }
    } else {
        None
    };

    let host = headers
        .get("host")
        .unwrap_or(&"localhost".to_string())
        .clone();
    let _url = format!("http://{}{}", host, path);

    // TODO: Call JavaScript handler via __andromeda_internal_handle_http_request
    let response_body = format!(
        "Hello from Andromeda HTTP server!\n\nRequest received:\n- Method: {}\n- Path: {}\n- Server ID: {}\n\nNote: JavaScript handler integration in progress.",
        method, path, server_id
    );

    let status_code = 200u16;
    let status_text = "OK";

    let mut response = format!("HTTP/1.1 {} {}\r\n", status_code, status_text);

    response.push_str("Content-Type: text/plain\r\n");
    response.push_str("X-Powered-By: Andromeda\r\n");
    response.push_str(&format!("Content-Length: {}\r\n", response_body.len()));

    response.push_str("\r\n");
    response.push_str(&response_body);

    stream.write_all(response.as_bytes()).await?;
    stream.flush().await?;
    stream.shutdown().await?;

    Ok(())
}

async fn send_error_response(
    stream: &TcpStreamWrapper,
    status_code: u16,
    status_text: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let response = format!(
        "HTTP/1.1 {} {}\r\n\
         Content-Type: text/plain; charset=UTF-8\r\n\
         Content-Length: {}\r\n\
         Connection: close\r\n\
         \r\n\
         {}",
        status_code,
        status_text,
        status_text.len(),
        status_text
    );

    stream.write_all(response.as_bytes()).await?;
    stream.flush().await?;
    stream.shutdown().await?;
    Ok(())
}
