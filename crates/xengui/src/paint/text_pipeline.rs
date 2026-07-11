// SPDX-License-Identifier: Apache-2.0
use crate::{
    FontDatabase, FontDescriptor, FontStyle, FontWeight, TextCommand, TextMetrics, TextRasterizer,
};
use wgpu_glyph::{GlyphBrushBuilder, Section, Text, ab_glyph};

pub struct TextPipeline {
    glyph_brush: wgpu_glyph::GlyphBrush<()>,
    font_db: FontDatabase,
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

                let (_locale, _region, sys_fonts) =
                    find_for_system_locale(system_fonts::FontStyle::Sans);
                let mut loaded_font = None;

                for font in sys_fonts {
                    if let system_fonts::FoundFontSource::Path(font_path) = font.source
                        && let Ok(font_bytes) = std::fs::read(&font_path)
                        && let Ok(font_arc) = ab_glyph::FontArc::try_from_vec(font_bytes)
                    {
                        loaded_font = Some(font_arc);
                        break;
                    }
                }

                loaded_font
                    .or_else(|| {
                        user_fonts.first().and_then(|(_, data)| {
                            ab_glyph::FontArc::try_from_vec(data.clone()).ok()
                        })
                    })
                    .ok_or_else(|| {
                        "No system font found and no fallback font provided.".to_string()
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

        let default_font = default_font_arc.clone();
        let mut glyph_brush =
            GlyphBrushBuilder::using_font(default_font_arc).build(device, surface_format);
        let mut font_db = FontDatabase::new(default_font);

        for (name, data) in user_fonts {
            if let Ok(user_font) = ab_glyph::FontArc::try_from_vec(data) {
                let id = glyph_brush.add_font(user_font.clone());
                font_db.register(name, user_font, id);
            }
        }

        Ok(Self {
            glyph_brush,
            font_db,
        })
    }

    pub fn draw(&mut self, scale_factor: f32, theme: winit::window::Theme, command: &TextCommand) {
        let color = command.style.color.unwrap_or(match theme {
            winit::window::Theme::Dark => crate::Color::WHITE,
            winit::window::Theme::Light => crate::Color::BLACK,
        });

        let scale = command
            .style
            .font_size
            .map(|s| s.to_physical(scale_factor))
            .unwrap_or(20.0 * scale_factor);

        let weight = command.style.font_weight.unwrap_or_default();
        let style = command.style.font_style.unwrap_or_default();
        let font_id = self
            .font_db
            .resolve_font_id(command.font.as_deref(), weight, style);

        let letter_spacing = command
            .style
            .letter_spacing
            .map(|ls| ls.value().to_physical(scale_factor))
            .unwrap_or(0.0);

        let rgba = [color.r(), color.g(), color.b(), color.a()];

        // Hızlı yol: harf aralığı yoksa tek section yeterli (glyph_brush kendi
        // shaping'ini uygular, en doğru sonuç budur).
        if letter_spacing.abs() < f32::EPSILON {
            let mut text = Text::new(&command.text).with_color(rgba).with_scale(scale);
            if let Some(id) = font_id {
                text = text.with_font_id(id);
            }
            self.glyph_brush.queue(
                Section::default()
                    .with_screen_position(command.position)
                    .add_text(text),
            );
            return;
        }

        // Harf aralığı varsa: her karakteri ayrı section olarak, manuel
        // kümülatif x ofsetiyle kuyruklarız. `measure()` ile BİREBİR aynı
        // ilerleme (advance) mantığını kullanmak zorunludur, aksi halde
        // layout kutusu ile gerçek çizim arasında (daha önce düzelttiğimiz
        // font_size uyuşmazlığı gibi) bir sapma oluşur.
        let font_arc = self
            .font_db
            .resolve_font(command.font.as_deref(), weight, style)
            .clone();
        let scaled = {
            use wgpu_glyph::ab_glyph::{Font, PxScale};
            font_arc.as_scaled(PxScale::from(scale))
        };

        let mut cursor_x = command.position.0;
        let mut buf = [0u8; 4];

        for ch in command.text.chars() {
            let ch_str = ch.encode_utf8(&mut buf);
            let mut text = Text::new(ch_str).with_color(rgba).with_scale(scale);
            if let Some(id) = font_id {
                text = text.with_font_id(id);
            }

            self.glyph_brush.queue(
                Section::default()
                    .with_screen_position((cursor_x, command.position.1))
                    .add_text(text),
            );

            use wgpu_glyph::ab_glyph::ScaleFont;
            let advance = scaled.h_advance(scaled.glyph_id(ch));
            cursor_x += advance + letter_spacing;
        }
    }

    pub fn measure(
        &self,
        text: &str,
        font: Option<&str>,
        font_size: f32,
        weight: FontWeight,
        style: FontStyle,
        letter_spacing: f32,
    ) -> (f32, f32) {
        use wgpu_glyph::ab_glyph::{Font, PxScale, ScaleFont};

        let font_arc = self.font_db.resolve_font(font, weight, style);
        let scaled = font_arc.as_scaled(PxScale::from(font_size));

        let chars: Vec<char> = text.chars().collect();
        let mut width = 0.0;

        for (i, &ch) in chars.iter().enumerate() {
            width += scaled.h_advance(scaled.glyph_id(ch));
            if i + 1 < chars.len() {
                width += letter_spacing;
            }
        }

        let height = scaled.height();
        (width, height)
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

impl TextRasterizer for TextPipeline {
    fn measure(&self, text: &str, font: &FontDescriptor) -> TextMetrics {
        let (width, height) = self.measure(
            text,
            font.family.as_deref(),
            font.size,
            font.weight,
            font.style,
            font.letter_spacing,
        );

        TextMetrics { width, height }
    }

    fn rasterize(&mut self, ch: char, font: &FontDescriptor) -> crate::GlyphBitmap {

        
    }
}
