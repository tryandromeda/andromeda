// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::collections::{HashMap, VecDeque};
use std::rc::Rc;
use std::time::Duration;

use winit::application::ApplicationHandler;
use winit::event::{ElementState, KeyEvent, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{ModifiersState, PhysicalKey};
use winit::platform::pump_events::EventLoopExtPumpEvents;
use winit::window::{Window, WindowAttributes, WindowId};

use super::events::{EventDetail, SerializedWindowEvent};

/// Options accepted by `createWindow(options)`, parsed from JSON on entry.
#[derive(Debug, Clone)]
pub struct CreateWindowOptions {
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub resizable: bool,
    pub visible: bool,
}

impl CreateWindowOptions {
    pub fn from_json(raw: &str) -> Result<Self, String> {
        let value: serde_json::Value =
            serde_json::from_str(raw).map_err(|e| format!("invalid options JSON: {e}"))?;
        let title = value
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("Andromeda")
            .to_string();
        let width = value.get("width").and_then(|v| v.as_u64()).unwrap_or(800) as u32;
        let height = value.get("height").and_then(|v| v.as_u64()).unwrap_or(600) as u32;
        if width == 0 || height == 0 {
            return Err("width and height must be positive".to_string());
        }
        let resizable = value
            .get("resizable")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        let visible = value
            .get("visible")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        Ok(Self {
            title,
            width,
            height,
            resizable,
            visible,
        })
    }
}

/// Per-window bookkeeping stored in the `WindowingApp`.
pub struct WindowData {
    pub window: Rc<Window>,
    pub closed: bool,
    pub surface: Option<wgpu::Surface<'static>>,
    pub surface_config: Option<wgpu::SurfaceConfiguration>,
}

/// Shared GPU context — one instance/adapter/device/queue for all windows.
/// Lazily initialized on first `present()`.
pub struct WindowingGpu {
    pub instance: wgpu::Instance,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    /// Blit shader + sampler + per-format pipeline cache for
    /// `presentCanvas`. Populated lazily on first canvas-present call.
    pub blit: Option<BlitResources>,
}

/// Shader module, bind-group layout, sampler, and cached render pipelines
/// for the canvas → window surface blit. One entry per destination format
/// (typically just one in practice — the window's swapchain format).
pub struct BlitResources {
    pub shader: wgpu::ShaderModule,
    pub sampler: wgpu::Sampler,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub pipeline_layout: wgpu::PipelineLayout,
    pub pipelines: std::collections::HashMap<wgpu::TextureFormat, wgpu::RenderPipeline>,
}

impl BlitResources {
    pub fn new(device: &wgpu::Device) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("andromeda-window-blit-shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("./blit.wgsl").into()),
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("andromeda-window-blit-sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("andromeda-window-blit-bgl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("andromeda-window-blit-pl"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });
        Self {
            shader,
            sampler,
            bind_group_layout,
            pipeline_layout,
            pipelines: std::collections::HashMap::new(),
        }
    }

    /// Get or build the blit pipeline for the given destination format.
    pub fn pipeline_for_format(
        &mut self,
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
    ) -> &wgpu::RenderPipeline {
        self.pipelines.entry(format).or_insert_with(|| {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("andromeda-window-blit-pipeline"),
                layout: Some(&self.pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &self.shader,
                    entry_point: Some("vs_main"),
                    buffers: &[],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &self.shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: Default::default(),
                }),
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
                cache: None,
            })
        })
    }
}

/// A pending window creation waiting for `ApplicationHandler::resumed` to fire.
pub struct PendingCreation {
    pub rid: u32,
    pub options: CreateWindowOptions,
}

/// The mutable "app" state passed to `pump_app_events`.
pub struct WindowingApp {
    pub windows: HashMap<u32, WindowData>,
    pub rid_by_window_id: HashMap<WindowId, u32>,
    pub next_rid: u32,
    pub pending_creations: Vec<PendingCreation>,
    pub pending_events: VecDeque<SerializedWindowEvent>,
    pub modifiers: ModifiersState,
    pub pointer: (f64, f64),
    pub mouse_buttons: u32,
}

impl Default for WindowingApp {
    fn default() -> Self {
        Self {
            windows: HashMap::new(),
            rid_by_window_id: HashMap::new(),
            // Start rids at 1 so 0 can signal "invalid" from TS if needed.
            next_rid: 1,
            pending_creations: Vec::new(),
            pending_events: VecDeque::new(),
            modifiers: ModifiersState::empty(),
            pointer: (0.0, 0.0),
            mouse_buttons: 0,
        }
    }
}

impl WindowingApp {
    pub fn reserve_rid(&mut self) -> u32 {
        let rid = self.next_rid;
        self.next_rid += 1;
        rid
    }

    pub fn get(&self, rid: u32) -> Option<&WindowData> {
        self.windows.get(&rid)
    }

    pub fn get_mut(&mut self, rid: u32) -> Option<&mut WindowData> {
        self.windows.get_mut(&rid)
    }

    pub fn remove(&mut self, rid: u32) -> Option<WindowData> {
        if let Some(data) = self.windows.remove(&rid) {
            let wid = data.window.id();
            self.rid_by_window_id.remove(&wid);
            Some(data)
        } else {
            None
        }
    }
}

/// Owns the event loop plus the app state. One instance lives in `OpsStorage`.
#[derive(Default)]
pub struct WindowingState {
    pub event_loop: Option<EventLoop<()>>,
    pub app: WindowingApp,
    pub gpu: Option<WindowingGpu>,
}

impl WindowingState {
    /// Lazily initialize the shared GPU context.
    pub fn ensure_gpu(&mut self) -> Result<&WindowingGpu, String> {
        if self.gpu.is_some() {
            return Ok(self.gpu.as_ref().unwrap());
        }
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        let adapter =
            futures::executor::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: None,
                force_fallback_adapter: false,
            }))
            .map_err(|e| format!("failed to request wgpu adapter: {e}"))?;
        let (device, queue) =
            futures::executor::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                label: Some("andromeda-window-device"),
                memory_hints: Default::default(),
                trace: wgpu::Trace::default(),
                experimental_features: wgpu::ExperimentalFeatures::default(),
            }))
            .map_err(|e| format!("failed to request wgpu device: {e}"))?;
        self.gpu = Some(WindowingGpu {
            instance,
            adapter,
            device,
            queue,
            blit: None,
        });
        Ok(self.gpu.as_ref().unwrap())
    }

    /// Lazily initialize the blit pipeline state.
    pub fn ensure_blit(&mut self) -> Result<&mut WindowingGpu, String> {
        let gpu = self
            .gpu
            .as_mut()
            .ok_or_else(|| "GPU not initialized".to_string())?;
        if gpu.blit.is_none() {
            gpu.blit = Some(BlitResources::new(&gpu.device));
        }
        Ok(gpu)
    }
}

impl WindowingState {
    pub fn ensure_event_loop(&mut self) -> Result<&mut EventLoop<()>, String> {
        if self.event_loop.is_none() {
            let event_loop =
                EventLoop::new().map_err(|e| format!("failed to initialize event loop: {e}"))?;
            self.event_loop = Some(event_loop);
        }
        Ok(self.event_loop.as_mut().unwrap())
    }

    pub fn pump(&mut self, timeout: Duration) {
        if let Some(event_loop) = self.event_loop.as_mut() {
            let _ = event_loop.pump_app_events(Some(timeout), &mut self.app);
        }
    }
}

impl ApplicationHandler<()> for WindowingApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let pending = std::mem::take(&mut self.pending_creations);
        for create in pending {
            let attrs = WindowAttributes::default()
                .with_title(&create.options.title)
                .with_inner_size(winit::dpi::LogicalSize::new(
                    create.options.width,
                    create.options.height,
                ))
                .with_resizable(create.options.resizable)
                .with_visible(create.options.visible);
            match event_loop.create_window(attrs) {
                Ok(window) => {
                    let window_id = window.id();
                    window.request_redraw();
                    window.focus_window();
                    let window = Rc::new(window);
                    self.windows.insert(
                        create.rid,
                        WindowData {
                            window: window.clone(),
                            closed: false,
                            surface: None,
                            surface_config: None,
                        },
                    );
                    self.rid_by_window_id.insert(window_id, create.rid);
                }
                Err(e) => {
                    eprintln!(
                        "[andromeda/window] failed to create window (rid {}): {e}",
                        create.rid
                    );
                    self.pending_events
                        .push_back(SerializedWindowEvent::close(create.rid));
                }
            }
        }
    }

    fn window_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(&rid) = self.rid_by_window_id.get(&window_id) else {
            return;
        };

        match event {
            WindowEvent::CloseRequested => {
                self.pending_events
                    .push_back(SerializedWindowEvent::close(rid));
            }
            WindowEvent::Resized(size) => {
                let scale = self
                    .windows
                    .get(&rid)
                    .map(|w| w.window.scale_factor())
                    .unwrap_or(1.0);
                // Report logical (CSS) pixels so JS-side canvas coordinates
                // line up 1:1 with the values users pass to createWindow.
                let logical_w = (size.width as f64 / scale).round() as u32;
                let logical_h = (size.height as f64 / scale).round() as u32;
                self.pending_events.push_back(SerializedWindowEvent::resize(
                    rid, logical_w, logical_h, scale,
                ));
            }
            WindowEvent::ModifiersChanged(mods) => {
                self.modifiers = mods.state();
            }
            WindowEvent::CursorMoved { position, .. } => {
                let scale = self
                    .windows
                    .get(&rid)
                    .map(|w| w.window.scale_factor())
                    .unwrap_or(1.0);
                self.pointer = (position.x / scale, position.y / scale);
                let detail = self.mouse_detail(-1);
                self.pending_events.push_back(SerializedWindowEvent {
                    rid,
                    kind: "mousemove",
                    detail,
                });
            }
            WindowEvent::MouseInput { state, button, .. } => {
                let btn = mouse_button_index(button);
                let shift = btn.clamp(0, 31) as u32;
                let bit = 1u32 << shift;
                match state {
                    ElementState::Pressed => self.mouse_buttons |= bit,
                    ElementState::Released => self.mouse_buttons &= !bit,
                }
                let kind = match state {
                    ElementState::Pressed => "mousedown",
                    ElementState::Released => "mouseup",
                };
                let detail = self.mouse_detail(btn);
                self.pending_events
                    .push_back(SerializedWindowEvent { rid, kind, detail });
            }
            WindowEvent::KeyboardInput {
                event: key_event, ..
            } => {
                let kind = match key_event.state {
                    ElementState::Pressed => "keydown",
                    ElementState::Released => "keyup",
                };
                let detail = self.key_detail(&key_event);
                self.pending_events
                    .push_back(SerializedWindowEvent { rid, kind, detail });
            }
            WindowEvent::RedrawRequested => {
                if let Some(data) = self.windows.get(&rid)
                    && !data.closed
                {
                    data.window.request_redraw();
                }
            }
            WindowEvent::Destroyed => {
                self.windows.remove(&rid);
                self.rid_by_window_id.remove(&window_id);
            }
            _ => {}
        }
    }
}

impl WindowingApp {
    fn mouse_detail(&self, button: i32) -> EventDetail {
        EventDetail::Mouse {
            x: self.pointer.0,
            y: self.pointer.1,
            button,
            buttons: self.mouse_buttons,
            alt_key: self.modifiers.alt_key(),
            ctrl_key: self.modifiers.control_key(),
            meta_key: self.modifiers.super_key(),
            shift_key: self.modifiers.shift_key(),
        }
    }

    fn key_detail(&self, ev: &KeyEvent) -> EventDetail {
        build_key_detail(
            &ev.physical_key,
            &ev.logical_key,
            ev.location,
            ev.repeat,
            self.modifiers,
        )
    }
}

fn build_key_detail(
    physical_key: &PhysicalKey,
    logical_key: &winit::keyboard::Key,
    location: winit::keyboard::KeyLocation,
    repeat: bool,
    modifiers: ModifiersState,
) -> EventDetail {
    use super::keymap;

    let (code, key_code) = match physical_key {
        PhysicalKey::Code(c) => (
            keymap::physical_key_to_code(*c).to_string(),
            keymap::physical_key_to_legacy_keycode(*c),
        ),
        PhysicalKey::Unidentified(_) => ("Unidentified".to_string(), 0),
    };
    let key = match logical_key {
        winit::keyboard::Key::Character(s) => s.as_str().to_string(),
        winit::keyboard::Key::Named(named) => keymap::named_key_to_spec(named).to_string(),
        winit::keyboard::Key::Unidentified(_) => "Unidentified".to_string(),
        winit::keyboard::Key::Dead(Some(c)) => c.to_string(),
        winit::keyboard::Key::Dead(None) => "Dead".to_string(),
    };
    EventDetail::Key {
        key,
        code,
        key_code,
        which: key_code,
        location: keymap::location_to_u8(location),
        alt_key: modifiers.alt_key(),
        ctrl_key: modifiers.control_key(),
        meta_key: modifiers.super_key(),
        shift_key: modifiers.shift_key(),
        repeat,
        is_composing: false,
    }
}

fn mouse_button_index(button: MouseButton) -> i32 {
    match button {
        MouseButton::Left => 0,
        MouseButton::Middle => 1,
        MouseButton::Right => 2,
        MouseButton::Back => 3,
        MouseButton::Forward => 4,
        MouseButton::Other(n) => 5 + n as i32,
    }
}
