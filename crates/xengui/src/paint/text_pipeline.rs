// SPDX-License-Identifier: Apache-2.0
use crate::TextCommand;
use wgpu_glyph::{GlyphBrushBuilder, Section, Text, ab_glyph};

pub struct TextPipeline {
    glyph_brush: wgpu_glyph::GlyphBrush<()>,
    font_map: std::collections::HashMap<String, wgpu_glyph::FontId>,
}

impl TextPipeline {
    pub fn new(
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        user_fonts: Vec<(String, Vec<u8>)>,
    ) -> Result<Self, String> {
        let default_font_arc = {
            #[cfg(not(target_arch = "wasm32"))]
            {
                use system_fonts::find_for_system_locale;

                let (_locale, _region, fonts) =
                    find_for_system_locale(system_fonts::FontStyle::Sans);
                let mut loaded_font = None;

                for font in fonts {
                    if let system_fonts::FoundFontSource::Path(font_path) = font.source
                        && let Ok(font_bytes) = std::fs::read(&font_path)
                        && let Ok(font_arc) = ab_glyph::FontArc::try_from_vec(font_bytes)
                    {
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
                    return Err("WASM target requires at least one font supplied.".to_string());
                }
                ab_glyph::FontArc::try_from_vec(user_fonts[0].1.clone())
                    .map_err(|_| "Invalid fallback font provided for WASM context.".to_string())?
            }
        };

        let mut glyph_brush =
            GlyphBrushBuilder::using_font(default_font_arc).build(device, surface_format);

        let mut font_map = std::collections::HashMap::new();

        // Dynamic font registering
        for (name, data) in user_fonts {
            if let Ok(user_font) = ab_glyph::FontArc::try_from_vec(data) {
                let id = glyph_brush.add_font(user_font);
                font_map.insert(name, id);
            }
        }

        Ok(Self {
            glyph_brush,
            font_map,
        })
    }

    pub fn draw(
        &mut self,
        _render_pass: &mut wgpu::RenderPass<'_>,
        scale_factor: f32,
        theme: winit::window::Theme,
        command: &TextCommand,
    ) {
        let color = command.style.color.unwrap_or(match theme {
            winit::window::Theme::Dark => crate::Color::WHITE,
            winit::window::Theme::Light => crate::Color::BLACK,
        });

        let scale = command
            .style
            .font_size
            .map(|s| s.to_physical(scale_factor))
            .unwrap_or(20.0);

        let mut text = Text::new(&command.text)
            .with_color([color.r(), color.g(), color.b(), color.a()])
            .with_scale(scale);

        if let Some(font_name) = &command.font
            && let Some(font_id) = self.font_map.get(font_name.as_str())
        {
            text = text.with_font_id(*font_id);
        }

        self.glyph_brush.queue(
            Section::default()
                .with_screen_position(command.position)
                .add_text(text),
        );
    }

    #[allow(clippy::too_many_arguments)]
    pub fn flush(
        &mut self,
        device: &wgpu::Device,
        staging_belt: &mut wgpu::util::StagingBelt,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        width: u32,
        height: u32,
    ) -> Result<(), String> {
        self.glyph_brush
            .draw_queued(device, staging_belt, encoder, view, width, height)
            .map_err(|e| e.to_string())
    }
}
