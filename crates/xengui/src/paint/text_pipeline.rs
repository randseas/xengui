// SPDX-License-Identifier: Apache-2.0
use crate::{FontStyle, FontWeight, TextCommand};
use glyphon::{
    Attrs, Buffer as GlyphonBuffer, Cache, Color as GlyphonColor, Family, FontSystem, Metrics,
    Resolution, Shaping, Style as GlyphonStyle, SwashCache, TextArea, TextAtlas, TextBounds,
    TextRenderer, Viewport, Weight as GlyphonWeight,
};
use std::collections::HashMap;

struct PendingText {
    buffer: GlyphonBuffer,
    position: (f32, f32),
    color: GlyphonColor,
    bounds: TextBounds,
}

pub struct TextPipeline {
    font_system: FontSystem,
    swash_cache: SwashCache,
    atlas: TextAtlas,
    renderer: TextRenderer,
    viewport: Viewport,
    /// Maps a user-supplied logical font name (as passed via `command.font`)
    /// to the family name glyphon/fontdb actually indexed it under.
    user_font_map: HashMap<String, String>,
    /// Family name to fall back to when no font is requested. `None` means
    /// "let fontdb pick a generic sans-serif", which is what we do on
    /// desktop where system fonts are auto-loaded by `FontSystem::new()`.
    default_family_name: Option<String>,
    pending: Vec<PendingText>,
}

impl TextPipeline {
    /// Snaps a physical pixel size to the nearest whole pixel, biased
    /// upward for small sizes.
    ///
    /// Without hinting, small glyphs (roughly under ~16px) lose enough
    /// stroke width on a plain `round()` that stems and serifs can vanish
    /// entirely at certain sizes. Rounding those up rather than to nearest
    /// keeps thin strokes from dropping below one rasterized pixel, which
    /// reads as sharper even though it's technically less precise.
    #[inline]
    fn snap(px: f32) -> f32 {
        //if px <= 20.0 { px.ceil() } else { px.round() }
        px.round()
    }

    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        surface_format: wgpu::TextureFormat,
        user_fonts: Vec<(String, Vec<u8>)>,
    ) -> Result<Self, String> {
        #[cfg(target_arch = "wasm32")]
        if user_fonts.is_empty() {
            return Err("WASM target requires at least one font supplied.".to_string());
        }

        // FontSystem::new() already loads system fonts on non-wasm targets
        // (fontdb::Database::load_system_fonts under the hood), so there's
        // no need to hunt for a default font manually the way the
        // ab_glyph/system_fonts version did.
        let mut font_system = FontSystem::new();
        let mut user_font_map: HashMap<String, String> = HashMap::new();

        for (name, data) in &user_fonts {
            let before = font_system.db().faces().count();
            font_system.db_mut().load_font_data(data.clone());

            if font_system.db().faces().count() > before
                && let Some(face) = font_system.db().faces().last()
                && let Some((family_name, _)) = face.families.first()
            {
                user_font_map.insert(name.clone(), family_name.clone());
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        let default_family_name: Option<String> = None;

        #[cfg(target_arch = "wasm32")]
        let default_family_name: Option<String> = {
            let name = user_fonts
                .first()
                .and_then(|(name, _)| user_font_map.get(name).cloned());
            match name {
                Some(n) => Some(n),
                None => {
                    return Err("Invalid fallback font provided for WASM context.".to_string());
                }
            }
        };

        let swash_cache = SwashCache::new();
        let cache = Cache::new(device);
        let mut atlas = TextAtlas::new(device, queue, &cache, surface_format);
        let renderer =
            TextRenderer::new(&mut atlas, device, wgpu::MultisampleState::default(), None);
        let viewport = Viewport::new(device, &cache);

        Ok(Self {
            font_system,
            swash_cache,
            atlas,
            renderer,
            viewport,
            user_font_map,
            default_family_name,
            pending: Vec::new(),
        })
    }

    /// Builds an `Attrs` tied to explicit field references rather than
    /// `&self`, so callers can hold it alongside a `&mut self.font_system`
    /// borrow without the borrow checker treating it as a whole-`self` loan.
    fn resolve_attrs<'a>(
        user_font_map: &'a HashMap<String, String>,
        default_family_name: &'a Option<String>,
        font: Option<&str>,
        weight: FontWeight,
        style: FontStyle,
    ) -> Attrs<'a> {
        let family = font
            .and_then(|n| user_font_map.get(n))
            .map(|s| Family::Name(s.as_str()))
            .unwrap_or_else(|| {
                default_family_name
                    .as_deref()
                    .map(Family::Name)
                    .unwrap_or(Family::SansSerif)
            });

        Attrs::new()
            .family(family)
            .weight(convert_weight(weight))
            .style(convert_style(style))
    }

    #[allow(clippy::too_many_arguments)]
    fn queue_run(
        &mut self,
        text: &str,
        font: Option<&str>,
        weight: FontWeight,
        style: FontStyle,
        scale: f32,
        position: (f32, f32),
        color: GlyphonColor,
    ) {
        let attrs = Self::resolve_attrs(
            &self.user_font_map,
            &self.default_family_name,
            font,
            weight,
            style,
        );

        let metrics = Metrics::new(scale, scale * 1.25);
        let mut buffer = GlyphonBuffer::new(&mut self.font_system, metrics);
        buffer.set_size(Some(f32::MAX), Some(f32::MAX));
        buffer.set_text(text, &attrs, Shaping::Advanced, None);
        buffer.shape_until_scroll(&mut self.font_system, false);

        self.pending.push(PendingText {
            buffer,
            position,
            color,
            bounds: TextBounds {
                left: position.0 as i32,
                top: position.1 as i32,
                right: i32::MAX,
                bottom: i32::MAX,
            },
        });
    }

    fn measure_raw(
        &mut self,
        text: &str,
        font: Option<&str>,
        weight: FontWeight,
        style: FontStyle,
        scale: f32,
        line_height: f32,
    ) -> (f32, f32) {
        let attrs = Self::resolve_attrs(
            &self.user_font_map,
            &self.default_family_name,
            font,
            weight,
            style,
        );
        let final_line_height = if line_height > 0.0 {
            line_height
        } else {
            scale * 1.2
        };
        let metrics = Metrics::new(scale, final_line_height);
        let mut buffer = GlyphonBuffer::new(&mut self.font_system, metrics);
        buffer.set_size(Some(f32::MAX), Some(f32::MAX));
        buffer.set_text(text, &attrs, Shaping::Advanced, None);
        buffer.shape_until_scroll(&mut self.font_system, false);

        let width = buffer
            .layout_runs()
            .map(|run| run.line_w)
            .fold(0.0_f32, f32::max);

        (width, metrics.line_height)
    }

    pub fn draw(&mut self, scale_factor: f32, theme: winit::window::Theme, command: &TextCommand) {
        let color = command.style.color.unwrap_or(match theme {
            winit::window::Theme::Dark => crate::Color::WHITE,
            winit::window::Theme::Light => crate::Color::BLACK,
        });

        // Font size is also snapped: a non-integer physical pixel size means
        // every glyph in the run is rasterized at a slightly different
        // scale than a whole-pixel grid, which independently softens edges
        // even when the origin is snapped.
        let scale = Self::snap(
            command
                .style
                .font_size
                .map(|s| s.to_physical(scale_factor))
                .unwrap_or(20.0 * scale_factor),
        );

        let weight = command.style.font_weight.unwrap_or_default();
        let style = command.style.font_style.unwrap_or_default();

        let letter_spacing = command
            .style
            .letter_spacing
            .map(|ls| ls.value().to_physical(scale_factor))
            .unwrap_or(0.0);

        let line_height = command
            .style
            .line_height
            .map(|lh| lh.value().to_physical(scale_factor))
            .unwrap_or(0.0);

        let glyphon_color = GlyphonColor::rgba(
            (color.r() * 255.0).round() as u8,
            (color.g() * 255.0).round() as u8,
            (color.b() * 255.0).round() as u8,
            (color.a() * 255.0).round() as u8,
        );

        let snapped_position = (
            Self::snap(command.position.0),
            Self::snap(command.position.1),
        );

        if letter_spacing.abs() < f32::EPSILON {
            self.queue_run(
                &command.text,
                command.font.as_deref(),
                weight,
                style,
                scale,
                snapped_position,
                glyphon_color,
            );
            return;
        }

        // glyphon/cosmic-text has no direct inter-glyph letter-spacing knob,
        // so - just like the previous ab_glyph implementation - we lay the
        // run out one character at a time and advance the cursor manually.
        let mut cursor_x = snapped_position.0;
        for ch in command.text.chars() {
            let mut buf = [0u8; 4];
            let ch_str = ch.encode_utf8(&mut buf);

            self.queue_run(
                ch_str,
                command.font.as_deref(),
                weight,
                style,
                scale,
                (Self::snap(cursor_x), snapped_position.1),
                glyphon_color,
            );

            let (advance, _) = self.measure_raw(
                ch_str,
                command.font.as_deref(),
                weight,
                style,
                scale,
                line_height,
            );
            cursor_x += advance + letter_spacing;
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn measure(
        &mut self,
        text: &str,
        font: Option<&str>,
        font_size: f32,
        weight: FontWeight,
        style: FontStyle,
        letter_spacing: f32,
        line_height: f32,
    ) -> (f32, f32) {
        // Must match the snapping done in `draw()`, otherwise the layout
        // box (computed here) and the actually-rendered glyph run diverge
        // by a pixel or two.
        let scale = Self::snap(font_size);

        let (width, height) = self.measure_raw(text, font, weight, style, scale, line_height);

        let extra = if text.is_empty() {
            0.0
        } else {
            letter_spacing * (text.chars().count() as f32 - 1.0)
        };

        (width + extra, height)
    }

    pub fn flush(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        width: u32,
        height: u32,
    ) -> Result<(), String> {
        self.viewport.update(queue, Resolution { width, height });

        let text_areas: Vec<TextArea> = self
            .pending
            .iter()
            .map(|p| TextArea {
                buffer: &p.buffer,
                left: p.position.0,
                top: p.position.1,
                scale: 1.0,
                bounds: p.bounds,
                default_color: p.color,
                custom_glyphs: &[],
            })
            .collect();

        self.renderer
            .prepare(
                device,
                queue,
                &mut self.font_system,
                &mut self.atlas,
                &self.viewport,
                text_areas,
                &mut self.swash_cache,
            )
            .map_err(|e| e.to_string())?;

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("text_pipeline_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            self.renderer
                .render(&self.atlas, &self.viewport, &mut pass)
                .map_err(|e| e.to_string())?;
        }

        self.atlas.trim();
        self.pending.clear();

        Ok(())
    }
}

/// NOTE: variant names assumed — adjust to match your actual `FontWeight` enum.
fn convert_weight(weight: FontWeight) -> GlyphonWeight {
    match weight {
        FontWeight::Thin => GlyphonWeight::THIN,
        FontWeight::ExtraLight => GlyphonWeight::EXTRA_LIGHT,
        FontWeight::Light => GlyphonWeight::LIGHT,
        FontWeight::Regular => GlyphonWeight::NORMAL,
        FontWeight::Medium => GlyphonWeight::MEDIUM,
        FontWeight::SemiBold => GlyphonWeight::SEMIBOLD,
        FontWeight::Bold => GlyphonWeight::BOLD,
        FontWeight::ExtraBold => GlyphonWeight::EXTRA_BOLD,
        FontWeight::Black => GlyphonWeight::BLACK,
    }
}

/// NOTE: variant names assumed — adjust to match your actual `FontStyle` enum.
fn convert_style(style: FontStyle) -> GlyphonStyle {
    match style {
        FontStyle::Normal => GlyphonStyle::Normal,
        FontStyle::Italic => GlyphonStyle::Italic,
        FontStyle::Oblique => GlyphonStyle::Oblique,
    }
}
