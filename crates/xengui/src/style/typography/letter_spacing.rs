// SPDX-License-Identifier: Apache-2.0
use crate::style::Length;

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct LetterSpacing(Length);

impl LetterSpacing {
    /// Creates a new letter spacing.
    pub fn new<L>(value: L) -> Self
    where
        L: Into<Length>,
    {
        Self(value.into())
    }

    /// Returns the spacing value.
    pub const fn value(&self) -> Length {
        self.0
    }
}

impl<L> From<L> for LetterSpacing
where
    L: Into<Length>,
{
    fn from(value: L) -> Self {
        Self::new(value)
    }
}