// SPDX-License-Identifier: Apache-2.0
use std::cell::{ Cell, RefCell };
use std::sync::Arc;
use winit::window::Window;

thread_local! {
    static ACTIVE_WINDOW: RefCell<Option<Arc<Window>>> = const { RefCell::new(None) };
    static CLOSE_REQUESTED: Cell<bool> = const { Cell::new(false) };
}

pub(crate) fn set_active_window(window: Arc<Window>) {
    ACTIVE_WINDOW.with(|cell| {
        *cell.borrow_mut() = Some(window);
    });
}

fn with_window(f: impl FnOnce(&Window)) {
    ACTIVE_WINDOW.with(|cell| {
        if let Some(window) = cell.borrow().as_ref() {
            f(window);
        }
    });
}

/// Moves the OS window under the cursor, as if its native titlebar had
/// been dragged. Intended to be called from a widget's own drag handling;
/// prefer `StyleBuilder::window_drag_region` for a custom titlebar instead
/// of calling this directly.
pub fn drag_window() {
    with_window(|w| {
        let _ = w.drag_window();
    });
}

pub fn minimize_window() {
    with_window(|w| w.set_minimized(true));
}

pub fn toggle_maximize_window() {
    with_window(|w| w.set_maximized(!w.is_maximized()));
}

pub fn is_window_maximized() -> bool {
    let mut result = false;
    with_window(|w| {
        result = w.is_maximized();
    });
    result
}

/// Requests the application to close. Actual shutdown happens on the next
/// `about_to_wait` pass, since a real `event_loop.exit()` needs the active
/// event loop, which isn't available from arbitrary widget callbacks.
pub fn close_window() {
    CLOSE_REQUESTED.with(|c| c.set(true));
}

pub(crate) fn take_close_requested() -> bool {
    CLOSE_REQUESTED.with(|c| c.replace(false))
}
