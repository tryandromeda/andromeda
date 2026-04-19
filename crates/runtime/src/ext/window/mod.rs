// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::time::Duration;

use andromeda_core::{Extension, ExtensionOp, HostData, OpsStorage};
use nova_vm::{
    ecmascript::{Agent, ArgumentsList, ExceptionType, JsResult, Value},
    engine::{Bindable, GcScope},
};

use crate::RuntimeMacroTask;

#[cfg(feature = "canvas")]
pub mod canvas_bridge;
pub mod events;
pub mod keymap;
pub mod state;

use state::{CreateWindowOptions, PendingCreation, WindowingState};

/// The windowing extension.
#[derive(Default)]
pub struct WindowExt;

impl WindowExt {
    pub fn new_extension() -> Extension {
        Extension {
            name: "window",
            ops: vec![
                ExtensionOp::new(
                    "internal_window_create",
                    Self::internal_window_create,
                    1,
                    false,
                ),
                ExtensionOp::new(
                    "internal_window_close",
                    Self::internal_window_close,
                    1,
                    false,
                ),
                ExtensionOp::new(
                    "internal_window_poll_events",
                    Self::internal_window_poll_events,
                    0,
                    false,
                ),
                ExtensionOp::new(
                    "internal_window_raw_handle",
                    Self::internal_window_raw_handle,
                    1,
                    false,
                ),
                ExtensionOp::new(
                    "internal_window_set_title",
                    Self::internal_window_set_title,
                    2,
                    false,
                ),
                ExtensionOp::new(
                    "internal_window_get_size",
                    Self::internal_window_get_size,
                    1,
                    false,
                ),
                ExtensionOp::new(
                    "internal_window_set_size",
                    Self::internal_window_set_size,
                    3,
                    false,
                ),
                ExtensionOp::new(
                    "internal_window_set_visible",
                    Self::internal_window_set_visible,
                    2,
                    false,
                ),
                ExtensionOp::new(
                    "internal_window_present_color",
                    Self::internal_window_present_color,
                    5,
                    false,
                ),
                #[cfg(feature = "canvas")]
                ExtensionOp::new(
                    "internal_window_present_canvas",
                    Self::internal_window_present_canvas,
                    2,
                    false,
                ),
            ],
            storage: Some(Box::new(|storage: &mut OpsStorage| {
                storage.insert(WindowingState::default());
            })),
            files: vec![include_str!("./mod.ts")],
        }
    }

    /// `internal_window_create(optionsJson)` -> rid as string.
    fn internal_window_create<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let opts_binding = args.get(0).to_string(agent, gc.reborrow()).unbind()?;
        let opts_str = opts_binding
            .as_str(agent)
            .expect("String is not valid UTF-8")
            .to_string();

        let options = match CreateWindowOptions::from_json(&opts_str) {
            Ok(o) => o,
            Err(e) => {
                return throw_window_err(agent, "createWindow", &e, gc);
            }
        };

        let host_data = agent
            .get_host_data()
            .downcast_ref::<HostData<RuntimeMacroTask>>()
            .unwrap();

        let rid = {
            let mut storage = host_data.storage.borrow_mut();
            let state: &mut WindowingState = storage.get_mut().unwrap();
            if let Err(e) = state.ensure_event_loop() {
                drop(storage);
                return throw_window_err(agent, "createWindow", &e, gc);
            }
            let rid = state.app.reserve_rid();
            state
                .app
                .pending_creations
                .push(PendingCreation { rid, options });
            rid
        };

        let deadline = std::time::Instant::now() + Duration::from_millis(1500);
        loop {
            {
                let mut storage = host_data.storage.borrow_mut();
                let state: &mut WindowingState = storage.get_mut().unwrap();
                state.pump(Duration::from_millis(10));
                if state.app.windows.contains_key(&rid) {
                    break;
                }
            }
            if std::time::Instant::now() >= deadline {
                return throw_window_err(
                    agent,
                    "createWindow",
                    "window creation did not complete within 1500ms (event loop did not reach resumed state)",
                    gc,
                );
            }
        }

        Ok(Value::from_string(agent, rid.to_string(), gc.nogc()).unbind())
    }

    /// `internal_window_close(rid)` -> undefined. Idempotent.
    fn internal_window_close<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let rid = args.get(0).to_int32(agent, gc.reborrow()).unbind()? as u32;
        let host_data = agent
            .get_host_data()
            .downcast_ref::<HostData<RuntimeMacroTask>>()
            .unwrap();
        let mut storage = host_data.storage.borrow_mut();
        let state: &mut WindowingState = storage.get_mut().unwrap();
        state.app.remove(rid);
        Ok(Value::Undefined)
    }

    /// `internal_window_poll_events()` -> JSON array string of pending events.
    fn internal_window_poll_events<'gc>(
        agent: &mut Agent,
        _this: Value,
        _args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let host_data = agent
            .get_host_data()
            .downcast_ref::<HostData<RuntimeMacroTask>>()
            .unwrap();
        let json = {
            let mut storage = host_data.storage.borrow_mut();
            let state: &mut WindowingState = storage.get_mut().unwrap();
            state.pump(Duration::ZERO);
            let drained: Vec<_> = state.app.pending_events.drain(..).collect();
            serde_json::to_string(&drained).unwrap_or_else(|_| "[]".to_string())
        };
        Ok(Value::from_string(agent, json, gc.nogc()).unbind())
    }
}

impl WindowExt {
    /// `internal_window_raw_handle(rid)` -> JSON string:
    fn internal_window_raw_handle<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let rid = args.get(0).to_int32(agent, gc.reborrow()).unbind()? as u32;
        let host_data = agent
            .get_host_data()
            .downcast_ref::<HostData<RuntimeMacroTask>>()
            .unwrap();
        let json = {
            let storage = host_data.storage.borrow();
            let state: &WindowingState = storage.get().unwrap();
            let data = match state.app.get(rid) {
                Some(d) if !d.closed => d,
                _ => {
                    drop(storage);
                    return throw_window_err(agent, "rawHandle", "window has been closed", gc);
                }
            };
            match raw_handle_json(&data.window) {
                Ok(j) => j,
                Err(e) => {
                    drop(storage);
                    return throw_window_err(agent, "rawHandle", &e, gc);
                }
            }
        };
        Ok(Value::from_string(agent, json, gc.nogc()).unbind())
    }

    fn internal_window_set_title<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let rid = args.get(0).to_int32(agent, gc.reborrow()).unbind()? as u32;
        let title_binding = args.get(1).to_string(agent, gc.reborrow()).unbind()?;
        let title = title_binding
            .as_str(agent)
            .expect("String is not valid UTF-8")
            .to_string();
        let host_data = agent
            .get_host_data()
            .downcast_ref::<HostData<RuntimeMacroTask>>()
            .unwrap();
        let mut storage = host_data.storage.borrow_mut();
        let state: &mut WindowingState = storage.get_mut().unwrap();
        match state.app.get_mut(rid) {
            Some(d) if !d.closed => {
                d.window.set_title(&title);
                Ok(Value::Undefined)
            }
            _ => {
                drop(storage);
                throw_window_err(agent, "setTitle", "window has been closed", gc)
            }
        }
    }

    fn internal_window_get_size<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let rid = args.get(0).to_int32(agent, gc.reborrow()).unbind()? as u32;
        let host_data = agent
            .get_host_data()
            .downcast_ref::<HostData<RuntimeMacroTask>>()
            .unwrap();
        let json = {
            let storage = host_data.storage.borrow();
            let state: &WindowingState = storage.get().unwrap();
            match state.app.get(rid) {
                Some(d) if !d.closed => {
                    let size = d.window.inner_size();
                    let scale = d.window.scale_factor();
                    format!(
                        "{{\"width\":{},\"height\":{},\"scaleFactor\":{}}}",
                        size.width, size.height, scale
                    )
                }
                _ => {
                    drop(storage);
                    return throw_window_err(agent, "getSize", "window has been closed", gc);
                }
            }
        };
        Ok(Value::from_string(agent, json, gc.nogc()).unbind())
    }

    fn internal_window_set_size<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let rid = args.get(0).to_int32(agent, gc.reborrow()).unbind()? as u32;
        let width = args.get(1).to_int32(agent, gc.reborrow()).unbind()? as u32;
        let height = args.get(2).to_int32(agent, gc.reborrow()).unbind()? as u32;
        if width == 0 || height == 0 {
            return throw_window_err(agent, "setSize", "width and height must be positive", gc);
        }
        let host_data = agent
            .get_host_data()
            .downcast_ref::<HostData<RuntimeMacroTask>>()
            .unwrap();
        let mut storage = host_data.storage.borrow_mut();
        let state: &mut WindowingState = storage.get_mut().unwrap();
        match state.app.get_mut(rid) {
            Some(d) if !d.closed => {
                let _ = d
                    .window
                    .request_inner_size(winit::dpi::LogicalSize::new(width, height));
                Ok(Value::Undefined)
            }
            _ => {
                drop(storage);
                throw_window_err(agent, "setSize", "window has been closed", gc)
            }
        }
    }

    /// Present a solid-color frame into the window's swapchain.
    fn internal_window_present_color<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let rid = args.get(0).to_int32(agent, gc.reborrow()).unbind()? as u32;
        let r = args
            .get(1)
            .to_number(agent, gc.reborrow())
            .unbind()?
            .into_f64(agent);
        let g = args
            .get(2)
            .to_number(agent, gc.reborrow())
            .unbind()?
            .into_f64(agent);
        let b = args
            .get(3)
            .to_number(agent, gc.reborrow())
            .unbind()?
            .into_f64(agent);
        let a = args
            .get(4)
            .to_number(agent, gc.reborrow())
            .unbind()?
            .into_f64(agent);
        let host_data = agent
            .get_host_data()
            .downcast_ref::<HostData<RuntimeMacroTask>>()
            .unwrap();
        let mut storage = host_data.storage.borrow_mut();
        let state: &mut WindowingState = storage.get_mut().unwrap();
        if let Err(e) = state.ensure_gpu() {
            drop(storage);
            return throw_window_err(agent, "present", &e, gc);
        }
        match present_color_on_window(state, rid, r as f32, g as f32, b as f32, a as f32) {
            Ok(()) => Ok(Value::Undefined),
            Err(e) => {
                drop(storage);
                throw_window_err(agent, "present", &e, gc)
            }
        }
    }

    /// Present the latest frame of an `OffscreenCanvas` into a window's
    /// swapchain.
    #[cfg(feature = "canvas")]
    fn internal_window_present_canvas<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let win_rid = args.get(0).to_int32(agent, gc.reborrow()).unbind()? as u32;
        let canvas_rid = args.get(1).to_int32(agent, gc.reborrow()).unbind()? as u32;
        let host_data = agent
            .get_host_data()
            .downcast_ref::<HostData<RuntimeMacroTask>>()
            .unwrap();
        match canvas_bridge::present_canvas_on_window(host_data, win_rid, canvas_rid) {
            Ok(()) => Ok(Value::Undefined),
            Err(e) => throw_window_err(agent, "presentCanvas", &e, gc),
        }
    }

    fn internal_window_set_visible<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let rid = args.get(0).to_int32(agent, gc.reborrow()).unbind()? as u32;
        // Match standard JS boolean coercion for the few value kinds that
        // can reach here. The TS shim always passes `!!visible`, but the
        // op is the stable contract and should not silently invert truthy
        // non-Boolean inputs.
        let visible = match args.get(1).unbind() {
            Value::Boolean(b) => b,
            Value::Undefined | Value::Null => false,
            Value::Integer(i) => i.into_i64() != 0,
            _ => true,
        };
        let host_data = agent
            .get_host_data()
            .downcast_ref::<HostData<RuntimeMacroTask>>()
            .unwrap();
        let mut storage = host_data.storage.borrow_mut();
        let state: &mut WindowingState = storage.get_mut().unwrap();
        match state.app.get_mut(rid) {
            Some(d) if !d.closed => {
                d.window.set_visible(visible);
                Ok(Value::Undefined)
            }
            _ => {
                drop(storage);
                throw_window_err(agent, "setVisible", "window has been closed", gc)
            }
        }
    }
}

fn raw_handle_json(window: &winit::window::Window) -> Result<String, String> {
    use raw_window_handle::{HasDisplayHandle, HasWindowHandle, RawDisplayHandle, RawWindowHandle};

    let wh = window
        .window_handle()
        .map_err(|e| format!("window_handle unavailable: {e}"))?;
    let dh = window
        .display_handle()
        .map_err(|e| format!("display_handle unavailable: {e}"))?;
    let size = window.inner_size();

    let (system, window_value) = match wh.as_raw() {
        RawWindowHandle::AppKit(h) => ("cocoa", h.ns_view.as_ptr() as u64),
        RawWindowHandle::Win32(h) => ("win32", h.hwnd.get() as u64),
        RawWindowHandle::Xlib(h) => ("x11", h.window as u64),
        RawWindowHandle::Xcb(h) => ("x11", h.window.get() as u64),
        RawWindowHandle::Wayland(h) => ("wayland", h.surface.as_ptr() as u64),
        other => {
            return Err(format!("unsupported window handle variant: {other:?}"));
        }
    };

    let display_value: u64 = match dh.as_raw() {
        RawDisplayHandle::AppKit(_) => 0,
        RawDisplayHandle::Xlib(h) => h.display.map_or(0, |p| p.as_ptr() as u64),
        RawDisplayHandle::Xcb(h) => h.connection.map_or(0, |p| p.as_ptr() as u64),
        RawDisplayHandle::Wayland(h) => h.display.as_ptr() as u64,
        // Win32 / UiKit / Orbital / Ohos etc. don't expose a separate
        // display pointer — 0 is the accepted sentinel.
        _ => 0,
    };

    Ok(format!(
        "{{\"system\":\"{system}\",\"windowHandle\":\"{window_value}\",\"displayHandle\":\"{display_value}\",\"width\":{},\"height\":{}}}",
        size.width, size.height
    ))
}

/// Pre-tick hook invoked once per iteration of `Runtime::run`. Drives the
/// winit event loop so window events flow even when JS is idle.
pub fn pump_windowing_state(host_data: &HostData<RuntimeMacroTask>) {
    let mut storage = host_data.storage.borrow_mut();
    if let Some(state) = storage.get_mut::<WindowingState>() {
        state.pump(Duration::ZERO);
    }
}

/// Ensure the window has a configured surface that matches its current size,
/// then clear to the given RGBA and present. Separated from the op body so
/// the mutable-borrow juggling stays readable.
fn present_color_on_window(
    state: &mut WindowingState,
    rid: u32,
    r: f32,
    g: f32,
    b: f32,
    a: f32,
) -> Result<(), String> {
    use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
    let gpu = state.gpu.as_ref().ok_or("GPU not initialized")?;
    let data = match state.app.windows.get_mut(&rid) {
        Some(d) if !d.closed => d,
        _ => return Err("window has been closed".to_string()),
    };

    // Create the surface on first call.
    if data.surface.is_none() {
        let target = wgpu::SurfaceTargetUnsafe::RawHandle {
            raw_display_handle: data
                .window
                .display_handle()
                .map_err(|e| format!("display_handle: {e}"))?
                .as_raw(),
            raw_window_handle: data
                .window
                .window_handle()
                .map_err(|e| format!("window_handle: {e}"))?
                .as_raw(),
        };
        let surface = unsafe { gpu.instance.create_surface_unsafe(target) }
            .map_err(|e| format!("create_surface: {e}"))?;
        data.surface = Some(surface);
    }

    let size = data.window.inner_size();
    let (width, height) = (size.width.max(1), size.height.max(1));

    let needs_config = match &data.surface_config {
        None => true,
        Some(cfg) => cfg.width != width || cfg.height != height,
    };
    if needs_config {
        let surface = data.surface.as_ref().unwrap();
        let caps = surface.get_capabilities(&gpu.adapter);
        let format = caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(caps.formats[0]);
        let present_mode = if caps.present_modes.contains(&wgpu::PresentMode::Fifo) {
            wgpu::PresentMode::Fifo
        } else {
            caps.present_modes[0]
        };
        let alpha_mode = caps.alpha_modes[0];
        let cfg = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width,
            height,
            present_mode,
            desired_maximum_frame_latency: 2,
            alpha_mode,
            view_formats: vec![],
        };
        surface.configure(&gpu.device, &cfg);
        data.surface_config = Some(cfg);
    }

    let surface = data.surface.as_ref().unwrap();
    let frame = surface
        .get_current_texture()
        .map_err(|e| format!("get_current_texture: {e}"))?;
    let view = frame
        .texture
        .create_view(&wgpu::TextureViewDescriptor::default());
    let mut encoder = gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("andromeda-window-present"),
        });
    {
        let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("andromeda-window-clear"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: r as f64,
                        g: g as f64,
                        b: b as f64,
                        a: a as f64,
                    }),
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
    }
    data.window.pre_present_notify();
    gpu.queue.submit(std::iter::once(encoder.finish()));
    frame.present();
    Ok(())
}

fn throw_window_err<'gc>(
    agent: &mut Agent,
    operation: &str,
    message: &str,
    gc: GcScope<'gc, '_>,
) -> JsResult<'gc, Value<'gc>> {
    let full = format!("window.{operation}: {message}");
    let err = agent.throw_exception(ExceptionType::Error, full, gc.nogc());
    Err(err.unbind())
}
