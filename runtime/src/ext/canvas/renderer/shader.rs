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
    stroke_color: vec4f,
    stroke_width: f32,
    is_stroke: u32,
    has_texture: u32,
    composite_operation: u32,  // 0-25 for different blend modes
    // Transformation matrix: [a, b, c, d, e, f]
    // Represents: | a c e |
    //             | b d f |
    //             | 0 0 1 |
    transform: mat3x3f,
};

struct VertexOutput {
    @builtin(position) position: vec4f,
    @location(0) tex_coord: vec2f,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;
@group(0) @binding(1)
var<storage, read> gradient: array<ColorStop>;
@group(0) @binding(2)
var texture_sampler: sampler;
@group(0) @binding(3)
var texture: texture_2d<f32>;

@vertex
fn vs_main(
    @location(0) position: vec2f,
    @location(1) uv: vec2f
) -> VertexOutput {
    var out: VertexOutput;
    out.position = vec4f(position, 0.0, 1.0);
    out.tex_coord = uv;
    return out;
}

// Helper functions for blend modes

// RGB to HSL conversion
fn rgb_to_hsl(rgb: vec3f) -> vec3f {
    let max_c = max(max(rgb.r, rgb.g), rgb.b);
    let min_c = min(min(rgb.r, rgb.g), rgb.b);
    let l = (max_c + min_c) * 0.5;
    
    if (max_c == min_c) {
        return vec3f(0.0, 0.0, l);
    }
    
    let d = max_c - min_c;
    var s: f32;
    if (l > 0.5) {
        s = d / (2.0 - max_c - min_c);
    } else {
        s = d / (max_c + min_c);
    }
    
    var h: f32;
    if (max_c == rgb.r) {
        h = (rgb.g - rgb.b) / d + select(0.0, 6.0, rgb.g < rgb.b);
    } else if (max_c == rgb.g) {
        h = (rgb.b - rgb.r) / d + 2.0;
    } else {
        h = (rgb.r - rgb.g) / d + 4.0;
    }
    h = h / 6.0;
    
    return vec3f(h, s, l);
}

// HSL to RGB conversion
fn hsl_to_rgb(hsl: vec3f) -> vec3f {
    if (hsl.y == 0.0) {
        return vec3f(hsl.z, hsl.z, hsl.z);
    }
    
    var q: f32;
    if (hsl.z < 0.5) {
        q = hsl.z * (1.0 + hsl.y);
    } else {
        q = hsl.z + hsl.y - hsl.z * hsl.y;
    }
    let p = 2.0 * hsl.z - q;
    
    let r = hue_to_rgb(p, q, hsl.x + 1.0 / 3.0);
    let g = hue_to_rgb(p, q, hsl.x);
    let b = hue_to_rgb(p, q, hsl.x - 1.0 / 3.0);
    
    return vec3f(r, g, b);
}

fn hue_to_rgb(p: f32, q: f32, t_in: f32) -> f32 {
    var t = t_in;
    if (t < 0.0) { t = t + 1.0; }
    if (t > 1.0) { t = t - 1.0; }
    if (t < 1.0 / 6.0) { return p + (q - p) * 6.0 * t; }
    if (t < 1.0 / 2.0) { return q; }
    if (t < 2.0 / 3.0) { return p + (q - p) * (2.0 / 3.0 - t) * 6.0; }
    return p;
}

// Get luminosity from RGB
fn luminosity(c: vec3f) -> f32 {
    return 0.3 * c.r + 0.59 * c.g + 0.11 * c.b;
}

// Get saturation from RGB
fn saturation(c: vec3f) -> f32 {
    return max(max(c.r, c.g), c.b) - min(min(c.r, c.g), c.b);
}

// Clip color to valid range
fn clip_color(c_in: vec3f) -> vec3f {
    var c = c_in;
    let l = luminosity(c);
    let n = min(min(c.r, c.g), c.b);
    let x = max(max(c.r, c.g), c.b);
    
    if (n < 0.0) {
        c = l + (((c - l) * l) / (l - n));
    }
    if (x > 1.0) {
        c = l + (((c - l) * (1.0 - l)) / (x - l));
    }
    
    return c;
}

// Set luminosity
fn set_lum(c: vec3f, l: f32) -> vec3f {
    return clip_color(c + (l - luminosity(c)));
}

// Set saturation
fn set_sat(c: vec3f, s: f32) -> vec3f {
    var result = c;
    let min_val = min(min(c.r, c.g), c.b);
    let max_val = max(max(c.r, c.g), c.b);
    
    if (max_val > min_val) {
        result = (c - min_val) * s / (max_val - min_val);
    } else {
        result = vec3f(0.0, 0.0, 0.0);
    }
    
    return result;
}

// Apply composite operation
fn apply_composite(src: vec4f, dst: vec4f, op: u32) -> vec4f {
    // Premultiply alpha for proper blending
    let src_rgb = src.rgb * src.a;
    let dst_rgb = dst.rgb * dst.a;
    
    var result_rgb: vec3f;
    var result_a: f32;
    
    switch op {
        case 0u: { // source-over
            result_rgb = src_rgb + dst_rgb * (1.0 - src.a);
            result_a = src.a + dst.a * (1.0 - src.a);
        }
        case 1u: { // source-in
            result_rgb = src_rgb * dst.a;
            result_a = src.a * dst.a;
        }
        case 2u: { // source-out
            result_rgb = src_rgb * (1.0 - dst.a);
            result_a = src.a * (1.0 - dst.a);
        }
        case 3u: { // source-atop
            result_rgb = src_rgb * dst.a + dst_rgb * (1.0 - src.a);
            result_a = dst.a;
        }
        case 4u: { // destination-over
            result_rgb = dst_rgb + src_rgb * (1.0 - dst.a);
            result_a = dst.a + src.a * (1.0 - dst.a);
        }
        case 5u: { // destination-in
            result_rgb = dst_rgb * src.a;
            result_a = dst.a * src.a;
        }
        case 6u: { // destination-out
            result_rgb = dst_rgb * (1.0 - src.a);
            result_a = dst.a * (1.0 - src.a);
        }
        case 7u: { // destination-atop
            result_rgb = dst_rgb * src.a + src_rgb * (1.0 - dst.a);
            result_a = src.a;
        }
        case 8u: { // lighter
            result_rgb = src_rgb + dst_rgb;
            result_a = src.a + dst.a;
        }
        case 9u: { // copy
            result_rgb = src_rgb;
            result_a = src.a;
        }
        case 10u: { // xor
            result_rgb = src_rgb * (1.0 - dst.a) + dst_rgb * (1.0 - src.a);
            result_a = src.a + dst.a - 2.0 * src.a * dst.a;
        }
        case 11u: { // multiply
            result_rgb = src_rgb * dst_rgb + src_rgb * (1.0 - dst.a) + dst_rgb * (1.0 - src.a);
            result_a = src.a + dst.a * (1.0 - src.a);
        }
        case 12u: { // screen
            result_rgb = src_rgb + dst_rgb - src_rgb * dst_rgb;
            result_a = src.a + dst.a * (1.0 - src.a);
        }
        case 13u: { // overlay
            let blend = select(
                2.0 * src.rgb * dst.rgb,
                1.0 - 2.0 * (1.0 - src.rgb) * (1.0 - dst.rgb),
                dst.rgb <= vec3f(0.5)
            );
            result_rgb = blend * src.a * dst.a + src_rgb * (1.0 - dst.a) + dst_rgb * (1.0 - src.a);
            result_a = src.a + dst.a * (1.0 - src.a);
        }
        case 14u: { // darken
            let blend = min(src.rgb, dst.rgb);
            result_rgb = blend * src.a * dst.a + src_rgb * (1.0 - dst.a) + dst_rgb * (1.0 - src.a);
            result_a = src.a + dst.a * (1.0 - src.a);
        }
        case 15u: { // lighten
            let blend = max(src.rgb, dst.rgb);
            result_rgb = blend * src.a * dst.a + src_rgb * (1.0 - dst.a) + dst_rgb * (1.0 - src.a);
            result_a = src.a + dst.a * (1.0 - src.a);
        }
        case 16u: { // color-dodge
            let blend = select(
                min(vec3f(1.0), dst.rgb / (1.0 - src.rgb)),
                vec3f(1.0),
                src.rgb >= vec3f(1.0)
            );
            result_rgb = blend * src.a * dst.a + src_rgb * (1.0 - dst.a) + dst_rgb * (1.0 - src.a);
            result_a = src.a + dst.a * (1.0 - src.a);
        }
        case 17u: { // color-burn
            let blend = select(
                1.0 - min(vec3f(1.0), (1.0 - dst.rgb) / src.rgb),
                vec3f(0.0),
                src.rgb <= vec3f(0.0)
            );
            result_rgb = blend * src.a * dst.a + src_rgb * (1.0 - dst.a) + dst_rgb * (1.0 - src.a);
            result_a = src.a + dst.a * (1.0 - src.a);
        }
        case 18u: { // hard-light
            let blend = select(
                2.0 * src.rgb * dst.rgb,
                1.0 - 2.0 * (1.0 - src.rgb) * (1.0 - dst.rgb),
                src.rgb <= vec3f(0.5)
            );
            result_rgb = blend * src.a * dst.a + src_rgb * (1.0 - dst.a) + dst_rgb * (1.0 - src.a);
            result_a = src.a + dst.a * (1.0 - src.a);
        }
        case 19u: { // soft-light
            var blend = vec3f(0.0);
            for (var i = 0; i < 3; i++) {
                if (src.rgb[i] <= 0.5) {
                    blend[i] = dst.rgb[i] - (1.0 - 2.0 * src.rgb[i]) * dst.rgb[i] * (1.0 - dst.rgb[i]);
                } else {
                    let d = select(
                        sqrt(dst.rgb[i]),
                        ((16.0 * dst.rgb[i] - 12.0) * dst.rgb[i] + 4.0) * dst.rgb[i],
                        dst.rgb[i] <= 0.25
                    );
                    blend[i] = dst.rgb[i] + (2.0 * src.rgb[i] - 1.0) * (d - dst.rgb[i]);
                }
            }
            result_rgb = blend * src.a * dst.a + src_rgb * (1.0 - dst.a) + dst_rgb * (1.0 - src.a);
            result_a = src.a + dst.a * (1.0 - src.a);
        }
        case 20u: { // difference
            let blend = abs(dst.rgb - src.rgb);
            result_rgb = blend * src.a * dst.a + src_rgb * (1.0 - dst.a) + dst_rgb * (1.0 - src.a);
            result_a = src.a + dst.a * (1.0 - src.a);
        }
        case 21u: { // exclusion
            let blend = dst.rgb + src.rgb - 2.0 * dst.rgb * src.rgb;
            result_rgb = blend * src.a * dst.a + src_rgb * (1.0 - dst.a) + dst_rgb * (1.0 - src.a);
            result_a = src.a + dst.a * (1.0 - src.a);
        }
        case 22u: { // hue
            let blend = set_lum(set_sat(src.rgb, saturation(dst.rgb)), luminosity(dst.rgb));
            result_rgb = blend * src.a * dst.a + src_rgb * (1.0 - dst.a) + dst_rgb * (1.0 - src.a);
            result_a = src.a + dst.a * (1.0 - src.a);
        }
        case 23u: { // saturation
            let blend = set_lum(set_sat(dst.rgb, saturation(src.rgb)), luminosity(dst.rgb));
            result_rgb = blend * src.a * dst.a + src_rgb * (1.0 - dst.a) + dst_rgb * (1.0 - src.a);
            result_a = src.a + dst.a * (1.0 - src.a);
        }
        case 24u: { // color
            let blend = set_lum(src.rgb, luminosity(dst.rgb));
            result_rgb = blend * src.a * dst.a + src_rgb * (1.0 - dst.a) + dst_rgb * (1.0 - src.a);
            result_a = src.a + dst.a * (1.0 - src.a);
        }
        case 25u: { // luminosity
            let blend = set_lum(dst.rgb, luminosity(src.rgb));
            result_rgb = blend * src.a * dst.a + src_rgb * (1.0 - dst.a) + dst_rgb * (1.0 - src.a);
            result_a = src.a + dst.a * (1.0 - src.a);
        }
        default: { // fallback to source-over
            result_rgb = src_rgb + dst_rgb * (1.0 - src.a);
            result_a = src.a + dst.a * (1.0 - src.a);
        }
    }
    
    // Clamp and unpremultiply alpha
    result_a = clamp(result_a, 0.0, 1.0);
    if (result_a > 0.0) {
        result_rgb = clamp(result_rgb / result_a, vec3f(0.0), vec3f(1.0));
    }
    
    return vec4f(result_rgb, result_a);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
    var src_color: vec4f;
    
    // Handle texture sampling for images
    if (uniforms.has_texture == 1u) {
        let tex_color = textureSample(texture, texture_sampler, in.tex_coord);
        src_color = vec4f(tex_color.rgb, tex_color.a * uniforms.global_alpha);
    } else if (uniforms.is_stroke == 1u) {
        src_color = uniforms.stroke_color;
        src_color.w *= uniforms.global_alpha;
    } else if(uniforms.fill_style == 0u) {
        src_color = uniforms.color;
        src_color.w *= uniforms.global_alpha;
    } else {
        var color = gradient[0].color;
        var ratio = 0f;
        if(uniforms.fill_style == 1u) {
            var pos_vec = in.position.xy - uniforms.gradient_start;
            var grad_vec = uniforms.gradient_end - uniforms.gradient_start;
            let grad_len_sq = dot(grad_vec, grad_vec);
            // Ensure numerical stability: avoid division by very small numbers
            if (grad_len_sq > 1e-6) {
                ratio = dot(pos_vec, grad_vec) / grad_len_sq;
            }
        } else if(uniforms.fill_style == 2u) {
            var pos_vec = in.position.xy - uniforms.gradient_start;
            var grad_vec = uniforms.gradient_end - uniforms.gradient_start;
            let pos_len = length(pos_vec);
            // Numerical stability: check for degenerate cases
            if (pos_len > 1e-6) {
                var b = -2.0 * dot(pos_vec, grad_vec) / pos_len;
                var c = dot(grad_vec, grad_vec) - uniforms.radius_end * uniforms.radius_end;
                let discriminant = b * b - 4.0 * c;
                if (discriminant >= 0.0) {
                    var total_length = (-b + sqrt(discriminant)) * 0.5;
                    let radius_diff = total_length - uniforms.radius_start;
                    if (abs(radius_diff) > 1e-6) {
                        ratio = (pos_len - uniforms.radius_start) / radius_diff;
                    }
                }
            }
        } else {
            var pos_vec = in.position.xy - uniforms.gradient_start;
            var start_angle = uniforms.radius_start - radians(90.0);
            var start_vec = vec2f(cos(start_angle), sin(start_angle));
            // atan2 is stable for all inputs
            ratio = atan2(
                dot(pos_vec, start_vec), 
                determinant(mat2x2f(pos_vec, start_vec))
            ) / radians(360.0) + 0.5;
        }
        for(var i = 0u; i < arrayLength(&gradient) - 1; i++) {
            color = mix(
                color, 
                gradient[i + 1].color, 
                smoothstep(gradient[i].offset, gradient[i + 1].offset, ratio)
            );
        }
        src_color = color;
        src_color.w *= uniforms.global_alpha;
    }
    
    // Note: For proper compositing with existing canvas content, we would need
    // to load the destination color from the render target. This requires
    // framebuffer fetch or a separate texture read. For now, we assume a
    // transparent destination (dst = vec4f(0.0, 0.0, 0.0, 0.0)).
    // In a full implementation, you would:
    // 1. Use a texture attachment to store the current canvas state
    // 2. Sample from it here: let dst_color = textureLoad(canvas_texture, ...);
    // 3. Apply compositing: return apply_composite(src_color, dst_color, uniforms.composite_operation);
    
    let dst_color = vec4f(0.0, 0.0, 0.0, 0.0); // Placeholder for destination
    
    // Apply composite operation
    if (uniforms.composite_operation == 0u) {
        // Fast path for source-over (most common case)
        return src_color;
    } else {
        return apply_composite(src_color, dst_color, uniforms.composite_operation);
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
    pub stroke_color: Color,
    pub stroke_width: f32,
    pub is_stroke: u32,
    pub has_texture: u32,
    pub composite_operation: u32, // 0-25 for different blend modes
    pub transform: [f32; 12],     // 3x3 matrix stored as 3 vec4s for alignment
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
        let mut bytes = Vec::new();
        bytes.extend(self.color.encode_gpu()); // 0-15: vec4f (16)
        bytes.extend(self.gradient_start.encode_gpu()); // 16-23: vec2f (8)
        bytes.extend(self.gradient_end.encode_gpu()); // 24-31: vec2f (8)
        bytes.extend(self.fill_style.to_ne_bytes()); // 32-35: u32 (4)
        bytes.extend(self.global_alpha.to_ne_bytes()); // 36-39: f32 (4)
        bytes.extend(self.radius_start.to_ne_bytes()); // 40-43: f32 (4)
        bytes.extend(self.radius_end.to_ne_bytes()); // 44-47: f32 (4)
        bytes.extend(self.stroke_color.encode_gpu()); // 48-63: vec4f (16)
        bytes.extend(self.stroke_width.to_ne_bytes()); // 64-67: f32 (4)
        bytes.extend(self.is_stroke.to_ne_bytes()); // 68-71: u32 (4)
        bytes.extend(self.has_texture.to_ne_bytes()); // 72-75: u32 (4)
        bytes.extend(self.composite_operation.to_ne_bytes()); // 76-79: u32 (4)
        // Add padding to align transform to 16 bytes (next multiple of 16 after 79 is 80)
        while bytes.len() % 16 != 0 {
            bytes.push(0);
        }
        // Now at byte 80, encode transformation matrix (3x3 as 3 vec4s: 12 floats total)
        for &val in &self.transform {
            bytes.extend(val.to_ne_bytes());
        }
        // After 12 floats (48 bytes), we're at byte 80+48=128
        // Add padding to reach 160 bytes total (shader alignment requirement with new field)
        while bytes.len() < 160 {
            bytes.push(0);
        }
        bytes
    }
}
