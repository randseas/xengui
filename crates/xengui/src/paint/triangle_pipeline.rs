// SPDX-License-Identifier: Apache-2.0
use crate::TriangleCommand;

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

pub struct TrianglePipeline {
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    vertex_capacity: usize,
}

const VERTICES_PER_TRIANGLE: usize = 3;
const DEFAULT_TRIANGLE_CAPACITY: usize = 64;

impl TrianglePipeline {
    pub fn new(device: &wgpu::Device, surface_format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Triangle Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/triangle.wgsl").into()),
        });

        let layout = device.create_pipeline_layout(
            &(wgpu::PipelineLayoutDescriptor {
                label: Some("Triangle Pipeline Layout"),
                bind_group_layouts: &[],
                immediate_size: 0,
            })
        );

        let pipeline = device.create_render_pipeline(
            &(wgpu::RenderPipelineDescriptor {
                label: Some("Triangle Pipeline"),
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

        let vertex_capacity = DEFAULT_TRIANGLE_CAPACITY * VERTICES_PER_TRIANGLE;
        let vertex_buffer = device.create_buffer(
            &(wgpu::BufferDescriptor {
                label: Some("Triangle Vertex Buffer"),
                size: (vertex_capacity * std::mem::size_of::<Vertex>()) as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        );

        Self {
            pipeline,
            vertex_buffer,
            vertex_capacity,
        }
    }

    pub fn draw_batch(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        render_pass: &mut wgpu::RenderPass<'_>,
        surface_width: u32,
        surface_height: u32,
        cmds: &[TriangleCommand]
    ) {
        if cmds.is_empty() {
            return;
        }

        let inv_w = 2.0 / (surface_width.max(1) as f32);
        let inv_h = 2.0 / (surface_height.max(1) as f32);
        let ndc = |px: f32, py: f32| -> [f32; 2] { [px * inv_w - 1.0, 1.0 - py * inv_h] };

        let mut vertices = Vec::with_capacity(cmds.len() * VERTICES_PER_TRIANGLE);

        for cmd in cmds {
            let color = cmd.color.to_f32_array();
            vertices.push(Vertex { position: ndc(cmd.p0.0, cmd.p0.1), color });
            vertices.push(Vertex { position: ndc(cmd.p1.0, cmd.p1.1), color });
            vertices.push(Vertex { position: ndc(cmd.p2.0, cmd.p2.1), color });
        }

        self.ensure_capacity(device, vertices.len());
        queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&vertices));

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_viewport(0.0, 0.0, surface_width as f32, surface_height as f32, 0.0, 1.0);

        let mut run_start = 0usize;
        let mut current_clip = cmds[0].clip_rect;

        for (i, cmd) in cmds.iter().enumerate().skip(1) {
            if cmd.clip_rect != current_clip {
                Self::draw_run(
                    render_pass,
                    run_start,
                    i,
                    current_clip,
                    surface_width,
                    surface_height
                );
                run_start = i;
                current_clip = cmd.clip_rect;
            }
        }
        Self::draw_run(
            render_pass,
            run_start,
            cmds.len(),
            current_clip,
            surface_width,
            surface_height
        );
    }

    fn draw_run(
        render_pass: &mut wgpu::RenderPass<'_>,
        start: usize,
        end: usize,
        clip: Option<(f32, f32, f32, f32)>,
        surface_width: u32,
        surface_height: u32
    ) {
        let (sx, sy, sw, sh) = crate::paint::draw_command::scissor_for_clip(
            clip,
            surface_width,
            surface_height
        );
        if sw == 0 || sh == 0 {
            return;
        }
        render_pass.set_scissor_rect(sx, sy, sw, sh);
        render_pass.draw(
            (start * VERTICES_PER_TRIANGLE) as u32..(end * VERTICES_PER_TRIANGLE) as u32,
            0..1
        );
    }

    fn ensure_capacity(&mut self, device: &wgpu::Device, required: usize) {
        if required <= self.vertex_capacity {
            return;
        }
        self.vertex_capacity = required.next_power_of_two();
        self.vertex_buffer = device.create_buffer(
            &(wgpu::BufferDescriptor {
                label: Some("Triangle Vertex Buffer"),
                size: (self.vertex_capacity * std::mem::size_of::<Vertex>()) as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        );
    }
}
