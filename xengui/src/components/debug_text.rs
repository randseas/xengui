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
// xengui/src/components/debug_text.rs
use crate::VNode;
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

    fn render(
        &mut self,
        _render_pass: &mut wgpu::RenderPass,
        glyph_brush: &mut wgpu_glyph::GlyphBrush<()>,
        theme: &Option<winit::window::Theme>,
        _debug_mode: &bool,
    ) {
        if !_debug_mode {
            return;
        }

        // Theme handling
        let text_color = match theme {
            Some(winit::window::Theme::Dark) => [1.0, 1.0, 1.0, 1.0], 
            Some(winit::window::Theme::Light) => [0.0, 0.0, 0.0, 1.0],
            None => [1.0, 1.0, 1.0, 1.0],
        };

        let section = Section::default()
            .with_screen_position((0.0, 0.0))
            .add_text(
                Text::new(&self.cached_text)
                    .with_color(text_color)
                    .with_scale(20.0),
            );

        glyph_brush.queue(section);

        if self.is_dirty {
            self.is_dirty = false;
        }
    }
}
