// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use andromeda_core::{HostData, Rid};

use crate::RuntimeMacroTask;
use crate::ext::canvas::render_canvas_to_texture;

use super::state::WindowingState;

/// Present the latest frame of the canvas identified by `canvas_rid` into
/// the window identified by `win_rid`. Caller-side errors (closed window,
/// unknown canvas, GPU init failure) bubble back as `Err(String)` for the
/// op wrapper to convert into a JS exception.
pub fn present_canvas_on_window(
    host_data: &HostData<RuntimeMacroTask>,
    win_rid: u32,
    canvas_rid_raw: u32,
) -> Result<(), String> {
    use raw_window_handle::{HasDisplayHandle, HasWindowHandle};

    let canvas_rid = Rid::from_index(canvas_rid_raw);

    let canvas_texture = {
        let mut storage = host_data.storage.borrow_mut();
        render_canvas_to_texture(&mut *storage, canvas_rid)
            .ok_or_else(|| format!("canvas rid {canvas_rid_raw} not found"))?
    };

    let mut storage = host_data.storage.borrow_mut();
    let state: &mut WindowingState = storage
        .get_mut()
        .ok_or_else(|| "window extension not initialized".to_string())?;

    state.ensure_gpu()?;
    state.ensure_blit()?;

    let data = state
        .app
        .windows
        .get_mut(&win_rid)
        .filter(|d| !d.closed)
        .ok_or_else(|| "window has been closed".to_string())?;

    let gpu = state.gpu.as_mut().unwrap();

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

    let surface_format = data.surface_config.as_ref().unwrap().format;
    let frame = data
        .surface
        .as_ref()
        .unwrap()
        .get_current_texture()
        .map_err(|e| format!("get_current_texture: {e}"))?;
    let view = frame
        .texture
        .create_view(&wgpu::TextureViewDescriptor::default());

    let canvas_view_format = match canvas_texture.format() {
        wgpu::TextureFormat::Bgra8Unorm => Some(wgpu::TextureFormat::Bgra8UnormSrgb),
        wgpu::TextureFormat::Rgba8Unorm => Some(wgpu::TextureFormat::Rgba8UnormSrgb),
        _ => None,
    };
    let canvas_view = canvas_texture.create_view(&wgpu::TextureViewDescriptor {
        format: canvas_view_format,
        ..Default::default()
    });
    let blit = gpu.blit.as_mut().unwrap();
    let pipeline = blit
        .pipeline_for_format(&gpu.device, surface_format)
        .clone();
    let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("andromeda-window-blit-bg"),
        layout: &blit.bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&canvas_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&blit.sampler),
            },
        ],
    });

    let mut encoder = gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("andromeda-window-blit-encoder"),
        });
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("andromeda-window-blit-pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.draw(0..3, 0..1);
    }
    data.window.pre_present_notify();
    gpu.queue.submit(std::iter::once(encoder.finish()));
    frame.present();
    Ok(())
}
