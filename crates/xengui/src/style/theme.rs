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
    pub foreground: Color,
    pub foreground_muted: Color,
    pub border: Color,
    pub border_hover: Color,

    pub hover: Color,
    pub pressed: Color,
    pub disabled: Color,

    pub radius_sm: Length,
    pub radius_md: Length,
    pub radius_lg: Length,

    pub padding_sm: Length,
    pub padding_md: Length,
    pub padding_lg: Length,

    pub gap_sm: Length,
    pub gap_md: Length,
    pub gap_lg: Length,

    pub border_width: Length,
}

impl Theme {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            mode: ThemeMode::Light,

            primary: Color::BLUE_500,
            accent: Color::VIOLET_500,
            background: Color::WHITE,
            surface: Color::NEUTRAL_050,
            foreground: Color::NEUTRAL_900,
            foreground_muted: Color::NEUTRAL_700,
            border: Color::NEUTRAL_200,
            border_hover: Color::NEUTRAL_300,

            hover: Color::NEUTRAL_300,
            pressed: Color::NEUTRAL_400,
            disabled: Color::NEUTRAL_300.with_alpha(50),

            radius_sm: Length::px(4.0),
            radius_md: Length::px(8.0),
            radius_lg: Length::px(16.0),

            padding_sm: Length::px(6.0),
            padding_md: Length::px(12.0),
            padding_lg: Length::px(20.0),

            gap_sm: Length::px(4.0),
            gap_md: Length::px(8.0),
            gap_lg: Length::px(16.0),

            border_width: Length::px(1.0),
        }
    }

    pub fn light() -> Self {
        Self::new("light")
    }

    pub fn dark() -> Self {
        Self::new("dark")
            .mode(ThemeMode::Dark)
            .primary(Color::BLUE_400)
            .accent(Color::VIOLET_400)
            .background(Color::NEUTRAL_950)
            .surface(Color::NEUTRAL_900)
            .foreground(Color::NEUTRAL_050)
            .border(Color::NEUTRAL_800)
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

    pub fn padding_sm(mut self, padding: impl Into<Length>) -> Self {
        self.padding_sm = padding.into();
        self
    }

    pub fn padding_md(mut self, padding: impl Into<Length>) -> Self {
        self.padding_md = padding.into();
        self
    }

    pub fn padding_lg(mut self, padding: impl Into<Length>) -> Self {
        self.padding_lg = padding.into();
        self
    }

    pub fn gap_sm(mut self, gap: impl Into<Length>) -> Self {
        self.gap_sm = gap.into();
        self
    }

    pub fn gap_md(mut self, gap: impl Into<Length>) -> Self {
        self.gap_md = gap.into();
        self
    }

    pub fn gap_lg(mut self, gap: impl Into<Length>) -> Self {
        self.gap_lg = gap.into();
        self
    }

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
    pub(crate) fn resolved_for_system(&self, system_is_dark: bool) -> Self {
        if !self.is_auto() {
            return self.clone();
        }
        let palette = if system_is_dark { Self::dark() } else { Self::light() };
        Self {
            primary: palette.primary,
            accent: palette.accent,
            background: palette.background,
            surface: palette.surface,
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
pub(crate) enum ThemeSwitch {
    Index(usize),
    Name(String),
}

thread_local! {
    static CURRENT_THEME: RefCell<Theme> = RefCell::new(Theme::default());
    static THEME_SWITCH: RefCell<Option<ThemeSwitch>> = const { RefCell::new(None) };
}

pub(crate) fn set_current_theme(theme: Theme) {
    CURRENT_THEME.with(|cell| {
        *cell.borrow_mut() = theme;
    });
}

pub(crate) fn take_theme_switch() -> Option<ThemeSwitch> {
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
