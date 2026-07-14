// SPDX-License-Identifier: Apache-2.0

use crate::{ platform, ClipboardError };

/// Cross-platform clipboard.
///
/// # Example
///
/// ```no_run
/// use xen_clipboard::Clipboard;
///
/// let clipboard = Clipboard::new();
///
/// clipboard.set_text("Hello, World!").unwrap();
///
/// clipboard.get_text(|result| {
///     println!("{result:?}");
/// });
/// ```
#[must_use]
pub struct Clipboard {
    backend: Box<dyn ClipboardBackend>,
}

impl Default for Clipboard {
    fn default() -> Self {
        Self::new()
    }
}

impl Clipboard {
    #[inline]
    pub fn new() -> Self {
        Self {
            backend: platform::create_backend(),
        }
    }

    /// Reads text from the clipboard.
    pub fn get_text<F>(&self, callback: F)
        where F: FnOnce(Result<Option<String>, ClipboardError>) + Send + 'static
    {
        self.backend.get_text(Box::new(callback));
    }

    /// Writes text to the clipboard.
    pub fn set_text(
        &self,
        text: String,
        callback: impl FnOnce(Result<(), ClipboardError>) + Send + 'static
    ) {
        self.backend.set_text(text, Box::new(callback));
    }

    /// Returns whether the clipboard currently contains text.
    pub fn has_text<F>(&self, callback: F)
        where F: FnOnce(Result<bool, ClipboardError>) + Send + 'static
    {
        self.backend.has_text(Box::new(callback));
    }
}

/// Platform clipboard backend.
pub(crate) trait ClipboardBackend {
    fn get_text(&self, callback: Box<dyn FnOnce(Result<Option<String>, ClipboardError>) + Send>);

    fn set_text(&self, text: String, callback: Box<dyn FnOnce(Result<(), ClipboardError>) + Send>);

    fn has_text(&self, callback: Box<dyn FnOnce(Result<bool, ClipboardError>) + Send>);
}
