// SPDX-License-Identifier: Apache-2.0
use crate::{
    DrawCommand, LayoutContext, LayoutEngine, PaintContext, RectPipeline, TextPipeline, VNode,
};
use std::sync::Arc;
use winit::window::Window;

pub struct XenRenderer {
    pub window: Arc<Window>,
    pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub staging_belt: wgpu::util::StagingBelt,
    pub config: wgpu::SurfaceConfiguration,
    pub text_pipeline: TextPipeline,
    pub rect_pipeline: RectPipeline,
    pub debug: bool,
}

impl XenRenderer {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn new(
        window: Arc<Window>,
        user_fonts: Vec<(String, Vec<u8>)>,
        debug: bool,
    ) -> Result<Self, String> {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            #[cfg(target_os = "windows")]
            backends: wgpu::Backends::DX12,
            #[cfg(target_os = "macos")]
            backends: wgpu::Backends::METAL,
            #[cfg(target_os = "linux")]
            backends: wgpu::Backends::VULKAN,
            #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let surface = instance
            .create_surface(window.clone())
            .map_err(|e| format!("Cannot create surface: {}", e))?;

        let adapter =
            pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
                .expect("Cannot find a compatible adapter");

        let (device, queue) =
            pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))
                .map_err(|e| format!("Cannot start GPU (device): {}", e))?;

        Self::init_common(window, surface, adapter, device, queue, user_fonts, debug)
    }

    #[cfg(target_arch = "wasm32")]
    pub async fn new(
        window: Arc<Window>,
        user_fonts: Vec<(String, Vec<u8>)>,
    ) -> Result<Self, String> {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = instance
            .create_surface(window.clone())
            .map_err(|e| format!("Cannot create surface: {}", e))?;

        // Zero-blocking async pipeline for the browser event loop
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .expect("Cannot find a compatible adapter");

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default())
            .await
            .map_err(|e| format!("Cannot start GPU (device): {}", e))?;

        Self::init_common(window, surface, adapter, device, queue, user_fonts)
    }

    fn init_common(
        window: Arc<Window>,
        surface: wgpu::Surface<'static>,
        adapter: wgpu::Adapter,
        device: wgpu::Device,
        queue: wgpu::Queue,
        user_fonts: Vec<(String, Vec<u8>)>,
        debug: bool,
    ) -> Result<Self, String> {
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| {
                f == &wgpu::TextureFormat::Bgra8UnormSrgb
                    || f == &wgpu::TextureFormat::Rgba8UnormSrgb
            })
            .unwrap_or(surface_caps.formats[0]);

        let text_pipeline = TextPipeline::new(&device, surface_format, user_fonts)?;
        let rect_pipeline = RectPipeline::new(&device, surface_format);

        let alpha_mode = surface_caps
            .alpha_modes
            .iter()
            .copied()
            .find(|&a| {
                a == wgpu::CompositeAlphaMode::PreMultiplied
                    || a == wgpu::CompositeAlphaMode::PostMultiplied
            })
            .unwrap_or(wgpu::CompositeAlphaMode::Auto);

        // Prevent zero-sized texture allocations on web target by defaulting to at least 1px
        let width = window.inner_size().width.max(1);
        let height = window.inner_size().height.max(1);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width,
            height,
            present_mode: wgpu::PresentMode::Fifo,
            desired_maximum_frame_latency: 2,
            alpha_mode,
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        let staging_belt = wgpu::util::StagingBelt::new(device.clone(), 1024 * 1024);

        Ok(Self {
            window,
            surface,
            device,
            queue,
            staging_belt,
            config,
            text_pipeline,
            rect_pipeline,
            debug,
        })
    }

    pub fn render_frame(
        &mut self,
        tree: &mut [Box<dyn VNode>],
        theme: &Option<winit::window::Theme>,
    ) {
        let frame = match self.surface.get_current_texture() {
            Ok(frame) => frame,
            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                log::warn!("Surface lost/outdated, reconfiguring.");
                self.surface.configure(&self.device, &self.config);
                return; // bir sonraki redraw'da yeni surface ile tekrar denenecek
            }
            Err(wgpu::SurfaceError::OutOfMemory) => {
                log::error!("GPU out of memory, cannot continue.");
                std::process::exit(1);
            }
            Err(wgpu::SurfaceError::Timeout) => {
                log::debug!("Surface timeout, skipping frame.");
                return;
            }
            Err(e) => {
                log::warn!("Unhandled surface error: {e:?}");
                return;
            }
        };
        let view = frame.texture.create_view(&Default::default());
        let mut encoder = self.device.create_command_encoder(&Default::default());
        {
            let background_color = match theme {
                Some(winit::window::Theme::Dark) => wgpu::Color::BLACK,
                Some(winit::window::Theme::Light) => wgpu::Color::WHITE,
                None => wgpu::Color::WHITE,
            };

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(background_color),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            // Layout Pass
            let layout_ctx = LayoutContext {
                text: &self.text_pipeline,
                scale_factor: self.window.scale_factor() as f32,
                debug: self.debug,
            };

            LayoutEngine::layout(tree, &layout_ctx);

            // Paint Pass — VNode ağacı çizim komutlarını üretir.
            let mut commands = Vec::new();
            {
                let mut paint_ctx = PaintContext::new(&mut commands, self.debug);
                for node in tree.iter() {
                    node.paint(&mut paint_ctx);
                }
            }

            // Komutları türlerine göre ayır. `commands` sahibi bizde olduğundan
            // klonlamaya gerek yok; `into_iter()` ile taşıyoruz.
            let mut rect_cmds = Vec::with_capacity(commands.len());
            let mut text_cmds = Vec::new();

            for command in commands.into_iter() {
                match command {
                    DrawCommand::Rect(cmd) => rect_cmds.push(cmd),
                    DrawCommand::Text(cmd) => text_cmds.push(cmd),
                }
            }

            // Tüm rect'ler TEK vertex buffer yazımı + TEK draw çağrısıyla çizilir.
            self.rect_pipeline.draw_batch(
                &self.device,
                &self.queue,
                &mut render_pass,
                self.config.width,
                self.config.height,
                &rect_cmds,
            );

            // Text komutları glyph_brush'a kuyruklanır; gerçek çizim `flush()` ile
            // render_pass'tan SONRA (yeni bir encoder pass'i içinde) yapılır.
            let resolved_theme = theme.unwrap_or(winit::window::Theme::Dark);
            for cmd in &text_cmds {
                self.text_pipeline
                    .draw(self.window.scale_factor() as f32, resolved_theme, cmd);
            }

            drop(render_pass);

            const MAX_TEXT_FLUSH_RETRIES: u32 = 3;

            let mut attempts = 0;
            loop {
                match self.text_pipeline.flush(
                    &self.device,
                    &mut self.staging_belt,
                    &mut encoder,
                    &view,
                    frame.texture.width(),
                    frame.texture.height(),
                ) {
                    Ok(()) => break,
                    Err(e) if attempts < MAX_TEXT_FLUSH_RETRIES => {
                        attempts += 1;
                        log::warn!(
                            "Text cache resize, retrying flush ({attempts}/{MAX_TEXT_FLUSH_RETRIES}): {e}"
                        );
                        // glyph_brush zaten queue'yu ve texture'ı büyüttü; aynı komutları
                        // yeniden queue'lamamız gerekir çünkü draw_queued başarısız kalanı boşaltmış olabilir.
                        for cmd in &text_cmds {
                            self.text_pipeline.draw(
                                self.window.scale_factor() as f32,
                                resolved_theme,
                                cmd,
                            );
                        }
                    }
                    Err(e) => {
                        log::error!("Text drawing failed permanently, skipping frame: {e}");
                        return;
                    }
                }
            }
        }

        // finish buffers
        self.staging_belt.finish();
        // complete pipeline and presentate
        self.queue.submit(Some(encoder.finish()));
        frame.present();
        self.staging_belt.recall();
    }

    pub fn resize(
        &mut self,
        tree: &mut [Box<dyn VNode>],
        theme: &Option<winit::window::Theme>,
        size: winit::dpi::PhysicalSize<u32>,
    ) {
        if size.width == self.config.width && size.height == self.config.height {
            return; // aynı boyutla gelen tekrarlı event'i atla
        }
        if size.width > 0 && size.height > 0 {
            self.config.width = size.width.max(1);
            self.config.height = size.height.max(1);
            self.surface.configure(&self.device, &self.config);
            for node in tree.iter_mut() {
                node.set_dirty(true);
            }
            self.render_frame(tree, theme);
        }
    }
}
