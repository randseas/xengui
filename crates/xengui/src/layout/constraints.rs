// SPDX-License-Identifier: Apache-2.0

/// Layout constraints passed to a widget during measurement.
///
/// This type is intentionally layout-engine agnostic. Widgets should not
/// depend on Taffy's internal types or APIs.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Constraints {
    /// Width already determined by the layout engine.
    ///
    /// When this value is `Some`, the widget should use it directly instead
    /// of calculating its own intrinsic width.
    pub known_width: Option<f32>,

    /// Height already determined by the layout engine.
    ///
    /// When this value is `Some`, the widget should use it directly instead
    /// of calculating its own intrinsic height.
    pub known_height: Option<f32>,

    /// Maximum width available to the widget.
    ///
    /// `None` means the available width is unbounded.
    pub max_width: Option<f32>,

    /// Maximum height available to the widget.
    ///
    /// `None` means the available height is unbounded.
    pub max_height: Option<f32>,
}

impl Constraints {
    /// Constraints with no width or height limits.
    pub const UNBOUNDED: Self = Self {
        known_width: None,
        known_height: None,
        max_width: None,
        max_height: None,
    };

    /// Creates an unconstrained set of layout constraints.
    pub const fn new() -> Self {
        Self::UNBOUNDED
    }

    /// Returns a copy with a known width.
    pub const fn with_known_width(mut self, width: f32) -> Self {
        self.known_width = Some(width);
        self
    }

    /// Returns a copy with a known height.
    pub const fn with_known_height(mut self, height: f32) -> Self {
        self.known_height = Some(height);
        self
    }

    /// Returns a copy with a maximum width.
    pub const fn with_max_width(mut self, width: f32) -> Self {
        self.max_width = Some(width);
        self
    }

    /// Returns a copy with a maximum height.
    pub const fn with_max_height(mut self, height: f32) -> Self {
        self.max_height = Some(height);
        self
    }

    /// Applies the current constraints to the given width.
    #[inline]
    pub fn constrain_width(&self, width: f32) -> f32 {
        let mut width = self.known_width.unwrap_or(width);

        if let Some(max) = self.max_width {
            width = width.min(max);
        }

        width
    }

    /// Applies the current constraints to the given height.
    #[inline]
    pub fn constrain_height(&self, height: f32) -> f32 {
        let mut height = self.known_height.unwrap_or(height);

        if let Some(max) = self.max_height {
            height = height.min(max);
        }

        height
    }

    /// Applies the current constraints to both width and height.
    #[inline]
    pub fn constrain_size(&self, width: f32, height: f32) -> (f32, f32) {
        (self.constrain_width(width), self.constrain_height(height))
    }
}

impl Default for Constraints {
    fn default() -> Self {
        Self::UNBOUNDED
    }
}
