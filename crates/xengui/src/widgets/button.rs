// SPDX-License-Identifier: Apache-2.0
use crate::{
    Background,
    EventCtx,
    EventStatus,
    InputEvent,
    Interaction,
    LayoutBox,
    LayoutContext,
    PaintContext,
    RectCommand,
    Style,
    StyleBuilder,
    StylePatch,
    TextCommand,
    Widget,
    WidgetContent,
};
use smol_str::SmolStr;
use std::cell::Cell;
use winit::window::CursorIcon;

pub struct Button {
    dirty: bool,
    label: SmolStr,
    font: Option<SmolStr>,
    style: Style,
    key: Option<SmolStr>,

    hover_style: Option<Style>,
    pressed_style: Option<Style>,
    disabled_style: Option<Style>,
    computed_style: Style,

    interaction: Interaction,

    layout_box: LayoutBox,
    content_size: Cell<(f32, f32)>,
}

impl Button {
    pub fn new() -> Self {
        let mut interaction = Interaction::new();
        interaction.focusable = true;
        interaction.hover_cursor = Some(CursorIcon::Pointer);

        Self {
            dirty: true,
            label: SmolStr::new(""),
            font: None,
            style: Style::default(),
            key: None,

            hover_style: None,
            pressed_style: None,
            disabled_style: None,
            computed_style: Style::default(),

            interaction,

            layout_box: LayoutBox::default(),
            content_size: Cell::new((0.0, 0.0)),
        }
    }

    /// Stable identity among siblings, kept across rebuilds even when this
    /// widget moves position (reorder, insert, remove). Use for list items
    /// instead of relying on array index.
    pub fn key(mut self, key: impl Into<SmolStr>) -> Self {
        self.key = Some(key.into());
        self
    }

    pub fn label(mut self, label: impl Into<SmolStr>) -> Self {
        self.label = label.into();
        self.mark_dirty();
        self
    }

    pub fn font(mut self, font: impl Into<SmolStr>) -> Self {
        self.font = Some(font.into());
        self.mark_dirty();
        self
    }

    /// Full style overlay to be applied during hover - includes every field of Style
    /// such as background, border, color, font_size, padding, margin, etc.
    /// Only the fields you provide will overwrite the base style.
    ///
    /// ```ignore
    /// Button::new()
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

    pub fn pressed_style(mut self, build: impl FnOnce(StylePatch) -> StylePatch) -> Self {
        self.pressed_style = Some(build(StylePatch::new()).build());
        self.mark_dirty();
        self
    }

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

        self.computed_style = match patch {
            Some(patch) => self.style.overlay(patch),
            None => self.style.clone(),
        };
    }
}

impl Default for Button {
    fn default() -> Self {
        Self::new()
    }
}

impl StyleBuilder for Button {
    fn style_mut(&mut self) -> &mut Style {
        &mut self.style
    }

    fn mark_dirty(&mut self) {
        self.dirty = true;
        self.recompute_style();
    }
}

impl WidgetContent for Button {
    fn with_content(self, content: impl Into<SmolStr>) -> Self {
        self.label(content)
    }
}

crate::impl_interaction_builders!(Button);

impl Widget for Button {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
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

    fn children(&self) -> &[Box<dyn Widget>] {
        &[]
    }

    fn interaction(&self) -> Option<&Interaction> {
        Some(&self.interaction)
    }

    fn interaction_mut(&mut self) -> Option<&mut Interaction> {
        Some(&mut self.interaction)
    }

    fn measure(&self, ctx: &mut LayoutContext) -> (f32, f32) {
        let scale_factor = ctx.scale_factor;
        let style = &self.computed_style;

        let font_size = style.font_size
            .map(|s| s.to_physical(scale_factor))
            .unwrap_or(20.0 * scale_factor);

        let letter_spacing = style.letter_spacing
            .map(|ls| ls.value().to_physical(scale_factor))
            .unwrap_or(0.0);

        let line_height = style.line_height
            .map(|lh| lh.value().to_physical(scale_factor))
            .unwrap_or(0.0);

        let (text_w, text_h) = ctx.text.measure(
            &self.label,
            self.font.as_deref(),
            font_size,
            style.font_weight.unwrap_or_default(),
            style.font_style.unwrap_or_default(),
            letter_spacing,
            line_height
        );

        self.content_size.set((text_w, text_h));

        let padding = &style.padding.unwrap_or_default();

        (
            text_w + padding.left.value() + padding.right.value(),
            text_h + padding.top.value() + padding.bottom.value(),
        )
    }

    fn layout(&mut self, rect: LayoutBox) {
        self.layout_box = rect;
    }

    fn layout_box(&self) -> &LayoutBox {
        &self.layout_box
    }

    fn paint(&self, ctx: &mut PaintContext) {
        let style = &self.computed_style;

        if let Some(background) = style.background.clone() {
            let border = style.border.as_ref();
            ctx.draw_rect(RectCommand {
                position: (self.layout_box.x, self.layout_box.y),
                size: (self.layout_box.width, self.layout_box.height),
                background: Some(background),
                border_radius: border.map(|b| b.radius),
                border_width: border.map(|b| b.width),
                border_color: border.map(|b| b.color),
            });
        }

        let (content_w, content_h) = self.content_size.get();
        let padding = style.padding.unwrap_or_default();
        let text_x =
            self.layout_box.x +
            padding.left.value() +
            (self.layout_box.width - padding.left.value() - padding.right.value() - content_w).max(
                0.0
            ) *
                0.5;

        let text_y =
            self.layout_box.y +
            padding.top.value() +
            (self.layout_box.height - padding.top.value() - padding.bottom.value() - content_h).max(
                0.0
            ) *
                0.5;

        ctx.draw_text(TextCommand {
            text: self.label.clone(),
            position: (text_x, text_y),
            style: style.clone(),
            font: self.font.clone(),
        });
    }

    fn event(&mut self, event: &InputEvent, ctx: &mut EventCtx) -> EventStatus {
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

    fn content_eq(&self, other: &dyn Widget) -> bool {
        let Some(other) = other.as_any().downcast_ref::<Button>() else {
            return false;
        };
        self.label == other.label &&
            self.font == other.font &&
            format!("{:?}", self.style) == format!("{:?}", other.style) &&
            format!("{:?}", self.hover_style) == format!("{:?}", other.hover_style) &&
            format!("{:?}", self.pressed_style) == format!("{:?}", other.pressed_style) &&
            format!("{:?}", self.disabled_style) == format!("{:?}", other.disabled_style)
    }

    /// Ensures that the newly transferred interaction (hover/pressed) is reflected
    /// in the computed_style. If we don't call this: a button that is hovered after a
    /// rebuild would stay with the wrong (non-hovered) style until the next mouse event.
    fn after_interaction_transfer(&mut self) {
        self.recompute_style();
    }

    fn transfer_measured_state(&mut self, old: &dyn Widget) {
        if let Some(old) = old.as_any().downcast_ref::<Button>() {
            self.content_size.set(old.content_size.get());
        }
    }
}
