use std::cell::{self, RefCell, RefMut};
use std::convert::Infallible;
use std::net::SocketAddr;
use std::rc::Rc;
use std::sync::{Arc, Mutex, OnceLock};

use andromeda_core::{Runtime, RuntimeConfig};
use andromeda_runtime::{
    recommended_builtins, recommended_eventloop_handler, recommended_extensions, RuntimeMacroTask,
};
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use nova_vm::ecmascript::{
    scripts_and_modules::script::{parse_script, script_evaluation},
    types,
};
use tokio::net::TcpListener;

async fn create_response(
    _: Request<hyper::body::Incoming>,
    response: String,
) -> Result<Response<Full<Bytes>>, Infallible> {
    Ok(Response::new(Full::new(Bytes::from(response))))
}

fn run_script_for_request(runtime: &mut Runtime<RuntimeMacroTask>) -> String {
    runtime.agent.run_in_realm(&runtime.realm_root, |agent| {
        let source_text = types::String::from_string(agent, String::from("serve()"));
        let realm = agent.current_realm_id();

        let script = match parse_script(agent, source_text, realm, !runtime.config.no_strict, None)
        {
            Ok(script) => script,
            Err(_) => {
                panic!("error");
            }
        };

        match script_evaluation(agent, script) {
            Ok(value) => match value.to_string(agent) {
                Ok(str) => str.as_str(agent).to_owned(),
                _ => panic!("error"),
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
            let mut runtime = Runtime::new(RuntimeConfig {
                no_strict: false,
                paths: vec![path],
                verbose: false,
                extensions: recommended_extensions(),
                builtins: recommended_builtins(),
                eventloop_handler: recommended_eventloop_handler,
            });
            _ = runtime.run();

            let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
            let listener = TcpListener::bind(addr).await.unwrap();
            loop {
                let (stream, _) = listener.accept().await.unwrap();
                let io = TokioIo::new(stream);

                if let Err(err) = http1::Builder::new()
                    // `service_fn` converts our function in a `Service`
                    .serve_connection(
                        io,
                        service_fn(|request| {
                            let res = run_script_for_request(&mut runtime);
                            create_response(request, res)
                        }),
                    )
                    .await
                {
                    eprintln!("Error serving connection: {:?}", err);
                }
            }
        })
}
