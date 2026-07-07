// SPDX-License-Identifier: Apache-2.0
use crate::{PaintContext, Style, StyleBuilder, TextCommand, VNode};
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
        ctx.draw_text(TextCommand {
            text: self.content.clone(),
            position: self.position,
            style: self.style.clone(),
            font: self.font.clone(),
        });
    }
}
