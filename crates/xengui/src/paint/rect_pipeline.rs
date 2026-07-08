// SPDX-License-Identifier: Apache-2.0
use crate::RectCommand;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 2],
    color: [f32; 4],
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
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

pub struct RectPipeline {
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    vertex_capacity: usize,
}

/// Bir rect kaç vertex'e denk geliyor (2 üçgen = 6 vertex).
const VERTICES_PER_RECT: usize = 6;
/// Frame ortasında sık sık realloc olmaması için makul bir başlangıç kapasitesi.
const DEFAULT_RECT_CAPACITY: usize = 256;

impl RectPipeline {
    pub fn new(device: &wgpu::Device, surface_format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Rect Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/rect.wgsl").into()),
        });

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Rect Pipeline Layout"),
            bind_group_layouts: &[],
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Rect Pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[Vertex::layout()],
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
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });

        let vertex_capacity = DEFAULT_RECT_CAPACITY * VERTICES_PER_RECT;

        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Rect Vertex Buffer"),
            size: (vertex_capacity * std::mem::size_of::<Vertex>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            pipeline,
            vertex_buffer,
            vertex_capacity,
        }
    }

    /// Bir frame'deki TÜM rect komutlarını tek vertex buffer yazımı ve
    /// tek draw çağrısıyla çizer. Komutlar tek tek çizilmez; aksi halde
    /// `queue.write_buffer` çağrıları `submit()` anına kadar sıraya girdiği
    /// için önceki rect'lerin verisi sonraki rect'inkiyle ezilir.
    pub fn draw_batch(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        render_pass: &mut wgpu::RenderPass<'_>,
        surface_width: u32,
        surface_height: u32,
        cmds: &[RectCommand],
    ) {
        if cmds.is_empty() {
            return;
        }

        let inv_w = 2.0 / surface_width.max(1) as f32;
        let inv_h = 2.0 / surface_height.max(1) as f32;
        let ndc = |px: f32, py: f32| -> [f32; 2] { [px * inv_w - 1.0, 1.0 - py * inv_h] };

        let mut vertices = Vec::with_capacity(cmds.len() * VERTICES_PER_RECT);

        for cmd in cmds {
            let background = match cmd.background.as_ref() {
                Some(crate::Background::Color(color)) => color,
                None => continue, // arka planı olmayan rect'i sessizce atla
            };
            let rgba = background.to_f32_array();

            let (x, y) = cmd.position;
            let (w, h) = cmd.size;

            let p0 = ndc(x, y);
            let p1 = ndc(x + w, y);
            let p2 = ndc(x, y + h);
            let p3 = ndc(x + w, y + h);

            vertices.extend_from_slice(&[
                Vertex {
                    position: p0,
                    color: rgba,
                },
                Vertex {
                    position: p1,
                    color: rgba,
                },
                Vertex {
                    position: p2,
                    color: rgba,
                },
                Vertex {
                    position: p2,
                    color: rgba,
                },
                Vertex {
                    position: p1,
                    color: rgba,
                },
                Vertex {
                    position: p3,
                    color: rgba,
                },
            ]);
        }

        if vertices.is_empty() {
            return;
        }

        self.ensure_capacity(device, vertices.len());

        queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&vertices));

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_viewport(
            0.0,
            0.0,
            surface_width as f32,
            surface_height as f32,
            0.0,
            1.0,
        );
        render_pass.set_scissor_rect(0, 0, surface_width, surface_height);

        render_pass.draw(0..vertices.len() as u32, 0..1);
    }

    fn ensure_capacity(&mut self, device: &wgpu::Device, required: usize) {
        if required <= self.vertex_capacity {
            return;
        }

        self.vertex_capacity = required.next_power_of_two();

        self.vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Rect Vertex Buffer"),
            size: (self.vertex_capacity * std::mem::size_of::<Vertex>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
    }
}
