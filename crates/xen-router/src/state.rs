// SPDX-License-Identifier: Apache-2.0
use std::cell::{ Cell, RefCell };
use xengui::hooks;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;

thread_local! {
    static CURRENT_PATH: RefCell<String> = RefCell::new(initial_path());
    static POPSTATE_INSTALLED: Cell<bool> = const { Cell::new(false) };
}

#[cfg(target_arch = "wasm32")]
fn initial_path() -> String {
    web_sys::window()
        .and_then(|w| w.location().pathname().ok())
        .unwrap_or_else(|| "/".to_string())
}

#[cfg(not(target_arch = "wasm32"))]
fn initial_path() -> String {
    "/".to_string()
}

pub fn current_path() -> String {
    ensure_popstate_listener();
    CURRENT_PATH.with(|p| p.borrow().clone())
}

/// Navigates to `path`, pushing a new browser history entry on wasm32.
pub fn navigate(path: impl Into<String>) {
    set_path(path.into(), true);
}

/// Like `navigate`, but replaces the current history entry instead of
/// pushing a new one - useful for redirects that shouldn't be reachable
/// via the back button.
pub fn replace(path: impl Into<String>) {
    set_path(path.into(), false);
}

fn set_path(path: String, _push: bool) {
    #[cfg(target_arch = "wasm32")]
    sync_browser_url(&path, _push);

    CURRENT_PATH.with(|p| {
        *p.borrow_mut() = path;
    });
    hooks::mark_dirty_and_redraw();
}

#[cfg(target_arch = "wasm32")]
fn sync_browser_url(path: &str, push: bool) {
    let Some(window) = web_sys::window() else {
        return;
    };
    let Ok(history) = window.history() else {
        return;
    };
    let _ = if push {
        history.push_state_with_url(&wasm_bindgen::JsValue::NULL, "", Some(path))
    } else {
        history.replace_state_with_url(&wasm_bindgen::JsValue::NULL, "", Some(path))
    };
}

// Installed lazily on first read instead of requiring a separate init
// call - simply depending on this crate and calling current_path()/
// navigate() is enough to get popstate sync on wasm32.
fn ensure_popstate_listener() {
    #[cfg(target_arch = "wasm32")]
    {
        if POPSTATE_INSTALLED.with(Cell::get) {
            return;
        }
        POPSTATE_INSTALLED.with(|f| f.set(true));

        let Some(window) = web_sys::window() else {
            return;
        };
        let window_for_closure = window.clone();

        let closure = wasm_bindgen::closure::Closure::<dyn FnMut(web_sys::Event)>::new(
            move |_event: web_sys::Event| {
                let path = window_for_closure
                    .location()
                    .pathname()
                    .unwrap_or_else(|_| "/".to_string());
                CURRENT_PATH.with(|p| {
                    *p.borrow_mut() = path;
                });
                hooks::mark_dirty_and_redraw();
            }
        );
        let _ = window.add_event_listener_with_callback(
            "popstate",
            closure.as_ref().unchecked_ref()
        );
        closure.forget();
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        POPSTATE_INSTALLED.with(|f| f.set(true));
    }
}