// SPDX-License-Identifier: Apache-2.0
use super::{
    AlignItems, Background, Border, Color, Display, Edges, FlexDirection, FlexWrap, FontStyle,
    FontWeight, GridPlacement, GridTrack, JustifyContent, Length, LetterSpacing, LineHeight,
    Position, Size, TextAlign, TextDecoration,
};

#[derive(Clone, Debug, Default)]
pub struct Style {
    // Typography
    pub color: Option<Color>,
    pub background: Option<Background>,
    pub font_size: Option<Length>,
    pub font_weight: Option<FontWeight>,
    pub font_style: Option<FontStyle>,
    pub text_align: Option<TextAlign>,
    pub text_decoration: Option<TextDecoration>,
    pub letter_spacing: Option<LetterSpacing>,
    pub line_height: Option<LineHeight>,

    // Box model
    pub padding: Option<Edges>,
    pub margin: Option<Edges>,
    pub border: Option<Border>,

    // Sizing
    pub size: Option<Size>,
    pub min_size: Option<Size>,
    pub max_size: Option<Size>,

    // Layout stratejisi
    pub display: Option<Display>,
    pub position: Option<Position>,

    // Flexbox
    pub flex_direction: Option<FlexDirection>,
    pub flex_wrap: Option<FlexWrap>,
    pub flex_grow: Option<f32>,
    pub flex_shrink: Option<f32>,
    pub flex_basis: Option<Length>,
    pub align_items: Option<AlignItems>,
    pub align_self: Option<AlignItems>,
    pub justify_content: Option<JustifyContent>,
    pub align_content: Option<JustifyContent>,
    pub gap: Option<(Length, Length)>,

    // Grid (basit)
    pub grid_template_columns: Option<Vec<GridTrack>>,
    pub grid_template_rows: Option<Vec<GridTrack>>,
    pub grid_column: Option<GridPlacement>,
    pub grid_row: Option<GridPlacement>,
}
