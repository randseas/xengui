use crate::Color;

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct ScrollbarStyle {
    pub thickness: Option<f32>,
    pub thumb_color: Option<Color>,
    pub track_color: Option<Color>,
    pub button_color: Option<Color>,
    pub arrow_color: Option<Color>,
    pub min_thumb_length: Option<f32>,
    pub thumb_radius: Option<f32>,
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
        }
    }

    pub fn resolve(&self) -> ResolvedScrollbar {
        let thickness = self.thickness.unwrap_or(8.0);
        let thumb_color = self.thumb_color.unwrap_or(Color::NEUTRAL_400.with_alpha(160));
        ResolvedScrollbar {
            thickness,
            thumb_color,
            track_color: self.track_color.unwrap_or(Color::TRANSPARENT),
            button_color: self.button_color.unwrap_or(thumb_color),
            arrow_color: self.arrow_color.unwrap_or(Color::WHITE),
            min_thumb_length: self.min_thumb_length.unwrap_or(thickness * 1.5),
            thumb_radius: self.thumb_radius.unwrap_or(thickness * 2.0),
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
}

impl Default for ResolvedScrollbar {
    fn default() -> Self {
        ScrollbarStyle::default().resolve()
    }
}
