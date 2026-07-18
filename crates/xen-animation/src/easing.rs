// SPDX-License-Identifier: Apache-2.0

/// A CSS-compatible cubic-bezier timing function. The curve's start and end
/// points are implicitly (0,0) and (1,1); only the two control points are
/// stored, matching the `cubic-bezier(x1, y1, x2, y2)` CSS syntax.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CubicBezier {
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
}

impl CubicBezier {
    /// Creates a curve from its two control points, matching the argument
    /// order of the CSS `cubic-bezier(x1, y1, x2, y2)` function.
    pub const fn new(x1: f32, y1: f32, x2: f32, y2: f32) -> Self {
        Self { x1, y1, x2, y2 }
    }

    // Polynomial coefficients for the x(t) component of the bezier curve.
    fn coefficients_x(&self) -> (f32, f32, f32) {
        let cx = 3.0 * self.x1;
        let bx = 3.0 * (self.x2 - self.x1) - cx;
        let ax = 1.0 - cx - bx;
        (cx, bx, ax)
    }

    // Polynomial coefficients for the y(t) component of the bezier curve.
    fn coefficients_y(&self) -> (f32, f32, f32) {
        let cy = 3.0 * self.y1;
        let by = 3.0 * (self.y2 - self.y1) - cy;
        let ay = 1.0 - cy - by;
        (cy, by, ay)
    }

    // Evaluates x(t) for the internal bezier parameter t.
    fn sample_curve_x(&self, t: f32) -> f32 {
        let (cx, bx, ax) = self.coefficients_x();
        ((ax * t + bx) * t + cx) * t
    }

    // Evaluates y(t) for the internal bezier parameter t.
    fn sample_curve_y(&self, t: f32) -> f32 {
        let (cy, by, ay) = self.coefficients_y();
        ((ay * t + by) * t + cy) * t
    }

    // dx/dt, used by Newton-Raphson to invert x(t) -> t.
    fn sample_curve_derivative_x(&self, t: f32) -> f32 {
        let (cx, bx, ax) = self.coefficients_x();
        (3.0 * ax * t + 2.0 * bx) * t + cx
    }

    // Solves for the bezier parameter t given progress x, using Newton-Raphson
    // with a bisection fallback for curves where the tangent gets too flat to
    // converge - the same approach browsers use for CSS cubic-bezier() timing.
    fn solve_t_for_x(&self, x: f32, epsilon: f32) -> f32 {
        let mut t = x;

        for _ in 0..8 {
            let x_est = self.sample_curve_x(t) - x;
            if x_est.abs() < epsilon {
                return t;
            }
            let d = self.sample_curve_derivative_x(t);
            if d.abs() < 1e-6 {
                break;
            }
            t -= x_est / d;
        }

        let mut lo = 0.0f32;
        let mut hi = 1.0f32;
        t = x.clamp(lo, hi);

        while lo < hi {
            let x_est = self.sample_curve_x(t);
            if (x_est - x).abs() < epsilon {
                return t;
            }
            if x > x_est {
                lo = t;
            } else {
                hi = t;
            }
            t = (hi + lo) * 0.5;
        }

        t
    }

    /// Evaluates the easing curve for a normalized progress in the range `[0.0, 1.0]`.
    ///
    /// Values outside that range are clamped to the curve's endpoints (`0.0`
    /// or `1.0`), matching how CSS handles cubic-bezier timing functions.
    pub fn solve(&self, x: f32) -> f32 {
        if x <= 0.0 {
            return 0.0;
        }
        if x >= 1.0 {
            return 1.0;
        }
        let t = self.solve_t_for_x(x, 1e-5);
        self.sample_curve_y(t)
    }
}

/// A named or custom timing function used to shape how progress `[0.0, 1.0]`
/// maps to eased progress over the course of a [`crate::Transition`].
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum Easing {
    /// Constant rate of change; progress and eased progress are equal.
    #[default]
    Linear,
    /// Starts slow and accelerates towards the end.
    EaseIn,
    /// Starts fast and decelerates towards the end.
    EaseOut,
    /// Starts slow, speeds up through the middle, and slows down again.
    EaseInOut,
    /// A user-supplied cubic-bezier curve, for timing functions not covered
    /// by the built-in presets.
    CubicBezier(CubicBezier),
}

impl Easing {
    // TailwindCSS --ease-in / --ease-out / --ease-in-out values.
    pub const EASE_IN: CubicBezier = CubicBezier::new(0.4, 0.0, 1.0, 1.0);
    pub const EASE_OUT: CubicBezier = CubicBezier::new(0.0, 0.0, 0.2, 1.0);
    pub const EASE_IN_OUT: CubicBezier = CubicBezier::new(0.4, 0.0, 0.2, 1.0);

    /// Shorthand for `Easing::CubicBezier(CubicBezier::new(x1, y1, x2, y2))`.
    pub const fn cubic_bezier(x1: f32, y1: f32, x2: f32, y2: f32) -> Self {
        Self::CubicBezier(CubicBezier::new(x1, y1, x2, y2))
    }

    /// Applies this easing function to a normalized progress value, clamping
    /// `t` to `[0.0, 1.0]` first.
    pub fn apply(self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        match self {
            Self::Linear => t,
            Self::EaseIn => Self::EASE_IN.solve(t),
            Self::EaseOut => Self::EASE_OUT.solve(t),
            Self::EaseInOut => Self::EASE_IN_OUT.solve(t),
            Self::CubicBezier(curve) => curve.solve(t),
        }
    }
}
