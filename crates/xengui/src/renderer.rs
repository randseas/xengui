// SPDX-License-Identifier: Apache-2.0
// crates/xengui/src/renderer.rs
use crate::VNode;
use std::sync::Arc;
use wgpu_glyph::{GlyphBrushBuilder, ab_glyph};
use winit::window::Window;

pub struct XenRenderer {
    pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub glyph_brush: wgpu_glyph::GlyphBrush<()>,
    pub staging_belt: wgpu::util::StagingBelt,
    pub config: wgpu::SurfaceConfiguration,
    pub font_map: std::collections::HashMap<String, wgpu_glyph::FontId>, // Name -> ID match
}

impl XenRenderer {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn new(window: Arc<Window>, user_fonts: Vec<(String, Vec<u8>)>) -> Result<Self, String> {
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

        Self::init_common(window, surface, adapter, device, queue, user_fonts)
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

        let default_font_arc = {
            #[cfg(not(target_arch = "wasm32"))]
            {
                use system_fonts::find_for_system_locale;

                let (_locale, _region, fonts) = find_for_system_locale(system_fonts::FontStyle::Sans);
                let mut loaded_font = None;

                for font in fonts {
                    if let system_fonts::FoundFontSource::Path(font_path) = font.source
                        && let Ok(font_bytes) = std::fs::read(&font_path)
                            && let Ok(font_arc) = ab_glyph::FontArc::try_from_vec(font_bytes) {
                                loaded_font = Some(font_arc);
                                break;
                            }
                }
                loaded_font.ok_or_else(|| {
                    "Failed to load any native system font from system paths.".to_string()
                })?
            }

            #[cfg(target_arch = "wasm32")]
            {
                if user_fonts.is_empty() {
                    return Err(
                        "WASM target requires at least one font supplied."
                            .to_string(),
                    );
                }
                ab_glyph::FontArc::try_from_vec(user_fonts[0].1.clone())
                    .map_err(|_| "Invalid fallback font provided for WASM context.".to_string())?
            }
        };

        let mut glyph_brush =
            GlyphBrushBuilder::using_font(default_font_arc).build(&device, surface_format);
        let mut font_map = std::collections::HashMap::new();

        // Dynamic font registering
        for (name, data) in user_fonts {
            if let Ok(user_font) = ab_glyph::FontArc::try_from_vec(data) {
                let id = glyph_brush.add_font(user_font);
                font_map.insert(name, id);
            }
        }

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
            surface,
            device,
            queue,
            glyph_brush,
            staging_belt,
            config,
            font_map,
        })
    }

    pub fn render_frame(
        &mut self,
        tree: &mut [Box<dyn VNode>],
        theme: &Option<winit::window::Theme>,
        debug_mode: bool,
    ) {
        let frame = match self.surface.get_current_texture() {
            Ok(frame) => frame,
            Err(_) => return,
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

            for vnode in tree.iter_mut() {
                vnode.render(
                    &mut render_pass,
                    &mut self.glyph_brush,
                    &self.font_map,
                    theme,
                    &debug_mode,
                );
            }

            drop(render_pass);

            self.glyph_brush
                .draw_queued(
                    &self.device,
                    &mut self.staging_belt,
                    &mut encoder,
                    &view,
                    frame.texture.width(),
                    frame.texture.height(),
                )
                .expect("Drawing failed.");
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
        debug_mode: bool,
        size: winit::dpi::PhysicalSize<u32>,
    ) {
        if size.width > 0 && size.height > 0 {
            self.config.width = size.width.max(1);
            self.config.height = size.height.max(1);
            self.surface.configure(&self.device, &self.config);
            for node in tree.iter_mut() {
                node.set_dirty(true);
            }
            self.render_frame(tree, theme, debug_mode);
        }
    }
}
