// SPDX-License-Identifier: Apache-2.0
use crate::{ WgpuPipelines, WindowChrome, WindowShadow };
use std::sync::Arc;
use xengui::{ FrameRenderer, SystemTheme, Widget };

/// Owns a wgpu device/surface for a native window and drives xengui's
/// `FrameRenderer` against it every frame. This is xenframe's default
/// integration point. Not winit-specific: `W` only needs to provide a
/// raw window/display handle, so any windowing crate works.
pub struct WgpuWindowRenderer {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pipelines: WgpuPipelines,
    frame: FrameRenderer,
    chrome: WindowChrome,
}

impl WgpuWindowRenderer {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn new<W>(
        window: Arc<W>,
        width: u32,
        height: u32,
        user_fonts: Vec<(String, Vec<u8>)>
    ) -> Result<Self, String>
        where W: wgpu::WindowHandle + raw_window_handle::HasDisplayHandle + 'static
    {
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
            .create_surface(window)
            .map_err(|e| format!("Cannot create surface: {}", e))?;

        let adapter = pollster
            ::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
            .expect("Cannot find a compatible adapter");

        log::trace!("adapter limits: {:?}", adapter.limits());

        let (device, queue) = pollster
            ::block_on(
                adapter.request_device(
                    &(wgpu::DeviceDescriptor {
                        required_limits: adapter.limits(),
                        ..Default::default()
                    })
                )
            )
            .map_err(|e| format!("Cannot start GPU (device): {}", e))?;

        Self::init_common(surface, &adapter, device, queue, width, height, user_fonts)
    }

    #[cfg(target_arch = "wasm32")]
    pub async fn new<W>(
        window: Arc<W>,
        width: u32,
        height: u32,
        user_fonts: Vec<(String, Vec<u8>)>
    ) -> Result<Self, String>
        where W: wgpu::WindowHandle + raw_window_handle::HasDisplayHandle + 'static
    {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            flags: wgpu::InstanceFlags::default(),
            memory_budget_thresholds: wgpu::MemoryBudgetThresholds::default(),
            backend_options: wgpu::BackendOptions::default(),
            display: None,
        });

        let t_surface = web_time::Instant::now();
        let surface = instance
            .create_surface(window)
            .map_err(|e| format!("Cannot create surface: {}", e))?;
        log::info!("phase: surface {:?}", t_surface.elapsed());

        let t_adapter = web_time::Instant::now();
        let adapter = instance
            .request_adapter(
                &(wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::HighPerformance,
                    compatible_surface: Some(&surface),
                    force_fallback_adapter: false,
                    apply_limit_buckets: false,
                })
            ).await
            .map_err(|e| format!("Cannot find a compatible adapter: {}", e))?;
        log::info!("phase: adapter {:?}", t_adapter.elapsed());

        let t_device = web_time::Instant::now();
        let (device, queue) = adapter
            .request_device(
                &(wgpu::DeviceDescriptor {
                    required_limits: adapter.limits(),
                    ..Default::default()
                })
            ).await
            .map_err(|e| format!("Cannot start GPU (device): {}", e))?;
        log::info!("phase: device {:?}", t_device.elapsed());

        let t_pipelines = web_time::Instant::now();
        let result = Self::init_common(surface, &adapter, device, queue, width, height, user_fonts);
        log::info!("phase: pipelines+fonts {:?}", t_pipelines.elapsed());
        result
    }

    fn init_common(
        surface: wgpu::Surface<'static>,
        adapter: &wgpu::Adapter,
        device: wgpu::Device,
        queue: wgpu::Queue,
        width: u32,
        height: u32,
        user_fonts: Vec<(String, Vec<u8>)>
    ) -> Result<Self, String> {
        let surface_caps = surface.get_capabilities(adapter);
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

        let pipelines = WgpuPipelines::new(&device, &queue, surface_format, user_fonts)?;

        let alpha_mode = surface_caps.alpha_modes
            .iter()
            .copied()
            .find(|&a| {
                a == wgpu::CompositeAlphaMode::PreMultiplied ||
                    a == wgpu::CompositeAlphaMode::PostMultiplied
            })
            .unwrap_or(wgpu::CompositeAlphaMode::Auto);

        log::info!(
            "surface alpha_mode selected: {:?} (available: {:?})",
            alpha_mode,
            surface_caps.alpha_modes
        );

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: width.max(1),
            height: height.max(1),
            present_mode: wgpu::PresentMode::Fifo,
            desired_maximum_frame_latency: 2,
            alpha_mode,
            view_formats: vec![],
            color_space: wgpu::SurfaceColorSpace::Auto,
        };
        surface.configure(&device, &config);

        Ok(Self {
            surface,
            device,
            queue,
            config,
            pipelines,
            frame: FrameRenderer::new(),
            chrome: WindowChrome::default(),
        })
    }

    /// Sets the window chrome (shadow, rounded corners, border) drawn
    /// around the widget tree's own output - only meaningful when the
    /// host window has no OS decorations (`decorations: false`).
    pub fn set_chrome(&mut self, chrome: WindowChrome) {
        self.chrome = chrome;
    }

    pub fn is_animating(&self) -> bool {
        self.frame.is_animating()
    }

    pub fn render_frame(
        &mut self,
        tree: &mut [Box<dyn Widget>],
        theme: SystemTheme,
        scale_factor: f32
    ) {
        let frame = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(t) => t,
            wgpu::CurrentSurfaceTexture::Suboptimal(t) => t,
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

        let mut chrome_shadow_drawn = false;
        if let Some(shadow) = self.chrome.shadow {
            self.draw_chrome_shadow(&mut encoder, &view, shadow, scale_factor);
            chrome_shadow_drawn = true;
        }

        {
            let mut backend = self.pipelines.begin_frame(
                &self.device,
                &self.queue,
                &mut encoder,
                &view,
                self.config.width,
                self.config.height
            );
            if chrome_shadow_drawn {
                backend.preserve_existing_content();
            }
            self.frame.render_frame(
                tree,
                &mut backend,
                theme,
                scale_factor,
                self.config.width,
                self.config.height
            );
        }

        if self.chrome.radius > 0.0 {
            self.punch_chrome_corners(&mut encoder, &view, scale_factor);
        }

        if let Some((width, color)) = self.chrome.border {
            self.draw_chrome_border(&mut encoder, &view, width, color, scale_factor);
        }

        self.queue.submit(Some(encoder.finish()));
        self.queue.present(frame);
    }

    fn draw_chrome_shadow(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        shadow: WindowShadow,
        scale_factor: f32
    ) {
        let margin = shadow.margin * scale_factor;
        let w = self.config.width as f32;
        let h = self.config.height as f32;
        let radius = self.chrome.radius * scale_factor;
        let blur = shadow.blur_radius * scale_factor;
        let spread = shadow.spread_radius * scale_factor;

        let box_position = (margin, margin);
        let box_size = ((w - margin * 2.0).max(0.0), (h - margin * 2.0).max(0.0));

        let cmd = xengui::BoxShadowCommand {
            shadow_position: (
                box_position.0 + shadow.offset.0 * scale_factor - spread,
                box_position.1 + shadow.offset.1 * scale_factor - spread,
            ),
            shadow_size: (box_size.0 + spread * 2.0, box_size.1 + spread * 2.0),
            shadow_radius: (radius + spread).max(0.0),
            blur,
            color: shadow.color,
            inset: false,
            box_position,
            box_size,
            box_radius: radius,
            clip_rect: None,
        };

        let mut pass = encoder.begin_render_pass(
            &(wgpu::RenderPassDescriptor {
                label: Some("xenframe window shadow pass"),
                color_attachments: &[
                    Some(wgpu::RenderPassColorAttachment {
                        view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
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
        self.pipelines.box_shadow.draw_batch(
            &self.device,
            &self.queue,
            &mut pass,
            self.config.width,
            self.config.height,
            &[cmd]
        );
    }

    fn punch_chrome_corners(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        scale_factor: f32
    ) {
        let margin = self.chrome.shadow.map(|s| s.margin).unwrap_or(0.0) * scale_factor;

        let mut pass = encoder.begin_render_pass(
            &(wgpu::RenderPassDescriptor {
                label: Some("xenframe window corner mask pass"),
                color_attachments: &[
                    Some(wgpu::RenderPassColorAttachment {
                        view,
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
        self.pipelines.window_mask.draw(
            &self.device,
            &self.queue,
            &mut pass,
            self.config.width,
            self.config.height,
            margin,
            self.chrome.radius * scale_factor
        );
    }

    fn draw_chrome_border(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        border_width: f32,
        color: xengui::Color,
        scale_factor: f32
    ) {
        let margin = self.chrome.shadow.map(|s| s.margin).unwrap_or(0.0) * scale_factor;
        let physical_width = border_width * scale_factor;
        let inset = physical_width * 0.5;

        let box_w = ((self.config.width as f32) - margin * 2.0).max(0.0);
        let box_h = ((self.config.height as f32) - margin * 2.0).max(0.0);

        let cmd = xengui::RectCommand {
            position: (margin + inset, margin + inset),
            size: ((box_w - physical_width).max(0.0), (box_h - physical_width).max(0.0)),
            background: None,
            border_radius: Some(
                xengui::Length::px((self.chrome.radius * scale_factor - inset).max(0.0))
            ),
            border_width: Some(xengui::Length::px(physical_width)),
            border_color: Some(color),
            clip_rect: None,
        };

        let mut pass = encoder.begin_render_pass(
            &(wgpu::RenderPassDescriptor {
                label: Some("xenframe window border pass"),
                color_attachments: &[
                    Some(wgpu::RenderPassColorAttachment {
                        view,
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
        self.pipelines.rect.draw_batch(
            &self.device,
            &self.queue,
            &mut pass,
            self.config.width,
            self.config.height,
            &[cmd]
        );
    }

    pub fn resize(
        &mut self,
        tree: &mut [Box<dyn Widget>],
        theme: SystemTheme,
        scale_factor: f32,
        width: u32,
        height: u32
    ) {
        if width == self.config.width && height == self.config.height {
            return;
        }
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
            self.frame.resize();
            self.render_frame(tree, theme, scale_factor);
        }
    }
}
