// SPDX-License-Identifier: Apache-2.0
use crate::WgpuPipelines;
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

        let (device, queue) = pollster
            ::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))
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

        let surface = instance
            .create_surface(window)
            .map_err(|e| format!("Cannot create surface: {}", e))?;

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

        Self::init_common(surface, &adapter, device, queue, width, height, user_fonts)
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
        })
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

        {
            let mut backend = self.pipelines.begin_frame(
                &self.device,
                &self.queue,
                &mut encoder,
                &view,
                self.config.width,
                self.config.height
            );
            self.frame.render_frame(
                tree,
                &mut backend,
                theme,
                scale_factor,
                self.config.width,
                self.config.height
            );
        }

        self.queue.submit(Some(encoder.finish()));
        self.queue.present(frame);
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
