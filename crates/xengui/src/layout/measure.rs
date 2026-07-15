// SPDX-License-Identifier: Apache-2.0

/// The result produced by a widget after measurement.
///
/// A widget reports its preferred size after applying the supplied layout
/// constraints.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct MeasureResult {
    /// The measured width in logical pixels.
    pub width: f32,

    /// The measured height in logical pixels.
    pub height: f32,

    /// The distance, in logical pixels, from the top of the widget to its
    /// first text baseline.
    ///
    /// Widgets without a meaningful baseline should leave this as `None`.
    pub baseline: Option<f32>,
}

impl MeasureResult {
    /// Creates a new measurement result.
    pub const fn new(width: f32, height: f32) -> Self {
        Self {
            width,
            height,
            baseline: None,
        }
    }

    /// Creates a new measurement result with an explicit baseline.
    pub const fn with_baseline(width: f32, height: f32, baseline: f32) -> Self {
        Self {
            width,
            height,
            baseline: Some(baseline),
        }
    }

    /// Returns a copy with the given baseline.
    #[inline]
    pub const fn baseline(mut self, baseline: f32) -> Self {
        self.baseline = Some(baseline);
        self
    }

    /// Returns the measured size as a tuple.
    #[inline]
    pub const fn into_tuple(self) -> (f32, f32) {
        (self.width, self.height)
    }
}

impl From<(f32, f32)> for MeasureResult {
    fn from((width, height): (f32, f32)) -> Self {
        Self::new(width, height)
    }
}

impl From<MeasureResult> for (f32, f32) {
    fn from(result: MeasureResult) -> Self {
        (result.width, result.height)
    }
}
