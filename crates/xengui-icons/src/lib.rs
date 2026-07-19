// SPDX-License-Identifier: Apache-2.0
//! Prebuilt icon widgets. Each function returns a fresh `xengui::Svg` using
//! `currentColor`, so it automatically follows the parent widget's text
//! color without any manual styling.

macro_rules! icon {
    ($name:ident, $svg:expr) => {
        pub fn $name() -> xengui::Svg {
            xengui::Svg::from_string($svg)
        }
    };
}

icon!(
    plus,
    r#"<svg viewBox="0 0 24 24"><path d="M12 5 L12 19 M5 12 L19 12" fill="none" stroke="currentColor" stroke-width="2"/></svg>"#
);

icon!(
    check,
    r#"<svg viewBox="0 0 24 24"><path d="M20 6 L9 17 L4 12" fill="none" stroke="currentColor" stroke-width="2"/></svg>"#
);

icon!(
    x,
    r#"<svg viewBox="0 0 24 24"><path d="M18 6 L6 18 M6 6 L18 18" fill="none" stroke="currentColor" stroke-width="2"/></svg>"#
);
