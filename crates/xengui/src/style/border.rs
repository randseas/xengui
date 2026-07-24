// SPDX-License-Identifier: Apache-2.0
use crate::{ Color, Length };

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Border {
    pub top: Length,
    pub right: Length,
    pub bottom: Length,
    pub left: Length,
    pub color: Color,
    pub radius: Option<Length>,
}

impl Default for Border {
    fn default() -> Self {
        Self {
            top: Length::px(0.0),
            right: Length::px(0.0),
            bottom: Length::px(0.0),
            left: Length::px(0.0),
            color: Color::TRANSPARENT,
            radius: None,
        }
    }
}

impl Border {
    pub fn new(width: impl Into<Length>, color: Color, radius: impl Into<Length>) -> Self {
        let width = width.into();
        Self {
            top: width,
            right: width,
            bottom: width,
            left: width,
            color,
            radius: Some(radius.into()),
        }
    }

    pub fn all(width: impl Into<Length>, color: Color) -> Self {
        let width = width.into();
        Self { top: width, right: width, bottom: width, left: width, color, radius: None }
    }

    pub fn sides(
        top: impl Into<Length>,
        right: impl Into<Length>,
        bottom: impl Into<Length>,
        left: impl Into<Length>,
        color: Color
    ) -> Self {
        Self {
            top: top.into(),
            right: right.into(),
            bottom: bottom.into(),
            left: left.into(),
            color,
            radius: None,
        }
    }

    pub fn top(width: impl Into<Length>, color: Color) -> Self {
        Self { top: width.into(), color, ..Self::default() }
    }

    pub fn right(width: impl Into<Length>, color: Color) -> Self {
        Self { right: width.into(), color, ..Self::default() }
    }

    pub fn bottom(width: impl Into<Length>, color: Color) -> Self {
        Self { bottom: width.into(), color, ..Self::default() }
    }

    pub fn left(width: impl Into<Length>, color: Color) -> Self {
        Self { left: width.into(), color, ..Self::default() }
    }

    pub fn horizontal(width: impl Into<Length>, color: Color) -> Self {
        let width = width.into();
        Self { left: width, right: width, color, ..Self::default() }
    }

    pub fn vertical(width: impl Into<Length>, color: Color) -> Self {
        let width = width.into();
        Self { top: width, bottom: width, color, ..Self::default() }
    }

    pub fn radius(mut self, radius: impl Into<Length>) -> Self {
        self.radius = Some(radius.into());
        self
    }

    pub fn is_uniform(&self) -> bool {
        self.top == self.right && self.right == self.bottom && self.bottom == self.left
    }
}
