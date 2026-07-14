use wasm_bindgen_futures::{ spawn_local, JsFuture };

use web_sys::window;

use crate::clipboard::{ ClipboardBackend, ClipboardError };

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
                    let text = value.as_string();
                    callback(Ok(text));
                }
                Err(_) => {
                    callback(Err(ClipboardError::ReadFailed));
                }
            }
        });
    }

    fn set_text(&self, text: &str) -> Result<(), ClipboardError> {
        let Some(window) = window() else {
            return Err(ClipboardError::Unsupported);
        };

        let clipboard = window.navigator().clipboard();
        let text = text.to_owned();

        spawn_local(async move {
            if let Err(err) = JsFuture::from(clipboard.write_text(&text)).await {
                log::error!("clipboard write failed: {:?}", err);
            }
        });

        Ok(())
    }

    fn has_text(&self, callback: Box<dyn FnOnce(Result<bool, ClipboardError>) + Send>) {
        self.get_text(
            Box::new(move |result| {
                callback(result.map(|text| text.is_some()));
            })
        );
    }
}
