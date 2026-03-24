// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::collections::HashMap;
use std::process::Stdio;

use andromeda_core::{
    ErrorReporter, Extension, ExtensionOp, HostData, MacroTask, OpsStorage, RuntimeError,
};
use nova_vm::{
    ecmascript::{Agent, ArgumentsList, JsResult, PromiseCapability, Value},
    engine::{Bindable, GcScope, Global},
};
use tokio::process::Command as TokioCommand;

use crate::RuntimeMacroTask;

struct CommandExtResources {
    child_processes: HashMap<u32, tokio::process::Child>,
    next_id: u32,
}

impl Default for CommandExtResources {
    fn default() -> Self {
        Self {
            child_processes: HashMap::new(),
            next_id: 1,
        }
    }
}

/// Parsed command options from JSON.
struct CommandOpts {
    args: Vec<String>,
    cwd: String,
    env: HashMap<String, String>,
    clear_env: bool,
    stdin: String,
    stdout: String,
    stderr: String,
    #[cfg(unix)]
    uid: Option<u32>,
    #[cfg(unix)]
    gid: Option<u32>,
    #[cfg(windows)]
    windows_raw_arguments: bool,
}

impl CommandOpts {
    fn from_json(json_str: &str) -> Self {
        // Minimal JSON parser for our known structure.
        // We parse manually to avoid adding serde as a dependency to the runtime.
        let mut opts = CommandOpts {
            args: Vec::new(),
            cwd: String::new(),
            env: HashMap::new(),
            clear_env: false,
            stdin: "null".to_string(),
            stdout: "piped".to_string(),
            stderr: "piped".to_string(),
            #[cfg(unix)]
            uid: None,
            #[cfg(unix)]
            gid: None,
            #[cfg(windows)]
            windows_raw_arguments: false,
        };

        if json_str.is_empty() {
            return opts;
        }

        // Use serde_json which is already in the workspace
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(json_str) {
            if let Some(args) = value.get("args").and_then(|v| v.as_array()) {
                opts.args = args
                    .iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect();
            }
            if let Some(cwd) = value.get("cwd").and_then(|v| v.as_str()) {
                opts.cwd = cwd.to_string();
            }
            if let Some(env) = value.get("env").and_then(|v| v.as_object()) {
                for (k, v) in env {
                    if let Some(val) = v.as_str() {
                        opts.env.insert(k.clone(), val.to_string());
                    }
                }
            }
            if let Some(clear) = value.get("clearEnv").and_then(|v| v.as_bool()) {
                opts.clear_env = clear;
            }
            if let Some(s) = value.get("stdin").and_then(|v| v.as_str()) {
                opts.stdin = s.to_string();
            }
            if let Some(s) = value.get("stdout").and_then(|v| v.as_str()) {
                opts.stdout = s.to_string();
            }
            if let Some(s) = value.get("stderr").and_then(|v| v.as_str()) {
                opts.stderr = s.to_string();
            }
            #[cfg(unix)]
            {
                if let Some(uid) = value.get("uid").and_then(|v| v.as_u64()) {
                    opts.uid = Some(uid as u32);
                }
                if let Some(gid) = value.get("gid").and_then(|v| v.as_u64()) {
                    opts.gid = Some(gid as u32);
                }
            }
            #[cfg(windows)]
            {
                if let Some(raw) = value.get("windowsRawArguments").and_then(|v| v.as_bool()) {
                    opts.windows_raw_arguments = raw;
                }
            }
        }

        opts
    }
}

fn parse_stdio(s: &str) -> Stdio {
    match s {
        "inherit" => Stdio::inherit(),
        "null" => Stdio::null(),
        _ => Stdio::piped(),
    }
}

fn escape_json_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

#[cfg(unix)]
fn get_signal_name(status: &std::process::ExitStatus) -> Option<&'static str> {
    use std::os::unix::process::ExitStatusExt;
    status.signal().map(|sig| match sig {
        1 => "SIGHUP",
        2 => "SIGINT",
        3 => "SIGQUIT",
        6 => "SIGABRT",
        9 => "SIGKILL",
        11 => "SIGSEGV",
        13 => "SIGPIPE",
        14 => "SIGALRM",
        15 => "SIGTERM",
        _ => "UNKNOWN",
    })
}

#[cfg(windows)]
fn get_signal_name(_status: &std::process::ExitStatus) -> Option<&'static str> {
    None
}

fn build_output_json(
    status: &std::process::ExitStatus,
    stdout: &[u8],
    stderr: &[u8],
    stdout_mode: &str,
    stderr_mode: &str,
) -> String {
    let code = status.code().unwrap_or(-1);
    let success = status.success();
    // Only include captured output if the mode was "piped"
    let stdout_str = if stdout_mode == "piped" {
        escape_json_string(&String::from_utf8_lossy(stdout))
    } else {
        String::new()
    };
    let stderr_str = if stderr_mode == "piped" {
        escape_json_string(&String::from_utf8_lossy(stderr))
    } else {
        String::new()
    };
    let signal = match get_signal_name(status) {
        Some(name) => format!("\"{}\"", name),
        None => "null".to_string(),
    };

    format!(
        "{{\"success\":{success},\"code\":{code},\"signal\":{signal},\"stdout\":\"{stdout_str}\",\"stderr\":\"{stderr_str}\"}}"
    )
}

fn build_status_json(status: &std::process::ExitStatus) -> String {
    let code = status.code().unwrap_or(-1);
    let success = status.success();
    let signal = match get_signal_name(status) {
        Some(name) => format!("\"{}\"", name),
        None => "null".to_string(),
    };
    format!("{{\"success\":{success},\"code\":{code},\"signal\":{signal}}}")
}

/// Command extension
/// This extension provides the ability to spawn and interact with subprocesses.
#[derive(Default)]
pub struct CommandExt;

#[hotpath::measure_all]
impl CommandExt {
    pub fn new_extension() -> Extension {
        Extension {
            name: "command",
            ops: vec![
                ExtensionOp::new(
                    "internal_command_output_sync",
                    Self::internal_command_output_sync,
                    2,
                    false,
                ),
                ExtensionOp::new(
                    "internal_command_output",
                    Self::internal_command_output,
                    2,
                    false,
                ),
                ExtensionOp::new(
                    "internal_command_spawn",
                    Self::internal_command_spawn,
                    2,
                    false,
                ),
                ExtensionOp::new(
                    "internal_command_kill",
                    Self::internal_command_kill,
                    2,
                    false,
                ),
                ExtensionOp::new(
                    "internal_command_wait",
                    Self::internal_command_wait,
                    1,
                    false,
                ),
                ExtensionOp::new(
                    "internal_command_child_output",
                    Self::internal_command_child_output,
                    1,
                    false,
                ),
            ],
            storage: Some(Box::new(|storage: &mut OpsStorage| {
                storage.insert(CommandExtResources::default());
            })),
            files: vec![],
        }
    }

    fn apply_opts_to_std_command(cmd: &mut std::process::Command, opts: &CommandOpts) {
        if !opts.args.is_empty() {
            cmd.args(&opts.args);
        }
        if !opts.cwd.is_empty() {
            cmd.current_dir(&opts.cwd);
        }
        if opts.clear_env {
            cmd.env_clear();
        }
        for (k, v) in &opts.env {
            cmd.env(k, v);
        }
        cmd.stdin(parse_stdio(&opts.stdin));
        cmd.stdout(parse_stdio(&opts.stdout));
        cmd.stderr(parse_stdio(&opts.stderr));
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt as UnixCommandExt;
            if let Some(uid) = opts.uid {
                cmd.uid(uid);
            }
            if let Some(gid) = opts.gid {
                cmd.gid(gid);
            }
        }
        #[cfg(windows)]
        {
            if opts.windows_raw_arguments {
                use std::os::windows::process::CommandExt as WinCommandExt;
                cmd.raw_arg(opts.args.join(" "));
            }
        }
    }

    fn apply_opts_to_tokio_command(cmd: &mut TokioCommand, opts: &CommandOpts) {
        if !opts.args.is_empty() {
            cmd.args(&opts.args);
        }
        if !opts.cwd.is_empty() {
            cmd.current_dir(&opts.cwd);
        }
        if opts.clear_env {
            cmd.env_clear();
        }
        for (k, v) in &opts.env {
            cmd.env(k, v);
        }
        cmd.stdin(parse_stdio(&opts.stdin));
        cmd.stdout(parse_stdio(&opts.stdout));
        cmd.stderr(parse_stdio(&opts.stderr));
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt as UnixCommandExt;
            if let Some(uid) = opts.uid {
                cmd.uid(uid);
            }
            if let Some(gid) = opts.gid {
                cmd.gid(gid);
            }
        }
    }

    /// Synchronously runs a command and returns its output as a JSON string.
    /// args[0] = program, args[1] = options JSON
    fn internal_command_output_sync<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let program_binding = args.get(0).to_string(agent, gc.reborrow()).unbind()?;
        let program = program_binding
            .as_str(agent)
            .expect("String is not valid UTF-8");

        let opts_binding = args.get(1).to_string(agent, gc.reborrow()).unbind()?;
        let opts_str = opts_binding
            .as_str(agent)
            .expect("String is not valid UTF-8");

        let opts = CommandOpts::from_json(opts_str);
        let mut cmd = std::process::Command::new(program);
        Self::apply_opts_to_std_command(&mut cmd, &opts);

        match cmd.output() {
            Ok(output) => {
                let json = build_output_json(
                    &output.status,
                    &output.stdout,
                    &output.stderr,
                    &opts.stdout,
                    &opts.stderr,
                );
                Ok(Value::from_string(agent, json, gc.nogc()).unbind())
            }
            Err(e) => {
                let error = RuntimeError::runtime_error(format!(
                    "Failed to execute command '{}': {}",
                    program, e
                ));
                let error_msg = ErrorReporter::format_error(&error);
                Ok(Value::from_string(agent, format!("Error: {error_msg}"), gc.nogc()).unbind())
            }
        }
    }

    /// Asynchronously runs a command and returns a promise resolving to output JSON.
    /// args[0] = program, args[1] = options JSON
    fn internal_command_output<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let program_binding = args.get(0).to_string(agent, gc.reborrow()).unbind()?;
        let program = program_binding
            .as_str(agent)
            .expect("String is not valid UTF-8")
            .to_string();

        let opts_binding = args.get(1).to_string(agent, gc.reborrow()).unbind()?;
        let opts_str = opts_binding
            .as_str(agent)
            .expect("String is not valid UTF-8")
            .to_string();

        let promise_capability = PromiseCapability::new(agent, gc.nogc());
        let root_value = Global::new(agent, Value::from(promise_capability.promise()).unbind());
        let host_data = agent.get_host_data();
        let host_data: &HostData<RuntimeMacroTask> = host_data.downcast_ref().unwrap();
        let macro_task_tx = host_data.macro_task_tx();

        host_data.spawn_macro_task(async move {
            let opts = CommandOpts::from_json(&opts_str);
            let mut cmd = TokioCommand::new(&program);
            Self::apply_opts_to_tokio_command(&mut cmd, &opts);

            match cmd.output().await {
                Ok(output) => {
                    let json = build_output_json(
                        &output.status,
                        &output.stdout,
                        &output.stderr,
                        &opts.stdout,
                        &opts.stderr,
                    );
                    macro_task_tx
                        .send(MacroTask::User(RuntimeMacroTask::ResolvePromiseWithString(
                            root_value, json,
                        )))
                        .unwrap();
                }
                Err(e) => {
                    let error = RuntimeError::runtime_error(format!(
                        "Failed to execute command '{}': {}",
                        program, e
                    ));
                    let error_msg = ErrorReporter::format_error(&error);
                    macro_task_tx
                        .send(MacroTask::User(RuntimeMacroTask::RejectPromise(
                            root_value,
                            format!("Error: {error_msg}"),
                        )))
                        .unwrap();
                }
            }
        });

        Ok(Value::Promise(promise_capability.promise()).unbind())
    }

    /// Spawns a child process. Returns JSON: {"rid":number,"pid":number|null}
    /// args[0] = program, args[1] = options JSON
    fn internal_command_spawn<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let program_binding = args.get(0).to_string(agent, gc.reborrow()).unbind()?;
        let program = program_binding
            .as_str(agent)
            .expect("String is not valid UTF-8");

        let opts_binding = args.get(1).to_string(agent, gc.reborrow()).unbind()?;
        let opts_str = opts_binding
            .as_str(agent)
            .expect("String is not valid UTF-8");

        let opts = CommandOpts::from_json(opts_str);
        let mut cmd = TokioCommand::new(program);
        Self::apply_opts_to_tokio_command(&mut cmd, &opts);

        match cmd.spawn() {
            Ok(child) => {
                let os_pid = child.id();
                let rid = {
                    let host_data = agent.get_host_data();
                    let host_data: &HostData<RuntimeMacroTask> =
                        host_data.downcast_ref().unwrap();
                    let mut storage = host_data.storage.borrow_mut();
                    let resources: &mut CommandExtResources = storage.get_mut().unwrap();

                    let rid = resources.next_id;
                    resources.next_id += 1;
                    resources.child_processes.insert(rid, child);
                    rid
                };

                let pid_json = match os_pid {
                    Some(p) => p.to_string(),
                    None => "null".to_string(),
                };
                let json = format!("{{\"rid\":{rid},\"pid\":{pid_json}}}");
                Ok(Value::from_string(agent, json, gc.nogc()).unbind())
            }
            Err(e) => {
                let error = RuntimeError::runtime_error(format!(
                    "Failed to spawn command '{}': {}",
                    program, e
                ));
                let error_msg = ErrorReporter::format_error(&error);
                Ok(Value::from_string(agent, format!("Error: {error_msg}"), gc.nogc()).unbind())
            }
        }
    }

    /// Kills a spawned child process.
    /// args[0] = rid (string), args[1] = signal (string, e.g. "SIGTERM" or empty)
    fn internal_command_kill<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let rid_binding = args.get(0).to_string(agent, gc.reborrow()).unbind()?;
        let rid_str = rid_binding
            .as_str(agent)
            .expect("String is not valid UTF-8");
        let rid: u32 = match rid_str.parse() {
            Ok(p) => p,
            Err(_) => {
                let error = RuntimeError::runtime_error("Invalid process ID");
                let error_msg = ErrorReporter::format_error(&error);
                return Ok(
                    Value::from_string(agent, format!("Error: {error_msg}"), gc.nogc()).unbind(),
                );
            }
        };

        // Signal argument (unused on Windows for now, but parsed for API parity)
        let _signal_binding = args.get(1).to_string(agent, gc.reborrow()).unbind()?;

        let kill_result = {
            let host_data = agent.get_host_data();
            let host_data: &HostData<RuntimeMacroTask> = host_data.downcast_ref().unwrap();
            let mut storage = host_data.storage.borrow_mut();
            let resources: &mut CommandExtResources = storage.get_mut().unwrap();

            match resources.child_processes.get_mut(&rid) {
                Some(child) => {
                    // On Unix we could send specific signals, but tokio's start_kill
                    // sends SIGKILL. For SIGTERM parity we use start_kill which is
                    // the cross-platform approach.
                    child.start_kill().map_err(|e| e.to_string())
                }
                None => Err(format!("No child process found with ID {}", rid)),
            }
        };

        match kill_result {
            Ok(()) => Ok(Value::Undefined),
            Err(e) => {
                let error = RuntimeError::runtime_error(e);
                let error_msg = ErrorReporter::format_error(&error);
                Ok(
                    Value::from_string(agent, format!("Error: {error_msg}"), gc.nogc())
                        .unbind(),
                )
            }
        }
    }

    /// Waits for a spawned child process to complete. Returns a promise with status JSON.
    /// args[0] = rid (string)
    fn internal_command_wait<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let rid_binding = args.get(0).to_string(agent, gc.reborrow()).unbind()?;
        let rid_str = rid_binding
            .as_str(agent)
            .expect("String is not valid UTF-8");
        let rid: u32 = match rid_str.parse() {
            Ok(p) => p,
            Err(_) => {
                let promise_capability = PromiseCapability::new(agent, gc.nogc());
                let root_value =
                    Global::new(agent, Value::from(promise_capability.promise()).unbind());
                let host_data = agent.get_host_data();
                let host_data: &HostData<RuntimeMacroTask> = host_data.downcast_ref().unwrap();
                let macro_task_tx = host_data.macro_task_tx();
                macro_task_tx
                    .send(MacroTask::User(RuntimeMacroTask::RejectPromise(
                        root_value,
                        "Error: Invalid process ID".to_string(),
                    )))
                    .unwrap();
                return Ok(Value::Promise(promise_capability.promise()).unbind());
            }
        };

        let (child, macro_task_tx) = {
            let host_data = agent.get_host_data();
            let host_data: &HostData<RuntimeMacroTask> = host_data.downcast_ref().unwrap();
            let child = {
                let mut storage = host_data.storage.borrow_mut();
                let resources: &mut CommandExtResources = storage.get_mut().unwrap();
                resources.child_processes.remove(&rid)
            };
            (child, host_data.macro_task_tx())
        };

        let promise_capability = PromiseCapability::new(agent, gc.nogc());
        let root_value = Global::new(agent, Value::from(promise_capability.promise()).unbind());

        match child {
            Some(mut child) => {
                let host_data = agent.get_host_data();
                let host_data: &HostData<RuntimeMacroTask> = host_data.downcast_ref().unwrap();
                host_data.spawn_macro_task(async move {
                    match child.wait().await {
                        Ok(status) => {
                            let json = build_status_json(&status);
                            macro_task_tx
                                .send(MacroTask::User(
                                    RuntimeMacroTask::ResolvePromiseWithString(root_value, json),
                                ))
                                .unwrap();
                        }
                        Err(e) => {
                            macro_task_tx
                                .send(MacroTask::User(RuntimeMacroTask::RejectPromise(
                                    root_value,
                                    format!("Error: Failed to wait for process: {}", e),
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
                        format!("Error: No child process found with ID {}", rid),
                    )))
                    .unwrap();
            }
        }

        Ok(Value::Promise(promise_capability.promise()).unbind())
    }

    /// Waits for a child and collects all output. Returns a promise with full output JSON.
    /// args[0] = rid (string)
    fn internal_command_child_output<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let rid_binding = args.get(0).to_string(agent, gc.reborrow()).unbind()?;
        let rid_str = rid_binding
            .as_str(agent)
            .expect("String is not valid UTF-8");
        let rid: u32 = match rid_str.parse() {
            Ok(p) => p,
            Err(_) => {
                let promise_capability = PromiseCapability::new(agent, gc.nogc());
                let root_value =
                    Global::new(agent, Value::from(promise_capability.promise()).unbind());
                let host_data = agent.get_host_data();
                let host_data: &HostData<RuntimeMacroTask> = host_data.downcast_ref().unwrap();
                let macro_task_tx = host_data.macro_task_tx();
                macro_task_tx
                    .send(MacroTask::User(RuntimeMacroTask::RejectPromise(
                        root_value,
                        "Error: Invalid process ID".to_string(),
                    )))
                    .unwrap();
                return Ok(Value::Promise(promise_capability.promise()).unbind());
            }
        };

        let (child, macro_task_tx) = {
            let host_data = agent.get_host_data();
            let host_data: &HostData<RuntimeMacroTask> = host_data.downcast_ref().unwrap();
            let child = {
                let mut storage = host_data.storage.borrow_mut();
                let resources: &mut CommandExtResources = storage.get_mut().unwrap();
                resources.child_processes.remove(&rid)
            };
            (child, host_data.macro_task_tx())
        };

        let promise_capability = PromiseCapability::new(agent, gc.nogc());
        let root_value = Global::new(agent, Value::from(promise_capability.promise()).unbind());

        match child {
            Some(child) => {
                let host_data = agent.get_host_data();
                let host_data: &HostData<RuntimeMacroTask> = host_data.downcast_ref().unwrap();
                host_data.spawn_macro_task(async move {
                    match child.wait_with_output().await {
                        Ok(output) => {
                            let json = build_output_json(
                                &output.status,
                                &output.stdout,
                                &output.stderr,
                                "piped",
                                "piped",
                            );
                            macro_task_tx
                                .send(MacroTask::User(
                                    RuntimeMacroTask::ResolvePromiseWithString(root_value, json),
                                ))
                                .unwrap();
                        }
                        Err(e) => {
                            macro_task_tx
                                .send(MacroTask::User(RuntimeMacroTask::RejectPromise(
                                    root_value,
                                    format!("Error: Failed to get process output: {}", e),
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
                        format!("Error: No child process found with ID {}", rid),
                    )))
                    .unwrap();
            }
        }

        Ok(Value::Promise(promise_capability.promise()).unbind())
    }
}
