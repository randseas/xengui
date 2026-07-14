use wasm_bindgen_futures::{ spawn_local, JsFuture };
use web_sys::window;

use crate::{ ClipboardBackend, ClipboardError };

pub struct WasmClipboard;

impl WasmClipboard {
    #[inline]
    pub fn new() -> Self {
        Self
    }
}

impl ClipboardBackend for WasmClipboard {
    fn get_text(&self, callback: Box<dyn FnOnce(Result<Option<String>, ClipboardError>) + Send>) {
        let Some(window) = window() else {
            callback(Err(ClipboardError::Unsupported));
            return;
        };

        let clipboard = window.navigator().clipboard();

        spawn_local(async move {
            match JsFuture::from(clipboard.read_text()).await {
                Ok(value) => {
                    callback(Ok(value.as_string()));
                }

                Err(err) => {
                    callback(Err(ClipboardError::PlatformError(format!("{err:?}"))));
                }
            }
        });
    }

    fn set_text(&self, text: &str, callback: Box<dyn FnOnce(Result<(), ClipboardError>) + Send>) {
        let Some(window) = window() else {
            callback(Err(ClipboardError::Unsupported));
            return;
        };

        let clipboard = window.navigator().clipboard();
        let text = text.to_owned();

        spawn_local(async move {
            match JsFuture::from(clipboard.write_text(&text)).await {
                Ok(_) => {
                    callback(Ok(()));
                }

                Err(err) => {
                    callback(Err(ClipboardError::PlatformError(format!("{err:?}"))));
                }
            }
        });
    }

    fn has_text(&self, callback: Box<dyn FnOnce(Result<bool, ClipboardError>) + Send>) {
        self.get_text(
            Box::new(move |result| {
                callback(result.map(|text| text.is_some()));
            })
        );
    }
}
