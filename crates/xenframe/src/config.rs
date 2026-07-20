use xengui::{ Theme };

#[cfg(not(target_arch = "wasm32"))]
use crate::WindowPosition;
use crate::window::Fullscreen;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum AppThemeMode {
    /// `active_theme` only changes via explicit calls to
    /// `set_active_theme`/`set_active_theme_by_name`.
    #[default]
    Fixed,
    /// `active_theme` automatically follows the OS light/dark appearance,
    /// switching between `dark_theme` and `light_theme`.
    System,
}

pub struct AppConfig {
    #[cfg(not(target_arch = "wasm32"))]
    pub title: String,

    #[cfg(not(target_arch = "wasm32"))]
    pub width: u32,
    #[cfg(not(target_arch = "wasm32"))]
    pub height: u32,

    pub theme: Option<winit::window::Theme>,

    #[cfg(not(target_arch = "wasm32"))]
    pub resizable: bool,

    pub fullscreen: Option<Fullscreen>,

    #[cfg(not(target_arch = "wasm32"))]
    pub position: WindowPosition,

    pub fonts: Vec<(String, Vec<u8>)>,

    /// Every theme registered for this app; switch between them at
    /// runtime with `set_active_theme`/`set_active_theme_by_name`.
    pub themes: Vec<Theme>,
    /// Index into `themes` currently in effect.
    pub active_theme: usize,
    /// Index into `themes` used when the resolved OS appearance is dark.
    pub dark_theme: usize,
    /// Index into `themes` used when the resolved OS appearance is light.
    pub light_theme: usize,
    /// Whether `active_theme` is picked manually or follows the OS appearance.
    pub theme_mode: AppThemeMode,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            #[cfg(not(target_arch = "wasm32"))]
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

            fonts: Vec::new(),

            themes: vec![Theme::light(), Theme::dark()],
            active_theme: 0,
            dark_theme: 1,
            light_theme: 0,
            theme_mode: AppThemeMode::System,
        }
    }
}
