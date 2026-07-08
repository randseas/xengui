// SPDX-License-Identifier: Apache-2.0
use crate::{Color, Length};

#[derive(Clone, Copy, Debug)]
pub struct Border {
    pub width: Length,
    pub color: Color,
    pub radius: Length,
}

impl Default for Border {
    fn default() -> Self {
        Self {
            width: Length::pixels(0.0),
            color: Color::TRANSPARENT,
            radius: Length::pixels(0.0),
        }
    }
}

impl Border {
    pub fn new(width: impl Into<Length>, color: Color, radius: impl Into<Length>) -> Self {
        Self {
            width: width.into(),
            color,
            radius: radius.into(),
        }
    }
}