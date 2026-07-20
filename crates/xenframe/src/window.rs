use winit::monitor::{ MonitorHandle, VideoModeHandle };

pub enum WindowPosition {
    Center,
    Default,
    Fixed(i32, i32),
}

pub enum Fullscreen {
    Exclusive(VideoModeHandle),
    /// Providing `None` to `Borderless` will fullscreen on the current monitor.
    Borderless(Option<MonitorHandle>),
}

pub fn system_theme(theme: Option<winit::window::Theme>) -> xengui::SystemTheme {
    match theme {
        Some(winit::window::Theme::Light) => xengui::SystemTheme::Light,
        _ => xengui::SystemTheme::Dark,
    }
}
