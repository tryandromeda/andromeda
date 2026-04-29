// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use andromeda_core::{Extension, ExtensionOp};
use nova_vm::{
    ecmascript::{Agent, ArgumentsList, JsResult, Value},
    engine::{Bindable, GcScope},
};
use std::os::fd::AsRawFd;
use sysinfo::System;

/// OS extension for Andromeda.
#[derive(Default)]
pub struct OsExt;

impl OsExt {
    pub fn new_extension() -> Extension {
        Extension {
            name: "os",
            ops: vec![
                ExtensionOp::new("internal_os_hostname", Self::hostname, 0, false),
                ExtensionOp::new("internal_os_release", Self::os_release, 0, false),
                ExtensionOp::new("internal_os_name", Self::os_name, 0, false),
                ExtensionOp::new("internal_os_uptime", Self::os_uptime, 0, false),
                ExtensionOp::new("internal_os_loadavg", Self::loadavg, 0, false),
                ExtensionOp::new("internal_os_memory_usage", Self::memory_usage, 0, false),
                ExtensionOp::new("internal_os_console_size", Self::console_size, 0, false),
            ],
            storage: None,
            files: vec![include_str!("./mod.ts")],
        }
    }

    /// `Andromeda.hostname() -> string`
    fn hostname<'gc>(
        agent: &mut Agent,
        _this: Value,
        _args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let name = System::host_name().unwrap_or_else(|| "unknown".to_string());
        Ok(Value::from_string(agent, name, gc.nogc()).unbind())
    }

    /// `Andromeda.osRelease() -> string` — kernel version string (e.g. `"23.4.0"` on macOS).
    fn os_release<'gc>(
        agent: &mut Agent,
        _this: Value,
        _args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let release = System::kernel_version().unwrap_or_else(|| "unknown".to_string());
        Ok(Value::from_string(agent, release, gc.nogc()).unbind())
    }

    /// Operating system identifier — matches Deno's `Deno.build.os`
    fn os_name<'gc>(
        agent: &mut Agent,
        _this: Value,
        _args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        // `std::env::consts::OS` gives "macos" / "linux" / "windows";
        // normalize the macOS name to match Deno ("darwin").
        let name = match std::env::consts::OS {
            "macos" => "darwin",
            other => other,
        };
        Ok(Value::from_string(agent, name.to_string(), gc.nogc()).unbind())
    }

    /// `Andromeda.osUptime() -> number` — seconds since boot.
    fn os_uptime<'gc>(
        agent: &mut Agent,
        _this: Value,
        _args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let uptime = System::uptime();
        Ok(Value::from_f64(agent, uptime as f64, gc.nogc()).unbind())
    }

    /// `Andromeda.loadavg() -> [number, number, number]` — 1 / 5 / 15-minute
    /// averages. Returns `[0, 0, 0]` on Windows (matches Deno).
    fn loadavg<'gc>(
        agent: &mut Agent,
        _this: Value,
        _args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let load = System::load_average();
        // Return as JSON array — avoids constructing a Nova Array here;
        // the TS shim parses it.
        let json = format!("[{}, {}, {}]", load.one, load.five, load.fifteen);
        Ok(Value::from_string(agent, json, gc.nogc()).unbind())
    }

    /// `Andromeda.memoryUsage() -> { rss, heapTotal, heapUsed, external }`
    ///
    /// `rss` is the process resident set size, sourced from sysinfo. Nova
    /// does not currently expose V8-style heap accounting, so
    /// `heapTotal` / `heapUsed` / `external` mirror the RSS figure (shape
    /// compat with Deno for code that only inspects `rss` — other fields
    /// are non-zero placeholders pending upstream Nova introspection).
    fn memory_usage<'gc>(
        agent: &mut Agent,
        _this: Value,
        _args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let mut sys = System::new();
        sys.refresh_memory();
        let pid = sysinfo::get_current_pid().ok();
        let rss = pid
            .and_then(|p| {
                sys.refresh_processes(sysinfo::ProcessesToUpdate::Some(&[p]), true);
                sys.process(p).map(|proc| proc.memory())
            })
            .unwrap_or(0);
        let json =
            format!("{{\"rss\":{rss},\"heapTotal\":{rss},\"heapUsed\":{rss},\"external\":0}}");
        Ok(Value::from_string(agent, json, gc.nogc()).unbind())
    }

    /// `Andromeda.consoleSize() -> { columns, rows }` — queries the TTY
    /// attached to stdout. Returns `{columns: 80, rows: 24}` when stdout is
    /// not a TTY (same fallback shape Deno uses when there is no terminal).
    fn console_size<'gc>(
        agent: &mut Agent,
        _this: Value,
        _args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let (cols, rows) = terminal_size().unwrap_or((80, 24));
        let json = format!("{{\"columns\":{cols},\"rows\":{rows}}}");
        Ok(Value::from_string(agent, json, gc.nogc()).unbind())
    }
}

#[cfg(unix)]
fn terminal_size() -> Option<(u16, u16)> {
    let mut ws: libc::winsize = unsafe { std::mem::zeroed() };
    let fd = std::io::stdout().as_raw_fd();
    let result = unsafe { libc::ioctl(fd, libc::TIOCGWINSZ, &mut ws) };
    if result == 0 && ws.ws_col > 0 && ws.ws_row > 0 {
        Some((ws.ws_col, ws.ws_row))
    } else {
        None
    }
}

#[cfg(windows)]
fn terminal_size() -> Option<(u16, u16)> {
    // Windows console-size detection needs Win32's GetConsoleScreenBufferInfo.
    // Not implemented in this pass — fallback `(80, 24)` is what Deno also
    // returns for non-TTY stdout. Tracked for a follow-up.
    None
}

#[cfg(not(any(unix, windows)))]
fn terminal_size() -> Option<(u16, u16)> {
    None
}
