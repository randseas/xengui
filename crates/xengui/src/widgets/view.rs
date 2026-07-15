// SPDX-License-Identifier: Apache-2.0
use crate::{
    Constraints,
    Interaction,
    LayoutBox,
    MeasureContext,
    MeasureResult,
    PaintContext,
    Style,
    StyleBuilder,
    Widget,
};
use smol_str::SmolStr;

pub struct View {
    dirty: bool,
    style: Style,
    layout_box: LayoutBox,
    children: Vec<Box<dyn Widget>>,
    interaction: Interaction,
    key: Option<SmolStr>,
}

impl View {
    pub fn new() -> Self {
        Self {
            dirty: true,
            style: Style::default(),
            layout_box: LayoutBox::default(),
            children: Vec::new(),
            interaction: Interaction::new(),
            key: None,
        }
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
        self.dirty = true;
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
        self
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
        self.paint_box(ctx);
    }

    fn content_eq(&self, other: &dyn Widget) -> bool {
        let Some(other) = other.as_any().downcast_ref::<View>() else {
            return false;
        };
        format!("{:?}", self.style) == format!("{:?}", other.style)
    }
}
