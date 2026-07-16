// SPDX-License-Identifier: Apache-2.0
use smol_str::SmolStr;

use crate::Outline;

use super::{
    AlignItems,
    Background,
    Border,
    Color,
    Display,
    Edges,
    FlexDirection,
    FlexWrap,
    FontStyle,
    FontWeight,
    GridPlacement,
    GridTrack,
    JustifyContent,
    Length,
    LetterSpacing,
    LineHeight,
    Position,
    Size,
    TextAlign,
    TextDecoration,
    Overflow,
};

#[derive(Default, Clone, Debug, PartialEq)]
pub enum StyleValue<T> {
    #[default]
    Default,
    Value(T),
    None,
}

impl<T> From<T> for StyleValue<T> {
    fn from(value: T) -> Self {
        Self::Value(value)
    }
}

impl<T: Clone> StyleValue<T> {
    pub fn overlay(&self, parent: &Self) -> Self {
        match self {
            Self::Default => parent.clone(),
            Self::Value(value) => Self::Value(value.clone()),
            Self::None => Self::None,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Style {
    // Typography
    pub color: Option<Color>,
    pub background: Option<Background>,
    pub font: Option<SmolStr>,
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
    pub outline: StyleValue<Outline>,
    pub focus_outline: StyleValue<Outline>,

    // Sizing
    pub size: Option<Size>,
    pub min_size: Option<Size>,
    pub max_size: Option<Size>,

    // Layout
    pub display: Option<Display>,
    pub position: Option<Position>,
    pub overflow_x: Option<Overflow>,
    pub overflow_y: Option<Overflow>,

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

    // Grid
    pub grid_template_columns: Option<Vec<GridTrack>>,
    pub grid_template_rows: Option<Vec<GridTrack>>,
    pub grid_column: Option<GridPlacement>,
    pub grid_row: Option<GridPlacement>,
}

impl Style {
    pub fn overlay(&self, patch: &Style) -> Style {
        #[allow(clippy::unnecessary_lazy_evaluations)]
        Style {
            color: patch.color.or_else(|| self.color),
            background: patch.background.clone().or_else(|| self.background.clone()),
            font: patch.font.clone().or_else(|| self.font.clone()),
            font_size: patch.font_size.or_else(|| self.font_size),
            font_weight: patch.font_weight.or_else(|| self.font_weight),
            font_style: patch.font_style.or_else(|| self.font_style),
            text_align: patch.text_align.or_else(|| self.text_align),
            text_decoration: patch.text_decoration.or_else(|| self.text_decoration),
            letter_spacing: patch.letter_spacing.or_else(|| self.letter_spacing),
            line_height: patch.line_height.or_else(|| self.line_height),

            padding: patch.padding.or_else(|| self.padding),
            margin: patch.margin.or_else(|| self.margin),
            border: patch.border.or_else(|| self.border),
            outline: match &patch.outline {
                StyleValue::Default => self.outline.clone(),
                value => value.clone(),
            },

            focus_outline: match &patch.focus_outline {
                StyleValue::Default => self.focus_outline.clone(),
                value => value.clone(),
            },

            size: patch.size.or_else(|| self.size),
            min_size: patch.min_size.or_else(|| self.min_size),
            max_size: patch.max_size.or_else(|| self.max_size),

            display: patch.display.or_else(|| self.display),
            position: patch.position.or_else(|| self.position),
            overflow_x: patch.overflow_x.or_else(|| self.overflow_x),
            overflow_y: patch.overflow_y.or_else(|| self.overflow_y),

            flex_direction: patch.flex_direction.or_else(|| self.flex_direction),
            flex_wrap: patch.flex_wrap.or_else(|| self.flex_wrap),
            flex_grow: patch.flex_grow.or_else(|| self.flex_grow),
            flex_shrink: patch.flex_shrink.or_else(|| self.flex_shrink),
            flex_basis: patch.flex_basis.or_else(|| self.flex_basis),
            align_items: patch.align_items.or_else(|| self.align_items),
            align_self: patch.align_self.or_else(|| self.align_self),
            justify_content: patch.justify_content.or_else(|| self.justify_content),
            align_content: patch.align_content.or_else(|| self.align_content),
            gap: patch.gap.or_else(|| self.gap),

            grid_template_columns: patch.grid_template_columns
                .clone()
                .or_else(|| self.grid_template_columns.clone()),
            grid_template_rows: patch.grid_template_rows
                .clone()
                .or_else(|| self.grid_template_rows.clone()),
            grid_column: patch.grid_column.or(self.grid_column),
            grid_row: patch.grid_row.or(self.grid_row),
        }
    }

    /// Fills in `patch`'s unset typography fields using `self` as the
    /// inherited parent style; every other field always comes from `patch`.
    pub fn inherit_typography(&self, patch: &Style) -> Style {
        Style {
            color: patch.color.or(self.color),
            font: patch.font.clone().or_else(|| self.font.clone()),
            font_size: patch.font_size.or(self.font_size),
            font_weight: patch.font_weight.or(self.font_weight),
            font_style: patch.font_style.or(self.font_style),
            text_align: patch.text_align.or(self.text_align),
            text_decoration: patch.text_decoration.or(self.text_decoration),
            letter_spacing: patch.letter_spacing.or(self.letter_spacing),
            line_height: patch.line_height.or(self.line_height),
            ..patch.clone()
        }
    }
}
