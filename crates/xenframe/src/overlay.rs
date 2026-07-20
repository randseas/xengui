#[cfg(target_arch = "wasm32")]
pub(crate) fn show_fatal_overlay(message: &str) {
    let Some(document) = web_sys::window().and_then(|w| w.document()) else {
        return;
    };
    let Some(body) = document.body() else {
        return;
    };
    let Ok(overlay) = document.create_element("pre") else {
        return;
    };
    let _ = overlay.set_attribute(
        "style",
        "position:fixed;inset:0;margin:0;background:#1a0000;color:#ff8080;\
         font:12px/1.5 monospace;padding:16px;white-space:pre-wrap;\
         z-index:2147483647;overflow:auto;"
    );
    overlay.set_text_content(Some(message));
    let _ = body.append_child(&overlay);
}
