use crate::style::Length;

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Default)]
pub struct LineHeight(Length);

impl LineHeight {
    /// Creates a new line height.
    pub fn new<L>(value: L) -> Self
    where
        L: Into<Length>,
    {
        Self(value.into())
    }

    /// Returns the line height value.
    pub const fn value(&self) -> Length {
        self.0
    }
}

impl<L> From<L> for LineHeight
where
    L: Into<Length>,
{
    fn from(value: L) -> Self {
        Self::new(value)
    }
}
