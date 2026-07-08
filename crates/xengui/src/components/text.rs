// SPDX-License-Identifier: Apache-2.0
use crate::{
    LayoutBox, LayoutContext, PaintContext, RectCommand, Style, StyleBuilder, TextCommand, VNode,
};
use smol_str::SmolStr;

#[macro_export]
macro_rules! props {
    ($($field:ident : $val:expr),* $(,)?) => {
        #[allow(clippy::needless_update)]
        TextProps {
            $( $field: Some(($val).into()), )*
            ..Default::default()
        }
    };
}

pub struct Text {
    pub key: String,
    pub is_dirty: bool,

    content: SmolStr,
    position: (f32, f32),

    font: Option<SmolStr>,

    pub style: Style,
    layout_box: LayoutBox,
}

impl Text {
    pub fn new(key: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            is_dirty: true,

            content: SmolStr::new(""),
            position: (0.0, 0.0),
            font: None,

            style: Style::default(),
            layout_box: LayoutBox::default(),
        }
    }

    // Builder methods
    pub fn text(mut self, text: impl Into<SmolStr>) -> Self {
        self.content = text.into();
        self.set_dirty(true);
        self
    }

    pub fn position(mut self, position: (f32, f32)) -> Self {
        self.position = position;
        self.set_dirty(true);
        self
    }

    pub fn font(mut self, font: impl Into<SmolStr>) -> Self {
        self.font = Some(font.into());
        self.set_dirty(true);
        self
    }
}

impl StyleBuilder for Text {
    fn style_mut(&mut self) -> &mut Style {
        &mut self.style
    }

    fn mark_dirty(&mut self) {
        self.set_dirty(true);
    }
}

impl VNode for Text {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn key(&self) -> &str {
        &self.key
    }

    fn is_dirty(&self) -> bool {
        self.is_dirty
    }

    fn set_dirty(&mut self, dirty: bool) {
        self.is_dirty = dirty;
    }

    fn paint(&self, ctx: &mut PaintContext) {
        if ctx.debug {
            log::info!(
                "paint -> '{}' x={} y={}",
                self.content,
                self.layout_box.x,
                self.layout_box.y,
            );
        }

        if self.style.background.is_some() {
            ctx.draw_rect(RectCommand {
                position: (self.layout_box.x, self.layout_box.y),
                size: (self.layout_box.width, self.layout_box.height),
                background: self.style.background.clone(),
                border_radius: None,
            });
        }

        ctx.draw_text(TextCommand {
            text: self.content.clone(),
            position: (self.layout_box.x, self.layout_box.y),
            style: self.style.clone(),
            font: self.font.clone(),
        });
    }

    fn measure(&self, ctx: &LayoutContext) -> (f32, f32) {
        let scale_factor = ctx.scale_factor;

        let font_size = self
            .style
            .font_size
            .map(|s| s.to_physical(scale_factor))
            .unwrap_or(20.0 * scale_factor);

        let letter_spacing = self
            .style
            .letter_spacing
            .map(|ls| ls.value().to_physical(scale_factor))
            .unwrap_or(0.0);

        ctx.text.measure(
            &self.content,
            self.font.as_deref(),
            font_size,
            self.style.font_weight.unwrap_or_default(),
            self.style.font_style.unwrap_or_default(),
            letter_spacing,
        )
    }

    fn layout(&mut self, rect: LayoutBox) {
        self.layout_box = rect;
    }

    fn layout_box(&self) -> &LayoutBox {
        &self.layout_box
    }
}
