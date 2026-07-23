// SPDX-License-Identifier: Apache-2.0
use xengui::Color;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WindowShadow {
    pub color: Color,
    pub blur_radius: f32,
    pub spread_radius: f32,
    pub offset: (f32, f32),
    /// Extra transparent padding (logical px) reserved around the visible
    /// window content so the blurred shadow has room to render instead of
    /// being clipped at the surface edge.
    pub margin: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct WindowChrome {
    pub radius: f32,
    pub shadow: Option<WindowShadow>,
    pub border: Option<(f32, Color)>,
}
