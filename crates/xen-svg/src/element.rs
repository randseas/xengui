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

/// Presentation attributes shared by every SVG element, mirroring the CSS
/// properties SVG borrows for fill/stroke/opacity.
#[derive(Clone, Debug, PartialEq)]
pub struct SvgAttributes {
    pub fill: SvgColor,
    pub stroke: SvgColor,
    pub stroke_width: f32,
    pub opacity: f32,
    pub transform: Transform2D,
}

impl Default for SvgAttributes {
    // Matches SVG's own defaults: shapes fill black unless told otherwise,
    // and have no stroke at all.
    fn default() -> Self {
        Self {
            fill: SvgColor::Solid(Color::BLACK),
            stroke: SvgColor::None,
            stroke_width: 1.0,
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
