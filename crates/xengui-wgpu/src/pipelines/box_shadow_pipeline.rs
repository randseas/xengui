// SPDX-License-Identifier: Apache-2.0
use xengui::{ BoxShadowCommand, paint };

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 2],
    local_pos: [f32; 2],
    half_size: [f32; 2],
    radius: f32,
    blur: f32,
    color: [f32; 4],
    inset: f32,
    box_local_pos: [f32; 2],
    box_half_size: [f32; 2],
    box_radius: f32,
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
                    format: wgpu::VertexFormat::Float32,
                },
                wgpu::VertexAttribute {
                    shader_location: 7,
                    offset: 52,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    shader_location: 8,
                    offset: 60,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    shader_location: 9,
                    offset: 68,
                    format: wgpu::VertexFormat::Float32,
                },
            ],
        }
    }
}

pub struct BoxShadowPipeline {
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    vertex_capacity: usize,
    write_offset: usize,
}

const VERTICES_PER_SHADOW: usize = 6;
const DEFAULT_SHADOW_CAPACITY: usize = 64;

impl BoxShadowPipeline {
    pub fn new(device: &wgpu::Device, surface_format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Box Shadow Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/box_shadow.wgsl").into()),
        });

        let layout = device.create_pipeline_layout(
            &(wgpu::PipelineLayoutDescriptor {
                label: Some("Box Shadow Pipeline Layout"),
                bind_group_layouts: &[],
                immediate_size: 0,
            })
        );

        let pipeline = device.create_render_pipeline(
            &(wgpu::RenderPipelineDescriptor {
                label: Some("Box Shadow Pipeline"),
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

        let vertex_capacity = DEFAULT_SHADOW_CAPACITY * VERTICES_PER_SHADOW;
        let vertex_buffer = device.create_buffer(
            &(wgpu::BufferDescriptor {
                label: Some("Box Shadow Vertex Buffer"),
                size: (vertex_capacity * std::mem::size_of::<Vertex>()) as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        );

        Self { pipeline, vertex_buffer, vertex_capacity, write_offset: 0 }
    }

    pub fn reset_frame(&mut self) {
        self.write_offset = 0;
    }

    pub fn draw_batch(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        render_pass: &mut wgpu::RenderPass<'_>,
        surface_width: u32,
        surface_height: u32,
        cmds: &[BoxShadowCommand]
    ) {
        if cmds.is_empty() {
            return;
        }

        let inv_w = 2.0 / (surface_width.max(1) as f32);
        let inv_h = 2.0 / (surface_height.max(1) as f32);
        let ndc = |px: f32, py: f32| -> [f32; 2] { [px * inv_w - 1.0, 1.0 - py * inv_h] };

        let mut vertices = Vec::with_capacity(cmds.len() * VERTICES_PER_SHADOW);

        for cmd in cmds {
            // Render quad follows whichever rect is actually visible: the
            // shadow rect for outset, the real box for inset (an inset
            // shadow never paints outside its own box).
            let (rx, ry) = if cmd.inset { cmd.box_position } else { cmd.shadow_position };
            let (rw, rh) = if cmd.inset { cmd.box_size } else { cmd.shadow_size };

            let pad = cmd.blur * 3.0 + 4.0;
            let cx = rx + rw * 0.5;
            let cy = ry + rh * 0.5;
            let quad_half_w = rw * 0.5 + pad;
            let quad_half_h = rh * 0.5 + pad;

            let shadow_half_size = [cmd.shadow_size.0 * 0.5, cmd.shadow_size.1 * 0.5];
            let shadow_center = (
                cmd.shadow_position.0 + shadow_half_size[0],
                cmd.shadow_position.1 + shadow_half_size[1],
            );
            let box_half_size = [cmd.box_size.0 * 0.5, cmd.box_size.1 * 0.5];
            let box_center = (
                cmd.box_position.0 + box_half_size[0],
                cmd.box_position.1 + box_half_size[1],
            );

            let color = cmd.color.to_f32_array();
            let inset = if cmd.inset { 1.0 } else { 0.0 };

            let corners = [
                (cx - quad_half_w, cy - quad_half_h),
                (cx + quad_half_w, cy - quad_half_h),
                (cx - quad_half_w, cy + quad_half_h),
                (cx + quad_half_w, cy + quad_half_h),
            ];

            let mk = |world: (f32, f32)| Vertex {
                position: ndc(world.0, world.1),
                local_pos: [world.0 - shadow_center.0, world.1 - shadow_center.1],
                half_size: shadow_half_size,
                radius: cmd.shadow_radius,
                blur: cmd.blur,
                color,
                inset,
                box_local_pos: [world.0 - box_center.0, world.1 - box_center.1],
                box_half_size,
                box_radius: cmd.box_radius,
            };

            vertices.extend_from_slice(
                &[
                    mk(corners[0]),
                    mk(corners[1]),
                    mk(corners[2]),
                    mk(corners[2]),
                    mk(corners[1]),
                    mk(corners[3]),
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

        let mut run_start = 0usize;
        let mut current_clip = cmds[0].clip_rect;

        for (i, cmd) in cmds.iter().enumerate().skip(1) {
            if cmd.clip_rect != current_clip {
                Self::draw_run(
                    render_pass,
                    base_vertex,
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
            base_vertex,
            run_start,
            cmds.len(),
            current_clip,
            surface_width,
            surface_height
        );
    }

    fn draw_run(
        render_pass: &mut wgpu::RenderPass<'_>,
        base_vertex: usize,
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
            (base_vertex + start * VERTICES_PER_SHADOW) as u32..(base_vertex +
                end * VERTICES_PER_SHADOW) as u32,
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
                label: Some("Box Shadow Vertex Buffer"),
                size: (self.vertex_capacity * std::mem::size_of::<Vertex>()) as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        );
    }
}
