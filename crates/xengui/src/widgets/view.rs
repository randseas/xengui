// SPDX-License-Identifier: Apache-2.0
use crate::{
    Background,
    Constraints,
    EventCtx,
    EventStatus,
    InputEvent,
    Interaction,
    LayoutBox,
    MeasureContext,
    MeasureResult,
    PaintContext,
    Style,
    StyleBuilder,
    StylePatch,
    Widget,
};
use smol_str::SmolStr;

pub struct View {
    key: Option<SmolStr>,

    dirty: bool,
    style: Style,
    inherited_style: Style,
    computed_style: Style,

    hover_style: Option<Style>,
    pressed_style: Option<Style>,
    disabled_style: Option<Style>,

    layout_box: LayoutBox,
    children: Vec<Box<dyn Widget>>,
    interaction: Interaction,
}

impl View {
    pub fn new() -> Self {
        let mut view = Self {
            key: None,

            dirty: true,
            style: Style::default(),
            inherited_style: Style::default(),
            computed_style: Style::default(),

            hover_style: None,
            pressed_style: None,
            disabled_style: None,

            layout_box: LayoutBox::default(),
            children: Vec::new(),
            interaction: Interaction::new(),
        };

        view.recompute_style();
        view
    }

    /// Stable identity among siblings, kept across rebuilds even when this
    /// widget moves position (reorder, insert, remove). Use for list items
    /// instead of relying on array index.
    pub fn key(mut self, key: impl Into<SmolStr>) -> Self {
        self.key = Some(key.into());
        self
    }

    pub fn font(mut self, font: impl Into<SmolStr>) -> Self {
        self.style.font = Some(font.into());
        self.mark_dirty();
        self
    }

    /// Full style overlay to be applied during hover state - includes every field of Style
    /// such as background, border, color, font_size, padding, margin, etc.
    /// Only the fields you provide will overwrite the base style.
    pub fn hover_style(mut self, build: impl FnOnce(StylePatch) -> StylePatch) -> Self {
        self.hover_style = Some(build(StylePatch::new()).build());
        self.mark_dirty();
        self
    }

    /// Full style overlay to be applied during pressed state - includes every field of Style
    /// such as background, border, color, font_size, padding, margin, etc.
    /// Only the fields you provide will overwrite the base style.
    pub fn pressed_style(mut self, build: impl FnOnce(StylePatch) -> StylePatch) -> Self {
        self.pressed_style = Some(build(StylePatch::new()).build());
        self.mark_dirty();
        self
    }

    /// Full style overlay to be applied during disabled state - includes every field of Style
    /// such as background, border, color, font_size, padding, margin, etc.
    /// Only the fields you provide will overwrite the base style.
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

    pub fn child(mut self, child: impl Widget + 'static) -> Self {
        self.children.push(Box::new(child));
        self
    }

    /// Bulk variant of `child` for dynamically built lists where each item
    /// is already a boxed trait object (e.g. produced inside a `.map()`).
    pub fn children_vec(mut self, children: Vec<Box<dyn Widget>>) -> Self {
        self.children = children;
        self
    }

    pub fn focusable(mut self, focusable: bool) -> Self {
        self.interaction.focusable = focusable;
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

        let base = self.inherited_style.overlay(&self.style);

        self.computed_style = match patch {
            Some(patch) => base.overlay(patch),
            None => base,
        };
    }
}

impl Default for View {
    fn default() -> Self {
        Self::new()
    }
}

impl StyleBuilder for View {
    fn style_mut(&mut self) -> &mut Style {
        &mut self.style
    }

    fn mark_dirty(&mut self) {
        self.dirty = true;
        self.recompute_style();
    }
}

crate::impl_interaction_builders!(View);

impl Widget for View {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn debug_name(&self) -> &'static str {
        "Widget#View"
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
        &self.children
    }

    fn children_mut(&mut self) -> Option<&mut Vec<Box<dyn Widget>>> {
        Some(&mut self.children)
    }

    fn interaction(&self) -> Option<&Interaction> {
        Some(&self.interaction)
    }

    fn interaction_mut(&mut self) -> Option<&mut Interaction> {
        Some(&mut self.interaction)
    }

    fn measure(&self, _ctx: &mut MeasureContext, _constraints: Constraints) -> MeasureResult {
        MeasureResult::new(0.0, 0.0)
    }

    fn layout(&mut self, rect: LayoutBox) {
        self.layout_box = rect;
    }

    fn layout_box(&self) -> &LayoutBox {
        &self.layout_box
    }

    fn paint(&self, ctx: &mut PaintContext) {
        log::trace!(
            "paint -> '{:?}' x={} y={} dirty={:?}",
            self.get_key(),
            self.layout_box.x,
            self.layout_box.y,
            self.is_dirty()
        );

        self.paint_box(ctx);
        self.paint_outline(ctx);
        self.paint_focus(ctx);
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
        let Some(other) = other.as_any().downcast_ref::<View>() else {
            return false;
        };

        self.style == other.style &&
            self.hover_style == other.hover_style &&
            self.pressed_style == other.pressed_style &&
            self.disabled_style == other.disabled_style
    }

    fn cascade_style(&mut self, parent: &Style) {
        self.inherited_style = parent.clone();
        self.recompute_style();
    }

    fn after_interaction_transfer(&mut self) {
        self.recompute_style();
    }
}
