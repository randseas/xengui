// SPDX-License-Identifier: Apache-2.0
use crate::{
    AnimationManager,
    DrawCommand,
    ImageCommand,
    ImagePipeline,
    LayoutContext,
    LayoutEngine,
    PaintContext,
    RectCommand,
    RectPipeline,
    RenderCache,
    TextCommand,
    TextPipeline,
    TriangleCommand,
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
    // Forces a full layout pass on the next render_frame without marking
    // individual widgets dirty, so unaffected widgets keep reusing their
    // cached measurement and paint commands.
    force_layout: bool,
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

        // compatible_surface ensures the adapter picked can actually render to
        // this surface - without it, browsers with partial WebGPU support (like
        // Safari) may hand back an adapter that later fails at device/surface
        // configuration instead of failing fast here with a clear error.
        let adapter = instance
            .request_adapter(
                &(wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::LowPower,
                    compatible_surface: Some(&surface),
                    force_fallback_adapter: false,
                    apply_limit_buckets: false,
                })
            ).await
            .map_err(|e| format!("Cannot find a compatible adapter: {}", e))?;

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
        let Some(surface_format) = surface_caps.formats
            .iter()
            .copied()
            .find(|f| {
                f == &wgpu::TextureFormat::Bgra8Unorm || f == &wgpu::TextureFormat::Rgba8Unorm
            })
            .or_else(|| surface_caps.formats.first().copied()) else {
            return Err(
                "Surface reports no supported texture formats (GPU/browser incompatibility).".to_string()
            );
        };

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
            force_layout: false,
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
            // uses the app's own active theme
            let app_background = crate::current_theme().background;
            let background_color = wgpu::Color {
                r: app_background.r() as f64,
                g: app_background.g() as f64,
                b: app_background.b() as f64,
                a: app_background.a() as f64,
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

            let needs_full_layout =
                std::mem::take(&mut self.force_layout) ||
                tree_is_dirty(tree) ||
                self.anim.active_keys().any(|k| k.property.affects_layout());

            let mut layout_ctx = LayoutContext {
                text: &mut self.text_pipeline,
                anim: &mut self.anim,
                scale_factor: self.window.scale_factor() as f32,
            };

            if needs_full_layout {
                LayoutEngine::layout(
                    tree,
                    &mut layout_ctx,
                    &mut self.render_cache,
                    self.config.width as f32,
                    self.config.height as f32
                );
            } else {
                LayoutEngine::cascade(tree, &mut layout_ctx);
            }

            let mut commands: Vec<(i32, DrawCommand)> = Vec::new();
            let mut focus_commands: Vec<RectCommand> = Vec::new();
            let mut live_keys: HashSet<String> = HashSet::new();

            let scale_factor = self.window.scale_factor() as f32;
            for (i, node) in tree.iter().enumerate() {
                let segment = crate::path_segment(node.as_ref(), i);
                paint_recursive(
                    node.as_ref(),
                    &segment,
                    &mut self.render_cache,
                    &mut commands,
                    &mut focus_commands,
                    &mut live_keys,
                    None,
                    scale_factor
                );
            }
            self.render_cache.retain_keys(&live_keys);

            for node in tree.iter_mut() {
                reset_dirty_recursive(node.as_mut());
            }

            // Stable sort keeps original paint order for widgets sharing
            // the same z-index; only different values get reordered.
            commands.sort_by_key(|(z, _)| *z);

            #[derive(PartialEq, Clone, Copy)]
            enum RunKind {
                Rect,
                Triangle,
                Image,
            }

            let mut current_kind: Option<RunKind> = None;
            let mut rect_buf: Vec<RectCommand> = Vec::new();
            let mut tri_buf: Vec<TriangleCommand> = Vec::new();
            let mut img_buf: Vec<ImageCommand> = Vec::new();
            let mut text_cmds: Vec<TextCommand> = Vec::new();

            macro_rules! flush_run {
                () => {
                    match current_kind {
                        Some(RunKind::Rect) => {
                            self.rect_pipeline.draw_batch(
                                &self.device,
                                &self.queue,
                                &mut render_pass,
                                self.config.width,
                                self.config.height,
                                &rect_buf
                            );
                        }
                        Some(RunKind::Triangle) => {
                            self.triangle_pipeline.draw_batch(
                                &self.device,
                                &self.queue,
                                &mut render_pass,
                                self.config.width,
                                self.config.height,
                                &tri_buf
                            );
                        }
                        Some(RunKind::Image) => {
                            self.image_pipeline.draw_batch(
                                &self.device,
                                &self.queue,
                                &mut render_pass,
                                self.config.width,
                                self.config.height,
                                &img_buf
                            );
                        }
                        None => {}
                    }
                    rect_buf.clear();
                    tri_buf.clear();
                    img_buf.clear();
                };
            }

            // Draws each contiguous run of same-type commands in the order
            // z-index (then paint order) puts them in, instead of always
            // drawing every rect, then every triangle, then every image.
            for (_, command) in commands {
                match command {
                    DrawCommand::Text(cmd) => {
                        text_cmds.push(*cmd);
                    }
                    DrawCommand::Rect(cmd) => {
                        if current_kind != Some(RunKind::Rect) {
                            flush_run!();
                            current_kind = Some(RunKind::Rect);
                        }
                        rect_buf.push(cmd);
                    }
                    DrawCommand::Triangle(cmd) => {
                        if current_kind != Some(RunKind::Triangle) {
                            flush_run!();
                            current_kind = Some(RunKind::Triangle);
                        }
                        tri_buf.push(cmd);
                    }
                    DrawCommand::Image(cmd) => {
                        if current_kind != Some(RunKind::Image) {
                            flush_run!();
                            current_kind = Some(RunKind::Image);
                        }
                        img_buf.push(*cmd);
                    }
                }
            }
            flush_run!();

            let resolved_theme = theme.unwrap_or(winit::window::Theme::Dark);
            for cmd in &text_cmds {
                self.text_pipeline.draw(self.window.scale_factor() as f32, resolved_theme, cmd);
            }

            // Underline/strike/overline quads produced while queueing text
            // above; drawn once, after every layer, instead of being
            // folded back into an already-drawn rect batch.
            let decorations = self.text_pipeline.take_decorations();
            if !decorations.is_empty() {
                self.rect_pipeline.draw_batch(
                    &self.device,
                    &self.queue,
                    &mut render_pass,
                    self.config.width,
                    self.config.height,
                    &decorations
                );
            }

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

            // Drawn in its own pass, after text, so the focus ring is
            // always visible above absolutely everything else in the frame.
            if !focus_commands.is_empty() {
                let mut focus_pass = encoder.begin_render_pass(
                    &(wgpu::RenderPassDescriptor {
                        label: Some("Focus Ring Pass"),
                        color_attachments: &[
                            Some(wgpu::RenderPassColorAttachment {
                                view: &view,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: wgpu::LoadOp::Load,
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
                self.rect_pipeline.draw_batch(
                    &self.device,
                    &self.queue,
                    &mut focus_pass,
                    self.config.width,
                    self.config.height,
                    &focus_commands
                );
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
            self.force_layout = true;
            self.render_frame(tree, theme);
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn paint_recursive(
    widget: &dyn Widget,
    path: &str,
    cache: &mut RenderCache,
    commands: &mut Vec<(i32, DrawCommand)>,
    focus_commands: &mut Vec<RectCommand>,
    live_keys: &mut HashSet<String>,
    clip_rect: Option<(f32, f32, f32, f32)>,
    scale_factor: f32
) {
    let layout_box = *widget.layout_box();

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

    let z_index = widget.computed_style().z_index.unwrap_or(0);

    let own_commands: Vec<DrawCommand> = match cache.try_reuse(path, layout_box, widget.is_dirty()) {
        Some(cached) => cached.to_vec(),
        None => {
            let mut local = Vec::new();
            {
                let mut paint_ctx = PaintContext::new(&mut local, scale_factor);
                widget.paint(&mut paint_ctx);
            }
            cache.store(path, layout_box, local.clone());
            local
        }
    };

    for mut command in own_commands {
        apply_clip(&mut command, clip_rect);
        commands.push((z_index, command));
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
            focus_commands,
            live_keys,
            child_clip,
            scale_factor
        );
    }

    // Painted after every descendant so overlays (scrollbar thumbs, etc.)
    // stay on top of this widget's own subtree; never cached since it
    // depends on live interaction state.
    let mut overlay = Vec::new();
    {
        let mut paint_ctx = PaintContext::new(&mut overlay, scale_factor);
        widget.paint_overlay(&mut paint_ctx);
    }
    for mut command in overlay {
        apply_clip(&mut command, clip_rect);
        commands.push((z_index, command));
    }

    // Collected separately from normal content so it can be drawn in its
    // own pass, above absolutely everything, regardless of z-index or
    // tree position; never cached since it depends on live focus state.
    let mut focus_local = Vec::new();
    {
        let mut paint_ctx = PaintContext::new(&mut focus_local, scale_factor);
        widget.paint_focus(&mut paint_ctx);
    }
    for mut command in focus_local {
        apply_clip(&mut command, clip_rect);
        if let DrawCommand::Rect(rect_cmd) = command {
            focus_commands.push(rect_cmd);
        }
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

fn tree_is_dirty(tree: &[Box<dyn Widget>]) -> bool {
    tree.iter().any(|w| widget_dirty_recursive(w.as_ref()))
}

fn widget_dirty_recursive(widget: &dyn Widget) -> bool {
    widget.is_dirty() ||
        widget
            .children()
            .iter()
            .any(|c| widget_dirty_recursive(c.as_ref()))
}
