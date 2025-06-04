// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use super::UNIFORM_LENGTH;

pub const SHADER2D: &str = r#"
struct Uniforms {
    color: vec4<f32>,
    mode: f32,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
};

@vertex
fn vs_main(@location(0) position: vec2<f32>) -> VertexOutput {
    var out: VertexOutput;
    out.position = vec4<f32>(position, 0.0, 1.0);
    return out;
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return uniforms.color;
}
"#;

#[derive(Debug)]
pub struct Buffers {
    pub vertex: Vec<wgpu::Buffer>,
    pub props: [u8; UNIFORM_LENGTH],
    pub uniforms: Vec<wgpu::Buffer>,
    pub bind_group: Vec<wgpu::BindGroup>,
    pub commands: Vec<RenderCommand>,

    pub background: wgpu::Texture,
}

#[derive(Debug)]
pub struct RenderCommand {
    pub vertex: usize,
    pub uniforms: usize,
    pub bind_group: usize,
    pub length: u32,
}
