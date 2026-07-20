use xengui::XenRenderer;

pub enum XenEvent {
    RendererReady(Box<XenRenderer>),
    CancelSelection,
    SystemThemeChanged(winit::window::Theme),
    NativeInputChanged(String),
}
