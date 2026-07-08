// SPDX-License-Identifier: Apache-2.0
use crate::{LayoutBox, LayoutContext, PaintContext, RectCommand, Style, StyleBuilder, Widget};

pub struct View {
    dirty: bool,
    style: Style,
    layout_box: LayoutBox,
    children: Vec<Box<dyn Widget>>,
}

impl View {
    pub fn new() -> Self {
        Self {
            dirty: true,
            style: Style::default(),
            layout_box: LayoutBox::default(),
            children: Vec::new(),
        }
    }

    pub fn child(mut self, child: impl Widget + 'static) -> Self {
        self.children.push(Box::new(child));
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

impl Widget for View {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
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

    /// Sadece boş (children'sız) VE explicit boyutu olmayan View'ler için
    /// çağrılır (bkz. layout_engine.rs::build_taffy_node). Bu durumda 0x0
    /// döner; gerçek boyutlandırma taffy tarafından style/children'a göre
    /// yapılır.
    fn measure(&self, _ctx: &LayoutContext) -> (f32, f32) {
        (0.0, 0.0)
    }

    fn layout(&mut self, rect: LayoutBox) {
        self.layout_box = rect;
    }
    fn layout_box(&self) -> &LayoutBox {
        &self.layout_box
    }

    fn paint(&self, ctx: &mut PaintContext) {
        if self.style.background.is_some() || self.style.border.is_some() {
            let border = self.style.border.as_ref();
            ctx.draw_rect(RectCommand {
                position: (self.layout_box.x, self.layout_box.y),
                size: (self.layout_box.width, self.layout_box.height),
                background: self.style.background.clone(),
                border_radius: border.map(|b| b.radius),
                border_width: border.map(|b| b.width),
                border_color: border.map(|b| b.color),
            });
        }
        // Not: renderer.rs artık ağacı recursive gezdiği için burada
        // children'ı elle paint etmeye gerek yok.
    }
}
