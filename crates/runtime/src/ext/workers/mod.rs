// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, AtomicU32, Ordering},
    mpsc,
};
use std::thread::JoinHandle;

use andromeda_core::{
    Extension, ExtensionOp, HostData, MacroTask, OpsStorage, Runtime, RuntimeConfig,
};
use nova_vm::{
    ecmascript::{
        Agent, ArgumentsList, ExceptionType, Function, HostDefined, InternalMethods, JsResult,
        PropertyKey, String as NovaString, Value, parse_module, parse_script, script_evaluation,
    },
    engine::{Bindable, GcScope},
};

thread_local! {
    /// Carries worker init data from `run_worker_thread` into the
    /// `WorkersResources` storage hook that fires during `Runtime::new`.
    static WORKER_INIT: std::cell::RefCell<Option<WorkerInitData>> =
        const { std::cell::RefCell::new(None) };
}

struct WorkerInitData {
    name: String,
    outbound: mpsc::Sender<WorkerOutbound>,
}

use crate::RuntimeMacroTask;

#[derive(Debug)]
pub(crate) enum WorkerInbound {
    Message(String),
    Terminate,
}

#[derive(Debug)]
pub(crate) enum WorkerOutbound {
    Message(String),
    MessageError(String),
    UncaughtError {
        message: String,
        filename: String,
        lineno: u32,
        colno: u32,
    },
}

pub struct WorkerRecord {
    pub id: u32,
    pub name: String,
    pub(crate) tx_inbound: mpsc::Sender<WorkerInbound>,
    pub terminate_flag: Arc<AtomicBool>,
    pub thread_join: Mutex<Option<JoinHandle<()>>>,
}

#[derive(Default)]
pub struct WorkersResources {
    pub workers: Mutex<HashMap<u32, Arc<WorkerRecord>>>,
    pub next_worker_id: AtomicU32,
    /// `true` if this runtime is itself running as a worker.
    pub is_worker: AtomicBool,
    /// Outbound channel back to the parent. `None` on the main runtime.
    pub(crate) outbound_to_parent: Mutex<Option<mpsc::Sender<WorkerOutbound>>>,
    /// Worker-local close flag, set by `self.close()`.
    pub self_close_flag: Arc<AtomicBool>,
    /// Name of this worker (when running in worker mode). Empty in the parent.
    pub worker_name: Mutex<String>,
}

#[derive(Default)]
pub struct WorkersExt;

impl WorkersExt {
    #[hotpath::measure]
    pub fn new_extension() -> Extension {
        Extension {
            name: "workers",
            ops: vec![
                ExtensionOp::new("op_worker_spawn", Self::op_worker_spawn, 3, false),
                ExtensionOp::new(
                    "op_worker_post_to_worker",
                    Self::op_worker_post_to_worker,
                    2,
                    false,
                ),
                ExtensionOp::new(
                    "op_worker_post_to_parent",
                    Self::op_worker_post_to_parent,
                    1,
                    false,
                ),
                ExtensionOp::new(
                    "op_worker_post_messageerror_to_parent",
                    Self::op_worker_post_messageerror_to_parent,
                    1,
                    false,
                ),
                ExtensionOp::new("op_worker_terminate", Self::op_worker_terminate, 1, false),
                ExtensionOp::new("op_worker_close_self", Self::op_worker_close_self, 0, false),
                ExtensionOp::new("op_worker_is_worker", Self::op_worker_is_worker, 0, false),
                ExtensionOp::new("op_worker_get_name", Self::op_worker_get_name, 0, false),
            ],
            storage: Some(Box::new(|storage: &mut OpsStorage| {
                let res = WorkersResources::default();
                WORKER_INIT.with(|cell| {
                    if let Some(init) = cell.borrow_mut().take() {
                        res.is_worker.store(true, Ordering::Release);
                        *res.worker_name.lock().unwrap() = init.name;
                        *res.outbound_to_parent.lock().unwrap() = Some(init.outbound);
                    }
                });
                storage.insert(res);
            })),
            files: vec![
                include_str!("./worker.ts"),
                include_str!("./worker_global.ts"),
                include_str!("./message_channel.ts"),
            ],
        }
    }

    fn with_resources<R>(agent: &mut Agent, f: impl FnOnce(&WorkersResources) -> R) -> R {
        let host_data = agent.get_host_data();
        let host_data: &HostData<RuntimeMacroTask> = host_data.downcast_ref().unwrap();
        let storage = host_data.storage.borrow();
        let resources: &WorkersResources = storage.get().unwrap();
        f(resources)
    }

    /// spawns the worker threadand returns the worker id.
    pub fn op_worker_spawn<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let specifier_val = args.get(0).to_string(agent, gc.reborrow()).unbind()?;
        let specifier = specifier_val
            .as_str(agent)
            .expect("worker specifier is not valid UTF-8")
            .to_string();

        let name_val = args.get(1).to_string(agent, gc.reborrow()).unbind()?;
        let name = name_val
            .as_str(agent)
            .expect("worker name is not valid UTF-8")
            .to_string();

        let type_val = args.get(2).to_string(agent, gc.reborrow()).unbind()?;
        let worker_type = type_val
            .as_str(agent)
            .expect("worker type is not valid UTF-8")
            .to_string();

        if worker_type != "module" {
            let gc = gc.into_nogc();
            return Err(agent
                .throw_exception(
                    ExceptionType::TypeError,
                    "Andromeda only supports module workers (type: \"module\")".to_string(),
                    gc,
                )
                .unbind());
        }

        // HTTP/HTTPS worker scripts are not supported — the worker entry module loader uses local filesystem reads.
        if specifier.starts_with("http://") || specifier.starts_with("https://") {
            let gc = gc.into_nogc();
            return Err(agent
                .throw_exception(
                    ExceptionType::TypeError,
                    "Andromeda workers do not support HTTP(S) script URLs yet — \
                     use a local file path or file:// URL"
                        .to_string(),
                    gc,
                )
                .unbind());
        }

        // Resolve the script path relative to the cwd. Absolute paths pass through unchanged.
        let resolved = {
            let p = PathBuf::from(&specifier);
            if p.is_absolute() {
                p.to_string_lossy().to_string()
            } else {
                std::env::current_dir()
                    .unwrap_or_else(|_| PathBuf::from("."))
                    .join(&p)
                    .to_string_lossy()
                    .to_string()
            }
        };

        let (tx_inbound, rx_inbound) = mpsc::channel::<WorkerInbound>();
        let (tx_outbound, rx_outbound) = mpsc::channel::<WorkerOutbound>();
        let terminate_flag = Arc::new(AtomicBool::new(false));

        let worker_id = Self::with_resources(agent, |res| {
            let id = res.next_worker_id.fetch_add(1, Ordering::Relaxed);
            let record = Arc::new(WorkerRecord {
                id,
                name: name.clone(),
                tx_inbound: tx_inbound.clone(),
                terminate_flag: terminate_flag.clone(),
                thread_join: Mutex::new(None),
            });
            res.workers.lock().unwrap().insert(id, record);
            id
        });

        let parent_macro_tx = {
            let host_data = agent.get_host_data();
            let host_data: &HostData<RuntimeMacroTask> = host_data.downcast_ref().unwrap();
            host_data.macro_task_tx()
        };

        let parent_macro_count = {
            let host_data = agent.get_host_data();
            let host_data: &HostData<RuntimeMacroTask> = host_data.downcast_ref().unwrap();
            host_data.macro_task_count.clone()
        };
        parent_macro_count.fetch_add(1, Ordering::Release);

        let forwarder_macro_tx = parent_macro_tx.clone();
        std::thread::spawn(move || {
            while let Ok(msg) = rx_outbound.recv() {
                let task = match msg {
                    WorkerOutbound::Message(payload) => {
                        RuntimeMacroTask::WorkerDeliverMessage { worker_id, payload }
                    }
                    WorkerOutbound::MessageError(reason) => {
                        RuntimeMacroTask::WorkerDeliverMessageError { worker_id, reason }
                    }
                    WorkerOutbound::UncaughtError {
                        message,
                        filename,
                        lineno,
                        colno,
                    } => RuntimeMacroTask::WorkerDeliverError {
                        worker_id,
                        message,
                        filename,
                        lineno,
                        colno,
                    },
                };
                if forwarder_macro_tx.send(MacroTask::User(task)).is_err() {
                    break;
                }
            }
            parent_macro_count.fetch_sub(1, Ordering::Release);
            let _ =
                forwarder_macro_tx.send(MacroTask::User(RuntimeMacroTask::WorkerForwarderClosed {
                    worker_id,
                }));
        });

        let worker_terminate = terminate_flag.clone();
        let worker_name = name.clone();
        let tokio_handle = tokio::runtime::Handle::try_current().ok();
        let join = std::thread::Builder::new()
            .name(format!("andromeda-worker-{worker_id}"))
            .spawn(move || {
                let _tokio_guard = tokio_handle.as_ref().map(|h| h.enter());
                run_worker_thread(
                    worker_name,
                    resolved,
                    rx_inbound,
                    tx_outbound,
                    worker_terminate,
                );
            })
            .expect("failed to spawn worker thread");

        Self::with_resources(agent, |res| {
            let workers = res.workers.lock().unwrap();
            if let Some(record) = workers.get(&worker_id) {
                *record.thread_join.lock().unwrap() = Some(join);
            }
        });

        let gc = gc.into_nogc();
        Ok(Value::from_f64(agent, worker_id as f64, gc).unbind())
    }

    pub fn op_worker_post_to_worker<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let id = args.get(0).to_uint32(agent, gc.reborrow()).unwrap();
        let payload_val = args.get(1).to_string(agent, gc.reborrow()).unbind()?;
        let payload = payload_val
            .as_str(agent)
            .expect("worker payload is not valid UTF-8")
            .to_string();

        Self::with_resources(agent, |res| {
            let workers = res.workers.lock().unwrap();
            if let Some(record) = workers.get(&id)
                && !record.terminate_flag.load(Ordering::Acquire)
            {
                let _ = record.tx_inbound.send(WorkerInbound::Message(payload));
            }
        });

        Ok(Value::Undefined)
    }

    pub fn op_worker_post_to_parent<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let payload_val = args.get(0).to_string(agent, gc.reborrow()).unbind()?;
        let payload = payload_val
            .as_str(agent)
            .expect("worker payload is not valid UTF-8")
            .to_string();

        Self::with_resources(agent, |res| {
            if let Some(tx) = res.outbound_to_parent.lock().unwrap().as_ref() {
                let _ = tx.send(WorkerOutbound::Message(payload));
            }
        });

        Ok(Value::Undefined)
    }

    pub fn op_worker_post_messageerror_to_parent<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let reason_val = args.get(0).to_string(agent, gc.reborrow()).unbind()?;
        let reason = reason_val
            .as_str(agent)
            .expect("reason is not valid UTF-8")
            .to_string();
        Self::with_resources(agent, |res| {
            if let Some(tx) = res.outbound_to_parent.lock().unwrap().as_ref() {
                let _ = tx.send(WorkerOutbound::MessageError(reason));
            }
        });
        Ok(Value::Undefined)
    }

    pub fn op_worker_terminate<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let id = args.get(0).to_uint32(agent, gc.reborrow()).unwrap();

        Self::with_resources(agent, |res| {
            let mut workers = res.workers.lock().unwrap();
            if let Some(record) = workers.remove(&id) {
                record.terminate_flag.store(true, Ordering::Release);
                let _ = record.tx_inbound.send(WorkerInbound::Terminate);
                drop(record);
            }
        });

        Ok(Value::Undefined)
    }

    pub fn op_worker_close_self<'gc>(
        agent: &mut Agent,
        _this: Value,
        _args: ArgumentsList,
        _gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        Self::with_resources(agent, |res| {
            res.self_close_flag.store(true, Ordering::Release);
        });
        let host_data = agent.get_host_data();
        let host_data: &HostData<RuntimeMacroTask> = host_data.downcast_ref().unwrap();
        let _ = host_data
            .macro_task_tx
            .send(MacroTask::User(RuntimeMacroTask::WorkerSelfClose));
        Ok(Value::Undefined)
    }

    pub fn op_worker_is_worker<'gc>(
        agent: &mut Agent,
        _this: Value,
        _args: ArgumentsList,
        _gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let is_worker = Self::with_resources(agent, |res| res.is_worker.load(Ordering::Acquire));
        Ok(if is_worker {
            Value::Boolean(true)
        } else {
            Value::Boolean(false)
        })
    }

    pub fn op_worker_get_name<'gc>(
        agent: &mut Agent,
        _this: Value,
        _args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let name = Self::with_resources(agent, |res| res.worker_name.lock().unwrap().clone());
        let gc = gc.into_nogc();
        Ok(Value::from_string(agent, name, gc).unbind())
    }
}

fn run_worker_thread(
    name: String,
    script_path: String,
    rx_inbound: mpsc::Receiver<WorkerInbound>,
    tx_outbound: mpsc::Sender<WorkerOutbound>,
    terminate_flag: Arc<AtomicBool>,
) {
    let (worker_macro_tx, worker_macro_rx) = mpsc::channel();
    let host_data = HostData::<RuntimeMacroTask>::new(worker_macro_tx.clone());

    let extensions = worker_recommended_extensions();

    host_data.macro_task_count.fetch_add(1, Ordering::Release);

    {
        let forwarder_tx = worker_macro_tx.clone();
        std::thread::spawn(move || {
            let mut sent_close = false;
            while let Ok(msg) = rx_inbound.recv() {
                let (task, is_close) = match msg {
                    WorkerInbound::Message(payload) => (
                        RuntimeMacroTask::WorkerSelfDeliverMessage { payload },
                        false,
                    ),
                    WorkerInbound::Terminate => (RuntimeMacroTask::WorkerSelfClose, true),
                };
                if forwarder_tx.send(MacroTask::User(task)).is_err() {
                    return;
                }
                if is_close {
                    sent_close = true;
                    break;
                }
            }
            if !sent_close {
                let _ = forwarder_tx.send(MacroTask::User(RuntimeMacroTask::WorkerSelfClose));
            }
        });
    }

    let config = RuntimeConfig {
        no_strict: false,
        files: vec![],
        verbose: false,
        extensions,
        builtins: vec![],
        eventloop_handler: crate::recommended_eventloop_handler,
        macro_task_rx: worker_macro_rx,
        import_map: None,
        pre_tick_hook: None,
    };

    WORKER_INIT.with(|cell| {
        *cell.borrow_mut() = Some(WorkerInitData {
            name: name.clone(),
            outbound: tx_outbound.clone(),
        });
    });

    let mut runtime = Runtime::new(config, host_data);

    let worker_name_for_init = name.clone();
    runtime
        .agent
        .run_in_realm(&runtime.realm_root, |agent, mut gc| {
            let global_obj = agent.current_realm(gc.nogc()).global_object(agent).unbind();
            let key =
                PropertyKey::from_static_str(agent, "__andromeda_init_worker_globals", gc.nogc())
                    .unbind();
            let init_fn = global_obj
                .internal_get(agent, key, Value::from(global_obj), gc.reborrow())
                .unbind();
            if let Ok(init_fn) = init_fn
                && let Ok(func) = Function::try_from(init_fn)
            {
                let name_val = Value::from_string(agent, worker_name_for_init, gc.nogc()).unbind();
                let _ = func.call(agent, Value::Undefined, &mut [name_val], gc.reborrow());
            }
        });

    runtime
        .agent
        .run_in_realm(&runtime.realm_root, |agent, mut gc| {
            for builtin in &[include_str!("../../../../../namespace/mod.ts")] {
                let realm = agent.current_realm(gc.nogc());
                let source_text = NovaString::from_str(agent, builtin, gc.nogc());
                let script = match parse_script(agent, source_text, realm, true, None, gc.nogc()) {
                    Ok(s) => s,
                    Err(_) => continue,
                };
                let _ = script_evaluation(agent, script.unbind(), gc.reborrow()).unbind();
            }
        });

    let script_path_clone = script_path.clone();
    let entry_result: Result<(), String> =
        runtime
            .agent
            .run_in_realm(&runtime.realm_root, |agent, mut gc| {
                let source = match std::fs::read_to_string(&script_path_clone) {
                    Ok(s) => s,
                    Err(e) => {
                        return Err(format!(
                            "Failed to read worker script {script_path_clone}: {e}"
                        ));
                    }
                };
                let source_text = NovaString::from_string(agent, source, gc.nogc());
                let realm = agent.current_realm(gc.nogc());
                let host_defined = Some(std::rc::Rc::new(script_path_clone.clone()) as HostDefined);
                let module = match parse_module(agent, source_text, realm, host_defined, gc.nogc())
                {
                    Ok(m) => m,
                    Err(diagnostics) => {
                        return Err(format!(
                            "Parse error in worker module {script_path_clone}: {diagnostics:?}"
                        ));
                    }
                };
                match agent
                    .run_module(module.unbind(), None, gc.reborrow())
                    .unbind()
                {
                    Ok(_) => Ok(()),
                    Err(e) => {
                        let msg = e
                            .value()
                            .string_repr(agent, gc.reborrow())
                            .as_str(agent)
                            .unwrap_or("<non-string error>")
                            .to_string();
                        Err(msg)
                    }
                }
            });

    let leaked_host_hooks = runtime.host_hooks;

    let entry_failed = entry_result.is_err();
    if let Err(msg) = entry_result {
        let _ = tx_outbound.send(WorkerOutbound::UncaughtError {
            message: msg,
            filename: script_path.clone(),
            lineno: 0,
            colno: 0,
        });
        terminate_flag.store(true, Ordering::Release);
    }
    drop(tx_outbound);

    if entry_failed {
        clear_leaked_outbound(leaked_host_hooks);
        return;
    }

    // Run the worker's main loop. Nova will return RuntimeOutput at end.
    let _ = runtime.run();
    clear_leaked_outbound(leaked_host_hooks);

    // Mark this worker as terminated.
    terminate_flag.store(true, Ordering::Release);
}

fn clear_leaked_outbound(host_hooks: &andromeda_core::RuntimeHostHooks<RuntimeMacroTask>) {
    let storage = host_hooks.host_data.storage.borrow();
    if let Some(res) = storage.get::<WorkersResources>() {
        let _ = res.outbound_to_parent.lock().unwrap().take();
    }
}

/// Build the recommended extension set for a worker realm. Same as the
/// main runtime's `recommended_extensions()` but without `WindowExt`
pub fn worker_recommended_extensions() -> Vec<Extension> {
    #[cfg(not(feature = "virtualfs"))]
    use crate::FsExt;
    #[cfg(feature = "virtualfs")]
    use crate::VirtualFsExt;
    use crate::{
        BroadcastChannelExt, CommandExt, ConsoleExt, CronExt, FetchExt, FfiExt, FileExt, NetExt,
        ProcessExt, StreamsExt, TimeExt, TlsExt, URLExt, WebExt, WebIDLExt, WebLocksExt,
    };

    vec![
        WebIDLExt::new_extension(),
        #[cfg(not(feature = "virtualfs"))]
        FsExt::new_extension(),
        #[cfg(feature = "virtualfs")]
        VirtualFsExt::new_extension(),
        ConsoleExt::new_extension(),
        TimeExt::new_extension(),
        CronExt::new_extension(),
        ProcessExt::new_extension(),
        CommandExt::new_extension(),
        URLExt::new_extension(),
        WebExt::new_extension(),
        WebLocksExt::new_extension(),
        FileExt::new_extension(),
        BroadcastChannelExt::new_extension(),
        FetchExt::new_extension(),
        NetExt::new_extension(),
        StreamsExt::new_extension(),
        TlsExt::new_extension(),
        FfiExt::new_extension(),
        // Workers also serve HTTP — required for `Andromeda.serve({ parallel })`.
        #[cfg(feature = "serve")]
        crate::ServeExt::new_extension(),
        #[cfg(feature = "canvas")]
        crate::CanvasExt::new_extension(),
        #[cfg(feature = "crypto")]
        crate::CryptoExt::new_extension(),
        #[cfg(feature = "storage")]
        crate::LocalStorageExt::new_extension(),
        #[cfg(feature = "storage")]
        crate::SqliteExt::new_extension(),
        #[cfg(feature = "storage")]
        crate::CacheStorageExt::new_extension(),
        // Workers can have workers.
        WorkersExt::new_extension(),
    ]
}

/// Resolve a function stashed on `globalThis` by property name and call
/// it with the given string arguments. Errors are silently dropped.
fn call_global_function(
    agent: &mut nova_vm::ecmascript::GcAgent,
    realm_root: &nova_vm::ecmascript::RealmRoot,
    property_name: &'static str,
    prefix_args_f64: &[f64],
    prefix_args_str: &[&str],
    payload_args: Vec<String>,
) {
    let _ = call_global_function_capturing_err(
        agent,
        realm_root,
        property_name,
        prefix_args_f64,
        prefix_args_str,
        payload_args,
    );
}

fn call_global_function_capturing_err(
    agent: &mut nova_vm::ecmascript::GcAgent,
    realm_root: &nova_vm::ecmascript::RealmRoot,
    property_name: &'static str,
    prefix_args_f64: &[f64],
    prefix_args_str: &[&str],
    payload_args: Vec<String>,
) -> Option<String> {
    agent.run_in_realm(realm_root, |agent, mut gc| -> Option<String> {
        let global_obj = agent.current_realm(gc.nogc()).global_object(agent).unbind();
        let key = PropertyKey::from_static_str(agent, property_name, gc.nogc()).unbind();
        let value = global_obj
            .internal_get(agent, key, Value::from(global_obj), gc.reborrow())
            .ok()?
            .unbind();
        let func: Function = value.try_into().ok()?;

        let mut js_args: Vec<Value<'static>> = Vec::new();
        for v in prefix_args_f64 {
            js_args.push(Value::from_f64(agent, *v, gc.nogc()).unbind());
        }
        for s in prefix_args_str {
            js_args.push(Value::from_string(agent, s.to_string(), gc.nogc()).unbind());
        }
        for s in payload_args.into_iter() {
            js_args.push(Value::from_string(agent, s, gc.nogc()).unbind());
        }

        let call_result = func
            .call(agent, Value::Undefined, &mut js_args[..], gc.reborrow())
            .map(|_| ())
            .map_err(|e| e.unbind());
        match call_result {
            Ok(()) => None,
            Err(e) => Some(
                e.value()
                    .string_repr(agent, gc.reborrow())
                    .as_str(agent)
                    .unwrap_or("<non-string error>")
                    .to_string(),
            ),
        }
    })
}

pub fn dispatch_parent_event(
    agent: &mut nova_vm::ecmascript::GcAgent,
    realm_root: &nova_vm::ecmascript::RealmRoot,
    worker_id: u32,
    kind: &'static str,
    payload_args: Vec<String>,
) {
    call_global_function(
        agent,
        realm_root,
        "__andromeda_dispatch_worker_event",
        &[worker_id as f64],
        &[kind],
        payload_args,
    );
}

pub fn dispatch_self_event(
    agent: &mut nova_vm::ecmascript::GcAgent,
    realm_root: &nova_vm::ecmascript::RealmRoot,
    host_data: &HostData<RuntimeMacroTask>,
    kind: &'static str,
    payload_args: Vec<String>,
) {
    let err_msg = call_global_function_capturing_err(
        agent,
        realm_root,
        "__andromeda_dispatch_self_event",
        &[],
        &[kind],
        payload_args,
    );
    if let Some(msg) = err_msg {
        // We are inside the worker — forward to the parent as ErrorEvent.
        let storage = host_data.storage.borrow();
        if let Some(res) = storage.get::<WorkersResources>()
            && let Some(tx) = res.outbound_to_parent.lock().unwrap().as_ref()
        {
            let _ = tx.send(WorkerOutbound::UncaughtError {
                message: msg,
                filename: String::new(),
                lineno: 0,
                colno: 0,
            });
        }
    }
}
