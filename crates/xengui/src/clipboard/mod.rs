mod platform;

#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "android")]
mod android;

#[cfg(target_os = "ios")]
mod ios;

#[cfg(target_arch = "wasm32")]
mod wasm;

#[cfg(
    not(
        any(
            target_os = "windows",
            target_os = "linux",
            target_os = "macos",
            target_os = "android",
            target_os = "ios",
            target_arch = "wasm32"
        )
    )
)]
compile_error!("[clipboard]: unsupported platform.");

#[derive(Debug, Clone)]
pub enum ClipboardError {
    OpenFailed,
    FormatUnavailable,
    ReadFailed,
    WriteFailed,
    AllocationFailed,
    LockFailed,
    Unsupported,
    PermissionDenied,
    PlatformError(String),
}

#[must_use]
pub struct Clipboard {
    backend: Box<dyn ClipboardBackend>,
}

impl Default for Clipboard {
    fn default() -> Self {
        Self {
            backend: platform::create_backend(),
        }
    }
}

impl Clipboard {
    pub fn get_text<F>(&self, callback: F)
        where F: FnOnce(Result<Option<String>, ClipboardError>) + Send + 'static
    {
        self.backend.get_text(Box::new(callback));
    }

    pub fn set_text(&self, text: &str) {
        self.backend.set_text(text);
    }

    pub fn has_text<F>(&self, callback: F)
        where F: FnOnce(Result<bool, ClipboardError>) + Send + 'static
    {
        self.backend.has_text(Box::new(callback));
    }
}

/// Trait that all platforms apply
pub(crate) trait ClipboardBackend: Send + Sync {
    fn get_text(&self, callback: Box<dyn FnOnce(Result<Option<String>, ClipboardError>) + Send>);

    fn set_text(&self, text: &str);

    fn has_text(&self, callback: Box<dyn FnOnce(Result<bool, ClipboardError>) + Send>);
}
