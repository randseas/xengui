// SPDX-License-Identifier: Apache-2.0
use super::{ SvgColor, Transform2D };
use crate::Color;

/// One segment of an SVG path's `d` attribute, already normalized to
/// absolute coordinates (relative commands are resolved during parsing).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PathCommand {
    MoveTo(f32, f32),
    LineTo(f32, f32),
    QuadTo(f32, f32, f32, f32),
    CubicTo(f32, f32, f32, f32, f32, f32),
    Close,
}

/// Value of the `stroke-linecap` presentation attribute.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum LineCap {
    #[default]
    Butt,
    Round,
    Square,
}

/// Value of the `stroke-linejoin` presentation attribute.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum LineJoin {
    #[default]
    Miter,
    Round,
    Bevel,
}

/// Value of the `fill-rule` presentation attribute.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum FillRule {
    #[default]
    NonZero,
    EvenOdd,
}

/// Presentation attributes shared by every SVG element, mirroring the CSS
/// properties SVG borrows for fill/stroke/opacity.
#[derive(Clone, Debug, PartialEq)]
pub struct SvgAttributes {
    pub fill: SvgColor,
    pub fill_rule: FillRule,
    pub stroke: SvgColor,
    pub stroke_width: f32,
    pub line_cap: LineCap,
    pub line_join: LineJoin,
    pub miter_limit: f32,
    pub opacity: f32,
    pub transform: Transform2D,
}

impl Default for SvgAttributes {
    // Matches SVG's own defaults: shapes fill black unless told otherwise,
    // have no stroke at all, and use the spec's default cap/join/miter.
    fn default() -> Self {
        Self {
            fill: SvgColor::Solid(Color::BLACK),
            fill_rule: FillRule::NonZero,
            stroke: SvgColor::None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            miter_limit: 4.0,
            opacity: 1.0,
            transform: Transform2D::IDENTITY,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum SvgElement {
    Path {
        commands: Vec<PathCommand>,
        attrs: SvgAttributes,
    },
    Rect {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        rx: f32,
        attrs: SvgAttributes,
    },
    Circle {
        cx: f32,
        cy: f32,
        r: f32,
        attrs: SvgAttributes,
    },
    Line {
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        attrs: SvgAttributes,
    },
    Group {
        children: Vec<SvgElement>,
        attrs: SvgAttributes,
    },
}

impl SvgElement {
    pub fn attrs(&self) -> &SvgAttributes {
        match self {
            | Self::Path { attrs, .. }
            | Self::Rect { attrs, .. }
            | Self::Circle { attrs, .. }
            | Self::Line { attrs, .. }
            | Self::Group { attrs, .. } => attrs,
        }
    }
}
