// SPDX-License-Identifier: Apache-2.0

use crate::{
    BoxShadow, Overflow, TransitionProperty, properties::StyleValue, style::{ FontStyle, FontWeight, IntoThemed, LetterSpacing },
};

use super::{
    AlignItems,
    Background,
    Border,
    Outline,
    Color,
    Cursor,
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

    fn width<M>(mut self, width: impl IntoThemed<Length, M>) -> Self {
        self.style_mut().size.get_or_insert_with(Default::default).width = Some(
            width.resolve_themed()
        );

        self.mark_dirty();

        self
    }

    fn height<M>(mut self, height: impl IntoThemed<Length, M>) -> Self {
        self.style_mut().size.get_or_insert_with(Default::default).height = Some(
            height.resolve_themed()
        );

        self.mark_dirty();

        self
    }

    fn size<W, H, MW, MH>(mut self, width: W, height: H) -> Self
        where W: IntoThemed<Length, MW>, H: IntoThemed<Length, MH>
    {
        self.style_mut().size = Some(Size::new(width.resolve_themed(), height.resolve_themed()));

        self.mark_dirty();

        self
    }

    fn padding<M>(mut self, padding: impl IntoThemed<Edges, M>) -> Self {
        self.style_mut().padding = Some(padding.resolve_themed());

        self.mark_dirty();

        self
    }

    fn color<M>(mut self, color: impl IntoThemed<Color, M>) -> Self {
        self.style_mut().color = Some(color.resolve_themed());

        self.mark_dirty();

        self
    }

    fn selection_color<M>(mut self, color: impl IntoThemed<Color, M>) -> Self {
        self.style_mut().selection_color = Some(color.resolve_themed());

        self.mark_dirty();

        self
    }

    fn selection_background<M>(mut self, color: impl IntoThemed<Color, M>) -> Self {
        self.style_mut().selection_background = Some(color.resolve_themed());

        self.mark_dirty();

        self
    }

    fn caret_color<M>(mut self, color: impl IntoThemed<Color, M>) -> Self {
        self.style_mut().caret_color = Some(color.resolve_themed());

        self.mark_dirty();

        self
    }

    fn selection_border_width<M>(mut self, width: impl IntoThemed<Length, M>) -> Self {
        self.style_mut().selection_border_width = Some(width.resolve_themed());

        self.mark_dirty();

        self
    }

    fn selection_border_color<M>(mut self, color: impl IntoThemed<Color, M>) -> Self {
        self.style_mut().selection_border_color = Some(color.resolve_themed());

        self.mark_dirty();

        self
    }

    fn selection_border_radius<M>(mut self, radius: impl IntoThemed<Length, M>) -> Self {
        self.style_mut().selection_border_radius = Some(radius.resolve_themed());

        self.mark_dirty();

        self
    }

    fn cursor(mut self, cursor: Cursor) -> Self {
        self.style_mut().cursor = Some(cursor);

        self.mark_dirty();

        self
    }

    fn background<M>(mut self, background: impl IntoThemed<Background, M>) -> Self {
        self.style_mut().background = Some(background.resolve_themed());

        self.mark_dirty();

        self
    }

    fn font_size<M>(mut self, size: impl IntoThemed<Length, M>) -> Self {
        self.style_mut().font_size = Some(size.resolve_themed());

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

    fn margin<M>(mut self, margin: impl IntoThemed<Edges, M>) -> Self {
        self.style_mut().margin = Some(margin.resolve_themed());

        self.mark_dirty();

        self
    }

    fn border<M>(mut self, border: impl IntoThemed<Border, M>) -> Self {
        self.style_mut().border = Some(border.resolve_themed());

        self.mark_dirty();

        self
    }

    fn outline<M>(mut self, outline: impl IntoThemed<StyleValue<Outline>, M>) -> Self {
        self.style_mut().outline = outline.resolve_themed();

        self.mark_dirty();

        self
    }

    fn box_shadow(mut self, shadows: impl Into<Vec<BoxShadow>>) -> Self {
        self.style_mut().box_shadow = Some(shadows.into());
        self.mark_dirty();
        self
    }

    fn box_shadow_none(mut self) -> Self {
        self.style_mut().box_shadow = None;
        self.mark_dirty();
        self
    }

    fn min_width<M>(mut self, width: impl IntoThemed<Length, M>) -> Self {
        self.style_mut().min_size.get_or_insert_with(Default::default).width = Some(
            width.resolve_themed()
        );

        self.mark_dirty();

        self
    }

    fn min_height<M>(mut self, height: impl IntoThemed<Length, M>) -> Self {
        self.style_mut().min_size.get_or_insert_with(Default::default).height = Some(
            height.resolve_themed()
        );

        self.mark_dirty();

        self
    }

    fn min_size<W, H, MW, MH>(mut self, width: W, height: H) -> Self
        where W: IntoThemed<Length, MW>, H: IntoThemed<Length, MH>
    {
        self.style_mut().min_size = Some(
            Size::new(width.resolve_themed(), height.resolve_themed())
        );

        self.mark_dirty();

        self
    }

    fn max_width<M>(mut self, width: impl IntoThemed<Length, M>) -> Self {
        self.style_mut().max_size.get_or_insert_with(Default::default).width = Some(
            width.resolve_themed()
        );

        self.mark_dirty();

        self
    }

    fn max_height<M>(mut self, height: impl IntoThemed<Length, M>) -> Self {
        self.style_mut().max_size.get_or_insert_with(Default::default).height = Some(
            height.resolve_themed()
        );

        self.mark_dirty();

        self
    }

    fn max_size<W, H, MW, MH>(mut self, width: W, height: H) -> Self
        where W: IntoThemed<Length, MW>, H: IntoThemed<Length, MH>
    {
        self.style_mut().max_size = Some(
            Size::new(width.resolve_themed(), height.resolve_themed())
        );

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

    /// Paint order relative to siblings; higher values paint later, on top.
    fn z_index(mut self, z_index: i32) -> Self {
        self.style_mut().z_index = Some(z_index);

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

    fn flex_basis<M>(mut self, basis: impl IntoThemed<Length, M>) -> Self {
        self.style_mut().flex_basis = Some(basis.resolve_themed());

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

    fn gap<W, H, MW, MH>(mut self, horizontal: W, vertical: H) -> Self
        where W: IntoThemed<Length, MW>, H: IntoThemed<Length, MH>
    {
        self.style_mut().gap = Some((horizontal.resolve_themed(), vertical.resolve_themed()));

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

    fn scrollbar_thumb_color<M>(mut self, color: impl IntoThemed<Color, M>) -> Self {
        self.style_mut().scrollbar.get_or_insert_with(ScrollbarStyle::default).thumb_color = Some(
            color.resolve_themed()
        );
        self.mark_dirty();
        self
    }

    fn scrollbar_thumb_radius<M>(mut self, radius: impl IntoThemed<f32, M>) -> Self {
        self.style_mut().scrollbar.get_or_insert_with(ScrollbarStyle::default).thumb_radius = Some(
            radius.resolve_themed()
        );
        self.mark_dirty();
        self
    }

    fn scrollbar_track_color<M>(mut self, color: impl IntoThemed<Color, M>) -> Self {
        self.style_mut().scrollbar.get_or_insert_with(ScrollbarStyle::default).track_color = Some(
            color.resolve_themed()
        );
        self.mark_dirty();
        self
    }

    fn scrollbar_button_color<M>(mut self, color: impl IntoThemed<Color, M>) -> Self {
        self.style_mut().scrollbar.get_or_insert_with(ScrollbarStyle::default).button_color = Some(
            color.resolve_themed()
        );
        self.mark_dirty();
        self
    }

    fn scrollbar_arrow_color<M>(mut self, color: impl IntoThemed<Color, M>) -> Self {
        self.style_mut().scrollbar.get_or_insert_with(ScrollbarStyle::default).arrow_color = Some(
            color.resolve_themed()
        );

        self.mark_dirty();

        self
    }

    fn scrollbar_thumb_border_color<M>(mut self, color: impl IntoThemed<Color, M>) -> Self {
        self.style_mut().scrollbar.get_or_insert_with(ScrollbarStyle::default).thumb_border_color =
            Some(color.resolve_themed());

        self.mark_dirty();

        self
    }

    fn scrollbar_track_border_color<M>(mut self, color: impl IntoThemed<Color, M>) -> Self {
        self.style_mut().scrollbar.get_or_insert_with(ScrollbarStyle::default).track_border_color =
            Some(color.resolve_themed());

        self.mark_dirty();

        self
    }

    // Overrides applied only while the pointer is over the scrollbar's own

    // track/thumb/buttons; unset fields fall back to the default scrollbar style.

    fn scrollbar_hover_thickness(mut self, thickness: f32) -> Self {
        self.style_mut().scrollbar_hover.get_or_insert_with(ScrollbarStyle::default).thickness =
            Some(thickness);

        self.mark_dirty();

        self
    }

    fn scrollbar_hover_min_thumb_length(mut self, length: f32) -> Self {
        self

            .style_mut()

            .scrollbar_hover.get_or_insert_with(ScrollbarStyle::default).min_thumb_length =
            Some(length);

        self.mark_dirty();

        self
    }

    fn scrollbar_hover_thumb_color<M>(mut self, color: impl IntoThemed<Color, M>) -> Self {
        self.style_mut().scrollbar_hover.get_or_insert_with(ScrollbarStyle::default).thumb_color =
            Some(color.resolve_themed());

        self.mark_dirty();

        self
    }

    fn scrollbar_hover_thumb_border_color<M>(mut self, color: impl IntoThemed<Color, M>) -> Self {
        self

            .style_mut()

            .scrollbar_hover.get_or_insert_with(ScrollbarStyle::default).thumb_border_color = Some(
            color.resolve_themed()
        );

        self.mark_dirty();

        self
    }

    fn scrollbar_hover_track_color<M>(mut self, color: impl IntoThemed<Color, M>) -> Self {
        self.style_mut().scrollbar_hover.get_or_insert_with(ScrollbarStyle::default).track_color =
            Some(color.resolve_themed());

        self.mark_dirty();

        self
    }

    fn scrollbar_hover_track_border_color<M>(mut self, color: impl IntoThemed<Color, M>) -> Self {
        self

            .style_mut()

            .scrollbar_hover.get_or_insert_with(ScrollbarStyle::default).track_border_color = Some(
            color.resolve_themed()
        );

        self.mark_dirty();

        self
    }

    fn scrollbar_hover_button_color<M>(mut self, color: impl IntoThemed<Color, M>) -> Self {
        self.style_mut().scrollbar_hover.get_or_insert_with(ScrollbarStyle::default).button_color =
            Some(color.resolve_themed());

        self.mark_dirty();

        self
    }

    fn scrollbar_hover_arrow_color<M>(mut self, color: impl IntoThemed<Color, M>) -> Self {
        self.style_mut().scrollbar_hover.get_or_insert_with(ScrollbarStyle::default).arrow_color =
            Some(color.resolve_themed());

        self.mark_dirty();

        self
    }

    // Active while the thumb is being dragged; unset fields fall back to

    // the hover style, then the default.

    fn scrollbar_pressed_thickness(mut self, thickness: f32) -> Self {
        self.style_mut().scrollbar_pressed.get_or_insert_with(ScrollbarStyle::default).thickness =
            Some(thickness);

        self.mark_dirty();

        self
    }

    fn scrollbar_pressed_min_thumb_length(mut self, length: f32) -> Self {
        self

            .style_mut()

            .scrollbar_pressed.get_or_insert_with(ScrollbarStyle::default).min_thumb_length =
            Some(length);

        self.mark_dirty();

        self
    }

    fn scrollbar_pressed_thumb_color<M>(mut self, color: impl IntoThemed<Color, M>) -> Self {
        self.style_mut().scrollbar_pressed.get_or_insert_with(ScrollbarStyle::default).thumb_color =
            Some(color.resolve_themed());

        self.mark_dirty();

        self
    }

    fn scrollbar_pressed_thumb_border_color<M>(mut self, color: impl IntoThemed<Color, M>) -> Self {
        self

            .style_mut()

            .scrollbar_pressed.get_or_insert_with(ScrollbarStyle::default).thumb_border_color =
            Some(color.resolve_themed());

        self.mark_dirty();

        self
    }

    fn scrollbar_pressed_track_color<M>(mut self, color: impl IntoThemed<Color, M>) -> Self {
        self.style_mut().scrollbar_pressed.get_or_insert_with(ScrollbarStyle::default).track_color =
            Some(color.resolve_themed());

        self.mark_dirty();

        self
    }

    fn scrollbar_pressed_track_border_color<M>(mut self, color: impl IntoThemed<Color, M>) -> Self {
        self

            .style_mut()

            .scrollbar_pressed.get_or_insert_with(ScrollbarStyle::default).track_border_color =
            Some(color.resolve_themed());

        self.mark_dirty();

        self
    }

    fn scrollbar_pressed_button_color<M>(mut self, color: impl IntoThemed<Color, M>) -> Self {
        self

            .style_mut()

            .scrollbar_pressed.get_or_insert_with(ScrollbarStyle::default).button_color = Some(
            color.resolve_themed()
        );

        self.mark_dirty();

        self
    }

    fn scrollbar_pressed_arrow_color<M>(mut self, color: impl IntoThemed<Color, M>) -> Self {
        self.style_mut().scrollbar_pressed.get_or_insert_with(ScrollbarStyle::default).arrow_color =
            Some(color.resolve_themed());

        self.mark_dirty();

        self
    }

    fn scale(mut self, scale: f32) -> Self {
        self.style_mut().scale = Some(scale);

        self.mark_dirty();

        self
    }

    fn content_scale(mut self, scale: f32) -> Self {
        self.style_mut().content_scale = Some(scale);

        self.mark_dirty();

        self
    }

    fn transition(mut self, transition: crate::Transition) -> Self {
        self.style_mut().transition = Some(transition);

        let props = self.style_mut().transition_properties.unwrap_or(TransitionProperty::NONE);

        self.style_mut().transition_properties = Some(props.union(TransitionProperty::DEFAULT));

        self.mark_dirty();

        self
    }

    fn transition_all(mut self, transition: crate::Transition) -> Self {
        self.style_mut().transition = Some(transition);

        let props = self.style_mut().transition_properties.unwrap_or(TransitionProperty::NONE);

        self.style_mut().transition_properties = Some(props.union(TransitionProperty::ALL));

        self.mark_dirty();

        self
    }

    fn transition_colors(mut self, transition: crate::Transition) -> Self {
        self.style_mut().transition_overrides.colors = Some(transition);

        let props = self.style_mut().transition_properties.unwrap_or(TransitionProperty::NONE);

        self.style_mut().transition_properties = Some(props.union(TransitionProperty::COLORS));

        self.mark_dirty();

        self
    }

    fn transition_opacity(mut self, transition: crate::Transition) -> Self {
        self.style_mut().transition_overrides.opacity = Some(transition);

        let props = self.style_mut().transition_properties.unwrap_or(TransitionProperty::NONE);

        self.style_mut().transition_properties = Some(props.union(TransitionProperty::OPACITY));

        self.mark_dirty();

        self
    }

    // Reserved for the future box-shadow system; has no visible effect yet.

    fn transition_shadow(mut self, transition: crate::Transition) -> Self {
        self.style_mut().transition_overrides.shadow = Some(transition);

        let props = self.style_mut().transition_properties.unwrap_or(TransitionProperty::NONE);

        self.style_mut().transition_properties = Some(props.union(TransitionProperty::SHADOW));

        self.mark_dirty();

        self
    }

    fn transition_transform(mut self, transition: crate::Transition) -> Self {
        self.style_mut().transition_overrides.transform = Some(transition);

        let props = self.style_mut().transition_properties.unwrap_or(TransitionProperty::NONE);

        self.style_mut().transition_properties = Some(props.union(TransitionProperty::TRANSFORM));

        self.mark_dirty();

        self
    }

    fn transition_none(mut self) -> Self {
        self.style_mut().transition = None;

        self.style_mut().transition_overrides = Default::default();

        self.style_mut().transition_properties = Some(TransitionProperty::NONE);

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
