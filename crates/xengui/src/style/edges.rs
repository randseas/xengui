// SPDX-License-Identifier: Apache-2.0
use super::Length;

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Edges {
    left: Length,
    top: Length,
    right: Length,
    bottom: Length,
}

impl Edges {
    pub fn all<L>(value: L) -> Self
    where
        L: Into<Length>,
    {
        let value = value.into();

        Self {
            left: value,
            top: value,
            right: value,
            bottom: value,
        }
    }

    pub fn symmetric<H, V>(horizontal: H, vertical: V) -> Self
    where
        H: Into<Length>,
        V: Into<Length>,
    {
        let horizontal = horizontal.into();
        let vertical = vertical.into();
        
        Self {
            left: horizontal,
            right: horizontal,
            top: vertical,
            bottom: vertical,
        }
    }

    pub fn only<L, T, R, B>(left: L, top: T, right: R, bottom: B) -> Self
    where
        L: Into<Length>,
        T: Into<Length>,
        R: Into<Length>,
        B: Into<Length>,
    {
        Self {
            left: left.into(),
            top: top.into(),
            right: right.into(),
            bottom: bottom.into(),
        }
    }

    pub const fn left(&self) -> Length {
        self.left
    }

    pub const fn top(&self) -> Length {
        self.top
    }

    pub const fn right(&self) -> Length {
        self.right
    }

    pub const fn bottom(&self) -> Length {
        self.bottom
    }
}
