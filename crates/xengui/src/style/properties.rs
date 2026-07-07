// SPDX-License-Identifier: Apache-2.0
use super::{
    Color, Edges, FontStyle, FontWeight, Length, LetterSpacing, LineHeight, TextAlign,
    TextDecoration, Size
};

#[derive(Clone, Debug, Default)]
pub struct Style {
    // Typography
    pub text_color: Option<Color>,
    pub font_size: Option<Length>,
    pub font_weight: Option<FontWeight>,
    pub font_style: Option<FontStyle>,
    pub text_align: Option<TextAlign>,
    pub text_decoration: Option<TextDecoration>,
    pub letter_spacing: Option<LetterSpacing>,
    pub line_height: Option<LineHeight>,

    // Layout
    pub padding: Option<Edges>,

    // Sizing
    pub size: Option<Size>,
    pub min_size: Option<Size>,
    pub max_size: Option<Size>,
}
