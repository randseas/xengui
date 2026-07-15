// SPDX-License-Identifier: Apache-2.0
use crate::text::TextMeasurer;

/// Context passed to widgets during intrinsic measurement.
///
/// Unlike `LayoutContext`, this context contains only the resources required
/// to calculate a widget's preferred size. It is intentionally lightweight
/// and independent from rendering.
pub struct MeasureContext<'a> {
    /// Text measurement backend.
    pub text: &'a mut dyn TextMeasurer,

    /// Device scale factor.
    ///
    /// Widgets should return measurements in logical pixels. This value can
    /// be used when converting between logical and physical units.
    pub scale_factor: f32,
}

impl<'a> MeasureContext<'a> {
    /// Creates a new measurement context.
    pub const fn new(text: &'a mut dyn TextMeasurer, scale_factor: f32) -> Self {
        Self {
            text,
            scale_factor,
        }
    }
}
