// SPDX-License-Identifier: Apache-2.0

use crate::{FontStyle, FontWeight, GlyphBitmap};

#[derive(Clone, Debug)]
pub struct FontDescriptor {
    pub family: Option<String>,
    pub weight: FontWeight,
    pub style: FontStyle,
    pub size: f32,
    pub letter_spacing: f32,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct TextMetrics {
    pub width: f32,
    pub height: f32,
}

pub trait TextRasterizer {
    fn rasterize(&mut self, ch: char, font: &FontDescriptor) -> GlyphBitmap;

    fn measure(&self, text: &str, font: &FontDescriptor) -> TextMetrics;
}
