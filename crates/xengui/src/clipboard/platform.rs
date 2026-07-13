use super::ClipboardBackend;

#[cfg(target_os = "windows")]
pub fn create_backend() -> Box<dyn ClipboardBackend> {
    Box::new(super::windows::WindowsClipboard::new())
}

#[cfg(target_os = "linux")]
pub fn create_backend() -> Box<dyn ClipboardBackend> {
    Box::new(super::linux::LinuxClipboard::new())
}

#[cfg(target_os = "macos")]
pub fn create_backend() -> Box<dyn ClipboardBackend> {
    Box::new(super::macos::MacClipboard::new())
}

#[cfg(target_os = "android")]
pub fn create_backend() -> Box<dyn ClipboardBackend> {
    Box::new(super::android::AndroidClipboard::new())
}

#[cfg(target_os = "ios")]
pub fn create_backend() -> Box<dyn ClipboardBackend> {
    Box::new(super::ios::IosClipboard::new())
}

#[cfg(target_arch = "wasm32")]
pub fn create_backend() -> Box<dyn ClipboardBackend> {
    Box::new(super::wasm::WasmClipboard::new())
}
