use xengui_wgpu::WgpuWindowRenderer;

pub enum XenEvent {
    RendererReady(Box<WgpuWindowRenderer>),
    CancelSelection,
    SystemThemeChanged(winit::window::Theme),
    NativeInputChanged(String),
}
