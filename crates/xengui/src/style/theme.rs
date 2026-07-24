use crate::{ Border, Outline, properties::StyleValue };

// SPDX-License-Identifier: Apache-2.0
use super::{ Background, Color, Edges, Length };
use std::cell::RefCell;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThemeMode {
    Light,
    Dark,
    Auto,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Theme {
    name: String,
    mode: ThemeMode,

    pub primary: Color,
    pub accent: Color,
    pub background: Color,
    pub surface: Color,
    pub surface_hover: Color,
    pub foreground: Color,
    pub foreground_muted: Color,
    pub border: Color,
    pub border_hover: Color,

    pub hover: Color,
    pub pressed: Color,
    pub disabled: Color,

    pub selection: Color,
    pub selection_color: Color,
    pub selection_border_color: Color,
    pub selection_border_width: Length,
    pub selection_border_radius: Length,
    pub caret_color: Color,

    pub radius_xs: Length,
    pub radius_sm: Length,
    pub radius_md: Length,
    pub radius_lg: Length,
    pub radius_xl: Length,
    pub radius_2xl: Length,
    pub radius_3xl: Length,
    pub radius_4xl: Length,

    pub space_xs: Length,
    pub space_sm: Length,
    pub space_md: Length,
    pub space_lg: Length,
    pub space_xl: Length,
    pub space_2xl: Length,
    pub space_3xl: Length,
    pub space_4xl: Length,

    pub border_width: Length,
}

impl Theme {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            mode: ThemeMode::Light,

            // Colors
            primary: Color::BLUE_500,
            accent: Color::BLUE_500,

            background: Color::WHITE,
            surface: Color::NEUTRAL_50,
            surface_hover: Color::NEUTRAL_100,

            foreground: Color::NEUTRAL_900,
            foreground_muted: Color::NEUTRAL_500,

            border: Color::NEUTRAL_200,
            border_hover: Color::NEUTRAL_300,

            hover: Color::NEUTRAL_100,
            pressed: Color::NEUTRAL_200,
            disabled: Color::NEUTRAL_300.with_alpha(50),

            selection: Color::BLUE_500.with_alpha(80),
            selection_color: Color::BLUE_200,
            selection_border_color: Color::TRANSPARENT,
            selection_border_width: Length::px(0.0),
            selection_border_radius: Length::px(4.0),
            caret_color: Color::WHITE,

            // Border radius
            radius_xs: Length::px(2.0), // rounded-sm
            radius_sm: Length::px(4.0), // rounded
            radius_md: Length::px(6.0), // rounded-md
            radius_lg: Length::px(8.0), // rounded-lg
            radius_xl: Length::px(12.0), // rounded-xl
            radius_2xl: Length::px(16.0), // rounded-2xl
            radius_3xl: Length::px(24.0), // rounded-3xl
            radius_4xl: Length::px(9999.0), // rounded-full

            // Spacing
            space_xs: Length::px(2.0),
            space_sm: Length::px(4.0),
            space_md: Length::px(8.0),
            space_lg: Length::px(12.0),
            space_xl: Length::px(16.0),
            space_2xl: Length::px(24.0),
            space_3xl: Length::px(32.0),
            space_4xl: Length::px(48.0),

            border_width: Length::px(1.0),
        }
    }

    pub fn light() -> Self {
        Self::new("light")
            .mode(ThemeMode::Light)
            .primary(Color::BLUE_500)
            .accent(Color::BLUE_500)

            .background(Color::WHITE)
            .surface(Color::NEUTRAL_50)
            .surface_hover(Color::NEUTRAL_100)

            .foreground(Color::NEUTRAL_900)
            .foreground_muted(Color::NEUTRAL_500)

            .border(Color::NEUTRAL_200)
            .border_hover(Color::NEUTRAL_300)

            .hover(Color::NEUTRAL_100)
            .pressed(Color::NEUTRAL_200)
            .disabled(Color::NEUTRAL_300.with_alpha(50))

            .selection(Color::BLUE_500.with_alpha(80))
            .selection_color(Color::BLUE_200)
            .caret_color(Color::BLUE_500)
            .selection_border_color(Color::TRANSPARENT)
            .selection_border_width(Length::px(0.0))
            .selection_border_radius(Length::px(4.0))
    }

    pub fn dark() -> Self {
        Self::new("dark")
            .mode(ThemeMode::Dark)
            .primary(Color::BLUE_400)
            .accent(Color::BLUE_400)

            .background(Color::NEUTRAL_950)
            .surface(Color::NEUTRAL_900)
            .surface_hover(Color::NEUTRAL_800)

            .foreground(Color::NEUTRAL_50)
            .foreground_muted(Color::NEUTRAL_300)

            .border(Color::NEUTRAL_800)
            .border_hover(Color::NEUTRAL_700)

            .hover(Color::NEUTRAL_800)
            .pressed(Color::NEUTRAL_700)
            .disabled(Color::NEUTRAL_700.with_alpha(50))

            .selection(Color::BLUE_500.with_alpha(80))
            .selection_color(Color::BLUE_200)
            .caret_color(Color::BLUE_400)
            .selection_border_color(Color::TRANSPARENT)
            .selection_border_width(Length::px(0.0))
            .selection_border_radius(Length::px(4.0))
    }

    pub fn auto() -> Self {
        let mut theme = Self::light();
        theme.mode = ThemeMode::Auto;
        theme
    }

    pub fn mode(mut self, mode: ThemeMode) -> Self {
        self.mode = mode;
        self
    }

    pub fn primary(mut self, color: Color) -> Self {
        self.primary = color;
        self
    }

    pub fn accent(mut self, color: Color) -> Self {
        self.accent = color;
        self
    }

    pub fn background(mut self, color: Color) -> Self {
        self.background = color;
        self
    }

    pub fn surface(mut self, color: Color) -> Self {
        self.surface = color;
        self
    }

    pub fn surface_hover(mut self, color: Color) -> Self {
        self.surface_hover = color;
        self
    }

    pub fn foreground(mut self, color: Color) -> Self {
        self.foreground = color;
        self
    }

    pub fn foreground_muted(mut self, color: Color) -> Self {
        self.foreground_muted = color;
        self
    }

    pub fn border(mut self, color: Color) -> Self {
        self.border = color;
        self
    }

    pub fn border_hover(mut self, color: Color) -> Self {
        self.border_hover = color;
        self
    }

    pub fn hover(mut self, color: Color) -> Self {
        self.hover = color;
        self
    }

    pub fn pressed(mut self, color: Color) -> Self {
        self.pressed = color;
        self
    }

    pub fn disabled(mut self, color: Color) -> Self {
        self.disabled = color;
        self
    }

    pub fn selection(mut self, color: Color) -> Self {
        self.selection = color;
        self
    }

    pub fn selection_color(mut self, color: Color) -> Self {
        self.selection_color = color;
        self
    }

    pub fn caret_color(mut self, color: Color) -> Self {
        self.caret_color = color;
        self
    }

    pub fn selection_border_width(mut self, width: Length) -> Self {
        self.selection_border_width = width;
        self
    }

    pub fn selection_border_color(mut self, color: Color) -> Self {
        self.selection_border_color = color;
        self
    }

    pub fn selection_border_radius(mut self, radius: Length) -> Self {
        self.selection_border_radius = radius;
        self
    }

    /* Radius: start */
    pub fn radius_xs(mut self, radius: impl Into<Length>) -> Self {
        self.radius_xs = radius.into();
        self
    }

    pub fn radius_sm(mut self, radius: impl Into<Length>) -> Self {
        self.radius_sm = radius.into();
        self
    }

    pub fn radius_md(mut self, radius: impl Into<Length>) -> Self {
        self.radius_md = radius.into();
        self
    }

    pub fn radius_lg(mut self, radius: impl Into<Length>) -> Self {
        self.radius_lg = radius.into();
        self
    }

    pub fn radius_xl(mut self, radius: impl Into<Length>) -> Self {
        self.radius_xl = radius.into();
        self
    }

    pub fn radius_2xl(mut self, radius: impl Into<Length>) -> Self {
        self.radius_2xl = radius.into();
        self
    }

    pub fn radius_3xl(mut self, radius: impl Into<Length>) -> Self {
        self.radius_3xl = radius.into();
        self
    }

    pub fn radius_4xl(mut self, radius: impl Into<Length>) -> Self {
        self.radius_4xl = radius.into();
        self
    }
    /* Radius: end */

    /* Padding: start */
    pub fn space_xs(mut self, space: impl Into<Length>) -> Self {
        self.space_xs = space.into();
        self
    }

    pub fn space_sm(mut self, space: impl Into<Length>) -> Self {
        self.space_sm = space.into();
        self
    }

    pub fn space_md(mut self, space: impl Into<Length>) -> Self {
        self.space_md = space.into();
        self
    }

    pub fn space_lg(mut self, space: impl Into<Length>) -> Self {
        self.space_lg = space.into();
        self
    }

    pub fn space_xl(mut self, space: impl Into<Length>) -> Self {
        self.space_xl = space.into();
        self
    }

    pub fn space_2xl(mut self, space: impl Into<Length>) -> Self {
        self.space_2xl = space.into();
        self
    }

    pub fn space_3xl(mut self, space: impl Into<Length>) -> Self {
        self.space_3xl = space.into();
        self
    }

    pub fn space_4xl(mut self, space: impl Into<Length>) -> Self {
        self.space_4xl = space.into();
        self
    }
    /* Padding: end */

    pub fn border_width(mut self, width: impl Into<Length>) -> Self {
        self.border_width = width.into();
        self
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub const fn is_dark(&self) -> bool {
        matches!(self.mode, ThemeMode::Dark)
    }

    pub const fn is_auto(&self) -> bool {
        matches!(self.mode, ThemeMode::Auto)
    }

    // Only the color set flips with the system theme; spacing/radius
    // tokens the user configured on this theme are preserved as-is.
    pub fn resolved_for_system(&self, system_is_dark: bool) -> Self {
        if !self.is_auto() {
            return self.clone();
        }
        let palette = if system_is_dark { Self::dark() } else { Self::light() };
        Self {
            primary: palette.primary,
            accent: palette.accent,
            background: palette.background,
            surface: palette.surface,
            surface_hover: palette.surface_hover,
            foreground: palette.foreground,
            border: palette.border,
            ..self.clone()
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::light()
    }
}

// Which theme should become active on the next render pass; requested via
// `set_active_theme`/`set_active_theme_by_name` from anywhere in user code.
pub enum ThemeSwitch {
    Index(usize),
    Name(String),
}

thread_local! {
    static CURRENT_THEME: RefCell<Theme> = RefCell::new(Theme::default());
    static THEME_SWITCH: RefCell<Option<ThemeSwitch>> = const { RefCell::new(None) };
}

pub fn set_current_theme(theme: Theme) {
    CURRENT_THEME.with(|cell| {
        *cell.borrow_mut() = theme;
    });
}

pub fn take_theme_switch() -> Option<ThemeSwitch> {
    THEME_SWITCH.with(|cell| cell.borrow_mut().take())
}

pub fn current_theme() -> Theme {
    CURRENT_THEME.with(|cell| cell.borrow().clone())
}

/// Switches the app's active theme by index into `AppConfig::themes`,
/// triggering a rebuild on the next frame.
pub fn set_active_theme(index: usize) {
    THEME_SWITCH.with(|cell| {
        *cell.borrow_mut() = Some(ThemeSwitch::Index(index));
    });
    crate::hooks::mark_dirty_and_redraw();
}

/// Switches the app's active theme by matching `Theme::name()` against
/// `AppConfig::themes`, triggering a rebuild on the next frame.
pub fn set_active_theme_by_name(name: impl Into<String>) {
    THEME_SWITCH.with(|cell| {
        *cell.borrow_mut() = Some(ThemeSwitch::Name(name.into()));
    });
    crate::hooks::mark_dirty_and_redraw();
}

pub struct ValueMarker;
pub struct FnMarker;

pub trait IntoThemed<T, Marker> {
    fn resolve_themed(self) -> T;
}

impl IntoThemed<Color, ValueMarker> for Color {
    fn resolve_themed(self) -> Color {
        self
    }
}

impl<F: FnOnce(&Theme) -> Color> IntoThemed<Color, FnMarker> for F {
    fn resolve_themed(self) -> Color {
        CURRENT_THEME.with(|cell| self(&cell.borrow()))
    }
}

impl IntoThemed<Background, ValueMarker> for Color {
    fn resolve_themed(self) -> Background {
        Background::Color(self)
    }
}

impl IntoThemed<Background, ValueMarker> for Background {
    fn resolve_themed(self) -> Background {
        self
    }
}

impl<F: FnOnce(&Theme) -> Color> IntoThemed<Background, FnMarker> for F {
    fn resolve_themed(self) -> Background {
        Background::Color(CURRENT_THEME.with(|cell| self(&cell.borrow())))
    }
}

impl<T: Into<Length>> IntoThemed<Length, ValueMarker> for T {
    fn resolve_themed(self) -> Length {
        self.into()
    }
}

impl<F: FnOnce(&Theme) -> Length> IntoThemed<Length, FnMarker> for F {
    fn resolve_themed(self) -> Length {
        CURRENT_THEME.with(|cell| self(&cell.borrow()))
    }
}

impl<T: Into<Edges>> IntoThemed<Edges, ValueMarker> for T {
    fn resolve_themed(self) -> Edges {
        self.into()
    }
}

impl<F: FnOnce(&Theme) -> Edges> IntoThemed<Edges, FnMarker> for F {
    fn resolve_themed(self) -> Edges {
        CURRENT_THEME.with(|cell| self(&cell.borrow()))
    }
}

impl IntoThemed<Border, ValueMarker> for Border {
    fn resolve_themed(self) -> Border {
        self
    }
}

impl<F: FnOnce(&Theme) -> Border> IntoThemed<Border, FnMarker> for F {
    fn resolve_themed(self) -> Border {
        CURRENT_THEME.with(|cell| self(&cell.borrow()))
    }
}

impl IntoThemed<StyleValue<Outline>, ValueMarker> for Outline {
    fn resolve_themed(self) -> StyleValue<Outline> {
        StyleValue::Value(self)
    }
}

impl IntoThemed<StyleValue<Outline>, ValueMarker> for StyleValue<Outline> {
    fn resolve_themed(self) -> StyleValue<Outline> {
        self
    }
}

impl<F: FnOnce(&Theme) -> Outline> IntoThemed<StyleValue<Outline>, FnMarker> for F {
    fn resolve_themed(self) -> StyleValue<Outline> {
        StyleValue::Value(CURRENT_THEME.with(|cell| self(&cell.borrow())))
    }
}

impl IntoThemed<f32, ValueMarker> for f32 {
    fn resolve_themed(self) -> f32 {
        self
    }
}

impl<F: FnOnce(&Theme) -> f32> IntoThemed<f32, FnMarker> for F {
    fn resolve_themed(self) -> f32 {
        CURRENT_THEME.with(|cell| self(&cell.borrow()))
    }
}
