use crate::ClipboardError;
use super::ClipboardBackend;

pub struct LinuxClipboard;

impl LinuxClipboard {
    #[inline]
    pub fn new() -> Self {
        Self
    }
}

impl ClipboardBackend for LinuxClipboard {
    fn get_text(&self, callback: Box<dyn FnOnce(Result<Option<String>, ClipboardError>) + Send>) {
        callback(Err(ClipboardError::Unsupported));
    }

    fn set_text(&self, _text: &str, callback: Box<dyn FnOnce(Result<(), ClipboardError>) + Send>) {
        callback(Err(ClipboardError::Unsupported));
    }

    fn has_text(&self, callback: Box<dyn FnOnce(Result<bool, ClipboardError>) + Send>) {
        callback(Ok(false));
    }
}
