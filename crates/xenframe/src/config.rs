use xengui::{ Theme };
use xengui_wgpu::{ WindowShadow };

#[cfg(not(target_arch = "wasm32"))]
use crate::WindowPosition;
use crate::window::Fullscreen;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
/// Controls how the active theme is selected.
pub enum AppThemeMode {
    /// Uses the currently selected theme.
    ///
    /// The active theme only changes when explicitly updated through
    /// `set_active_theme` or `set_active_theme_by_name`.
    #[default]
    Fixed,

    /// Automatically switches between the configured light and dark themes
    /// based on the operating system appearance preference.
    System,
}

pub struct AppConfig {
    pub title: String,

    /// Initial window width in pixels.
    #[cfg(not(target_arch = "wasm32"))]
    pub width: u32,

    /// Initial window height in pixels.
    #[cfg(not(target_arch = "wasm32"))]
    pub height: u32,

    /// Whether the window can be resized by the user.
    #[cfg(not(target_arch = "wasm32"))]
    pub resizable: bool,

    /// Initial position of the window on the screen.
    #[cfg(not(target_arch = "wasm32"))]
    pub position: WindowPosition,

    /// Whether the OS draws the native title bar and window border.
    ///
    /// Set to `false` to build a custom titlebar out of ordinary widgets;
    /// combine with `StyleBuilder::window_drag_region` for the draggable
    /// area and `xenframe::minimize_window`/`toggle_maximize_window`/
    /// `close_window` for the window control buttons. No effect on wasm32.
    #[cfg(not(target_arch = "wasm32"))]
    pub decorations: bool,

    /// Native window theme hint used by the operating system.
    pub theme: Option<winit::window::Theme>,

    /// Initial fullscreen state of the application window.
    pub fullscreen: Option<Fullscreen>,

    /// Fonts available to the application.
    ///
    /// Each entry contains a font name and its binary font data.
    pub fonts: Vec<(String, Vec<u8>)>,

    /// Themes registered for this application.
    ///
    /// Themes can be switched at runtime using `set_active_theme`
    /// or `set_active_theme_by_name`.
    pub themes: Vec<Theme>,

    /// Index of the theme currently applied to the application.
    pub active_theme: usize,

    /// Index of the theme used when the system appearance is dark.
    pub dark_theme: usize,

    /// Index of the theme used when the system appearance is light.
    pub light_theme: usize,

    /// Determines whether the application theme is manually controlled
    /// or synchronized with the system appearance.
    pub theme_mode: AppThemeMode,

    /// Enables Ctrl+R / Cmd+R keyboard shortcut support for triggering
    /// `App::reload`.
    ///
    /// Disabled by default. Applications must explicitly opt in.
    pub reload_shortcut: bool,

    /// Rounded-corner radius (logical px) drawn on the window itself when
    /// `decorations` is false - the OS no longer rounds the window, so
    /// xenframe punches the corners transparent in the wgpu surface instead.
    pub window_radius: f32,

    /// Soft drop shadow drawn behind the window when `decorations` is
    /// false. The window must be sized with `shadow.margin` logical px of
    /// extra transparent padding around the visual content for the blur
    /// to have room to render.
    pub window_shadow: Option<WindowShadow>,

    /// Border stroke (width, color) drawn around the window edge when
    /// `decorations` is false.
    pub window_border: Option<(f32, xengui::Color)>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            title: "XenGui App".to_string(),

            #[cfg(not(target_arch = "wasm32"))]
            width: 800,
            #[cfg(not(target_arch = "wasm32"))]
            height: 600,

            theme: None,

            #[cfg(not(target_arch = "wasm32"))]
            resizable: true,

            fullscreen: None,

            #[cfg(not(target_arch = "wasm32"))]
            position: WindowPosition::Center,

            #[cfg(not(target_arch = "wasm32"))]
            decorations: true,

            fonts: Vec::new(),

            themes: vec![Theme::light(), Theme::dark()],
            active_theme: 0,
            dark_theme: 1,
            light_theme: 0,
            theme_mode: AppThemeMode::System,

            reload_shortcut: false,

            window_radius: 0.0,
            window_shadow: None,
            window_border: None,
        }
    }
}
