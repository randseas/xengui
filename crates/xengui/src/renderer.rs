/*
 * Copyright (C) 2026 randseas
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program. If not, see <https://www.gnu.org/licenses/>.
 */
// xengui/src/renderer.rs
use pollster::block_on;
use std::sync::Arc;
use wgpu_glyph::{ab_glyph, GlyphBrushBuilder};
use winit::window::Window;

use crate::VNode;

pub struct XenRenderer {
    pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub glyph_brush: wgpu_glyph::GlyphBrush<()>,
    pub staging_belt: wgpu::util::StagingBelt,
    pub config: wgpu::SurfaceConfiguration,
}

impl XenRenderer {
    pub fn new(window: Arc<Window>) -> Result<Self, String> {
        // Initialize WGPU boilerplate
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            #[cfg(target_os = "windows")]
            backends: wgpu::Backends::DX12,

            #[cfg(target_os = "macos")]
            backends: wgpu::Backends::METAL,

            #[cfg(target_os = "linux")]
            backends: wgpu::Backends::VULKAN,

            #[cfg(target_arch = "wasm32")]
            backends: wgpu::Backends::GL,

            #[cfg(not(any(
                target_os = "windows",
                target_os = "macos",
                target_os = "linux",
                target_arch = "wasm32"
            )))]
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let surface = instance
            .create_surface(window.clone())
            .map_err(|e| format!("Cannot create surface: {}", e))?;

        let adapter = block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
            .map_err(|e| format!("Cannot find a compatible adapter: {}", e))?;

        let (device, queue) = block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))
            .map_err(|e| format!("Cannot start GPU (device): {}", e))?;

        // Load font for debug overlay
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

        let font: &[u8] = include_bytes!("../fonts/Inter_Regular.ttf");
        let font_arc = ab_glyph::FontArc::try_from_slice(font)
            .expect("Cannot load system font: Inter_Regular.ttf corrupted or missing.");

        let glyph_brush = GlyphBrushBuilder::using_font(font_arc).build(&device, surface_format);

        let alpha_mode = surface_caps
            .alpha_modes
            .iter()
            .copied()
            .find(|&a| {
                a == wgpu::CompositeAlphaMode::PreMultiplied
                    || a == wgpu::CompositeAlphaMode::PostMultiplied
            })
            .unwrap_or(wgpu::CompositeAlphaMode::Auto);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: window.inner_size().width,
            height: window.inner_size().height,
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
            // RenderContext'i burada oluştur
            let background_color = match theme {
                Some(winit::window::Theme::Dark) => wgpu::Color::BLACK,
                Some(winit::window::Theme::Light) => wgpu::Color::WHITE,
                None => wgpu::Color::WHITE,
            };
            // RenderPass artık mut edilebilir bir değişken
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

            // MİMARİ DÜZELTME: render_pass yaşam döngüsündeyken node'lara iletiliyor.
            for vnode in tree.iter_mut() {
                vnode.render(&mut render_pass, &mut self.glyph_brush, theme, &debug_mode);
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
        mut tree: &mut [Box<dyn VNode>],
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
            self.render_frame(&mut tree, theme, debug_mode);
        }
    }
}
