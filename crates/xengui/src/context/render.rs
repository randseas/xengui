use std::collections::HashMap;
use wgpu_glyph::{FontId};

pub struct RenderContext<'glyph> {
    font_map: &'glyph HashMap<String, FontId>,
    theme: winit::window::Theme,
    debug_mode: bool,
}

impl<'glyph> RenderContext<'glyph> {
    pub fn new(
        font_map: &'glyph HashMap<String, FontId>,
        theme: winit::window::Theme,
        debug_mode: bool,
    ) -> Self {
        Self {
            font_map,
            theme,
            debug_mode,
        }
    }
    pub fn theme(&self) -> winit::window::Theme {
        self.theme
    }
    pub fn debug_mode(&self) -> bool {
        self.debug_mode
    }
    pub fn font(&self, name: &str) -> Option<FontId> {
        self.font_map.get(name).copied()
    }
}
