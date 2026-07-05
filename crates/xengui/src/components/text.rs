// SPDX-License-Identifier: Apache-2.0
// xengui/src/components/text.rs
use crate::VNode;
use smol_str::SmolStr;
use wgpu_glyph::{Section, Text as WGPUText};

#[macro_export]
macro_rules! props {
    ($($field:ident : $val:expr),* $(,)?) => {
        TextProps {
            $( $field: Some(($val).into()), )*
            ..Default::default()
        }
    };
}

#[derive(Default)]
pub struct TextProps {
    pub text: Option<SmolStr>,
    pub scale: Option<f32>,
    pub position: Option<(f32, f32)>,
    pub color: Option<[f32; 4]>,
}

pub struct Text {
    pub key: String,
    pub is_dirty: bool,

    pub props: TextProps,
}

impl Text {
    pub fn new(key: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            is_dirty: true,
            props: TextProps::default(),
        }
    }

    // Builder methods
    pub fn text(mut self, text: impl Into<SmolStr>) -> Self {
        self.props.text = Some(text.into());
        self.set_dirty(true);
        self
    }

    pub fn scale(mut self, scale: f32) -> Self {
        self.props.scale = Some(scale);
        self.set_dirty(true);
        self
    }

    pub fn position(mut self, position: (f32, f32)) -> Self {
        self.props.position = Some(position);
        self.set_dirty(true);
        self
    }

    pub fn color(mut self, color: [f32; 4]) -> Self {
        self.props.color = Some(color);
        self.set_dirty(true);
        self
    }

    pub fn set_props(&mut self, props: TextProps) {
        let mut changed = false;

        if let Some(t) = props.text {
            self.props.text = Some(t);
            changed = true;
        }
        if let Some(s) = props.scale {
            self.props.scale = Some(s);
            changed = true;
        }
        if let Some(p) = props.position {
            self.props.position = Some(p);
            changed = true;
        }
        if let Some(c) = props.color {
            self.props.color = Some(c);
            changed = true;
        }

        if changed {
            self.set_dirty(true);
        }
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

    fn render(
        &mut self,
        _render_pass: &mut wgpu::RenderPass,
        glyph_brush: &mut wgpu_glyph::GlyphBrush<()>,
        theme: &Option<winit::window::Theme>,
        _debug_mode: &bool,
    ) {
        let text = self.props.text.as_deref().unwrap_or("");
        let scale = self.props.scale.unwrap_or(20.0);
        let position = self.props.position.unwrap_or((0.0, 0.0));
        let text_color = self.props.color.unwrap_or_else(|| match theme {
            Some(winit::window::Theme::Dark) => [1.0, 1.0, 1.0, 1.0],
            Some(winit::window::Theme::Light) => [0.0, 0.0, 0.0, 1.0],
            None => [1.0, 1.0, 1.0, 1.0],
        });

        let section = Section::default().with_screen_position(position).add_text(
            WGPUText::new(&text)
                .with_color(text_color)
                .with_scale(scale),
        );

        glyph_brush.queue(section);

        if self.is_dirty {
            self.is_dirty = false;
        }
    }
}
