// SPDX-License-Identifier: Apache-2.0

/// 2D affine transform matrix, laid out the same way as SVG's `matrix(a,b,c,d,e,f)`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Transform2D {
    pub a: f32,
    pub b: f32,
    pub c: f32,
    pub d: f32,
    pub e: f32,
    pub f: f32,
}

impl Transform2D {
    pub const IDENTITY: Self = Self { a: 1.0, b: 0.0, c: 0.0, d: 1.0, e: 0.0, f: 0.0 };

    pub const fn new(a: f32, b: f32, c: f32, d: f32, e: f32, f: f32) -> Self {
        Self { a, b, c, d, e, f }
    }

    pub fn translate(tx: f32, ty: f32) -> Self {
        Self::new(1.0, 0.0, 0.0, 1.0, tx, ty)
    }

    pub fn scale(sx: f32, sy: f32) -> Self {
        Self::new(sx, 0.0, 0.0, sy, 0.0, 0.0)
    }

    pub fn rotate(degrees: f32) -> Self {
        let rad = degrees.to_radians();
        let (sin, cos) = rad.sin_cos();
        Self::new(cos, sin, -sin, cos, 0.0, 0.0)
    }

    pub fn skew_x(degrees: f32) -> Self {
        Self::new(1.0, 0.0, degrees.to_radians().tan(), 1.0, 0.0, 0.0)
    }

    pub fn skew_y(degrees: f32) -> Self {
        Self::new(1.0, degrees.to_radians().tan(), 0.0, 1.0, 0.0, 0.0)
    }

    pub fn then(self, next: Self) -> Self {
        Self {
            a: self.a * next.a + self.b * next.c,
            b: self.a * next.b + self.b * next.d,
            c: self.c * next.a + self.d * next.c,
            d: self.c * next.b + self.d * next.d,
            e: self.e * next.a + self.f * next.c + next.e,
            f: self.e * next.b + self.f * next.d + next.f,
        }
    }

    pub fn apply(self, x: f32, y: f32) -> (f32, f32) {
        (self.a * x + self.c * y + self.e, self.b * x + self.d * y + self.f)
    }
}

impl Default for Transform2D {
    fn default() -> Self {
        Self::IDENTITY
    }
}

/// Parses an SVG `transform` attribute value, e.g. `"translate(4 5) rotate(45)"`.
/// Unknown or malformed functions are skipped rather than aborting the whole parse.
pub fn parse_transform(input: &str) -> Transform2D {
    let mut result = Transform2D::IDENTITY;

    for func in input.split(')') {
        let func = func.trim();
        if func.is_empty() {
            continue;
        }
        let Some((name, args)) = func.split_once('(') else {
            continue;
        };
        let nums: Vec<f32> = args
            .split([',', ' '])
            .filter(|s| !s.is_empty())
            .filter_map(|s| s.parse().ok())
            .collect();

        let matrix = match name.trim() {
            "translate" =>
                match nums.as_slice() {
                    [tx] => Transform2D::translate(*tx, 0.0),
                    [tx, ty] => Transform2D::translate(*tx, *ty),
                    _ => {
                        continue;
                    }
                }
            "scale" =>
                match nums.as_slice() {
                    [s] => Transform2D::scale(*s, *s),
                    [sx, sy] => Transform2D::scale(*sx, *sy),
                    _ => {
                        continue;
                    }
                }
            "rotate" =>
                match nums.as_slice() {
                    [deg] => Transform2D::rotate(*deg),
                    [deg, cx, cy] =>
                        Transform2D::translate(*cx, *cy)
                            .then(Transform2D::rotate(*deg))
                            .then(Transform2D::translate(-cx, -cy)),
                    _ => {
                        continue;
                    }
                }
            "skewX" =>
                match nums.as_slice() {
                    [deg] => Transform2D::skew_x(*deg),
                    _ => {
                        continue;
                    }
                }
            "skewY" =>
                match nums.as_slice() {
                    [deg] => Transform2D::skew_y(*deg),
                    _ => {
                        continue;
                    }
                }
            "matrix" =>
                match nums.as_slice() {
                    [a, b, c, d, e, f] => Transform2D::new(*a, *b, *c, *d, *e, *f),
                    _ => {
                        continue;
                    }
                }
            _ => {
                continue;
            }
        };

        result = result.then(matrix);
    }

    result
}
