// SPDX-License-Identifier: Apache-2.0
use crate::{
    LayoutBox,
    LayoutContext,
    PaintContext,
    RectCommand,
    Style,
    StyleBuilder,
    TextCommand,
    Widget,
};
use smol_str::SmolStr;

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
    dirty: bool,
    content: SmolStr,
    font: Option<SmolStr>,
    style: Style,
    layout_box: LayoutBox,
    selectable: bool,
}

impl Label {
    pub fn new() -> Self {
        Self {
            dirty: true,
            content: SmolStr::new(""),
            font: None,
            style: Style::default(),
            layout_box: LayoutBox::default(),
            selectable: false,
        }
    }

    // Builder methods
    pub fn label(mut self, label: impl Into<SmolStr>) -> Self {
        self.content = label.into();
        self.set_dirty(true);
        self
    }

    pub fn font(mut self, font: impl Into<SmolStr>) -> Self {
        self.font = Some(font.into());
        self.set_dirty(true);
        self
    }

    pub fn selectable(mut self, selectable: bool) -> Self {
        self.selectable = selectable;
        self.set_dirty(true);
        self
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
    }
}

impl Widget for Label {
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

    fn paint(&self, ctx: &mut PaintContext) {
        log::trace!("paint -> '{}' x={} y={}", self.content, self.layout_box.x, self.layout_box.y);

        if self.style.background.is_some() {
            ctx.draw_rect(RectCommand {
                position: (self.layout_box.x, self.layout_box.y),
                size: (self.layout_box.width, self.layout_box.height),
                background: self.style.background.clone(),
                border_radius: None,
                border_color: None,
                border_width: None,
            });
        }

        ctx.draw_text(TextCommand {
            text: self.content.clone(),
            position: (self.layout_box.x, self.layout_box.y),
            style: self.style.clone(),
            font: self.font.clone(),
        });
    }

    fn measure(&self, ctx: &mut LayoutContext) -> (f32, f32) {
        let scale_factor = ctx.scale_factor;

        let font_size = self.style.font_size
            .map(|s| s.to_physical(scale_factor))
            .unwrap_or(20.0 * scale_factor);

        let letter_spacing = self.style.letter_spacing
            .map(|ls| ls.value().to_physical(scale_factor))
            .unwrap_or(0.0);

        let line_height = self.style.line_height
            .map(|lh| lh.value().to_physical(scale_factor))
            .unwrap_or(0.0);

        ctx.text.measure(
            &self.content,
            self.font.as_deref(),
            font_size,
            self.style.font_weight.unwrap_or_default(),
            self.style.font_style.unwrap_or_default(),
            letter_spacing,
            line_height
        )
    }

    fn layout(&mut self, rect: LayoutBox) {
        self.layout_box = rect;
    }

    fn layout_box(&self) -> &LayoutBox {
        &self.layout_box
    }

    fn children(&self) -> &[Box<dyn Widget>] {
        &[]
    }

    fn content_eq(&self, other: &dyn Widget) -> bool {
        let Some(other) = other.as_any().downcast_ref::<Label>() else {
            return false;
        };
        self.content == other.content &&
            self.font == other.font &&
            format!("{:?}", self.style) == format!("{:?}", other.style)
    }
}
