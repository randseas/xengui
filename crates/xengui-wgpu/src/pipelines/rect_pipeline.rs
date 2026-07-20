// SPDX-License-Identifier: Apache-2.0
use xengui::{ Background, RectCommand, paint };

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 2],
    local_pos: [f32; 2],
    half_size: [f32; 2],
    radius: f32,
    border_width: f32,
    fill_color: [f32; 4],
    border_color: [f32; 4],
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
                wgpu::VertexAttribute {
                    shader_location: 4,
                    offset: 28,
                    format: wgpu::VertexFormat::Float32,
                },
                wgpu::VertexAttribute {
                    shader_location: 5,
                    offset: 32,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    shader_location: 6,
                    offset: 48,
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

const VERTICES_PER_RECT: usize = 6;
const DEFAULT_RECT_CAPACITY: usize = 256;

impl RectPipeline {
    pub fn new(device: &wgpu::Device, surface_format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Rect Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/rect.wgsl").into()),
        });

        let layout = device.create_pipeline_layout(
            &(wgpu::PipelineLayoutDescriptor {
                label: Some("Rect Pipeline Layout"),
                bind_group_layouts: &[],
                immediate_size: 0,
            })
        );

        let pipeline = device.create_render_pipeline(
            &(wgpu::RenderPipelineDescriptor {
                label: Some("Rect Pipeline"),
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

        let vertex_capacity = DEFAULT_RECT_CAPACITY * VERTICES_PER_RECT;
        let vertex_buffer = device.create_buffer(
            &(wgpu::BufferDescriptor {
                label: Some("Rect Vertex Buffer"),
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
        cmds: &[RectCommand]
    ) {
        if cmds.is_empty() {
            return;
        }

        let inv_w = 2.0 / (surface_width.max(1) as f32);
        let inv_h = 2.0 / (surface_height.max(1) as f32);
        let ndc = |px: f32, py: f32| -> [f32; 2] { [px * inv_w - 1.0, 1.0 - py * inv_h] };

        let mut vertices = Vec::with_capacity(cmds.len() * VERTICES_PER_RECT);

        for cmd in cmds {
            let fill_color = match cmd.background.as_ref() {
                Some(Background::Color(color)) => color.to_f32_array(),
                None => [0.0, 0.0, 0.0, 0.0],
            };

            let (x, y) = cmd.position;
            let (w, h) = cmd.size;
            let half_w = w * 0.5;
            let half_h = h * 0.5;

            let radius = cmd.border_radius
                .map(|r| r.value())
                .unwrap_or(0.0)
                .clamp(0.0, half_w.min(half_h));

            let border_width = cmd.border_width.map(|bw| bw.value()).unwrap_or(0.0);
            let border_color = cmd.border_color
                .map(|c| c.to_f32_array())
                .unwrap_or([0.0, 0.0, 0.0, 0.0]);

            let half_size = [half_w, half_h];
            let p0 = ndc(x, y);
            let p1 = ndc(x + w, y);
            let p2 = ndc(x, y + h);
            let p3 = ndc(x + w, y + h);

            let local = |lx: f32, ly: f32| [lx, ly];

            let mk = |screen: [f32; 2], local_pos: [f32; 2]| Vertex {
                position: screen,
                local_pos,
                half_size,
                radius,
                border_width,
                fill_color,
                border_color,
            };

            vertices.extend_from_slice(
                &[
                    mk(p0, local(-half_w, -half_h)),
                    mk(p1, local(half_w, -half_h)),
                    mk(p2, local(-half_w, half_h)),
                    mk(p2, local(-half_w, half_h)),
                    mk(p1, local(half_w, -half_h)),
                    mk(p3, local(half_w, half_h)),
                ]
            );
        }

        self.ensure_capacity(device, vertices.len());
        queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&vertices));

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_viewport(0.0, 0.0, surface_width as f32, surface_height as f32, 0.0, 1.0);

        // Consecutive commands sharing the same clip rect are drawn in one
        // call; the scissor rect only changes when the clip actually does,
        // which keeps paint order intact while still clipping per-widget.
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
        let (sx, sy, sw, sh) = paint::draw_command::scissor_for_clip(
            clip,
            surface_width,
            surface_height
        );
        if sw == 0 || sh == 0 {
            return;
        }
        render_pass.set_scissor_rect(sx, sy, sw, sh);
        render_pass.draw(
            (start * VERTICES_PER_RECT) as u32..(end * VERTICES_PER_RECT) as u32,
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
                label: Some("Rect Vertex Buffer"),
                size: (self.vertex_capacity * std::mem::size_of::<Vertex>()) as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        );
    }
}
