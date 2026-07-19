// SPDX-License-Identifier: Apache-2.0
//! Bridges xen-svg's framework-agnostic color type to xengui's own `Color`.
//! Lives here (not in xen-svg) since xen-svg must stay independent of
//! xengui; the conversion trait is local to this crate so it doesn't run
//! into orphan-rule restrictions on either foreign type.

use crate::Color;

pub trait IntoSvgColor {
    fn into_svg_color(self) -> xen_svg::SvgColor;
}

impl IntoSvgColor for Color {
    fn into_svg_color(self) -> xen_svg::SvgColor {
        xen_svg::SvgColor::Solid(xen_svg::Color::rgba_f32(self.r(), self.g(), self.b(), self.a()))
    }
}

impl IntoSvgColor for xen_svg::SvgColor {
    fn into_svg_color(self) -> xen_svg::SvgColor {
        self
    }
}

pub fn from_svg_color(color: xen_svg::Color) -> Color {
    Color::rgba_f32(color.r(), color.g(), color.b(), color.a())
}
