use crate::Color;

/// Scrollbar thickness while neither hovered nor pressed.
pub const DEFAULT_SCROLLBAR_THICKNESS: f32 = 5.0;
/// Scrollbar thickness while hovered or pressed, unless overridden.
pub const DEFAULT_SCROLLBAR_HOVER_THICKNESS: f32 = 10.0;

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct ScrollbarStyle {
    pub thickness: Option<f32>,
    pub thumb_color: Option<Color>,
    pub track_color: Option<Color>,
    pub button_color: Option<Color>,
    pub arrow_color: Option<Color>,
    pub min_thumb_length: Option<f32>,
    pub thumb_radius: Option<f32>,
    pub thumb_border_width: Option<f32>,
    pub thumb_border_color: Option<Color>,
    pub track_border_width: Option<f32>,
    pub track_border_color: Option<Color>,
}

impl ScrollbarStyle {
    pub fn overlay(&self, patch: &Self) -> Self {
        Self {
            thickness: patch.thickness.or(self.thickness),
            thumb_color: patch.thumb_color.or(self.thumb_color),
            track_color: patch.track_color.or(self.track_color),
            button_color: patch.button_color.or(self.button_color),
            arrow_color: patch.arrow_color.or(self.arrow_color),
            min_thumb_length: patch.min_thumb_length.or(self.min_thumb_length),
            thumb_radius: patch.thumb_radius.or(self.thumb_radius),
            thumb_border_width: patch.thumb_border_width.or(self.thumb_border_width),
            thumb_border_color: patch.thumb_border_color.or(self.thumb_border_color),
            track_border_width: patch.track_border_width.or(self.track_border_width),
            track_border_color: patch.track_border_color.or(self.track_border_color),
        }
    }

    pub fn resolve(&self) -> ResolvedScrollbar {
        let thickness = self.thickness.unwrap_or(DEFAULT_SCROLLBAR_THICKNESS);
        let thumb_color = self.thumb_color.unwrap_or(Color::NEUTRAL_400.with_alpha(160));
        ResolvedScrollbar {
            thickness,
            thumb_color,
            track_color: self.track_color.unwrap_or(Color::TRANSPARENT),
            button_color: self.button_color.unwrap_or(thumb_color),
            arrow_color: self.arrow_color.unwrap_or(Color::WHITE),
            min_thumb_length: self.min_thumb_length.unwrap_or(thickness * 1.5),
            thumb_radius: self.thumb_radius.unwrap_or(thickness * 2.0),
            thumb_border_width: self.thumb_border_width.unwrap_or(0.0),
            thumb_border_color: self.thumb_border_color.unwrap_or(Color::TRANSPARENT),
            track_border_width: self.track_border_width.unwrap_or(0.0),
            track_border_color: self.track_border_color.unwrap_or(Color::TRANSPARENT),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ResolvedScrollbar {
    pub thickness: f32,
    pub thumb_color: Color,
    pub track_color: Color,
    pub button_color: Color,
    pub arrow_color: Color,
    pub min_thumb_length: f32,
    pub thumb_radius: f32,
    pub thumb_border_width: f32,
    pub thumb_border_color: Color,
    pub track_border_width: f32,
    pub track_border_color: Color,
}

impl ResolvedScrollbar {
    // Applies a hover/pressed patch on top of an already-resolved base;
    // only thickness needs an external fallback since every other field
    // already has a resolved value from the base to fall back to.
    pub fn patched(&self, patch: &ScrollbarStyle, default_thickness: f32) -> Self {
        Self {
            thickness: patch.thickness.unwrap_or(default_thickness),
            thumb_color: patch.thumb_color.unwrap_or(self.thumb_color),
            track_color: patch.track_color.unwrap_or(self.track_color),
            button_color: patch.button_color.unwrap_or(self.button_color),
            arrow_color: patch.arrow_color.unwrap_or(self.arrow_color),
            min_thumb_length: patch.min_thumb_length.unwrap_or(self.min_thumb_length),
            thumb_radius: patch.thumb_radius.unwrap_or(self.thumb_radius),
            thumb_border_width: patch.thumb_border_width.unwrap_or(self.thumb_border_width),
            thumb_border_color: patch.thumb_border_color.unwrap_or(self.thumb_border_color),
            track_border_width: patch.track_border_width.unwrap_or(self.track_border_width),
            track_border_color: patch.track_border_color.unwrap_or(self.track_border_color),
        }
    }
}

impl Default for ResolvedScrollbar {
    fn default() -> Self {
        ScrollbarStyle::default().resolve()
    }
}
