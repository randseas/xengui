// SPDX-License-Identifier: Apache-2.0
use super::Color;

#[derive(Clone, Debug, PartialEq)]
pub enum Background {
    Color(Color),
    // LinearGradient(LinearGradient),
    // RadialGradient(RadialGradient),
    // Image(ImageBrush),
}
impl From<Color> for Background {
    fn from(color: Color) -> Self {
        Self::Color(color)
    }
}