// SPDX-License-Identifier: Apache-2.0
use crate::{ Background, Color, Length, Style };
use smol_str::SmolStr;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct RectCommand {
    pub position: (f32, f32),
    pub size: (f32, f32),
    pub background: Option<Background>,
    pub border_radius: Option<Length>,
    pub border_width: Option<Length>,
    pub border_color: Option<Color>,
    pub clip_rect: Option<(f32, f32, f32, f32)>,
}

#[derive(Clone, Debug)]
pub struct TextCommand {
    pub text: SmolStr,
    pub position: (f32, f32),
    pub style: Style,
    pub max_width: Option<f32>,
    pub clip_rect: Option<(f32, f32, f32, f32)>,
}

#[derive(Clone, Debug)]
pub struct ImageData {
    pub id: u64,
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

#[derive(Clone, Debug)]
pub struct ImageCommand {
    pub position: (f32, f32),
    pub size: (f32, f32),
    pub image: Arc<ImageData>,
    pub border_radius: Option<Length>,
    pub tint: Option<Color>,
    pub clip_rect: Option<(f32, f32, f32, f32)>,
}

#[derive(Clone, Debug)]
pub struct TriangleCommand {
    pub p0: (f32, f32),
    pub p1: (f32, f32),
    pub p2: (f32, f32),
    pub color: Color,
    pub clip_rect: Option<(f32, f32, f32, f32)>,
}

#[derive(Clone, Debug)]
pub struct BoxShadowCommand {
    /// Rect used for the blurred rounded-rect SDF. For an outset shadow
    /// this is the box shifted by the offset and grown by the spread; for
    /// an inset shadow it's the box shifted/shrunk instead - the "light"
    /// rect the inner shadow appears to be cast from.
    pub shadow_position: (f32, f32),
    pub shadow_size: (f32, f32),
    pub shadow_radius: f32,
    pub blur: f32,
    pub color: Color,
    pub inset: bool,
    /// The widget's real box; an inset shadow is masked to stay inside it.
    pub box_position: (f32, f32),
    pub box_size: (f32, f32),
    pub box_radius: f32,
    pub clip_rect: Option<(f32, f32, f32, f32)>,
}

#[derive(Clone, Debug)]
pub enum DrawCommand {
    Rect(RectCommand),
    Triangle(TriangleCommand),
    Text(Box<TextCommand>),
    Image(Box<ImageCommand>),
    BoxShadow(BoxShadowCommand),
}

// Converts a logical clip rect (top-left origin) into a physical scissor
// rect clamped to the surface bounds. `None` means the full surface.
pub fn scissor_for_clip(
    clip: Option<(f32, f32, f32, f32)>,
    surface_width: u32,
    surface_height: u32
) -> (u32, u32, u32, u32) {
    let Some((x, y, w, h)) = clip else {
        return (0, 0, surface_width, surface_height);
    };

    let x0 = x.max(0.0).min(surface_width as f32);
    let y0 = y.max(0.0).min(surface_height as f32);
    let x1 = (x + w).max(0.0).min(surface_width as f32);
    let y1 = (y + h).max(0.0).min(surface_height as f32);

    (
        x0.round() as u32,
        y0.round() as u32,
        (x1 - x0).round().max(0.0) as u32,
        (y1 - y0).round().max(0.0) as u32,
    )
}
