// SPDX-License-Identifier: Apache-2.0
use crate::{
    DrawCommand, LayoutContext, LayoutEngine, PaintContext, RectPipeline, RenderCache,
    TextPipeline, Widget,
};
use std::{collections::HashSet, sync::Arc};
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
    pub render_cache: RenderCache,
    pub debug: bool,
}

impl XenRenderer {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn new(
        window: Arc<Window>,
        user_fonts: Vec<(String, Vec<u8>)>,
        debug: bool,
    ) -> Result<Self, String> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: if cfg!(target_os = "windows") {
                wgpu::Backends::DX12
            } else if cfg!(target_os = "macos") {
                wgpu::Backends::METAL
            } else if cfg!(target_os = "linux") {
                wgpu::Backends::VULKAN
            } else {
                wgpu::Backends::PRIMARY
            },
            flags: wgpu::InstanceFlags::default(),
            memory_budget_thresholds: wgpu::MemoryBudgetThresholds::default(),
            backend_options: wgpu::BackendOptions::default(),
            display: None,
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
        debug: bool,
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

        Self::init_common(window, surface, adapter, device, queue, user_fonts, debug)
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
                f == &wgpu::TextureFormat::Bgra8Unorm || f == &wgpu::TextureFormat::Rgba8Unorm
            })
            .unwrap_or(surface_caps.formats[0]);

        let text_pipeline = TextPipeline::new(&device, &queue, surface_format, user_fonts)?;
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
            color_space: wgpu::SurfaceColorSpace::Auto,
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
            render_cache: RenderCache::new(),
            debug,
        })
    }

    pub fn render_frame(
        &mut self,
        tree: &mut [Box<dyn Widget>],
        theme: &Option<winit::window::Theme>,
    ) {
        let frame = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(surface_texture) => surface_texture,
            wgpu::CurrentSurfaceTexture::Suboptimal(surface_texture) => surface_texture,
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                log::warn!("Surface lost/outdated, reconfiguring.");
                self.surface.configure(&self.device, &self.config);
                return; // bir sonraki redraw'da yeni surface ile tekrar denenecek
            }
            wgpu::CurrentSurfaceTexture::Timeout => {
                log::debug!("Surface timeout, skipping frame.");
                return;
            }
            wgpu::CurrentSurfaceTexture::Occluded => {
                log::debug!("Surface occluded, skipping frame.");
                return;
            }
            wgpu::CurrentSurfaceTexture::Validation => {
                log::warn!("Surface validation error, skipping frame.");
                return;
            }
            #[allow(unreachable_patterns)]
            _ => {
                log::warn!("Unhandled surface texture state, skipping frame.");
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

            // Layout Pass — artık taffy ile (flex/grid destekli).
            let mut layout_ctx = LayoutContext {
                text: &mut self.text_pipeline,
                scale_factor: self.window.scale_factor() as f32,
                debug: self.debug,
            };
            LayoutEngine::layout(
                tree,
                &mut layout_ctx,
                &self.render_cache,
                self.config.width as f32,
                self.config.height as f32,
            );

            // Paint Pass — TÜM ağaç (children dahil) recursive gezilir.
            // Cache key'i artık `Widget::key()`'e değil, ağaçtaki konuma
            // (path, ör. "0.1.2") dayanır; bu, key() hiç set edilmemiş
            // widget'larda (View gibi) önceki `.unwrap()` panic'ini de
            // kalıcı olarak ortadan kaldırır.
            let mut commands = Vec::new();
            let mut live_keys: HashSet<String> = HashSet::new();

            for (i, node) in tree.iter().enumerate() {
                paint_recursive(
                    node.as_ref(),
                    &i.to_string(),
                    &mut self.render_cache,
                    &mut commands,
                    &mut live_keys,
                    self.debug,
                );
            }
            self.render_cache.retain_keys(&live_keys);

            for node in tree.iter_mut() {
                reset_dirty_recursive(node.as_mut());
            }

            let mut rect_cmds = Vec::with_capacity(commands.len());
            let mut text_cmds = Vec::new();
            for command in commands.into_iter() {
                match command {
                    DrawCommand::Rect(cmd) => rect_cmds.push(cmd),
                    DrawCommand::Text(cmd) => text_cmds.push(cmd),
                }
            }

            self.rect_pipeline.draw_batch(
                &self.device,
                &self.queue,
                &mut render_pass,
                self.config.width,
                self.config.height,
                &rect_cmds,
            );

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
                    &self.queue,
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
        self.queue.present(frame);
        self.staging_belt.recall();
    }

    pub fn resize(
        &mut self,
        tree: &mut [Box<dyn Widget>],
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
                set_dirty_recursive(node.as_mut());
            }
            self.render_frame(tree, theme);
        }
    }
}

/// Bir widget'ı ve TÜM alt ağacını (children) recursive olarak paint eder.
/// Dirty olmayan ve layout box'ı değişmemiş dallar için cache'ten aynen
/// yeniden kullanılır — RenderCache'in sağladığı optimizasyon artık tek
/// seviyeli değil, tüm ağaç derinliğinde çalışıyor.
fn paint_recursive(
    widget: &dyn Widget,
    path: &str,
    cache: &mut RenderCache,
    commands: &mut Vec<DrawCommand>,
    live_keys: &mut HashSet<String>,
    debug: bool,
) {
    live_keys.insert(path.to_string());
    let layout_box = *widget.layout_box();

    if let Some(cached) = cache.try_reuse(path, layout_box, widget.is_dirty()) {
        commands.extend_from_slice(cached);
    } else {
        let mut local = Vec::new();
        {
            let mut paint_ctx = PaintContext::new(&mut local, debug);
            widget.paint(&mut paint_ctx);
        }
        cache.store(path, layout_box, local.clone());
        commands.extend(local);
    }

    for (i, child) in widget.children().iter().enumerate() {
        paint_recursive(
            child.as_ref(),
            &format!("{path}.{i}"),
            cache,
            commands,
            live_keys,
            debug,
        );
    }
}

fn reset_dirty_recursive(widget: &mut dyn Widget) {
    widget.set_dirty(false);
    if let Some(children) = widget.children_mut() {
        for child in children.iter_mut() {
            reset_dirty_recursive(child.as_mut());
        }
    }
}

fn set_dirty_recursive(widget: &mut dyn Widget) {
    widget.set_dirty(true);
    if let Some(children) = widget.children_mut() {
        for child in children.iter_mut() {
            set_dirty_recursive(child.as_mut());
        }
    }
}
