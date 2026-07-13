// SPDX-License-Identifier: Apache-2.0

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct GlyphKey {
    pub font: u32,
    pub glyph_id: u32,
    pub size: u32,
}

#[derive(Clone, Debug)]
pub struct GlyphBitmap {
    pub width: u32,
    pub height: u32,

    pub left: i32,
    pub top: i32,

    pub advance: f32,

    // 8-bit alpha bitmap.
    pub pixels: Vec<u8>,
}