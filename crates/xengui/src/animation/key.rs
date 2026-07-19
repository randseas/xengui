// SPDX-License-Identifier: Apache-2.0
use crate::WidgetId;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum AnimLayer {
    #[default]
    Root,
    Background,
    Content,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AnimProperty {
    BackgroundColor,
    TextColor,
    BorderColor,
    Opacity,
    Scale,
    ContentScale,
    ShadowColor,
    BorderWidth,
    BorderRadius,
    Width,
    Height,
    PaddingLeft,
    PaddingTop,
    PaddingRight,
    PaddingBottom,
    MarginLeft,
    MarginTop,
    MarginRight,
    MarginBottom,
    GapX,
    GapY,
}

impl AnimProperty {
    /// Whether an in-flight transition of this property changes the box
    /// model and therefore requires a real layout pass, as opposed to
    /// colors, opacity, or transform-only properties which only need a
    /// repaint on every animation frame.
    pub const fn affects_layout(self) -> bool {
        matches!(
            self,
            Self::BorderWidth |
                Self::BorderRadius |
                Self::Width |
                Self::Height |
                Self::PaddingLeft |
                Self::PaddingTop |
                Self::PaddingRight |
                Self::PaddingBottom |
                Self::MarginLeft |
                Self::MarginTop |
                Self::MarginRight |
                Self::MarginBottom |
                Self::GapX |
                Self::GapY
        )
    }
}

/// Identifies one animatable value on one widget's layer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct AnimKey {
    pub widget: WidgetId,
    pub layer: AnimLayer,
    pub property: AnimProperty,
}
