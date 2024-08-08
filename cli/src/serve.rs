use std::cell::RefCell;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::rc::Rc;

use andromeda_core::{Runtime, RuntimeConfig};
use andromeda_runtime::{
    recommended_builtins, recommended_eventloop_handler, recommended_extensions, RuntimeMacroTask,
};
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use nova_vm::ecmascript::execution::agent::RealmRoot;
use nova_vm::ecmascript::types::InternalMethods;
use nova_vm::ecmascript::{
    scripts_and_modules::script::{parse_script, script_evaluation},
    types,
};
use nova_vm::SmallString;
use tokio::net::TcpListener;

async fn run_script_for_request(
    req: Request<hyper::body::Incoming>,
    runtime: Rc<RefCell<Runtime<RuntimeMacroTask>>>,
) -> Result<Response<Full<Bytes>>, Infallible> {
    let (realm_root, strict_mode) = {
        let runtime = runtime.borrow();
        // TODO(Beast): RealmRoot in Nova maybe can derive Clone to make this not as cursed
        let realm_root = unsafe {
            std::mem::transmute_copy::<&RealmRoot, &'static RealmRoot>(&&runtime.realm_root)
        };
        (realm_root, runtime.config.no_strict)
    };

    runtime
        .borrow_mut()
        .agent
        .run_in_realm(&realm_root, |agent| {
            let source_text =
                types::String::from_string(agent, format!("serve(\"{}\")", req.uri().to_string()));
            let realm = agent.current_realm_id();

            let script = match parse_script(agent, source_text, realm, strict_mode, None) {
                Ok(script) => script,
                Err(_) => {
                    panic!("error");
                }
            };

            match script_evaluation(agent, script) {
                Ok(value) => match value {
                    types::Value::String(_) => match value.to_string(agent) {
                        Ok(str) => Ok(Response::new(Full::new(Bytes::from(
                            str.as_str(agent).to_owned(),
                        )))),
                        _ => panic!("error"),
                    },
                    types::Value::Object(obj) => {
                        let body = {
                            let body_property_descriptor = obj
                                .internal_get_own_property(
                                    agent,
                                    types::PropertyKey::SmallString(
                                        SmallString::from_str_unchecked("body"),
                                    ),
                                )
                                .unwrap()
                                .unwrap();
                            let body_value = body_property_descriptor.value.unwrap();
                            body_value
                                .to_string(agent)
                                .unwrap()
                                .clone()
                                .as_str(agent)
                                .to_owned()
                        };
                        let status = {
                            let status_code = obj
                                .internal_get_own_property(
                                    agent,
                                    types::PropertyKey::SmallString(
                                        SmallString::from_str_unchecked("status"),
                                    ),
                                )
                                .unwrap()
                                .unwrap();
                            let status_value = status_code.value.unwrap();
                            status_value.to_int32(agent).unwrap()
                        };
                        let status_code = StatusCode::from_u16(status as u16).unwrap();

                        let res = Response::builder()
                            .status(&status_code)
                            .body(Full::new(Bytes::from(body)))
                            .unwrap();
                        Ok(res)
                    }
                    _ => panic!("serve function must return a string or an object"),
                },
                _ => panic!("error"),
            }
        })
}

pub fn serve(path: String) {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async move {
            // setup code
            let runtime = Rc::new(RefCell::new(Runtime::new(RuntimeConfig {
                no_strict: false,
                paths: vec![path],
                verbose: false,
                extensions: recommended_extensions(),
                builtins: recommended_builtins(),
                eventloop_handler: recommended_eventloop_handler,
            })));
            _ = runtime.borrow_mut().run();

            let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
            let listener = TcpListener::bind(addr).await.unwrap();
            loop {
                let (stream, _) = listener.accept().await.unwrap();
                let io = TokioIo::new(stream);

                if let Err(err) = http1::Builder::new()
                    // `service_fn` converts our function in a `Service`
                    .serve_connection(
                        io,
                        service_fn(|request| run_script_for_request(request, runtime.clone())),
                    )
                    .await
                {
                    eprintln!("Error serving connection: {:?}", err);
                }
            }
        })
}
