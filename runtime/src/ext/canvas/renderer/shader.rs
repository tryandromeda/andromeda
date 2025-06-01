use super::UNIFORM_LENGTH;

pub const SHADER2D: &str = r#"
struct Uniforms {
    color: vec4<f32>;
    mode: f32;
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

struct VertexOutput {
    @builtin(position) position: vec4<f32>;
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
    pub result: wgpu::Buffer,
}

#[derive(Debug)]
pub struct RenderCommand {
    pub vertex: usize,
    pub uniforms: usize,
    pub bind_group: usize,
    pub length: u32,
}
