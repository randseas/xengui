// SPDX-License-Identifier: Apache-2.0

/// Minimal RGBA color, kept independent of any host framework's own color
/// type so this crate has no rendering/GUI dependency.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Color {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}

impl Color {
    pub const BLACK: Self = Self { r: 0.0, g: 0.0, b: 0.0, a: 1.0 };

    pub const fn rgba_f32(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub const fn r(&self) -> f32 {
        self.r
    }

    pub const fn g(&self) -> f32 {
        self.g
    }

    pub const fn b(&self) -> f32 {
        self.b
    }

    pub const fn a(&self) -> f32 {
        self.a
    }

    // Parses `#RGB`, `#RGBA`, `#RRGGBB`, `#RRGGBBAA`, with or without the
    // leading `#`. Falls back to opaque black on malformed input.
    pub fn hex(hex: &str) -> Self {
        let hex = hex.strip_prefix('#').unwrap_or(hex);

        let expanded: String;
        let hex = match hex.len() {
            3 | 4 => {
                expanded = hex
                    .chars()
                    .flat_map(|c| [c, c])
                    .collect();
                expanded.as_str()
            }
            _ => hex,
        };

        let parse_channel = |s: &str| -> Option<f32> {
            u8::from_str_radix(s, 16)
                .ok()
                .map(|v| (v as f32) / 255.0)
        };

        let parsed = match hex.len() {
            6 =>
                (
                    parse_channel(&hex[0..2]),
                    parse_channel(&hex[2..4]),
                    parse_channel(&hex[4..6]),
                    Some(1.0),
                ),
            8 =>
                (
                    parse_channel(&hex[0..2]),
                    parse_channel(&hex[2..4]),
                    parse_channel(&hex[4..6]),
                    parse_channel(&hex[6..8]),
                ),
            _ => (None, None, None, None),
        };

        match parsed {
            (Some(r), Some(g), Some(b), Some(a)) => Self::rgba_f32(r, g, b, a),
            _ => Self::rgba_f32(0.0, 0.0, 0.0, 1.0),
        }
    }
}

/// Paint value for SVG fill/stroke attributes.
///
/// Mirrors CSS/SVG's `currentColor` keyword: `SvgColor::CURRENT` defers the
/// actual color to whatever the host widget's inherited text color is at
/// render time, instead of a fixed color baked in up front.
#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub enum SvgColor {
    #[default]
    None,
    Solid(Color),
    Current,
}

impl SvgColor {
    pub const CURRENT: SvgColor = SvgColor::Current;
    pub const NONE: SvgColor = SvgColor::None;

    pub fn resolve(self, inherited: Color) -> Option<Color> {
        match self {
            SvgColor::Solid(color) => Some(color),
            SvgColor::Current => Some(inherited),
            SvgColor::None => None,
        }
    }
}

impl From<Color> for SvgColor {
    fn from(color: Color) -> Self {
        SvgColor::Solid(color)
    }
}
