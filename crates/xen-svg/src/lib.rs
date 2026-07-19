// SPDX-License-Identifier: Apache-2.0
//! Platform-agnostic SVG support: a small element model, a `d`/`viewBox`
//! parser, and a triangle tessellator. Has no rendering or windowing
//! dependencies of its own - any GUI framework can consume the tessellated
//! triangle list and draw it through its own pipeline.

mod color;
mod document;
mod element;
mod parser;
mod tessellate;
mod transform;

pub use color::{ Color, SvgColor };
pub use document::SvgDocument;
pub use element::{ FillRule, LineCap, LineJoin, PathCommand, SvgAttributes, SvgElement };
pub use parser::parse_svg;
pub use tessellate::{ tessellate_document, SvgTriangle };
pub use transform::{ parse_transform, Transform2D };
