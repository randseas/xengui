// SPDX-License-Identifier: Apache-2.0
use xengui::{ ImageCommand, ImageData, paint };
use std::collections::HashMap;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 2],
    uv: [f32; 2],
    local_pos: [f32; 2],
    half_size: [f32; 2],
    radius: f32,
    tint: [f32; 4],
}

impl Vertex {
    fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    shader_location: 0,
                    offset: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    shader_location: 1,
                    offset: 8,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    shader_location: 2,
                    offset: 16,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    shader_location: 3,
                    offset: 24,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    shader_location: 4,
                    offset: 32,
                    format: wgpu::VertexFormat::Float32,
                },
                wgpu::VertexAttribute {
                    shader_location: 5,
                    offset: 36,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

#[allow(dead_code)]
struct CachedTexture {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    sampler: wgpu::Sampler,
    bind_group: wgpu::BindGroup,
}

pub struct ImagePipeline {
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    vertex_buffer: wgpu::Buffer,
    vertex_capacity: usize,
    textures: HashMap<u64, CachedTexture>,
    write_offset: usize,
}

const VERTICES_PER_IMAGE: usize = 6;
const DEFAULT_IMAGE_CAPACITY: usize = 64;

impl ImagePipeline {
    pub fn new(device: &wgpu::Device, surface_format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Image Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/image.wgsl").into()),
        });

        let bind_group_layout = device.create_bind_group_layout(
            &(wgpu::BindGroupLayoutDescriptor {
                label: Some("Image Bind Group Layout"),
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
            })
        );

        let layout = device.create_pipeline_layout(
            &(wgpu::PipelineLayoutDescriptor {
                label: Some("Image Pipeline Layout"),
                bind_group_layouts: &[Some(&bind_group_layout)],
                immediate_size: 0,
            })
        );

        let pipeline = device.create_render_pipeline(
            &(wgpu::RenderPipelineDescriptor {
                label: Some("Image Pipeline"),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    compilation_options: Default::default(),
                    buffers: &[Some(Vertex::layout())],
                },
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    ..Default::default()
                },
                depth_stencil: None,
                multisample: Default::default(),
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    compilation_options: Default::default(),
                    targets: &[
                        Some(wgpu::ColorTargetState {
                            format: surface_format,
                            blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                            write_mask: wgpu::ColorWrites::ALL,
                        }),
                    ],
                }),
                multiview_mask: None,
                cache: None,
            })
        );

        let vertex_capacity = DEFAULT_IMAGE_CAPACITY * VERTICES_PER_IMAGE;
        let vertex_buffer = device.create_buffer(
            &(wgpu::BufferDescriptor {
                label: Some("Image Vertex Buffer"),
                size: (vertex_capacity * std::mem::size_of::<Vertex>()) as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        );

        Self {
            pipeline,
            bind_group_layout,
            vertex_buffer,
            vertex_capacity,
            textures: HashMap::new(),
            write_offset: 0,
        }
    }

    pub fn reset_frame(&mut self) {
        self.write_offset = 0;
    }

    fn ensure_texture(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        image: &std::sync::Arc<ImageData>
    ) {
        if self.textures.contains_key(&image.id) {
            return;
        }

        let width = image.width.max(1);
        let height = image.height.max(1);
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(
            &(wgpu::TextureDescriptor {
                label: Some("xengui image texture"),
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            })
        );

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &image.rgba,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            size
        );

        let view = texture.create_view(&Default::default());
        let sampler = device.create_sampler(
            &(wgpu::SamplerDescriptor {
                label: Some("xengui image sampler"),
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                ..Default::default()
            })
        );

        let bind_group = device.create_bind_group(
            &(wgpu::BindGroupDescriptor {
                label: Some("xengui image bind group"),
                layout: &self.bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&sampler),
                    },
                ],
            })
        );

        self.textures.insert(image.id, CachedTexture {
            texture,
            view,
            sampler,
            bind_group,
        });
    }

    pub fn draw_batch(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        render_pass: &mut wgpu::RenderPass<'_>,
        surface_width: u32,
        surface_height: u32,
        cmds: &[ImageCommand]
    ) {
        let live_ids: std::collections::HashSet<u64> = cmds
            .iter()
            .map(|c| c.image.id)
            .collect();
        self.textures.retain(|id, _| live_ids.contains(id));

        if cmds.is_empty() {
            return;
        }

        for cmd in cmds {
            self.ensure_texture(device, queue, &cmd.image);
        }

        let inv_w = 2.0 / (surface_width.max(1) as f32);
        let inv_h = 2.0 / (surface_height.max(1) as f32);
        let ndc = |px: f32, py: f32| -> [f32; 2] { [px * inv_w - 1.0, 1.0 - py * inv_h] };

        let mut vertices = Vec::with_capacity(cmds.len() * VERTICES_PER_IMAGE);

        for cmd in cmds {
            let (x, y) = cmd.position;
            let (w, h) = cmd.size;
            let half_w = w * 0.5;
            let half_h = h * 0.5;

            let radius = cmd.border_radius
                .map(|r| r.value())
                .unwrap_or(0.0)
                .clamp(0.0, half_w.min(half_h));

            let tint = cmd.tint.map(|c| c.to_f32_array()).unwrap_or([1.0, 1.0, 1.0, 1.0]);

            let half_size = [half_w, half_h];
            let p0 = ndc(x, y);
            let p1 = ndc(x + w, y);
            let p2 = ndc(x, y + h);
            let p3 = ndc(x + w, y + h);

            let local = |lx: f32, ly: f32| [lx, ly];

            let mk = |screen: [f32; 2], local_pos: [f32; 2], uv: [f32; 2]| Vertex {
                position: screen,
                uv,
                local_pos,
                half_size,
                radius,
                tint,
            };

            vertices.extend_from_slice(
                &[
                    mk(p0, local(-half_w, -half_h), [0.0, 0.0]),
                    mk(p1, local(half_w, -half_h), [1.0, 0.0]),
                    mk(p2, local(-half_w, half_h), [0.0, 1.0]),
                    mk(p2, local(-half_w, half_h), [0.0, 1.0]),
                    mk(p1, local(half_w, -half_h), [1.0, 0.0]),
                    mk(p3, local(half_w, half_h), [1.0, 1.0]),
                ]
            );
        }

        let base_vertex = self.write_offset;
        self.ensure_capacity(device, base_vertex + vertices.len());
        queue.write_buffer(
            &self.vertex_buffer,
            (base_vertex * std::mem::size_of::<Vertex>()) as u64,
            bytemuck::cast_slice(&vertices)
        );
        self.write_offset += vertices.len();

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_viewport(0.0, 0.0, surface_width as f32, surface_height as f32, 0.0, 1.0);

        for (i, cmd) in cmds.iter().enumerate() {
            let Some(cached) = self.textures.get(&cmd.image.id) else {
                continue;
            };

            let (sx, sy, sw, sh) = paint::draw_command::scissor_for_clip(
                cmd.clip_rect,
                surface_width,
                surface_height
            );
            if sw == 0 || sh == 0 {
                continue;
            }
            render_pass.set_scissor_rect(sx, sy, sw, sh);

            render_pass.set_bind_group(0, &cached.bind_group, &[]);
            let start = (base_vertex + i * VERTICES_PER_IMAGE) as u32;
            render_pass.draw(start..start + (VERTICES_PER_IMAGE as u32), 0..1);
        }
    }

    fn ensure_capacity(&mut self, device: &wgpu::Device, required: usize) {
        if required <= self.vertex_capacity {
            return;
        }
        self.vertex_capacity = required.next_power_of_two();
        self.vertex_buffer = device.create_buffer(
            &(wgpu::BufferDescriptor {
                label: Some("Image Vertex Buffer"),
                size: (self.vertex_capacity * std::mem::size_of::<Vertex>()) as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        );
    }
}
