// SPDX-License-Identifier: Apache-2.0
use crate::{ FontStyle, FontWeight, MeasureResult, TextCommand, TextMeasurer };
use glyphon::{
    Attrs,
    Buffer as GlyphonBuffer,
    Cache,
    Color as GlyphonColor,
    Family,
    FontSystem,
    Metrics,
    Resolution,
    Shaping,
    Style as GlyphonStyle,
    SwashCache,
    TextArea,
    TextAtlas,
    TextBounds,
    TextRenderer,
    Viewport,
    Weight as GlyphonWeight,
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
    user_font_map: HashMap<String, String>,
    default_family_name: Option<String>,
    pending: Vec<PendingText>,
}

impl TextPipeline {
    #[inline]
    fn snap(px: f32) -> f32 {
        px.round()
    }

    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        surface_format: wgpu::TextureFormat,
        user_fonts: Vec<(String, Vec<u8>)>
    ) -> Result<Self, String> {
        #[cfg(target_arch = "wasm32")]
        if user_fonts.is_empty() {
            return Err("WASM target requires at least one font supplied.".to_string());
        }

        let mut font_system = FontSystem::new();
        let mut user_font_map: HashMap<String, String> = HashMap::new();

        for (name, data) in &user_fonts {
            let before = font_system.db().faces().count();
            font_system.db_mut().load_font_data(data.clone());

            if
                font_system.db().faces().count() > before &&
                let Some(face) = font_system.db().faces().last() &&
                let Some((family_name, _)) = face.families.first()
            {
                user_font_map.insert(name.clone(), family_name.clone());
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        let default_family_name: Option<String> = None;

        #[cfg(target_arch = "wasm32")]
        let default_family_name: Option<String> = {
            let name = user_fonts.first().and_then(|(name, _)| user_font_map.get(name).cloned());
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
        let renderer = TextRenderer::new(
            &mut atlas,
            device,
            wgpu::MultisampleState::default(),
            None
        );
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

    fn resolve_attrs<'a>(
        user_font_map: &'a HashMap<String, String>,
        default_family_name: &'a Option<String>,
        font: Option<&str>,
        weight: FontWeight,
        style: FontStyle
    ) -> Attrs<'a> {
        let family = font
            .and_then(|n| user_font_map.get(n))
            .map(|s| Family::Name(s.as_str()))
            .unwrap_or_else(|| {
                default_family_name.as_deref().map(Family::Name).unwrap_or(Family::SansSerif)
            });

        Attrs::new().family(family).weight(convert_weight(weight)).style(convert_style(style))
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
        max_width: Option<f32>,
        clip_rect: Option<(f32, f32, f32, f32)>
    ) {
        let attrs = Self::resolve_attrs(
            &self.user_font_map,
            &self.default_family_name,
            font,
            weight,
            style
        );

        let metrics = Metrics::new(scale, scale * 1.25);
        let mut buffer = GlyphonBuffer::new(&mut self.font_system, metrics);
        buffer.set_size(Some(max_width.unwrap_or(f32::MAX)), Some(f32::MAX));
        buffer.set_text(text, &attrs, Shaping::Advanced, None);
        buffer.shape_until_scroll(&mut self.font_system, false);

        // Glyphon clips anything outside these bounds, which is what makes
        // scrolled-off text in a single-line input disappear at the edge
        // instead of overflowing past the widget.
        let bounds = match clip_rect {
            Some((x, y, w, h)) =>
                TextBounds {
                    left: x.round() as i32,
                    top: y.round() as i32,
                    right: (x + w).round() as i32,
                    bottom: (y + h).round() as i32,
                },
            None =>
                TextBounds {
                    left: position.0 as i32,
                    top: position.1 as i32,
                    right: i32::MAX,
                    bottom: i32::MAX,
                },
        };

        self.pending.push(PendingText {
            buffer,
            position,
            color,
            bounds,
        });
    }

    #[allow(clippy::too_many_arguments)]
    fn measure_raw(
        &mut self,
        text: &str,
        font: Option<&str>,
        weight: FontWeight,
        style: FontStyle,
        scale: f32,
        line_height: f32,
        max_width: Option<f32>
    ) -> (f32, f32) {
        let attrs = Self::resolve_attrs(
            &self.user_font_map,
            &self.default_family_name,
            font,
            weight,
            style
        );
        let final_line_height = if line_height > 0.0 { line_height } else { scale * 1.2 };
        let metrics = Metrics::new(scale, final_line_height);
        let mut buffer = GlyphonBuffer::new(&mut self.font_system, metrics);
        // A bounded width here is what makes glyphon break the text into
        // multiple lines instead of one long run.
        buffer.set_size(Some(max_width.unwrap_or(f32::MAX)), Some(f32::MAX));
        buffer.set_text(text, &attrs, Shaping::Advanced, None);
        buffer.shape_until_scroll(&mut self.font_system, false);

        let width = buffer
            .layout_runs()
            .map(|run| run.line_w)
            .fold(0.0_f32, f32::max);

        // Total height grows with the number of wrapped lines instead of
        // always reporting a single line's height.
        let line_count = buffer.layout_runs().count().max(1) as f32;
        let height = final_line_height * line_count;

        (width, height)
    }

    pub fn draw(&mut self, scale_factor: f32, theme: winit::window::Theme, command: &TextCommand) {
        let color = command.style.color.unwrap_or(match theme {
            winit::window::Theme::Dark => crate::Color::WHITE,
            winit::window::Theme::Light => crate::Color::BLACK,
        });

        let scale = Self::snap(
            command.style.font_size
                .map(|s| s.to_physical(scale_factor))
                .unwrap_or(20.0 * scale_factor)
        );

        let weight = command.style.font_weight.unwrap_or_default();
        let style = command.style.font_style.unwrap_or_default();

        let letter_spacing = command.style.letter_spacing
            .map(|ls| ls.value().to_physical(scale_factor))
            .unwrap_or(0.0);

        let line_height = command.style.line_height
            .map(|lh| lh.value().to_physical(scale_factor))
            .unwrap_or(0.0);

        let glyphon_color = GlyphonColor::rgba(
            (color.r() * 255.0).round() as u8,
            (color.g() * 255.0).round() as u8,
            (color.b() * 255.0).round() as u8,
            (color.a() * 255.0).round() as u8
        );

        let snapped_position = (Self::snap(command.position.0), Self::snap(command.position.1));

        if letter_spacing.abs() < f32::EPSILON {
            self.queue_run(
                &command.text,
                command.style.font.as_deref(),
                weight,
                style,
                scale,
                snapped_position,
                glyphon_color,
                command.max_width,
                command.clip_rect
            );
            return;
        }

        let mut cursor_x = snapped_position.0;
        for ch in command.text.chars() {
            let mut buf = [0u8; 4];
            let ch_str = ch.encode_utf8(&mut buf);

            self.queue_run(
                ch_str,
                command.style.font.as_deref(),
                weight,
                style,
                scale,
                (Self::snap(cursor_x), snapped_position.1),
                glyphon_color,
                None,
                command.clip_rect
            );

            let (advance, _) = self.measure_raw(
                ch_str,
                command.style.font.as_deref(),
                weight,
                style,
                scale,
                line_height,
                None
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
        max_width: Option<f32>
    ) -> (f32, f32) {
        let scale = Self::snap(font_size);

        let (width, height) = self.measure_raw(
            text,
            font,
            weight,
            style,
            scale,
            line_height,
            max_width
        );

        let extra = if text.is_empty() {
            0.0
        } else {
            letter_spacing * ((text.chars().count() as f32) - 1.0)
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
        height: u32
    ) -> Result<(), String> {
        self.viewport.update(queue, Resolution { width, height });

        let text_areas: Vec<TextArea> = self.pending
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
                &mut self.swash_cache
            )
            .map_err(|e| e.to_string())?;

        {
            let mut pass = encoder.begin_render_pass(
                &(wgpu::RenderPassDescriptor {
                    label: Some("text_pipeline_pass"),
                    color_attachments: &[
                        Some(wgpu::RenderPassColorAttachment {
                            view,
                            resolve_target: None,
                            depth_slice: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Load,
                                store: wgpu::StoreOp::Store,
                            },
                        }),
                    ],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                    multiview_mask: None,
                })
            );

            self.renderer
                .render(&self.atlas, &self.viewport, &mut pass)
                .map_err(|e| e.to_string())?;
        }

        self.atlas.trim();
        self.pending.clear();

        Ok(())
    }
}

impl TextMeasurer for TextPipeline {
    fn measure(
        &mut self,
        text: &str,
        font: Option<&str>,
        font_size: f32,
        weight: FontWeight,
        style: FontStyle,
        letter_spacing: f32,
        line_height: f32,
        max_width: Option<f32>
    ) -> MeasureResult {
        let (width, height) = TextPipeline::measure(
            self,
            text,
            font,
            font_size,
            weight,
            style,
            letter_spacing,
            line_height,
            max_width
        );

        MeasureResult {
            width,
            height,
            baseline: Some(height * 0.8),
        }
    }

    fn character_offsets(
        &mut self,
        text: &str,
        font: Option<&str>,
        font_size: f32,
        font_weight: FontWeight,
        font_style: FontStyle,
        letter_spacing: f32,
        line_height: f32
    ) -> Vec<f32> {
        let scale = Self::snap(font_size);
        let chars: Vec<char> = text.chars().collect();
        let mut offsets = Vec::with_capacity(chars.len() + 1);
        offsets.push(0.0);

        let mut cursor = 0.0;
        let last_index = chars.len().saturating_sub(1);

        for (i, ch) in chars.iter().enumerate() {
            let mut buf = [0u8; 4];
            let ch_str = ch.encode_utf8(&mut buf);

            let (advance, _) = self.measure_raw(
                ch_str,
                font,
                font_weight,
                font_style,
                scale,
                line_height,
                None
            );

            cursor += advance;
            offsets.push(cursor);

            if i != last_index {
                cursor += letter_spacing;
            }
        }

        offsets
    }

    fn ascent(
        &mut self,
        font: Option<&str>,
        font_size: f32,
        font_weight: FontWeight,
        font_style: FontStyle
    ) -> f32 {
        let scale = Self::snap(font_size);
        let (_, height) = self.measure_raw(" ", font, font_weight, font_style, scale, 0.0, None);
        height * 0.8
    }

    fn descent(
        &mut self,
        font: Option<&str>,
        font_size: f32,
        font_weight: FontWeight,
        font_style: FontStyle
    ) -> f32 {
        let scale = Self::snap(font_size);
        let (_, height) = self.measure_raw(" ", font, font_weight, font_style, scale, 0.0, None);
        height * 0.2
    }

    fn line_height(
        &mut self,
        font: Option<&str>,
        font_size: f32,
        font_weight: FontWeight,
        font_style: FontStyle
    ) -> f32 {
        let scale = Self::snap(font_size);
        let (_, height) = self.measure_raw(" ", font, font_weight, font_style, scale, 0.0, None);
        height
    }
}

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

fn convert_style(style: FontStyle) -> GlyphonStyle {
    match style {
        FontStyle::Normal => GlyphonStyle::Normal,
        FontStyle::Italic => GlyphonStyle::Italic,
        FontStyle::Oblique => GlyphonStyle::Oblique,
    }
}
