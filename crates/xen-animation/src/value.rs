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

    /// Interpolates two RGBA colors (packed as [r, g, b, a]) using
    /// premultiplied alpha. A straight per-channel lerp drags the RGB
    /// toward whichever endpoint has near-zero alpha (its RGB is otherwise
    /// meaningless), which shows up as a dark flash mid-fade whenever one
    /// endpoint is transparent.
    pub fn lerp_premultiplied(self, other: Self, t: f32) -> Self {
        let a_alpha = self.0[3];
        let b_alpha = other.0[3];
        let out_alpha = a_alpha + (b_alpha - a_alpha) * t;

        let premultiply = |v: [f32; 4]| [v[0] * v[3], v[1] * v[3], v[2] * v[3]];
        let pa = premultiply(self.0);
        let pb = premultiply(other.0);

        if out_alpha <= 0.0001 {
            return Self([0.0, 0.0, 0.0, 0.0]);
        }

        Self([
            (pa[0] + (pb[0] - pa[0]) * t) / out_alpha,
            (pa[1] + (pb[1] - pa[1]) * t) / out_alpha,
            (pa[2] + (pb[2] - pa[2]) * t) / out_alpha,
            out_alpha,
        ])
    }
}
