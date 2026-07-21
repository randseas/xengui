// SPDX-License-Identifier: Apache-2.0
use crate::{
    AnimationManager,
    Background,
    Constraints,
    EventCtx,
    EventStatus,
    InputEvent,
    Interaction,
    IntoThemed,
    LayoutBox,
    Length,
    MeasureContext,
    MeasureResult,
    PaintContext,
    RectCommand,
    Style,
    StyleBuilder,
    TextCommand,
    Widget,
    WidgetBase,
    WidgetContent,
    WidgetId,
    properties::{ DEFAULT_CURSOR_ICON, DEFAULT_FONT_SIZE, DEFAULT_POINTER_CURSOR_ICON },
};
use smol_str::SmolStr;
use std::cell::Cell;

/// A clickable widget that performs an action when activated.
///
/// A `Button` displays a text label and responds to user interactions such as
/// pointer clicks and keyboard activation. Its appearance can be customized
/// through the styling API, including typography, colors, borders, spacing,
/// and sizing.
///
/// Buttons can be enabled or disabled. When disabled, they remain visible but
/// do not respond to user input.
///
/// ## Example
///
/// ```no_run
/// use xengui::prelude::*;
///
/// let button = Button::new()
///     .label("Click me");
/// ```
pub struct Button {
    base: WidgetBase,
    content: SmolStr,
    layout_box: LayoutBox,
    content_size: Cell<(f32, f32)>,
    anim_id: WidgetId,
}

impl Button {
    pub fn new() -> Self {
        let mut interaction = Interaction::new();
        interaction.focusable = true;
        interaction.hover_cursor = Some(DEFAULT_POINTER_CURSOR_ICON);

        Self {
            base: WidgetBase::new(interaction),
            content: SmolStr::new(""),
            layout_box: LayoutBox::default(),
            content_size: Cell::new((0.0, 0.0)),
            anim_id: WidgetId::new_unique(),
        }
    }

    /// Assigns a stable identifier to this widget.
    ///
    /// Keys are used to preserve widget identity across rebuilds, allowing state
    /// to remain associated with the same logical widget even if its position in
    /// the widget tree changes.
    pub fn key(mut self, key: impl Into<SmolStr>) -> Self {
        self.base.key = Some(key.into());
        self
    }

    /// Sets the text displayed by this widget.
    pub fn label(mut self, content: impl Into<SmolStr>) -> Self {
        self.content = content.into();
        self.mark_dirty();
        self
    }

    /// Sets the font family used to render the widget's text.
    ///
    /// The font name must correspond to a font that has been registered with the
    /// application.
    pub fn font(mut self, font: impl Into<SmolStr>) -> Self {
        self.base.style.font = Some(font.into());
        self.mark_dirty();
        self
    }

    pub fn hover_background<M>(mut self, background: impl IntoThemed<Background, M>) -> Self {
        self.base.hover_style.get_or_insert_with(Style::default).background = Some(
            background.resolve_themed()
        );
        self.mark_dirty();
        self
    }

    pub fn pressed_background<M>(mut self, background: impl IntoThemed<Background, M>) -> Self {
        self.base.pressed_style.get_or_insert_with(Style::default).background = Some(
            background.resolve_themed()
        );
        self.mark_dirty();
        self
    }

    pub fn disabled_background<M>(mut self, background: impl IntoThemed<Background, M>) -> Self {
        self.base.disabled_style.get_or_insert_with(Style::default).background = Some(
            background.resolve_themed()
        );
        self.mark_dirty();
        self
    }

    /// Enables or disables user interaction for this widget.
    ///
    /// When disabled, the widget does not receive or respond to user input events
    /// such as pointer or keyboard interactions.
    ///
    /// This is equivalent to calling [`Self::disabled`] with the opposite value.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.base.interaction.set_enabled(enabled);
        self.mark_dirty();
        self
    }

    /// Enables or disables the widget using an inverted boolean.
    ///
    /// Passing `true` disables the widget, while `false` enables it.
    ///
    /// This is equivalent to calling [`Self::enabled`] with the negated value.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.base.interaction.set_enabled(!disabled);
        self.mark_dirty();
        self
    }

    // Widget-specific extra step (hover cursor) stays local; the shared
    // style-overlay logic now lives in WidgetBase.
    fn recompute_style(&mut self) {
        self.base.recompute_style();
        self.base.interaction.hover_cursor = self.base.computed_style.cursor.or(
            Some(
                if self.base.interaction.enabled {
                    DEFAULT_POINTER_CURSOR_ICON
                } else {
                    DEFAULT_CURSOR_ICON
                }
            )
        );
    }
}

impl Default for Button {
    fn default() -> Self {
        Self::new()
    }
}

impl StyleBuilder for Button {
    fn style_mut(&mut self) -> &mut Style {
        &mut self.base.style
    }

    fn mark_dirty(&mut self) {
        self.base.dirty = true;
        self.recompute_style();
    }
}

impl WidgetContent for Button {
    fn with_content(self, content: impl Into<SmolStr>) -> Self {
        self.label(content)
    }
}

crate::impl_interaction_builders!(base Button);
crate::impl_themed_style_builders!(base Button; hover_style => hover_style, pressed_style => pressed_style, disabled_style => disabled_style, focus_style => focus_style);

impl Widget for Button {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn debug_name(&self) -> &'static str {
        "Widget#Button"
    }

    fn get_key(&self) -> Option<&SmolStr> {
        self.base.key.as_ref()
    }

    fn is_dirty(&self) -> bool {
        self.base.dirty
    }

    fn set_dirty(&mut self, dirty: bool) {
        self.base.dirty = dirty;
    }

    fn style(&self) -> &Style {
        &self.base.style
    }

    fn style_mut(&mut self) -> &mut Style {
        &mut self.base.style
    }

    fn computed_style(&self) -> &Style {
        &self.base.computed_style
    }

    fn children(&self) -> &[Box<dyn Widget>] {
        &[]
    }

    fn interaction(&self) -> Option<&Interaction> {
        Some(&self.base.interaction)
    }

    fn interaction_mut(&mut self) -> Option<&mut Interaction> {
        Some(&mut self.base.interaction)
    }

    fn layout(&mut self, rect: LayoutBox) {
        self.layout_box = rect;
    }

    fn layout_box(&self) -> &LayoutBox {
        &self.layout_box
    }

    fn measure(&self, ctx: &mut MeasureContext, constraints: Constraints) -> MeasureResult {
        let scale_factor = ctx.scale_factor;
        let style = &self.base.computed_style;

        let font_size = style.font_size
            .map(|s| s.to_physical(scale_factor))
            .unwrap_or(DEFAULT_FONT_SIZE.to_physical(scale_factor));

        let letter_spacing = style.letter_spacing
            .map(|ls| ls.value().to_physical(scale_factor))
            .unwrap_or(0.0);

        let line_height = style.line_height
            .map(|lh| lh.value().to_physical(scale_factor))
            .unwrap_or(0.0);

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

        let padding = style.padding.unwrap_or_default();

        let width =
            result.width +
            padding.left.to_physical(scale_factor) +
            padding.right.to_physical(scale_factor);
        let height =
            result.height +
            padding.top.to_physical(scale_factor) +
            padding.bottom.to_physical(scale_factor);
        let (width, height) = constraints.constrain_size(width, height);

        MeasureResult::new(width, height)
    }

    fn paint(&self, ctx: &mut PaintContext) {
        log::trace!(
            "paint -> '{}' x={} y={} dirty={:?}",
            self.content,
            self.layout_box.x,
            self.layout_box.y,
            self.is_dirty()
        );

        let style = &self.base.computed_style;
        let sf = ctx.scale_factor;

        // Background is painted through its own scaled rect instead of
        // paint_box(), so a scale transition applies independently of
        // the content layer below.
        let scale = style.scale.unwrap_or(1.0);
        let background_box = crate::scaled_layout_box(self.layout_box, scale);

        if style.background.is_some() || style.border.is_some() {
            let border = style.border.as_ref();
            ctx.draw_rect(RectCommand {
                position: (background_box.x, background_box.y),
                size: (background_box.width, background_box.height),
                background: style.background.clone(),
                border_radius: border.map(|b| Length::px(b.radius.to_physical(sf))),
                border_color: border.map(|b| b.color),
                border_width: border.map(|b| Length::px(b.width.to_physical(sf))),
                clip_rect: None,
            });
        }

        self.paint_outline(ctx);

        let (content_w, content_h) = self.content_size.get();
        let padding = style.padding.unwrap_or_default();
        let (pad_l, pad_r, pad_t, pad_b) = (
            padding.left.to_physical(sf),
            padding.right.to_physical(sf),
            padding.top.to_physical(sf),
            padding.bottom.to_physical(sf),
        );
        let available_w = self.layout_box.width - pad_l - pad_r;
        let draw_max_width = available_w.max(content_w);

        let text_x = self.layout_box.x + pad_l + (available_w - content_w).max(0.0) * 0.5;
        let text_y =
            self.layout_box.y +
            pad_t +
            (self.layout_box.height - pad_t - pad_b - content_h).max(0.0) * 0.5;

        let content_scale = style.content_scale.unwrap_or(scale);
        let content_box = crate::scaled_layout_box(
            LayoutBox { x: text_x, y: text_y, width: content_w, height: content_h },
            content_scale
        );

        let mut text_style = style.clone();
        let base_font_size = text_style.font_size
            .map(|f| f.value())
            .unwrap_or(DEFAULT_FONT_SIZE.value());
        text_style.font_size = Some(Length::px(base_font_size * content_scale));

        ctx.draw_text(TextCommand {
            text: self.content.clone(),
            position: (content_box.x, content_box.y),
            style: text_style,
            max_width: Some(draw_max_width),
            clip_rect: None,
        });
    }

    fn event(&mut self, event: &InputEvent, ctx: &mut EventCtx) -> EventStatus {
        if !self.base.interaction.is_active() {
            return EventStatus::Ignored;
        }

        let before_style = self.base.computed_style.clone();
        let before_focus_visible = self.base.interaction.focus_visible;

        let status = self.base.interaction.handle(event, ctx);

        if matches!(status, EventStatus::Handled) {
            self.recompute_style();

            if
                self.base.computed_style != before_style ||
                self.base.interaction.focus_visible != before_focus_visible
            {
                self.base.dirty = true;
                ctx.request_redraw();
            }
        }

        status
    }

    fn content_eq(&self, other: &dyn Widget) -> bool {
        let Some(other) = other.as_any().downcast_ref::<Button>() else {
            return false;
        };

        self.content == other.content &&
            self.base.style == other.base.style &&
            self.base.hover_style == other.base.hover_style &&
            self.base.pressed_style == other.base.pressed_style &&
            self.base.disabled_style == other.base.disabled_style &&
            self.base.focus_style == other.base.focus_style
    }

    fn cascade_style(&mut self, parent: &Style, anim: &mut AnimationManager) {
        self.base.inherited_style = parent.clone();
        self.recompute_style();
        if crate::animate_computed_style(self.anim_id, &mut self.base.computed_style, anim) {
            self.base.dirty = true;
        }
    }

    fn after_interaction_transfer(&mut self) {
        self.recompute_style();
    }

    fn transfer_measured_state(&mut self, old: &dyn Widget) {
        if let Some(old) = old.as_any().downcast_ref::<Button>() {
            self.content_size.set(old.content_size.get());
            self.anim_id = old.anim_id;
        }
    }

    fn anim_id(&self) -> WidgetId {
        self.anim_id
    }
}
