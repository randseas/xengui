// SPDX-License-Identifier: Apache-2.0
use crate::{ Color, Length };

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct TextDecoration {
    underline: bool,
    strike: bool,
    overline: bool,
    color: Option<Color>,
    width: Option<Length>,
}

impl TextDecoration {
    pub const NONE: TextDecoration = TextDecoration {
        underline: false,
        strike: false,
        overline: false,
        color: None,
        width: None,
    };

    pub const UNDERLINE: TextDecoration = TextDecoration {
        underline: true,
        strike: false,
        overline: false,
        color: None,
        width: None,
    };

    pub const STRIKETHROUGH: TextDecoration = TextDecoration {
        underline: false,
        strike: true,
        overline: false,
        color: None,
        width: None,
    };

    pub const OVERLINE: TextDecoration = TextDecoration {
        underline: false,
        strike: false,
        overline: true,
        color: None,
        width: None,
    };

    pub fn underline(&self) -> bool {
        self.underline
    }

    pub fn strike(&self) -> bool {
        self.strike
    }

    pub fn overline(&self) -> bool {
        self.overline
    }

    pub fn color(&self) -> Option<Color> {
        self.color
    }

    pub fn width(&self) -> Option<Length> {
        self.width
    }

    pub fn with_underline(mut self, value: bool) -> Self {
        self.underline = value;
        self
    }

    pub fn with_strike(mut self, value: bool) -> Self {
        self.strike = value;
        self
    }

    pub fn with_overline(mut self, value: bool) -> Self {
        self.overline = value;
        self
    }

    pub fn with_color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    pub fn with_width(mut self, width: impl Into<Length>) -> Self {
        self.width = Some(width.into());
        self
    }
}
