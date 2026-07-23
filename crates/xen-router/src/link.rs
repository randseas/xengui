// SPDX-License-Identifier: Apache-2.0
use smol_str::SmolStr;
use xengui::Button;

/// A `Button` pre-wired to navigate to `path` on click instead of firing a
/// custom callback - the SPA equivalent of an `<a href>`, without the full
/// page reload `xengui::Link::href` would cause.
///
/// Returns a plain `Button`, so every other builder method (`.label`,
/// `.padding`, hover/pressed styles, ...) still applies normally.
pub fn router_link(path: impl Into<SmolStr>) -> Button {
    let path = path.into();
    Button::new().on_click(move |_ctx| crate::navigate(path.to_string()))
}
