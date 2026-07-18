// SPDX-License-Identifier: Apache-2.0

/// Generic animatable payload; every property (color, scale, opacity...)
/// is flattened into up to 4 floats so the manager can interpolate
/// anything without knowing what it represents.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AnimValue(pub [f32; 4]);

impl AnimValue {
    /// Linearly interpolates component-wise between `self` and `other`.
    ///
    /// `t` is expected to lie in `[0.0, 1.0]` but is not clamped here, so
    /// callers relying on overshoot easings can pass values outside that
    /// range and get correct extrapolation.
    pub fn lerp(self, other: Self, t: f32) -> Self {
        let mut out = [0.0; 4];

        for ((out, a), b) in out.iter_mut().zip(self.0.iter()).zip(other.0.iter()) {
            *out = *a + (*b - *a) * t;
        }

        Self(out)
    }
}
