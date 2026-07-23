// SPDX-License-Identifier: Apache-2.0
#![cfg(target_arch = "wasm32")]

//! Browser-specific bootstrap helpers for running a xenframe `App` in wasm.
//! Call `init_panic_hook` (and optionally `init_logger`) once, before
//! creating the `App` - typically from a `#[wasm_bindgen(start)]` function
//! in the host crate.

/// Forwards Rust panics to `console.error` with a full stack trace instead
/// of the browser's default opaque "unreachable executed" message. Safe to
/// call more than once; only the first call has any effect.
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}

/// Routes the `log` crate's output to the browser console at `level`. Safe
/// to call more than once.
pub fn init_logger(level: log::LevelFilter) {
    let _ = console_log::init_with_level(level.to_level().unwrap_or(log::Level::Info));
}

/// Reads the current viewport size in CSS pixels, preferring
/// `VisualViewport` (matches what `handler.rs` uses for canvas sync) and
/// falling back to `window.inner_width/height` on browsers without it.
pub fn viewport_size() -> Option<(f64, f64)> {
    let window = web_sys::window()?;

    if let Some(vv) = window.visual_viewport() {
        return Some((vv.width(), vv.height()));
    }

    let w = window.inner_width().ok()?.as_f64()?;
    let h = window.inner_height().ok()?.as_f64()?;
    Some((w, h))
}
