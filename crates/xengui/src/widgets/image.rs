// SPDX-License-Identifier: Apache-2.0
use crate::{
    Color,
    ImageCommand,
    ImageData,
    LayoutBox,
    LayoutContext,
    PaintContext,
    Style,
    StyleBuilder,
    Widget,
};
use std::hash::{ Hash, Hasher };
use std::sync::Arc;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ObjectFit {
    #[default]
    Fill,

    Contain,

    Cover,

    None,
}

pub type ImageSource = Arc<ImageData>;

fn hash_bytes(bytes: &[u8]) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    bytes.hash(&mut hasher);
    hasher.finish()
}

pub fn image_source_from_rgba8(rgba: Vec<u8>, width: u32, height: u32) -> ImageSource {
    debug_assert_eq!(rgba.len() as u64, (width as u64) * (height as u64) * 4);
    let id = hash_bytes(&rgba);
    Arc::new(ImageData {
        id,
        width,
        height,
        rgba,
    })
}

pub fn image_source_from_bytes(bytes: &[u8]) -> Result<ImageSource, String> {
    let decoded = image
        ::load_from_memory(bytes)
        .map_err(|e| e.to_string())?
        .to_rgba8();
    let (width, height) = decoded.dimensions();
    let id = hash_bytes(bytes);
    Ok(
        Arc::new(ImageData {
            id,
            width,
            height,
            rgba: decoded.into_raw(),
        })
    )
}

#[cfg(not(target_arch = "wasm32"))]
pub fn image_source_from_path(path: impl AsRef<std::path::Path>) -> Result<ImageSource, String> {
    let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
    image_source_from_bytes(&bytes)
}

pub struct Image {
    dirty: bool,
    style: Style,
    layout_box: LayoutBox,
    source: Option<ImageSource>,
    object_fit: ObjectFit,
    tint: Option<Color>,
}

impl Image {
    pub fn new() -> Self {
        Self {
            dirty: true,
            style: Style::default(),
            layout_box: LayoutBox::default(),
            source: None,
            object_fit: ObjectFit::default(),
            tint: None,
        }
    }

    pub fn source(mut self, source: ImageSource) -> Self {
        self.source = Some(source);
        self.mark_dirty();
        self
    }

    pub fn bytes(mut self, bytes: &[u8]) -> Self {
        match image_source_from_bytes(bytes) {
            Ok(source) => {
                self.source = Some(source);
            }
            Err(err) => log::error!("Image::bytes decode hatası: {err}"),
        }
        self.mark_dirty();
        self
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn path(mut self, path: impl AsRef<std::path::Path>) -> Self {
        match image_source_from_path(path.as_ref()) {
            Ok(source) => {
                self.source = Some(source);
            }
            Err(err) =>
                log::error!("Image::path('{}') decode hatası: {err}", path.as_ref().display()),
        }
        self.mark_dirty();
        self
    }

    pub fn object_fit(mut self, fit: ObjectFit) -> Self {
        self.object_fit = fit;
        self.mark_dirty();
        self
    }

    pub fn tint(mut self, color: Color) -> Self {
        self.tint = Some(color);
        self.mark_dirty();
        self
    }

    fn intrinsic_size(&self) -> (f32, f32) {
        match &self.source {
            Some(src) => (src.width as f32, src.height as f32),
            None => (0.0, 0.0),
        }
    }

    fn fitted_rect(&self) -> ((f32, f32), (f32, f32)) {
        let b = &self.layout_box;
        let (iw, ih) = self.intrinsic_size();

        if iw <= 0.0 || ih <= 0.0 {
            return ((b.x, b.y), (b.width, b.height));
        }

        match self.object_fit {
            ObjectFit::Fill => ((b.x, b.y), (b.width, b.height)),
            ObjectFit::None => {
                let x = b.x + (b.width - iw) * 0.5;
                let y = b.y + (b.height - ih) * 0.5;
                ((x, y), (iw, ih))
            }
            ObjectFit::Contain | ObjectFit::Cover => {
                let scale_x = b.width / iw;
                let scale_y = b.height / ih;
                let scale = if self.object_fit == ObjectFit::Contain {
                    scale_x.min(scale_y)
                } else {
                    scale_x.max(scale_y)
                };
                let w = iw * scale;
                let h = ih * scale;
                let x = b.x + (b.width - w) * 0.5;
                let y = b.y + (b.height - h) * 0.5;
                ((x, y), (w, h))
            }
        }
    }
}

impl Default for Image {
    fn default() -> Self {
        Self::new()
    }
}

impl StyleBuilder for Image {
    fn style_mut(&mut self) -> &mut Style {
        &mut self.style
    }
    fn mark_dirty(&mut self) {
        self.dirty = true;
    }
}

impl Widget for Image {
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
        &[]
    }

    fn measure(&self, ctx: &mut LayoutContext) -> (f32, f32) {
        let (iw, ih) = self.intrinsic_size();
        (iw * ctx.scale_factor, ih * ctx.scale_factor)
    }

    fn layout(&mut self, rect: LayoutBox) {
        self.layout_box = rect;
    }

    fn layout_box(&self) -> &LayoutBox {
        &self.layout_box
    }

    fn paint(&self, ctx: &mut PaintContext) {
        let Some(source) = self.source.clone() else {
            return;
        };

        let (position, size) = self.fitted_rect();
        let border = self.style.border.as_ref();

        ctx.draw_image(ImageCommand {
            position,
            size,
            image: source,
            border_radius: border.map(|b| b.radius),
            tint: self.tint,
        });
    }
}
