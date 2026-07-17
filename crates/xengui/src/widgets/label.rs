// SPDX-License-Identifier: Apache-2.0
use crate::{
    Background,
    Color,
    Constraints,
    EventCtx,
    EventStatus,
    InputEvent,
    Interaction,
    LayoutBox,
    MeasureContext,
    MeasureResult,
    PaintContext,
    RectCommand,
    Style,
    StyleBuilder,
    StylePatch,
    TextCommand,
    Widget,
    properties::DEFAULT_FONT_SIZE,
};
use smol_str::SmolStr;
use std::cell::{ Cell, RefCell };
use winit::event::{ ElementState, MouseButton };
use winit::window::CursorIcon;

#[macro_export]
macro_rules! props {
    ($($field:ident: $val:expr),* $(,)?) => {
        #[allow(clippy::needless_update)]
        TextProps {
            $( $field: Some(($val).into()), )*
            ..Default::default()
        }
    };
}

pub struct Label {
    key: Option<SmolStr>,

    dirty: bool,
    style: Style,
    inherited_style: Style,
    computed_style: Style,

    hover_style: Option<Style>,
    pressed_style: Option<Style>,
    disabled_style: Option<Style>,

    interaction: Interaction,
    selectable: bool,

    content: SmolStr,
    layout_box: LayoutBox,
    content_size: Cell<(f32, f32)>,
    measured_max_width: Cell<Option<f32>>,

    char_offsets: RefCell<Vec<f32>>,
    selection_anchor: Cell<Option<usize>>,
    selection_cursor: Cell<Option<usize>>,
    dragging: Cell<bool>,
}

impl Label {
    pub fn new() -> Self {
        let mut interaction = Interaction::new();
        interaction.focusable = false;
        interaction.hover_cursor = Some(CursorIcon::Text);

        let mut label = Self {
            key: None,

            dirty: true,
            style: Style::default(),
            inherited_style: Style::default(),
            computed_style: Style::default(),

            hover_style: None,
            pressed_style: None,
            disabled_style: None,

            interaction,
            selectable: false,

            content: SmolStr::new(""),
            layout_box: LayoutBox::default(),
            content_size: Cell::new((0.0, 0.0)),
            measured_max_width: Cell::new(None),

            char_offsets: RefCell::new(Vec::new()),
            selection_anchor: Cell::new(None),
            selection_cursor: Cell::new(None),
            dragging: Cell::new(false),
        };

        label.recompute_style();
        label
    }

    /// Stable identity among siblings, kept across rebuilds even when this
    /// widget moves position (reorder, insert, remove). Use for list items
    /// instead of relying on array index.
    pub fn key(mut self, key: impl Into<SmolStr>) -> Self {
        self.key = Some(key.into());
        self
    }

    // Builder methods
    pub fn label(mut self, content: impl Into<SmolStr>) -> Self {
        self.content = content.into();
        self.mark_dirty();
        self
    }

    pub fn font(mut self, font: impl Into<SmolStr>) -> Self {
        self.style.font = Some(font.into());
        self.mark_dirty();
        self
    }

    pub fn selectable(mut self, selectable: bool) -> Self {
        self.selectable = selectable;
        self.mark_dirty();
        self
    }

    /// Full style overlay to be applied during hover state - includes every field of Style
    /// such as background, border, color, font_size, padding, margin, etc.
    /// Only the fields you provide will overwrite the base style.
    ///
    /// ```ignore
    /// Label::new()
    ///     .background(Color::NEUTRAL_200)
    ///     .border(Border::new(1, Color::NEUTRAL_200, Length::px(6.0)))
    ///     .hover_style(|s| s
    ///         .background(Color::NEUTRAL_300)
    ///         .border(Border::new(1, Color::NEUTRAL_400, Length::px(6.0)))
    ///     )
    /// ```
    pub fn hover_style(mut self, build: impl FnOnce(StylePatch) -> StylePatch) -> Self {
        self.hover_style = Some(build(StylePatch::new()).build());
        self.mark_dirty();
        self
    }

    /// Full style overlay to be applied during pressed state - includes every field of Style
    /// such as background, border, color, font_size, padding, margin, etc.
    /// Only the fields you provide will overwrite the base style.
    ///
    /// ```ignore
    /// Label::new()
    ///     .background(Color::NEUTRAL_200)
    ///     .border(Border::new(1, Color::NEUTRAL_200, Length::px(6.0)))
    ///     .pressed_style(|s| s
    ///         .background(Color::NEUTRAL_300)
    ///         .border(Border::new(1, Color::NEUTRAL_400, Length::px(6.0)))
    ///     )
    /// ```
    pub fn pressed_style(mut self, build: impl FnOnce(StylePatch) -> StylePatch) -> Self {
        self.pressed_style = Some(build(StylePatch::new()).build());
        self.mark_dirty();
        self
    }

    /// Full style overlay to be applied during disabled state - includes every field of Style
    /// such as background, border, color, font_size, padding, margin, etc.
    /// Only the fields you provide will overwrite the base style.
    ///
    /// ```ignore
    /// Label::new()
    ///     .background(Color::NEUTRAL_200)
    ///     .border(Border::new(1, Color::NEUTRAL_200, Length::px(6.0)))
    ///     .disabled_style(|s| s
    ///         .background(Color::NEUTRAL_300)
    ///         .border(Border::new(1, Color::NEUTRAL_400, Length::px(6.0)))
    ///     )
    /// ```
    pub fn disabled_style(mut self, build: impl FnOnce(StylePatch) -> StylePatch) -> Self {
        self.disabled_style = Some(build(StylePatch::new()).build());
        self.mark_dirty();
        self
    }

    pub fn hover_background<B: Into<Background>>(mut self, background: B) -> Self {
        self.hover_style.get_or_insert_with(Style::default).background = Some(background.into());
        self.mark_dirty();
        self
    }

    pub fn pressed_background<B: Into<Background>>(mut self, background: B) -> Self {
        self.pressed_style.get_or_insert_with(Style::default).background = Some(background.into());
        self.mark_dirty();
        self
    }

    pub fn disabled_background<B: Into<Background>>(mut self, background: B) -> Self {
        self.disabled_style.get_or_insert_with(Style::default).background = Some(background.into());
        self.mark_dirty();
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.interaction.set_enabled(enabled);
        self.mark_dirty();
        self
    }

    fn recompute_style(&mut self) {
        let patch = if !self.interaction.enabled {
            self.disabled_style.as_ref()
        } else if self.interaction.pressed {
            self.pressed_style.as_ref().or(self.hover_style.as_ref())
        } else if self.interaction.hovered {
            self.hover_style.as_ref()
        } else {
            None
        };

        let base = self.inherited_style.inherit_style(&self.style);

        self.computed_style = match patch {
            Some(patch) => base.overlay(patch),
            None => base,
        };

        self.interaction.hover_cursor = self.computed_style.cursor
            .map(crate::Cursor::to_winit)
            .or(Some(CursorIcon::Text));
    }

    fn index_for_offset(&self, local_x: f32) -> usize {
        let offsets = self.char_offsets.borrow();
        if offsets.len() <= 1 {
            return 0;
        }
        let mut best = 0;
        let mut best_dist = f32::MAX;
        for (i, &off) in offsets.iter().enumerate() {
            let dist = (off - local_x).abs();
            if dist < best_dist {
                best_dist = dist;
                best = i;
            }
        }
        best
    }
}

impl Default for Label {
    fn default() -> Self {
        Self::new()
    }
}

impl StyleBuilder for Label {
    fn style_mut(&mut self) -> &mut Style {
        &mut self.style
    }

    fn mark_dirty(&mut self) {
        self.dirty = true;
        self.recompute_style();
    }
}

impl Widget for Label {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn debug_name(&self) -> &'static str {
        "Widget#Label"
    }

    fn get_key(&self) -> Option<&SmolStr> {
        self.key.as_ref()
    }

    fn is_dirty(&self) -> bool {
        self.dirty
    }

    fn set_dirty(&mut self, dirty: bool) {
        self.dirty = dirty;
    }

    fn style(&self) -> &Style {
        &self.style
    }

    fn style_mut(&mut self) -> &mut Style {
        &mut self.style
    }

    fn computed_style(&self) -> &Style {
        &self.computed_style
    }

    fn children(&self) -> &[Box<dyn Widget>] {
        &[]
    }

    fn interaction(&self) -> Option<&Interaction> {
        Some(&self.interaction)
    }

    fn interaction_mut(&mut self) -> Option<&mut Interaction> {
        Some(&mut self.interaction)
    }

    fn layout(&mut self, rect: LayoutBox) {
        self.layout_box = rect;
    }

    fn layout_box(&self) -> &LayoutBox {
        &self.layout_box
    }

    fn measure(&self, ctx: &mut MeasureContext, constraints: Constraints) -> MeasureResult {
        let scale_factor = ctx.scale_factor;
        let style = &self.computed_style;

        let font_size = style.font_size
            .map(|s| s.to_physical(scale_factor))
            .unwrap_or(DEFAULT_FONT_SIZE.to_physical(scale_factor));

        let letter_spacing = style.letter_spacing
            .map(|ls| ls.value().to_physical(scale_factor))
            .unwrap_or(0.0);

        let line_height = style.line_height
            .map(|lh| lh.value().to_physical(scale_factor))
            .unwrap_or(0.0);

        self.measured_max_width.set(constraints.max_width);

        let result = ctx.text.measure(
            &self.content,
            style.font.as_deref(),
            font_size,
            style.font_weight.unwrap_or_default(),
            style.font_style.unwrap_or_default(),
            letter_spacing,
            line_height,
            constraints.max_width
        );

        self.content_size.set((result.width, result.height));

        if self.selectable {
            *self.char_offsets.borrow_mut() = ctx.text.character_offsets(
                &self.content,
                style.font.as_deref(),
                font_size,
                style.font_weight.unwrap_or_default(),
                style.font_style.unwrap_or_default(),
                letter_spacing,
                line_height
            );
        }

        let padding = style.padding.unwrap_or_default();
        let width = result.width + padding.left.value() + padding.right.value();
        let height = result.height + padding.top.value() + padding.bottom.value();
        let (width, height) = constraints.constrain_size(width, height);

        MeasureResult {
            width,
            height,
            baseline: result.baseline,
        }
    }

    fn paint(&self, ctx: &mut PaintContext) {
        let style = &self.computed_style;

        log::trace!(
            "paint -> '{}' x={} y={} dirty={:?}",
            self.content,
            self.layout_box.x,
            self.layout_box.y,
            self.is_dirty()
        );

        self.paint_box(ctx);
        self.paint_outline(ctx);
        self.paint_focus(ctx);

        let padding = style.padding.unwrap_or_default();

        let text_x = self.layout_box.x + padding.left.value();
        let text_y = self.layout_box.y + padding.top.value();

        let mut text_style = style.clone();
        text_style.font_size.get_or_insert(DEFAULT_FONT_SIZE);

        if self.selectable && let Some((start, end)) = self.text_selection() {
            let offsets = self.char_offsets.borrow();
            if let (Some(&start_x), Some(&end_x)) = (offsets.get(start), offsets.get(end)) {
                let (_, content_h) = self.content_size.get();
                ctx.draw_rect(RectCommand {
                    position: (text_x + start_x, text_y),
                    size: (end_x - start_x, content_h.max(1.0)),
                    background: Some(Background::Color(Color::rgba(90, 140, 230, 100))),
                    border_radius: None,
                    border_width: None,
                    border_color: None,
                    clip_rect: None,
                });
            }
        }

        ctx.draw_text(TextCommand {
            text: self.content.clone(),
            position: (text_x, text_y),
            style: text_style,
            max_width: self.measured_max_width.get(),
            clip_rect: None,
        });
    }

    fn event(&mut self, event: &InputEvent, ctx: &mut EventCtx) -> EventStatus {
        if
            self.selectable &&
            let InputEvent::MouseInput { state, button: MouseButton::Left, position } = event
        {
            let padding_left = self.computed_style.padding.unwrap_or_default().left.value();
            let local_x = position.0 - self.layout_box.x - padding_left;
            let idx = self.index_for_offset(local_x);

            match state {
                ElementState::Pressed => {
                    self.selection_anchor.set(Some(idx));
                    self.selection_cursor.set(Some(idx));
                    self.dragging.set(true);
                }
                ElementState::Released => {
                    self.dragging.set(false);
                }
            }
            self.dirty = true;
            ctx.request_redraw();
        }

        if
            self.selectable &&
            self.dragging.get() &&
            let InputEvent::MouseMoved { position } = event
        {
            let padding_left = self.computed_style.padding.unwrap_or_default().left.value();
            let local_x = position.0 - self.layout_box.x - padding_left;
            self.selection_cursor.set(Some(self.index_for_offset(local_x)));
            self.dirty = true;
            ctx.request_redraw();
            return EventStatus::Handled;
        }

        if !self.interaction.is_active() {
            return EventStatus::Ignored;
        }

        let status = self.interaction.handle(event, ctx);

        if matches!(status, EventStatus::Handled) {
            self.recompute_style();
            self.dirty = true;
        }

        status
    }

    fn selectable_text(&self) -> Option<&str> {
        self.selectable.then_some(self.content.as_str())
    }

    fn text_selection(&self) -> Option<(usize, usize)> {
        let anchor = self.selection_anchor.get()?;
        let cursor = self.selection_cursor.get()?;
        (anchor != cursor).then(|| (anchor.min(cursor), anchor.max(cursor)))
    }

    fn set_text_selection(&mut self, range: Option<(usize, usize)>) {
        let (anchor, cursor) = range.map_or((None, None), |(s, e)| (Some(s), Some(e)));
        self.selection_anchor.set(anchor);
        self.selection_cursor.set(cursor);
    }

    fn select_all_text(&mut self) {
        if !self.selectable {
            return;
        }
        self.selection_anchor.set(Some(0));
        self.selection_cursor.set(Some(self.content.chars().count()));
        self.dirty = true;
    }

    fn content_eq(&self, other: &dyn Widget) -> bool {
        let Some(other) = other.as_any().downcast_ref::<Label>() else {
            return false;
        };

        self.content == other.content &&
            self.style == other.style &&
            self.hover_style == other.hover_style &&
            self.pressed_style == other.pressed_style &&
            self.disabled_style == other.disabled_style &&
            self.selectable == other.selectable
    }

    fn cascade_style(&mut self, parent: &Style) {
        self.inherited_style = parent.clone();
        self.recompute_style();
    }

    fn after_interaction_transfer(&mut self) {
        self.recompute_style();
    }

    fn transfer_measured_state(&mut self, old: &dyn Widget) {
        if let Some(old) = old.as_any().downcast_ref::<Label>() {
            self.content_size.set(old.content_size.get());
            self.measured_max_width.set(old.measured_max_width.get());
            self.char_offsets.replace(old.char_offsets.borrow().clone());
            self.selection_anchor.set(old.selection_anchor.get());
            self.selection_cursor.set(old.selection_cursor.get());
        }
    }
}
