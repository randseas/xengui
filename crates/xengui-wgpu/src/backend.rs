// SPDX-License-Identifier: Apache-2.0
use crate::pipelines::{ ImagePipeline, RectPipeline, TextPipeline, TrianglePipeline };
use xengui::{
    Color,
    ImageCommand,
    RectCommand,
    RenderBackend,
    SystemTheme,
    TextCommand,
    TextMeasurer,
    TriangleCommand,
};

/// Owns the four wgpu render pipelines xengui needs, built once against a
/// device and reused across every frame via `begin_frame`.
pub struct WgpuPipelines {
    rect: RectPipeline,
    triangle: TrianglePipeline,
    image: ImagePipeline,
    text: TextPipeline,
}

impl WgpuPipelines {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        surface_format: wgpu::TextureFormat,
        user_fonts: Vec<(String, Vec<u8>)>
    ) -> Result<Self, String> {
        Ok(Self {
            rect: RectPipeline::new(device, surface_format),
            triangle: TrianglePipeline::new(device, surface_format),
            image: ImagePipeline::new(device, surface_format),
            text: TextPipeline::new(device, queue, surface_format, user_fonts)?,
        })
    }

    /// Borrows this pipeline set for exactly one frame, wired to the
    /// caller's own device/queue/encoder/render target. This is the entry
    /// point a host embedding xengui into its own render graph (e.g. a
    /// Bevy render node) should use directly instead of `WgpuWindowRenderer`.
    pub fn begin_frame<'a>(
        &'a mut self,
        device: &'a wgpu::Device,
        queue: &'a wgpu::Queue,
        encoder: &'a mut wgpu::CommandEncoder,
        view: &'a wgpu::TextureView,
        width: u32,
        height: u32
    ) -> WgpuFrame<'a> {
        self.rect.reset_frame();
        self.triangle.reset_frame();
        self.image.reset_frame();

        WgpuFrame {
            pipelines: self,
            device,
            queue,
            encoder,
            view,
            width,
            height,
            background: Color::TRANSPARENT,
            scale_factor: 1.0,
            shape_pass_open: false,
            text_cmds: Vec::new(),
        }
    }
}

/// A single frame's worth of borrowed GPU resources; implements
/// `RenderBackend` so `xengui::FrameRenderer` can draw into it without
/// knowing anything about wgpu.
pub struct WgpuFrame<'a> {
    pipelines: &'a mut WgpuPipelines,
    device: &'a wgpu::Device,
    queue: &'a wgpu::Queue,
    encoder: &'a mut wgpu::CommandEncoder,
    view: &'a wgpu::TextureView,
    width: u32,
    height: u32,
    background: Color,
    scale_factor: f32,
    shape_pass_open: bool,
    // Kept only to redraw glyph buffers if `flush_text` needs a retry
    // after a text-atlas resize.
    text_cmds: Vec<(SystemTheme, TextCommand)>,
}

impl<'a> WgpuFrame<'a> {
    // First shape draw of the frame clears with the app background;
    // every later one just loads what's already on screen.
    fn shape_pass_load(&mut self) -> wgpu::LoadOp<wgpu::Color> {
        if self.shape_pass_open {
            return wgpu::LoadOp::Load;
        }
        self.shape_pass_open = true;
        let bg = self.background;
        wgpu::LoadOp::Clear(wgpu::Color {
            r: bg.r() as f64,
            g: bg.g() as f64,
            b: bg.b() as f64,
            a: bg.a() as f64,
        })
    }
}

impl<'a> RenderBackend for WgpuFrame<'a> {
    fn text_measurer(&mut self) -> &mut dyn TextMeasurer {
        &mut self.pipelines.text
    }

    fn begin_frame(&mut self, background: Color, _width: u32, _height: u32) -> bool {
        self.background = background;
        true
    }

    fn draw_rects(&mut self, cmds: &[RectCommand]) {
        if cmds.is_empty() {
            return;
        }
        let load = self.shape_pass_load();
        let mut pass = self.encoder.begin_render_pass(
            &(wgpu::RenderPassDescriptor {
                label: Some("xengui shape pass"),
                color_attachments: &[
                    Some(wgpu::RenderPassColorAttachment {
                        view: self.view,
                        resolve_target: None,
                        ops: wgpu::Operations { load, store: wgpu::StoreOp::Store },
                        depth_slice: None,
                    }),
                ],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            })
        );
        self.pipelines.rect.draw_batch(
            self.device,
            self.queue,
            &mut pass,
            self.width,
            self.height,
            cmds
        );
    }

    fn draw_triangles(&mut self, cmds: &[TriangleCommand]) {
        if cmds.is_empty() {
            return;
        }
        let load = self.shape_pass_load();
        let mut pass = self.encoder.begin_render_pass(
            &(wgpu::RenderPassDescriptor {
                label: Some("xengui shape pass"),
                color_attachments: &[
                    Some(wgpu::RenderPassColorAttachment {
                        view: self.view,
                        resolve_target: None,
                        ops: wgpu::Operations { load, store: wgpu::StoreOp::Store },
                        depth_slice: None,
                    }),
                ],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            })
        );
        self.pipelines.triangle.draw_batch(
            self.device,
            self.queue,
            &mut pass,
            self.width,
            self.height,
            cmds
        );
    }

    fn draw_images(&mut self, cmds: &[ImageCommand]) {
        if cmds.is_empty() {
            return;
        }
        let load = self.shape_pass_load();
        let mut pass = self.encoder.begin_render_pass(
            &(wgpu::RenderPassDescriptor {
                label: Some("xengui shape pass"),
                color_attachments: &[
                    Some(wgpu::RenderPassColorAttachment {
                        view: self.view,
                        resolve_target: None,
                        ops: wgpu::Operations { load, store: wgpu::StoreOp::Store },
                        depth_slice: None,
                    }),
                ],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            })
        );
        self.pipelines.image.draw_batch(
            self.device,
            self.queue,
            &mut pass,
            self.width,
            self.height,
            cmds
        );
    }

    fn draw_text(&mut self, theme: SystemTheme, scale_factor: f32, cmd: &TextCommand) {
        self.scale_factor = scale_factor;
        self.pipelines.text.draw(scale_factor, theme, cmd);
        self.text_cmds.push((theme, cmd.clone()));
    }

    fn take_text_decorations(&mut self) -> Vec<RectCommand> {
        self.pipelines.text.take_decorations()
    }

    fn flush_text(&mut self) {
        const MAX_RETRIES: u32 = 3;
        let mut attempts = 0;
        loop {
            match
                self.pipelines.text.flush(
                    self.device,
                    self.queue,
                    self.encoder,
                    self.view,
                    self.width,
                    self.height
                )
            {
                Ok(()) => {
                    break;
                }
                Err(e) if attempts < MAX_RETRIES => {
                    attempts += 1;
                    log::warn!("text cache resize, retrying flush ({attempts}/{MAX_RETRIES}): {e}");
                    for (theme, cmd) in &self.text_cmds {
                        self.pipelines.text.draw(self.scale_factor, *theme, cmd);
                    }
                }
                Err(e) => {
                    log::error!("text drawing failed permanently, skipping frame: {e}");
                    break;
                }
            }
        }
    }

    fn end_frame(&mut self) {
        self.pipelines.text.trim_atlas();
    }

    fn resize(&mut self, _width: u32, _height: u32) {}
}
