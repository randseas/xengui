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
