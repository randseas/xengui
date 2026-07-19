// SPDX-License-Identifier: Apache-2.0
use super::SvgElement;

/// Parsed SVG document: the `viewBox` (used as the local coordinate space
/// every element is authored in) plus the top-level element tree.
#[derive(Clone, Debug, PartialEq)]
pub struct SvgDocument {
    pub view_box: (f32, f32, f32, f32),
    pub elements: Vec<SvgElement>,
}

impl SvgDocument {
    pub fn new(view_box: (f32, f32, f32, f32)) -> Self {
        Self { view_box, elements: Vec::new() }
    }
}

impl Default for SvgDocument {
    fn default() -> Self {
        Self::new((0.0, 0.0, 24.0, 24.0))
    }
}
