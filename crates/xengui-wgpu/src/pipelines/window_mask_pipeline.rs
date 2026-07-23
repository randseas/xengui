// SPDX-License-Identifier: Apache-2.0

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 2],
    local_pos: [f32; 2],
    half_size: [f32; 2],
    radius: f32,
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
                    format: wgpu::VertexFormat::Float32,
                },
            ],
        }
    }
}

/// Punches a frameless, transparent wgpu surface's four corners
/// transparent so it reads as a rounded window instead of a rectangular
/// one, since the OS decorations that would normally clip it are disabled.
pub struct WindowMaskPipeline {
    pipeline: wgpu::RenderPipeline,
}

impl WindowMaskPipeline {
    pub fn new(device: &wgpu::Device, surface_format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Window Mask Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/window_mask.wgsl").into()),
        });

        let layout = device.create_pipeline_layout(
            &(wgpu::PipelineLayoutDescriptor {
                label: Some("Window Mask Pipeline Layout"),
                bind_group_layouts: &[],
                immediate_size: 0,
            })
        );

        // Erases pixels outside the rounded rect: dst *= (1 - outside).
        let erase_blend = wgpu::BlendState {
            color: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::Zero,
                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                operation: wgpu::BlendOperation::Add,
            },
            alpha: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::Zero,
                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                operation: wgpu::BlendOperation::Add,
            },
        };

        let pipeline = device.create_render_pipeline(
            &(wgpu::RenderPipelineDescriptor {
                label: Some("Window Mask Pipeline"),
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
                            blend: Some(erase_blend),
                            write_mask: wgpu::ColorWrites::ALL,
                        }),
                    ],
                }),
                multiview_mask: None,
                cache: None,
            })
        );

        Self { pipeline }
    }

    pub fn draw(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        render_pass: &mut wgpu::RenderPass<'_>,
        surface_width: u32,
        surface_height: u32,
        radius: f32
    ) {
        let w = surface_width as f32;
        let h = surface_height as f32;
        let half_w = w * 0.5;
        let half_h = h * 0.5;

        let ndc = |px: f32, py: f32| -> [f32; 2] { [px / half_w - 1.0, 1.0 - py / half_h] };

        let mk = |screen: [f32; 2], local: [f32; 2]| Vertex {
            position: screen,
            local_pos: local,
            half_size: [half_w, half_h],
            radius,
        };

        let vertices = [
            mk(ndc(0.0, 0.0), [-half_w, -half_h]),
            mk(ndc(w, 0.0), [half_w, -half_h]),
            mk(ndc(0.0, h), [-half_w, half_h]),
            mk(ndc(0.0, h), [-half_w, half_h]),
            mk(ndc(w, 0.0), [half_w, -half_h]),
            mk(ndc(w, h), [half_w, half_h]),
        ];

        let vertex_buffer = device.create_buffer(
            &(wgpu::BufferDescriptor {
                label: Some("Window Mask Vertex Buffer"),
                size: (vertices.len() * std::mem::size_of::<Vertex>()) as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        );
        queue.write_buffer(&vertex_buffer, 0, bytemuck::cast_slice(&vertices));

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        render_pass.set_viewport(0.0, 0.0, w, h, 0.0, 1.0);
        render_pass.draw(0..6, 0..1);
    }
}
