// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::ext::canvas::renderer::Color;

pub const SHADER2D: &str = r#"
struct ColorStop {
    color: vec4f,
    offset: f32,
};

struct Uniforms {
    color: vec4f,
    gradient_start: vec2f,
    gradient_end: vec2f,
    fill_style: u32,
    global_alpha: f32,
    radius_start: f32,
    radius_end: f32,
};

struct VertexOutput {
    @builtin(position) position: vec4f,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;
@group(0) @binding(1)
var<storage, read> gradient: array<ColorStop>;

@vertex
fn vs_main(@location(0) position: vec2f) -> VertexOutput {
    var out: VertexOutput;
    out.position = vec4f(position, 0.0, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
    if(uniforms.fill_style == 0) {
        var color = uniforms.color;
        color.w *= uniforms.global_alpha;
        return color;
    } else {
        var pos_vec = in.position.xy - uniforms.gradient_start;
        var grad_vec = uniforms.gradient_end - uniforms.gradient_start;
	  	var color = gradient[0].color;
        var ratio = 0f;
        if(uniforms.fill_style == 1) {
            ratio = dot(pos_vec, grad_vec) / pow(length(grad_vec), 2);
        } else {
            var b = -2 * dot(pos_vec, grad_vec) / length(pos_vec);
            var c = pow(length(grad_vec), 2) - pow(uniforms.radius_end, 2);
            var total_length = (-b + sqrt(b * b - 4 * c)) / 2;
            ratio = (length(pos_vec) - uniforms.radius_start) 
                / (total_length - uniforms.radius_start);
        }
	  	for(var i = 0u; i < arrayLength(&gradient) - 1; i++) {
		  color = mix(
			color, 
			gradient[i + 1].color, 
			smoothstep(gradient[i].offset, gradient[i + 1].offset, ratio)
		  );
		}
        color.w *= uniforms.global_alpha;
        return color;
    }
}
"#;

#[derive(Debug)]
pub struct RenderCommand {
    pub vertex: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
    pub length: u32,
}

#[derive(Debug)]
pub struct RenderData {
    pub vertex: Vec<Coordinate>,
    pub fill_data: FillData,
    pub length: u32,
}

#[derive(Debug)]
pub struct FillData {
    pub uniforms: Uniforms,
    pub gradient: Vec<ColorStop>,
}

pub type Coordinate = (f32, f32);

#[derive(Clone, Debug)]
pub struct ColorStop {
    pub color: Color,
    pub offset: f32,
}

#[derive(Debug)]
pub struct Uniforms {
    pub color: Color,
    pub gradient_start: Coordinate,
    pub gradient_end: Coordinate,
    pub fill_style: u32,
    pub global_alpha: f32,
    pub radius_start: f32,
    pub radius_end: f32,
}

pub trait EncodeGPU {
    fn encode_gpu(&self) -> Vec<u8>;
}

impl EncodeGPU for Coordinate {
    fn encode_gpu(&self) -> Vec<u8> {
        [self.0.to_ne_bytes(), self.1.to_ne_bytes()].concat()
    }
}

impl EncodeGPU for Color {
    fn encode_gpu(&self) -> Vec<u8> {
        [
            self[0].to_ne_bytes(),
            self[1].to_ne_bytes(),
            self[2].to_ne_bytes(),
            self[3].to_ne_bytes(),
        ]
        .concat()
    }
}

impl EncodeGPU for ColorStop {
    fn encode_gpu(&self) -> Vec<u8> {
        [
            self.color.encode_gpu(),
            self.offset.to_ne_bytes().to_vec(),
            vec![0; 12],
        ]
        .concat()
    }
}

impl EncodeGPU for Vec<Coordinate> {
    fn encode_gpu(&self) -> Vec<u8> {
        let mut buf = vec![];
        for elem in self.iter() {
            buf.extend(elem.encode_gpu());
        }
        buf
    }
}

impl EncodeGPU for Vec<ColorStop> {
    fn encode_gpu(&self) -> Vec<u8> {
        let mut buf = vec![];
        for elem in self.iter() {
            buf.extend(elem.encode_gpu());
        }
        buf
    }
}

impl EncodeGPU for Uniforms {
    fn encode_gpu(&self) -> Vec<u8> {
        [
            self.color.encode_gpu(),
            self.gradient_start.encode_gpu(),
            self.gradient_end.encode_gpu(),
            self.fill_style.to_ne_bytes().to_vec(),
            self.global_alpha.to_ne_bytes().to_vec(),
            self.radius_start.to_ne_bytes().to_vec(),
            self.radius_end.to_ne_bytes().to_vec(),
        ]
        .concat()
    }
}
