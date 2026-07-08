use std::collections::HashMap;

// SPDX-License-Identifier: Apache-2.0
use crate::{FontStyle, FontWeight, TextCommand};
use wgpu_glyph::{GlyphBrushBuilder, Section, Text, ab_glyph};

pub struct TextPipeline {
    glyph_brush: wgpu_glyph::GlyphBrush<()>,
    font_map: HashMap<String, wgpu_glyph::FontId>,
    fonts: HashMap<String, ab_glyph::FontArc>,
    default_font: ab_glyph::FontArc,
}

impl TextPipeline {
    pub fn new(
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        user_fonts: Vec<(String, Vec<u8>)>,
    ) -> Result<Self, String> {
        let mut fonts = HashMap::new();

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
        let mut font_map = HashMap::new();

        for (name, data) in user_fonts {
            if let Ok(user_font) = ab_glyph::FontArc::try_from_vec(data) {
                fonts.insert(name.clone(), user_font.clone());
                let id = glyph_brush.add_font(user_font);
                font_map.insert(name, id);
            }
        }

        Ok(Self {
            glyph_brush,
            font_map,
            fonts,
            default_font,
        })
    }

    /// `{family}-{Weight}-{Style}` -> `{family}-{Weight}` -> `{family}` sırasıyla
    /// en spesifik eşleşen kayıtlı fontu bulur. Hiçbiri yoksa `None` döner
    /// (çağıran taraf default fonta düşer).
    fn resolve_font_id(
        &self,
        family: Option<&str>,
        weight: FontWeight,
        style: FontStyle,
    ) -> Option<wgpu_glyph::FontId> {
        let family = family?;

        let composite = format!("{family}-{weight:?}-{style:?}");
        if let Some(id) = self.font_map.get(&composite) {
            return Some(*id);
        }

        let weight_only = format!("{family}-{weight:?}");
        if let Some(id) = self.font_map.get(&weight_only) {
            return Some(*id);
        }

        self.font_map.get(family).copied()
    }

    fn resolve_font_arc(
        &self,
        family: Option<&str>,
        weight: FontWeight,
        style: FontStyle,
    ) -> &ab_glyph::FontArc {
        let Some(family) = family else {
            return &self.default_font;
        };

        let composite = format!("{family}-{weight:?}-{style:?}");
        if let Some(font) = self.fonts.get(&composite) {
            return font;
        }

        let weight_only = format!("{family}-{weight:?}");
        if let Some(font) = self.fonts.get(&weight_only) {
            return font;
        }

        self.fonts.get(family).unwrap_or(&self.default_font)
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
        let font_id = self.resolve_font_id(command.font.as_deref(), weight, style);

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
            .resolve_font_arc(command.font.as_deref(), weight, style)
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

        let font_arc = self.resolve_font_arc(font, weight, style);
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
