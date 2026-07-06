// SPDX-License-Identifier: Apache-2.0
// crates/xengui/src/components/debug_text.rs
use crate::{RenderContext, VNode};
use wgpu_glyph::{Section, Text};

pub struct DebugText {
    pub text: String,
    pub is_dirty: bool,
    cached_text: String,
}

impl DebugText {
    pub fn new(initial_text: String) -> Self {
        Self {
            text: initial_text.clone(),
            is_dirty: true,
            cached_text: initial_text,
        }
    }
    pub fn update_logs(&mut self, logs: &[String]) {
        self.cached_text.clear();
        for (i, log) in logs.iter().enumerate() {
            self.cached_text.push_str(log);
            if i < logs.len() - 1 {
                self.cached_text.push('\n');
            }
        }
        self.is_dirty = false;
    }
}

impl VNode for DebugText {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
    fn key(&self) -> &str {
        "debug_text"
    }
    fn is_dirty(&self) -> bool {
        self.is_dirty
    }
    fn set_dirty(&mut self, dirty: bool) {
        self.is_dirty = dirty;
    }

    fn render(&mut self, _render_pass: &mut wgpu::RenderPass, ctx: &mut RenderContext) {
        if !ctx.debug_mode() {
            return;
        }

        let text = &self.cached_text;
        let text_color = match ctx.theme() {
            winit::window::Theme::Dark => [1.0, 1.0, 1.0, 1.0],
            winit::window::Theme::Light => [0.0, 0.0, 0.0, 1.0],
        };

        let wgpu_text = Text::new(text)
            .with_color(text_color)
            .with_scale(20.0)
            .with_font_id(wgpu_glyph::FontId(0));

        let section = Section::default()
            .with_screen_position((0.0, 0.0))
            .add_text(wgpu_text);
        ctx.glyph_brush.queue(section);

        if self.is_dirty {
            self.is_dirty = false;
        }
    }
}
