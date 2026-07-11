// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;

use crate::{FontStyle, FontWeight};
use wgpu_glyph::{FontId, ab_glyph};

pub struct FontDatabase {
    font_map: HashMap<String, FontId>,
    fonts: HashMap<String, ab_glyph::FontArc>,
    default_font: ab_glyph::FontArc,
}

impl FontDatabase {
    pub fn new(default_font: ab_glyph::FontArc) -> Self {
        Self {
            font_map: HashMap::new(),
            fonts: HashMap::new(),
            default_font,
        }
    }

    pub fn register(&mut self, name: String, font: ab_glyph::FontArc, id: FontId) {
        self.font_map.insert(name.clone(), id);
        self.fonts.insert(name, font);
    }

    pub fn resolve_font_id(
        &self,
        family: Option<&str>,
        weight: FontWeight,
        style: FontStyle,
    ) -> Option<FontId> {
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

    pub fn resolve_font(
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

    pub fn default_font(&self) -> &ab_glyph::FontArc {
        &self.default_font
    }
}
