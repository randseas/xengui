// SPDX-License-Identifier: Apache-2.0
use crate::{
    AnimationManager,
    DrawCommand,
    ImagePipeline,
    LayoutContext,
    LayoutEngine,
    PaintContext,
    RectPipeline,
    RenderCache,
    TextPipeline,
    TrianglePipeline,
    Widget,
};
use std::{ collections::HashSet, sync::Arc };
use winit::window::Window;
use web_time::Instant;

pub struct XenRenderer {
    pub window: Arc<Window>,
    pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub staging_belt: wgpu::util::StagingBelt,
    pub config: wgpu::SurfaceConfiguration,
    pub text_pipeline: TextPipeline,
    pub rect_pipeline: RectPipeline,
    pub triangle_pipeline: TrianglePipeline,
    pub image_pipeline: ImagePipeline,
    pub render_cache: RenderCache,
    pub anim: AnimationManager,
    last_tick: Instant,
}

impl XenRenderer {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn new(window: Arc<Window>, user_fonts: Vec<(String, Vec<u8>)>) -> Result<Self, String> {
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

        let adapter = pollster
            ::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
            .expect("Cannot find a compatible adapter");

        let (device, queue) = pollster
            ::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))
            .map_err(|e| format!("Cannot start GPU (device): {}", e))?;

        Self::init_common(window, surface, adapter, device, queue, user_fonts)
    }

    #[cfg(target_arch = "wasm32")]
    pub async fn new(
        window: Arc<Window>,
        user_fonts: Vec<(String, Vec<u8>)>
    ) -> Result<Self, String> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            flags: wgpu::InstanceFlags::default(),
            memory_budget_thresholds: wgpu::MemoryBudgetThresholds::default(),
            backend_options: wgpu::BackendOptions::default(),
            display: None,
        });

        let surface = instance
            .create_surface(window.clone())
            .map_err(|e| format!("Cannot create surface: {}", e))?;

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default()).await
            .expect("Cannot find a compatible adapter");

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default()).await
            .map_err(|e| format!("Cannot start GPU (device): {}", e))?;

        Self::init_common(window, surface, adapter, device, queue, user_fonts)
    }

    fn init_common(
        window: Arc<Window>,
        surface: wgpu::Surface<'static>,
        adapter: wgpu::Adapter,
        device: wgpu::Device,
        queue: wgpu::Queue,
        user_fonts: Vec<(String, Vec<u8>)>
    ) -> Result<Self, String> {
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats
            .iter()
            .copied()
            .find(|f| {
                f == &wgpu::TextureFormat::Bgra8Unorm || f == &wgpu::TextureFormat::Rgba8Unorm
            })
            .unwrap_or(surface_caps.formats[0]);

        let text_pipeline = TextPipeline::new(&device, &queue, surface_format, user_fonts)?;
        let rect_pipeline = RectPipeline::new(&device, surface_format);
        let triangle_pipeline = TrianglePipeline::new(&device, surface_format);
        let image_pipeline = ImagePipeline::new(&device, surface_format);

        let alpha_mode = surface_caps.alpha_modes
            .iter()
            .copied()
            .find(|&a| {
                a == wgpu::CompositeAlphaMode::PreMultiplied ||
                    a == wgpu::CompositeAlphaMode::PostMultiplied
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
            triangle_pipeline,
            image_pipeline,
            render_cache: RenderCache::new(),
            anim: AnimationManager::new(),
            last_tick: Instant::now(),
        })
    }

    pub fn render_frame(
        &mut self,
        tree: &mut [Box<dyn Widget>],
        theme: &Option<winit::window::Theme>
    ) {
        let now = Instant::now();
        let dt = now.duration_since(self.last_tick);
        self.last_tick = now;
        self.anim.tick(dt);

        let frame = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(surface_texture) => surface_texture,
            wgpu::CurrentSurfaceTexture::Suboptimal(surface_texture) => surface_texture,
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                log::warn!("Surface lost/outdated, reconfiguring.");
                self.surface.configure(&self.device, &self.config);
                return;
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

            let mut render_pass = encoder.begin_render_pass(
                &(wgpu::RenderPassDescriptor {
                    label: Some("Render Pass"),
                    color_attachments: &[
                        Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(background_color),
                                store: wgpu::StoreOp::Store,
                            },
                            depth_slice: None,
                        }),
                    ],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                    multiview_mask: None,
                })
            );

            let mut layout_ctx = LayoutContext {
                text: &mut self.text_pipeline,
                anim: &mut self.anim,
                scale_factor: self.window.scale_factor() as f32,
            };
            
            LayoutEngine::layout(
                tree,
                &mut layout_ctx,
                &mut self.render_cache,
                self.config.width as f32,
                self.config.height as f32
            );

            let mut commands = Vec::new();
            let mut live_keys: HashSet<String> = HashSet::new();

            for (i, node) in tree.iter().enumerate() {
                let segment = crate::path_segment(node.as_ref(), i);
                paint_recursive(
                    node.as_ref(),
                    &segment,
                    &mut self.render_cache,
                    &mut commands,
                    &mut live_keys,
                    None
                );
            }
            self.render_cache.retain_keys(&live_keys);

            for node in tree.iter_mut() {
                reset_dirty_recursive(node.as_mut());
            }

            let mut rect_cmds = Vec::with_capacity(commands.len());
            let mut triangle_cmds = Vec::new();
            let mut image_cmds = Vec::new();
            let mut text_cmds = Vec::new();
            for command in commands.into_iter() {
                match command {
                    DrawCommand::Rect(cmd) => rect_cmds.push(cmd),
                    DrawCommand::Triangle(cmd) => triangle_cmds.push(cmd),
                    DrawCommand::Text(cmd) => text_cmds.push(cmd),
                    DrawCommand::Image(cmd) => image_cmds.push(*cmd),
                }
            }

            self.rect_pipeline.draw_batch(
                &self.device,
                &self.queue,
                &mut render_pass,
                self.config.width,
                self.config.height,
                &rect_cmds
            );

            self.triangle_pipeline.draw_batch(
                &self.device,
                &self.queue,
                &mut render_pass,
                self.config.width,
                self.config.height,
                &triangle_cmds
            );

            self.image_pipeline.draw_batch(
                &self.device,
                &self.queue,
                &mut render_pass,
                self.config.width,
                self.config.height,
                &image_cmds
            );

            let resolved_theme = theme.unwrap_or(winit::window::Theme::Dark);
            for cmd in &text_cmds {
                self.text_pipeline.draw(self.window.scale_factor() as f32, resolved_theme, cmd);
            }

            // RectPipeline draws through a single shared GPU buffer, so all
            // rects - including underline/strike/overline quads produced
            // above - must go through one draw_batch call per frame.
            rect_cmds.extend(self.text_pipeline.take_decorations());

            self.rect_pipeline.draw_batch(
                &self.device,
                &self.queue,
                &mut render_pass,
                self.config.width,
                self.config.height,
                &rect_cmds
            );

            self.triangle_pipeline.draw_batch(
                &self.device,
                &self.queue,
                &mut render_pass,
                self.config.width,
                self.config.height,
                &triangle_cmds
            );

            self.image_pipeline.draw_batch(
                &self.device,
                &self.queue,
                &mut render_pass,
                self.config.width,
                self.config.height,
                &image_cmds
            );

            drop(render_pass);

            const MAX_TEXT_FLUSH_RETRIES: u32 = 3;

            let mut attempts = 0;
            loop {
                match
                    self.text_pipeline.flush(
                        &self.device,
                        &self.queue,
                        &mut encoder,
                        &view,
                        frame.texture.width(),
                        frame.texture.height()
                    )
                {
                    Ok(()) => {
                        break;
                    }
                    Err(e) if attempts < MAX_TEXT_FLUSH_RETRIES => {
                        attempts += 1;
                        log::warn!(
                            "Text cache resize, retrying flush ({attempts}/{MAX_TEXT_FLUSH_RETRIES}): {e}"
                        );
                        for cmd in &text_cmds {
                            self.text_pipeline.draw(
                                self.window.scale_factor() as f32,
                                resolved_theme,
                                cmd
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
        size: winit::dpi::PhysicalSize<u32>
    ) {
        if size.width == self.config.width && size.height == self.config.height {
            return;
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

fn paint_recursive(
    widget: &dyn Widget,
    path: &str,
    cache: &mut RenderCache,
    commands: &mut Vec<DrawCommand>,
    live_keys: &mut HashSet<String>,
    clip_rect: Option<(f32, f32, f32, f32)>
) {
    let layout_box = *widget.layout_box();

    // Off-screen subtree: nothing here can be visible, so skip painting
    // and recursing into it entirely. This is what makes scrolling cheap
    // regardless of how many rows exist outside the viewport.
    if let Some((cx, cy, cw, ch)) = clip_rect {
        let visible =
            layout_box.x < cx + cw &&
            layout_box.x + layout_box.width > cx &&
            layout_box.y < cy + ch &&
            layout_box.y + layout_box.height > cy;
        if !visible {
            return;
        }
    }

    live_keys.insert(path.to_string());

    let own_commands: Vec<DrawCommand> = match cache.try_reuse(path, layout_box, widget.is_dirty()) {
        Some(cached) => cached.to_vec(),
        None => {
            let mut local = Vec::new();
            {
                let mut paint_ctx = PaintContext::new(&mut local);
                widget.paint(&mut paint_ctx);
            }
            cache.store(path, layout_box, local.clone());
            local
        }
    };

    for mut command in own_commands {
        apply_clip(&mut command, clip_rect);
        commands.push(command);
    }

    let child_clip = match widget.clip_children() {
        Some(rect) => Some(clip_intersect(clip_rect, rect)),
        None => clip_rect,
    };

    for (i, child) in widget.children().iter().enumerate() {
        let segment = crate::path_segment(child.as_ref(), i);
        paint_recursive(
            child.as_ref(),
            &format!("{path}.{segment}"),
            cache,
            commands,
            live_keys,
            child_clip
        );
    }

    // Painted after every descendant so overlays (scrollbar thumbs, etc.)
    // stay on top; never cached since they depend on live interaction state.
    let mut overlay = Vec::new();
    {
        let mut paint_ctx = PaintContext::new(&mut overlay);
        widget.paint_overlay(&mut paint_ctx);
    }
    for mut command in overlay {
        apply_clip(&mut command, clip_rect);
        commands.push(command);
    }
}

fn clip_intersect(
    existing: Option<(f32, f32, f32, f32)>,
    ancestor: (f32, f32, f32, f32)
) -> (f32, f32, f32, f32) {
    let Some((ex, ey, ew, eh)) = existing else {
        return ancestor;
    };
    let (ax, ay, aw, ah) = ancestor;
    let x0 = ex.max(ax);
    let y0 = ey.max(ay);
    let x1 = (ex + ew).min(ax + aw);
    let y1 = (ey + eh).min(ay + ah);
    (x0, y0, (x1 - x0).max(0.0), (y1 - y0).max(0.0))
}

fn apply_clip(command: &mut DrawCommand, clip_rect: Option<(f32, f32, f32, f32)>) {
    let Some(ancestor_clip) = clip_rect else {
        return;
    };
    let target = match command {
        DrawCommand::Rect(cmd) => &mut cmd.clip_rect,
        DrawCommand::Image(cmd) => &mut cmd.clip_rect,
        DrawCommand::Text(cmd) => &mut cmd.clip_rect,
        DrawCommand::Triangle(cmd) => &mut cmd.clip_rect,
    };
    *target = Some(clip_intersect(*target, ancestor_clip));
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
