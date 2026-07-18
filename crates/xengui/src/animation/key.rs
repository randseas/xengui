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

/// Identifies one animatable value on one widget's layer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct AnimKey {
    pub widget: WidgetId,
    pub layer: AnimLayer,
    pub property: AnimProperty,
}
