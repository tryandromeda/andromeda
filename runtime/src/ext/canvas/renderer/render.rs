#![allow(dead_code)]
use image;
use wgpu::PollType;

use super::*;

pub struct Renderer {
    pub device: wgpu::Device,
    pub pipeline: wgpu::RenderPipeline,
    pub dimensions: Dimensions,
    pub buffers: Buffers,
    pub view: wgpu::TextureView,
    pub queue: wgpu::Queue,
    pub encoder: Option<wgpu::CommandEncoder>,
    pub bind_group_layout: wgpu::BindGroupLayout,
}

const U32_SIZE: u32 = std::mem::size_of::<u32>() as u32;
pub const UNIFORM_LENGTH: usize = 32;

impl Renderer {
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
        let bind_group_desc = wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0, // all uniforms
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        };
        let bind_group_layout = device.create_bind_group_layout(&bind_group_desc);
        let layout_desc = &wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        };
        let pipeline_layout = device.create_pipeline_layout(layout_desc);
        // Define color target with optional state as per wgpu newer API
        let color_targets: [Option<wgpu::ColorTargetState>; 1] = [Some(wgpu::ColorTargetState {
            format,
            blend: None,
            write_mask: wgpu::ColorWrites::ALL,
        })];
        let pipeline_desc = wgpu::RenderPipelineDescriptor {
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
                targets: &color_targets,
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
        };
        let pipeline = device.create_render_pipeline(&pipeline_desc);
        let encoder =
            Some(device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None }));

        let background = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
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
        let uniforms = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            mapped_at_creation: false,
            size: UNIFORM_LENGTH as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
        });
        let buffers = Buffers {
            vertex: Vec::new(),
            uniforms: vec![uniforms],
            props: [0; UNIFORM_LENGTH],
            bind_group: Vec::new(),
            commands: Vec::new(),
            background,
        };
        let view = buffers.background.create_view(&Default::default());

        Self {
            device,
            pipeline,
            dimensions,
            buffers,
            view,
            queue,
            encoder,
            bind_group_layout,
        }
    }

    pub fn render_all(&mut self) {
        let encoder = self.encoder.as_mut().unwrap();
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &self.view,
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
        for command in &self.buffers.commands {
            pass.set_bind_group(0, &self.buffers.bind_group[command.bind_group], &[]);
            pass.set_vertex_buffer(0, self.buffers.vertex[command.vertex].slice(..));
            pass.draw(0..command.length, 0..1);
        }
    }

    pub async fn create_bitmap(&mut self) -> Vec<u8> {
        // Calculate bytes per row with proper alignment
        let bytes_per_pixel = U32_SIZE; // 4 bytes per pixel for RGBA
        let unpadded_bytes_per_row = self.dimensions.width * bytes_per_pixel;
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let padded_bytes_per_row = ((unpadded_bytes_per_row + align - 1) / align) * align;

        // Create a new buffer with the correct size to accommodate padding
        let padded_buffer_size =
            (padded_bytes_per_row * self.dimensions.height) as wgpu::BufferAddress;
        let result_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            mapped_at_creation: false,
            size: padded_buffer_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        });

        let encoder = self.encoder.as_mut().unwrap();
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                aspect: wgpu::TextureAspect::All,
                mip_level: 0,
                texture: &self.buffers.background,
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
        let encoder = self.encoder.take().unwrap();
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

    pub fn set_uniform_at(&mut self, data: Vec<f32>, offset: usize) {
        let mut buf = Vec::new();
        for bytes in data {
            buf.extend(bytes.to_le_bytes());
        }
        self.buffers.props[..buf.len()].copy_from_slice(&buf[..]);
        if let Some(last) = &self.buffers.commands.last() {
            if self.buffers.uniforms.len() > last.uniforms {
                return self.queue.write_buffer(
                    self.buffers.uniforms.last().unwrap(),
                    offset as u64,
                    &self.buffers.props,
                );
            }
        }
        let uniforms = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            mapped_at_creation: false,
            size: self.buffers.props.len() as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
        });
        self.buffers.uniforms.push(uniforms);
        self.queue.write_buffer(
            self.buffers.uniforms.last().unwrap(),
            offset as u64,
            &self.buffers.props,
        );
    }

    pub fn render_rect(&mut self, rect: Rect, color: Color) {
        // Set the color uniform
        self.set_uniform_at(vec![color[0], color[1], color[2], color[3]], 0);

        let start = translate_coords(&rect.start, &self.dimensions);
        let x1 = (start.0 as f32).to_le_bytes();
        let y1 = (start.1 as f32).to_le_bytes();
        let end = translate_coords(&rect.end, &self.dimensions);
        let x2 = (end.0 as f32).to_le_bytes();
        let y2 = (end.1 as f32).to_le_bytes();
        let data = [x1, y1, x1, y2, x2, y1, x2, y2];
        let mut buf = Vec::new();
        for bytes in &data {
            buf.extend(bytes);
        }
        self.buffers.commands.push(RenderCommand {
            bind_group: self.buffers.bind_group.len(),
            uniforms: self.buffers.uniforms.len(),
            vertex: self.buffers.vertex.len(),
            length: 4,
        });
        let vertex = self.create_vertex_buffer(32);
        self.buffers.vertex.push(vertex);
        let bind_group = self.create_bind_group(self.buffers.uniforms.last().unwrap());
        self.buffers.bind_group.push(bind_group);
        self.queue
            .write_buffer(self.buffers.vertex.last().unwrap(), 0, &buf);
    }

    pub fn render_polygon(&mut self, polygon: Path) {
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
        let mut buf = Vec::new();
        for point in data {
            let (x, y) = translate_coords(point, &self.dimensions);
            buf.extend((x as f32).to_le_bytes());
            buf.extend((y as f32).to_le_bytes());
        }
        self.buffers.commands.push(RenderCommand {
            bind_group: self.buffers.bind_group.len(),
            uniforms: self.buffers.uniforms.len(),
            vertex: self.buffers.vertex.len(),
            length: polygon.len() as u32,
        });
        let vertex = self.create_vertex_buffer((polygon.len() as u64) * 8);
        self.buffers.vertex.push(vertex);
        let bind_group = self.create_bind_group(self.buffers.uniforms.last().unwrap());
        self.buffers.bind_group.push(bind_group);
        self.queue
            .write_buffer(self.buffers.vertex.last().unwrap(), 0, &buf);
    }

    pub async fn save_as_png(&mut self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        // First render all pending operations
        self.render_all();

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

    fn create_vertex_buffer(&self, size: u64) -> wgpu::Buffer {
        self.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            mapped_at_creation: false,
            size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX,
        })
    }

    fn create_bind_group(&self, buffer: &wgpu::Buffer) -> wgpu::BindGroup {
        self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer,
                    offset: 0,
                    size: None,
                }),
            }],
        })
    }
}

pub fn translate_coords(point: &Point, dimensions: &Dimensions) -> (f64, f64) {
    let x = point.x / (dimensions.width as f64) * 2.0 - 1.0;
    let y = point.y / (dimensions.height as f64) * -2.0 + 1.0;
    (x, y)
}
