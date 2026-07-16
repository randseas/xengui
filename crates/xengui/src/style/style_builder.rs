// SPDX-License-Identifier: Apache-2.0
use crate::{ Overflow, properties::StyleValue, style::{ FontStyle, FontWeight, LetterSpacing } };

use super::{
    AlignItems,
    Background,
    Border,
    Outline,
    Color,
    Display,
    Edges,
    FlexDirection,
    FlexWrap,
    GridPlacement,
    GridTrack,
    JustifyContent,
    Length,
    LineHeight,
    Position,
    ScrollbarStyle,
    Size,
    Style,
    TextAlign,
    TextDecoration,
};

pub trait StyleBuilder: Sized {
    fn style_mut(&mut self) -> &mut Style;
    fn mark_dirty(&mut self) {}

    fn width<L: Into<Length>>(mut self, width: L) -> Self {
        self.style_mut().size.get_or_insert_with(Default::default).width = Some(width.into());
        self.mark_dirty();
        self
    }

    fn height<L: Into<Length>>(mut self, height: L) -> Self {
        self.style_mut().size.get_or_insert_with(Default::default).height = Some(height.into());
        self.mark_dirty();
        self
    }

    fn size<W: Into<Length>, H: Into<Length>>(mut self, width: W, height: H) -> Self {
        self.style_mut().size = Some(Size::new(width.into(), height.into()));
        self.mark_dirty();
        self
    }

    fn padding<E: Into<Edges>>(mut self, padding: E) -> Self {
        self.style_mut().padding = Some(padding.into());
        self.mark_dirty();
        self
    }

    fn color(mut self, color: Color) -> Self {
        self.style_mut().color = Some(color);
        self.mark_dirty();
        self
    }

    fn background<B: Into<Background>>(mut self, background: B) -> Self {
        self.style_mut().background = Some(background.into());
        self.mark_dirty();
        self
    }

    fn font_size<L: Into<Length>>(mut self, size: L) -> Self {
        self.style_mut().font_size = Some(size.into());
        self.mark_dirty();
        self
    }
    fn font_weight(mut self, weight: FontWeight) -> Self {
        self.style_mut().font_weight = Some(weight);
        self.mark_dirty();
        self
    }

    fn font_style(mut self, style: FontStyle) -> Self {
        self.style_mut().font_style = Some(style);
        self.mark_dirty();
        self
    }

    fn letter_spacing(mut self, spacing: impl Into<LetterSpacing>) -> Self {
        self.style_mut().letter_spacing = Some(spacing.into());
        self.mark_dirty();
        self
    }

    fn text_align(mut self, align: TextAlign) -> Self {
        self.style_mut().text_align = Some(align);
        self.mark_dirty();
        self
    }

    fn text_decoration(mut self, decoration: TextDecoration) -> Self {
        self.style_mut().text_decoration = Some(decoration);
        self.mark_dirty();
        self
    }

    fn line_height(mut self, height: impl Into<LineHeight>) -> Self {
        self.style_mut().line_height = Some(height.into());
        self.mark_dirty();
        self
    }

    fn margin<E: Into<Edges>>(mut self, margin: E) -> Self {
        self.style_mut().margin = Some(margin.into());
        self.mark_dirty();
        self
    }

    fn border(mut self, border: Border) -> Self {
        self.style_mut().border = Some(border);
        self.mark_dirty();
        self
    }

    fn outline(mut self, outline: impl Into<StyleValue<Outline>>) -> Self {
        self.style_mut().outline = outline.into();
        self.mark_dirty();
        self
    }

    fn focus_outline(mut self, focus_outline: impl Into<StyleValue<Outline>>) -> Self {
        self.style_mut().focus_outline = focus_outline.into();
        self.mark_dirty();
        self
    }

    fn min_width<L: Into<Length>>(mut self, width: L) -> Self {
        self.style_mut().min_size.get_or_insert_with(Default::default).width = Some(width.into());
        self.mark_dirty();
        self
    }

    fn min_height<L: Into<Length>>(mut self, height: L) -> Self {
        self.style_mut().min_size.get_or_insert_with(Default::default).height = Some(height.into());
        self.mark_dirty();
        self
    }

    fn min_size<W: Into<Length>, H: Into<Length>>(mut self, width: W, height: H) -> Self {
        self.style_mut().min_size = Some(Size::new(width.into(), height.into()));
        self.mark_dirty();
        self
    }

    fn max_width<L: Into<Length>>(mut self, width: L) -> Self {
        self.style_mut().max_size.get_or_insert_with(Default::default).width = Some(width.into());
        self.mark_dirty();
        self
    }

    fn max_height<L: Into<Length>>(mut self, height: L) -> Self {
        self.style_mut().max_size.get_or_insert_with(Default::default).height = Some(height.into());
        self.mark_dirty();
        self
    }

    fn max_size<W: Into<Length>, H: Into<Length>>(mut self, width: W, height: H) -> Self {
        self.style_mut().max_size = Some(Size::new(width.into(), height.into()));
        self.mark_dirty();
        self
    }

    fn display(mut self, display: Display) -> Self {
        self.style_mut().display = Some(display);
        self.mark_dirty();
        self
    }

    fn position(mut self, position: Position) -> Self {
        self.style_mut().position = Some(position);
        self.mark_dirty();
        self
    }

    fn overflow_x(mut self, overflow: Overflow) -> Self {
        self.style_mut().overflow_x = Some(overflow);
        self.mark_dirty();
        self
    }

    fn overflow_y(mut self, overflow: Overflow) -> Self {
        self.style_mut().overflow_y = Some(overflow);
        self.mark_dirty();
        self
    }

    fn overflow(mut self, x: Overflow, y: Overflow) -> Self {
        self.style_mut().overflow_x = Some(x);
        self.style_mut().overflow_y = Some(y);
        self.mark_dirty();
        self
    }

    fn flex_direction(mut self, direction: FlexDirection) -> Self {
        self.style_mut().flex_direction = Some(direction);
        self.mark_dirty();
        self
    }

    fn flex_wrap(mut self, wrap: FlexWrap) -> Self {
        self.style_mut().flex_wrap = Some(wrap);
        self.mark_dirty();
        self
    }

    fn flex_grow(mut self, grow: f32) -> Self {
        self.style_mut().flex_grow = Some(grow);
        self.mark_dirty();
        self
    }

    fn flex_shrink(mut self, shrink: f32) -> Self {
        self.style_mut().flex_shrink = Some(shrink);
        self.mark_dirty();
        self
    }

    fn flex_basis<L: Into<Length>>(mut self, basis: L) -> Self {
        self.style_mut().flex_basis = Some(basis.into());
        self.mark_dirty();
        self
    }

    fn align_items(mut self, align: AlignItems) -> Self {
        self.style_mut().align_items = Some(align);
        self.mark_dirty();
        self
    }

    fn align_self(mut self, align: AlignItems) -> Self {
        self.style_mut().align_self = Some(align);
        self.mark_dirty();
        self
    }

    fn justify_content(mut self, justify: JustifyContent) -> Self {
        self.style_mut().justify_content = Some(justify);
        self.mark_dirty();
        self
    }

    fn align_content(mut self, align: JustifyContent) -> Self {
        self.style_mut().align_content = Some(align);
        self.mark_dirty();
        self
    }

    fn gap<W: Into<Length>, H: Into<Length>>(mut self, horizontal: W, vertical: H) -> Self {
        self.style_mut().gap = Some((horizontal.into(), vertical.into()));
        self.mark_dirty();
        self
    }

    fn grid_template_columns(mut self, columns: impl Into<Vec<GridTrack>>) -> Self {
        self.style_mut().grid_template_columns = Some(columns.into());
        self.mark_dirty();
        self
    }

    fn grid_template_rows(mut self, rows: impl Into<Vec<GridTrack>>) -> Self {
        self.style_mut().grid_template_rows = Some(rows.into());
        self.mark_dirty();
        self
    }

    fn grid_column(mut self, column: GridPlacement) -> Self {
        self.style_mut().grid_column = Some(column);
        self.mark_dirty();
        self
    }

    fn grid_row(mut self, row: GridPlacement) -> Self {
        self.style_mut().grid_row = Some(row);
        self.mark_dirty();
        self
    }

    fn scrollbar_thickness(mut self, thickness: f32) -> Self {
        self.style_mut().scrollbar.get_or_insert_with(ScrollbarStyle::default).thickness =
            Some(thickness);
        self.mark_dirty();
        self
    }

    fn scrollbar_min_thumb_length(mut self, length: f32) -> Self {
        self.style_mut().scrollbar.get_or_insert_with(ScrollbarStyle::default).min_thumb_length =
            Some(length);
        self.mark_dirty();
        self
    }

    fn scrollbar_thumb_color(mut self, color: Color) -> Self {
        self.style_mut().scrollbar.get_or_insert_with(ScrollbarStyle::default).thumb_color =
            Some(color);
        self.mark_dirty();
        self
    }

    fn scrollbar_track_color(mut self, color: Color) -> Self {
        self.style_mut().scrollbar.get_or_insert_with(ScrollbarStyle::default).track_color =
            Some(color);
        self.mark_dirty();
        self
    }

    fn scrollbar_button_color(mut self, color: Color) -> Self {
        self.style_mut().scrollbar.get_or_insert_with(ScrollbarStyle::default).button_color =
            Some(color);
        self.mark_dirty();
        self
    }

    fn scrollbar_arrow_color(mut self, color: Color) -> Self {
        self.style_mut().scrollbar.get_or_insert_with(ScrollbarStyle::default).arrow_color =
            Some(color);
        self.mark_dirty();
        self
    }
}

#[derive(Default)]
pub struct StylePatch(Style);

impl StylePatch {
    pub fn new() -> Self {
        Self(Style::default())
    }

    pub fn build(self) -> Style {
        self.0
    }
}

impl StyleBuilder for StylePatch {
    fn style_mut(&mut self) -> &mut Style {
        &mut self.0
    }
}
