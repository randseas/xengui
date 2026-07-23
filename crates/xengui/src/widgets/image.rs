// SPDX-License-Identifier: Apache-2.0
use crate::{
    AnimationManager,
    Color,
    Constraints,
    EventCtx,
    EventStatus,
    ImageCommand,
    ImageData,
    InputEvent,
    Interaction,
    LayoutBox,
    Length,
    MeasureContext,
    MeasureResult,
    PaintContext,
    Style,
    StyleBuilder,
    Widget,
    WidgetBase,
    WidgetId,
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

pub fn image_source_from_rgba8(mut rgba: Vec<u8>, width: u32, height: u32) -> ImageSource {
    let expected_len = ((width as u64) * (height as u64) * 4) as usize;
    if rgba.len() != expected_len {
        log::error!(
            "image_source_from_rgba8: buffer size mismatch (expected {expected_len}, got {})",
            rgba.len()
        );
        rgba.resize(expected_len, 0);
    }
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
    base: WidgetBase,
    anim_id: WidgetId,
    layout_box: LayoutBox,
    source: Option<ImageSource>,
    object_fit: ObjectFit,
    tint: Option<Color>,
}

impl Image {
    pub fn new() -> Self {
        let interaction = Interaction::new();

        let mut image = Self {
            base: WidgetBase::new(interaction),
            anim_id: WidgetId::new_unique(),
            layout_box: LayoutBox::default(),
            source: None,
            object_fit: ObjectFit::default(),
            tint: None,
        };

        image.recompute_style();
        image
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
            Err(err) => log::error!("Image::bytes decode error: {err}"),
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
                log::error!("Image::path('{}') decode error: {err}", path.as_ref().display()),
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

    fn recompute_style(&mut self) {
        self.base.recompute_style();
        self.base.interaction.hover_cursor = self.base.computed_style.cursor;
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
        &mut self.base.style
    }

    fn mark_dirty(&mut self) {
        self.base.dirty = true;
        self.recompute_style();
    }
}

crate::impl_interaction_builders!(base Image);
crate::impl_common_style_builders!(base Image);
crate::impl_themed_style_builders!(base Image; hover_style => hover_style, pressed_style => pressed_style, disabled_style => disabled_style, focus_style => focus_style);

impl Widget for Image {
    crate::impl_widget_boilerplate!();

    fn debug_name(&self) -> &'static str {
        "Widget#Image"
    }

    fn measure(&self, ctx: &mut MeasureContext, _constraints: Constraints) -> MeasureResult {
        let (iw, ih) = self.intrinsic_size();
        MeasureResult::new(iw * ctx.scale_factor, ih * ctx.scale_factor)
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

        let Some(source) = self.source.clone() else {
            return;
        };

        let (position, size) = self.fitted_rect();
        let border = self.base.computed_style.border.as_ref();
        let sf = ctx.scale_factor;

        ctx.draw_image(ImageCommand {
            position,
            size,
            image: source,
            border_radius: border.map(|b| Length::px(b.radius.to_physical(sf))),
            tint: self.tint,
            clip_rect: None,
        });
    }

    fn event(&mut self, event: &InputEvent, ctx: &mut EventCtx) -> EventStatus {
        if !self.base.interaction.is_active() {
            return EventStatus::Ignored;
        }

        let before_style = self.base.computed_style.clone();

        let status = self.base.interaction.handle(event, ctx);

        if matches!(status, EventStatus::Handled) {
            self.recompute_style();

            if self.base.computed_style != before_style {
                self.base.dirty = true;
                ctx.request_redraw();
            }
        }

        status
    }

    fn content_eq(&self, other: &dyn Widget) -> bool {
        let Some(other) = other.as_any().downcast_ref::<Image>() else {
            return false;
        };

        let source_eq = match (&self.source, &other.source) {
            (Some(a), Some(b)) => a.id == b.id,
            (None, None) => true,
            _ => false,
        };

        source_eq &&
            self.object_fit == other.object_fit &&
            self.tint == other.tint &&
            self.base.style == other.base.style &&
            self.base.hover_style == other.base.hover_style &&
            self.base.pressed_style == other.base.pressed_style &&
            self.base.disabled_style == other.base.disabled_style &&
            self.base.focus_style == other.base.focus_style
    }

    fn after_interaction_transfer(&mut self) {
        self.recompute_style();
    }

    fn cascade_style(&mut self, parent: &Style, anim: &mut AnimationManager) {
        self.base.inherited_style = parent.clone();
        self.recompute_style();
        if crate::animate_computed_style(self.anim_id, &mut self.base.computed_style, anim) {
            self.base.dirty = true;
        }
    }

    fn transfer_measured_state(&mut self, old: &dyn Widget) {
        if let Some(old) = old.as_any().downcast_ref::<Image>() {
            self.anim_id = old.anim_id;
        }
    }

    fn anim_id(&self) -> WidgetId {
        self.anim_id
    }
}
