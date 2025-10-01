// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use wgpu::PollType;

use crate::FillStyle;

use super::*;
pub struct Renderer {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub pipeline: wgpu::RenderPipeline,
    pub background: wgpu::Texture,
    pub default_sampler: wgpu::Sampler,
    pub default_texture: wgpu::Texture,

    pub dimensions: Dimensions,
    pub commands: Vec<RenderCommand>,
}

const U32_SIZE: u32 = std::mem::size_of::<u32>() as u32;

#[allow(dead_code)]
impl Renderer {
    pub fn create_stroke_data(
        &self,
        render_state: &RenderState,
        stroke_color: [f32; 4],
        stroke_width: f32,
    ) -> FillData {
        FillData {
            uniforms: Uniforms {
                color: [0.0, 0.0, 0.0, 0.0],
                gradient_start: (0.0, 0.0),
                gradient_end: (0.0, 0.0),
                fill_style: 0,
                global_alpha: render_state.global_alpha,
                radius_start: 0.0,
                radius_end: 0.0,
                stroke_color,
                stroke_width,
                is_stroke: 1,
                transform: transform_to_mat3(&render_state.transform),
                has_texture: 0,
            },
            gradient: vec![],
        }
    }

    pub fn render_stroke_rect(
        &mut self,
        rect: Rect,
        render_state: &RenderState,
        stroke_color: [f32; 4],
        stroke_width: f32,
    ) {
        let stroke_data = self.create_stroke_data(render_state, stroke_color, stroke_width);
        // Apply transformation to all four corners
        let top_left = transform_point(&rect.start, &render_state.transform);
        let top_right = transform_point(
            &Point {
                x: rect.end.x,
                y: rect.start.y,
            },
            &render_state.transform,
        );
        let bottom_right = transform_point(&rect.end, &render_state.transform);
        let bottom_left = transform_point(
            &Point {
                x: rect.start.x,
                y: rect.end.y,
            },
            &render_state.transform,
        );

        let tl = translate_coords(&top_left, &self.dimensions);
        let tr = translate_coords(&top_right, &self.dimensions);
        let br = translate_coords(&bottom_right, &self.dimensions);
        let bl = translate_coords(&bottom_left, &self.dimensions);

        let vertex = vec![
            (tl.0, tl.1),
            (tr.0, tr.1),
            (br.0, br.1),
            (bl.0, bl.1),
            (tl.0, tl.1), // close the loop
        ];
        self.create_render_command(RenderData {
            vertex,
            fill_data: stroke_data,
            length: 5,
        });
    }
    pub fn new(
        device: wgpu::Device,
        queue: wgpu::Queue,
        format: wgpu::TextureFormat,
        dimensions: Dimensions,
    ) -> Self {
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::from(SHADER2D)),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            cache: None,
            vertex: wgpu::VertexState {
                module: &module,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: 2 * 4,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float32x2,
                        offset: 0,
                        shader_location: 0,
                    }],
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &module,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        let background = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Background"),
            dimension: wgpu::TextureDimension::D2,
            format,
            mip_level_count: 1,
            sample_count: 1,
            size: wgpu::Extent3d {
                depth_or_array_layers: 1,
                height: dimensions.height,
                width: dimensions.width,
            },
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });

        // Create default sampler for texture operations
        let default_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Default Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        // Create a default 1x1 white texture for when no texture is being used
        let default_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Default Texture"),
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            mip_level_count: 1,
            sample_count: 1,
            size: wgpu::Extent3d {
                depth_or_array_layers: 1,
                height: 1,
                width: 1,
            },
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        // Initialize the default texture with white color
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &default_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &[255u8, 255u8, 255u8, 255u8],
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4),
                rows_per_image: Some(1),
            },
            wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
        );

        Self {
            device,
            queue,
            pipeline,
            background,
            default_sampler,
            default_texture,
            dimensions,
            commands: vec![],
        }
    }

    pub fn render_all(&mut self) {
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.background.create_view(&Default::default()),
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 1.0,
                            g: 1.0,
                            b: 1.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                    resolve_target: None,
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            pass.set_pipeline(&self.pipeline);
            for command in &self.commands {
                pass.set_bind_group(0, &command.bind_group, &[]);
                pass.set_vertex_buffer(0, command.vertex.slice(..));
                pass.draw(0..command.length, 0..1);
            }
        }

        self.queue.submit([encoder.finish()]);
    }

    pub async fn create_bitmap(&mut self) -> Vec<u8> {
        // Calculate bytes per row with proper alignment
        let bytes_per_pixel = U32_SIZE; // 4 bytes per pixel for RGBA
        let unpadded_bytes_per_row = self.dimensions.width * bytes_per_pixel;
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let padded_bytes_per_row = unpadded_bytes_per_row.div_ceil(align) * align;

        // Create a new buffer with the correct size to accommodate padding
        let padded_buffer_size =
            (padded_bytes_per_row * self.dimensions.height) as wgpu::BufferAddress;
        let result_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Bitmap"),
            mapped_at_creation: false,
            size: padded_buffer_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        });

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                aspect: wgpu::TextureAspect::All,
                mip_level: 0,
                texture: &self.background,
                origin: wgpu::Origin3d::ZERO,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &result_buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    bytes_per_row: Some(padded_bytes_per_row),
                    rows_per_image: Some(self.dimensions.height),
                    offset: 0,
                },
            },
            wgpu::Extent3d {
                depth_or_array_layers: 1,
                height: self.dimensions.height,
                width: self.dimensions.width,
            },
        );

        self.queue.submit([encoder.finish()]);

        let data = {
            let buffer_slice = result_buffer.slice(..);
            // map buffer for reading (callback-based API)
            buffer_slice.map_async(wgpu::MapMode::Read, |_| {});
            // poll until mapping is ready
            self.device.poll(PollType::Wait).unwrap();
            // now read mapped data
            let mapped_data = buffer_slice.get_mapped_range().to_vec();

            // Remove padding from each row to get the actual image data
            let mut unpadded_data = Vec::new();
            for row in 0..self.dimensions.height {
                let row_start = (row * padded_bytes_per_row) as usize;
                let row_end = row_start + (unpadded_bytes_per_row as usize);
                unpadded_data.extend_from_slice(&mapped_data[row_start..row_end]);
            }
            unpadded_data
        };
        result_buffer.unmap();

        data
    }

    pub fn create_render_command(&mut self, data: RenderData) {
        let uniforms = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Uniforms"),
            mapped_at_creation: false,
            size: 144, // Size required by shader with mat3x3f alignment (each column aligned to vec4)
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
        });
        self.queue
            .write_buffer(&uniforms, 0, &data.fill_data.uniforms.encode_gpu());

        let gradient_len = 32 * data.fill_data.gradient.len() as u64;
        let gradient = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Gradient"),
            mapped_at_creation: false,
            size: if gradient_len > 0 { gradient_len } else { 32 },
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::STORAGE,
        });
        self.queue
            .write_buffer(&gradient, 0, &data.fill_data.gradient.encode_gpu());

        let vertex_len = 32 * data.vertex.len() as u64;
        let vertex = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Vertex"),
            mapped_at_creation: false,
            size: if vertex_len > 0 { vertex_len } else { 16 },
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX,
        });
        self.queue
            .write_buffer(&vertex, 0, &data.vertex.encode_gpu());

        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.pipeline.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &uniforms,
                        offset: 0,
                        size: None,
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &gradient,
                        offset: 0,
                        size: None,
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&self.default_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(
                        &self.default_texture.create_view(&Default::default()),
                    ),
                },
            ],
        });

        self.commands.push(RenderCommand {
            vertex,
            bind_group,
            length: data.length,
        })
    }

    pub fn create_fill_data(&self, render_state: &RenderState) -> FillData {
        match &render_state.fill_style {
            FillStyle::Color { r, g, b, a } => FillData {
                uniforms: Uniforms {
                    color: [*r, *g, *b, *a],
                    gradient_start: (0.0, 0.0),
                    gradient_end: (0.0, 0.0),
                    fill_style: 0,
                    global_alpha: render_state.global_alpha,
                    radius_start: 0.0,
                    radius_end: 0.0,
                    stroke_color: [0.0, 0.0, 0.0, 0.0],
                    stroke_width: 0.0,
                    is_stroke: 0,
                    transform: transform_to_mat3(&render_state.transform),
                    has_texture: 0,
                },
                gradient: vec![],
            },
            FillStyle::LinearGradient(gradient) => FillData {
                uniforms: Uniforms {
                    color: [0.0, 0.0, 0.0, 0.0],
                    gradient_start: gradient.start,
                    gradient_end: gradient.end,
                    fill_style: 1,
                    global_alpha: render_state.global_alpha,
                    radius_start: 0.0,
                    radius_end: 0.0,
                    stroke_color: [0.0, 0.0, 0.0, 0.0],
                    stroke_width: 0.0,
                    is_stroke: 0,
                    transform: transform_to_mat3(&render_state.transform),
                    has_texture: 0,
                },
                gradient: gradient.color_stops.clone(),
            },
            FillStyle::RadialGradient(gradient) => FillData {
                uniforms: Uniforms {
                    color: [0.0, 0.0, 0.0, 0.0],
                    gradient_start: gradient.start,
                    gradient_end: gradient.end,
                    fill_style: 2,
                    global_alpha: render_state.global_alpha,
                    radius_start: gradient.start_radius,
                    radius_end: gradient.end_radius,
                    stroke_color: [0.0, 0.0, 0.0, 0.0],
                    stroke_width: 0.0,
                    is_stroke: 0,
                    transform: transform_to_mat3(&render_state.transform),
                    has_texture: 0,
                },
                gradient: gradient.color_stops.clone(),
            },
            FillStyle::ConicGradient(gradient) => FillData {
                uniforms: Uniforms {
                    color: [0.0, 0.0, 0.0, 0.0],
                    gradient_start: gradient.center,
                    gradient_end: (0.0, 0.0),
                    fill_style: 3,
                    global_alpha: render_state.global_alpha,
                    radius_start: gradient.start_angle,
                    radius_end: 0.0,
                    stroke_color: [0.0, 0.0, 0.0, 0.0],
                    stroke_width: 0.0,
                    is_stroke: 0,
                    transform: transform_to_mat3(&render_state.transform),
                    has_texture: 0,
                },
                gradient: gradient.color_stops.clone(),
            },
            _ => unimplemented!(),
        }
    }

    pub async fn save_as_png(&mut self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Extract pixel data from GPU
        let pixel_data = self.create_bitmap().await;

        // Convert from BGRA to RGBA (wgpu typically uses BGRA format)
        let mut rgba_data = Vec::new();
        for chunk in pixel_data.chunks(4) {
            if chunk.len() == 4 {
                // Convert BGRA -> RGBA
                rgba_data.push(chunk[2]); // R
                rgba_data.push(chunk[1]); // G
                rgba_data.push(chunk[0]); // B
                rgba_data.push(chunk[3]); // A
            }
        }

        // Save as PNG using the image crate
        let img =
            image::RgbaImage::from_raw(self.dimensions.width, self.dimensions.height, rgba_data)
                .ok_or("Failed to create image from pixel data")?;

        img.save(path)?;
        Ok(())
    }

    pub fn render_rect(&mut self, rect: Rect, render_state: &RenderState) {
        let fill_data = self.create_fill_data(render_state);
        // Apply transformation to all four corners of the rectangle
        let top_left = transform_point(&rect.start, &render_state.transform);
        let bottom_right = transform_point(&rect.end, &render_state.transform);
        let top_right = transform_point(
            &Point {
                x: rect.end.x,
                y: rect.start.y,
            },
            &render_state.transform,
        );
        let bottom_left = transform_point(
            &Point {
                x: rect.start.x,
                y: rect.end.y,
            },
            &render_state.transform,
        );

        let tl = translate_coords(&top_left, &self.dimensions);
        let tr = translate_coords(&top_right, &self.dimensions);
        let bl = translate_coords(&bottom_left, &self.dimensions);
        let br = translate_coords(&bottom_right, &self.dimensions);

        let vertex = vec![(tl.0, tl.1), (bl.0, bl.1), (tr.0, tr.1), (br.0, br.1)];

        self.create_render_command(RenderData {
            vertex,
            fill_data,
            length: 4,
        });
    }

    pub fn render_polygon(&mut self, polygon: Path, render_state: &RenderState) {
        let fill_data = self.create_fill_data(render_state);
        let mut data = Vec::new();
        if let 0 = polygon.len() % 2 {
            for i in 0..(polygon.len() / 2) {
                data.push(&polygon[i]);
                data.push(&polygon[polygon.len() - 1 - i]);
            }
        } else {
            for i in 0..((polygon.len() - 1) / 2) {
                data.push(&polygon[i]);
                data.push(&polygon[polygon.len() - 1 - i]);
            }
            data.push(&polygon[(polygon.len() - 1) / 2]);
        };
        // Apply transformation to each point before translating to clip space
        let vertex = data
            .iter()
            .map(|point| {
                let transformed = transform_point(point, &render_state.transform);
                translate_coords(&transformed, &self.dimensions)
            })
            .collect::<Vec<Coordinate>>();

        self.create_render_command(RenderData {
            vertex,
            fill_data,
            length: polygon.len() as u32,
        });
    }

    /// Renders a quadratic Bezier curve from start point through control point to end point
    pub fn render_quadratic_bezier(
        &mut self,
        start: Point,
        control: Point,
        end: Point,
        render_state: &RenderState,
        segments: usize,
        line_width: f64,
    ) {
        // Tessellate the curve into line segments
        let points = self.tessellate_quadratic_bezier(start, control, end, segments);

        // Render the line segments as a connected path
        self.render_polyline(points, render_state, line_width);
    }

    /// Renders a cubic Bezier curve from start through two control points to end point
    #[allow(clippy::too_many_arguments)]
    pub fn render_cubic_bezier(
        &mut self,
        start: Point,
        control1: Point,
        control2: Point,
        end: Point,
        render_state: &RenderState,
        segments: usize,
        line_width: f64,
    ) {
        // Tessellate the curve into line segments
        let points = self.tessellate_cubic_bezier(start, control1, control2, end, segments);

        // Render the line segments as a connected path
        self.render_polyline(points, render_state, line_width);
    }

    /// Renders a connected series of line segments with a specified width
    pub fn render_polyline(
        &mut self,
        points: Vec<Point>,
        render_state: &RenderState,
        line_width: f64,
    ) {
        if points.len() < 2 {
            return; // Need at least two points to make a line
        }

        // Create a vector of triangles to form the line with width
        let mut vertices = Vec::new();

        for i in 0..points.len() - 1 {
            let p1 = &points[i];
            let p2 = &points[i + 1];

            // Calculate the direction vector
            let dx = p2.x - p1.x;
            let dy = p2.y - p1.y;

            // Normalize the direction
            let length = (dx * dx + dy * dy).sqrt();
            if length < 0.0001 {
                continue; // Skip zero-length segments
            }

            let nx = dx / length;
            let ny = dy / length;

            // Calculate the perpendicular vector with half line width
            let half_width = line_width / 2.0;
            let px = -ny * half_width;
            let py = nx * half_width;

            // Create four corners of the line segment as a rectangle
            let a = Point {
                x: p1.x + px,
                y: p1.y + py,
            };
            let b = Point {
                x: p1.x - px,
                y: p1.y - py,
            };
            let c = Point {
                x: p2.x + px,
                y: p2.y + py,
            };
            let d = Point {
                x: p2.x - px,
                y: p2.y - py,
            };

            // Add rectangle as two triangles
            vertices.push(a.clone());
            vertices.push(b.clone());
            vertices.push(c.clone());

            vertices.push(b.clone());
            vertices.push(d.clone());
            vertices.push(c.clone());
        }

        // Render the triangles
        self.render_polygon(vertices, render_state);
    }

    #[allow(dead_code)]
    /// Render an arc as a filled polygon (fan) using tessellation
    pub fn render_arc(
        &mut self,
        center_x: f64,
        center_y: f64,
        radius: f64,
        start_angle: f64,
        end_angle: f64,
        render_state: &RenderState,
    ) {
        // Tessellate arc into points
        const SEGMENTS: usize = 32;
        let angle_diff = end_angle - start_angle;
        let step = angle_diff / SEGMENTS as f64;
        let mut points = Vec::new();
        // Center point for fan
        points.push(crate::ext::canvas::renderer::Point {
            x: center_x,
            y: center_y,
        });
        for i in 0..=SEGMENTS {
            let angle = start_angle + (i as f64 * step);
            let x = center_x + radius * angle.cos();
            let y = center_y + radius * angle.sin();
            points.push(crate::ext::canvas::renderer::Point { x, y });
        }
        // Render as polygon (triangle fan)
        self.render_polygon(points, render_state);
    }

    /// Tessellate a quadratic Bezier curve into line segments
    fn tessellate_quadratic_bezier(
        &self,
        start: Point,
        control: Point,
        end: Point,
        segments: usize,
    ) -> Vec<Point> {
        let mut points = Vec::with_capacity(segments + 1);

        for i in 0..=segments {
            let t = i as f64 / segments as f64;
            let t_inv = 1.0 - t;

            // Quadratic Bezier formula: B(t) = (1-t)²P₀ + 2(1-t)tP₁ + t²P₂
            let x = t_inv * t_inv * start.x + 2.0 * t_inv * t * control.x + t * t * end.x;
            let y = t_inv * t_inv * start.y + 2.0 * t_inv * t * control.y + t * t * end.y;

            points.push(Point { x, y });
        }

        points
    }

    /// Tessellate a cubic Bezier curve into line segments
    fn tessellate_cubic_bezier(
        &self,
        start: Point,
        control1: Point,
        control2: Point,
        end: Point,
        segments: usize,
    ) -> Vec<Point> {
        let mut points = Vec::with_capacity(segments + 1);

        for i in 0..=segments {
            let t = i as f64 / segments as f64;
            let t_inv = 1.0 - t;

            // Cubic Bezier formula: B(t) = (1-t)³P₀ + 3(1-t)²tP₁ + 3(1-t)t²P₂ + t³P₃
            let t_inv_sq = t_inv * t_inv;
            let t_sq = t * t;

            let x = t_inv_sq * t_inv * start.x
                + 3.0 * t_inv_sq * t * control1.x
                + 3.0 * t_inv * t_sq * control2.x
                + t_sq * t * end.x;

            let y = t_inv_sq * t_inv * start.y
                + 3.0 * t_inv_sq * t * control1.y
                + 3.0 * t_inv * t_sq * control2.y
                + t_sq * t * end.y;

            points.push(Point { x, y });
        }

        points
    }

    /// Renders an ellipse as a filled polygon using tessellation
    #[allow(clippy::too_many_arguments)]
    pub fn render_ellipse(
        &mut self,
        center_x: f64,
        center_y: f64,
        radius_x: f64,
        radius_y: f64,
        rotation: f64,
        start_angle: f64,
        end_angle: f64,
        render_state: &RenderState,
    ) {
        // Tessellate ellipse into points
        const SEGMENTS: usize = 64;
        let angle_diff = if end_angle > start_angle {
            end_angle - start_angle
        } else {
            (end_angle + 2.0 * std::f64::consts::PI) - start_angle
        };
        let step = angle_diff / SEGMENTS as f64;
        let mut points = Vec::new();

        for i in 0..=SEGMENTS {
            let angle = start_angle + (i as f64 * step);
            // Apply ellipse parameters
            let x = radius_x * angle.cos();
            let y = radius_y * angle.sin();

            // Apply rotation if needed
            let (rotated_x, rotated_y) = if rotation != 0.0 {
                let cos_rot = rotation.cos();
                let sin_rot = rotation.sin();
                (x * cos_rot - y * sin_rot, x * sin_rot + y * cos_rot)
            } else {
                (x, y)
            };

            points.push(crate::ext::canvas::renderer::Point {
                x: center_x + rotated_x,
                y: center_y + rotated_y,
            });
        }

        // Render as polygon
        self.render_polygon(points, render_state);
    }

    /// Renders a rounded rectangle as a filled polygon using tessellation
    pub fn render_rounded_rect(
        &mut self,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        radius: f64,
        render_state: &RenderState,
    ) {
        // Clamp radius to valid range
        let max_radius = (width.min(height)) / 2.0;
        let radius = radius.min(max_radius).max(0.0);

        if radius <= 0.0 {
            // Just render as regular rectangle
            let rect = Rect {
                start: Point { x, y },
                end: Point {
                    x: x + width,
                    y: y + height,
                },
            };
            self.render_rect(rect, render_state);
            return;
        }

        let mut points = Vec::new();
        const CORNER_SEGMENTS: usize = 8;
        let step = std::f64::consts::PI / (2.0 * CORNER_SEGMENTS as f64);

        // Top-left corner
        for i in 0..=CORNER_SEGMENTS {
            let angle = std::f64::consts::PI + (i as f64 * step);
            let corner_x = x + radius + radius * angle.cos();
            let corner_y = y + radius + radius * angle.sin();
            points.push(Point {
                x: corner_x,
                y: corner_y,
            });
        }

        // Top-right corner
        for i in 0..=CORNER_SEGMENTS {
            let angle = 1.5 * std::f64::consts::PI + (i as f64 * step);
            let corner_x = x + width - radius + radius * angle.cos();
            let corner_y = y + radius + radius * angle.sin();
            points.push(Point {
                x: corner_x,
                y: corner_y,
            });
        }

        // Bottom-right corner
        for i in 0..=CORNER_SEGMENTS {
            let angle = 0.0 + (i as f64 * step);
            let corner_x = x + width - radius + radius * angle.cos();
            let corner_y = y + height - radius + radius * angle.sin();
            points.push(Point {
                x: corner_x,
                y: corner_y,
            });
        }

        // Bottom-left corner
        for i in 0..=CORNER_SEGMENTS {
            let angle = 0.5 * std::f64::consts::PI + (i as f64 * step);
            let corner_x = x + radius + radius * angle.cos();
            let corner_y = y + height - radius + radius * angle.sin();
            points.push(Point {
                x: corner_x,
                y: corner_y,
            });
        }

        // Render as polygon
        self.render_polygon(points, render_state);
    }
}

pub fn translate_coords(point: &Point, dimensions: &Dimensions) -> (f32, f32) {
    let x = (point.x / (dimensions.width as f64) * 2.0 - 1.0) as f32;
    let y = (point.y / (dimensions.height as f64) * -2.0 + 1.0) as f32;
    (x, y)
}
