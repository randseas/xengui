// SPDX-License-Identifier: Apache-2.0

//! Platform runtime for xengui: window creation, the winit event loop,
//! clipboard/cursor/IME integration, and (on wasm) canvas + hidden-input
//! handling. `xengui` itself is platform-agnostic; everything here
//! translates OS/browser events into `xengui::InputEvent` and drives
//! `xengui::XenRenderer`.

pub mod app;
pub mod window;
pub mod handler;

pub mod cursor;
pub mod keyboard;
pub mod mouse;
pub mod text_agent;

pub mod config;
pub mod event;
pub mod redraw;

pub mod window_controls;

#[cfg(target_os = "windows")]
pub mod win32_chrome;

#[cfg(target_arch = "wasm32")]
pub mod web;
pub mod overlay;

pub use app::{ App, request_reload };
pub use config::*;
pub use window::WindowPosition;

pub use window_controls::{
    close_window,
    drag_window,
    is_window_maximized,
    minimize_window,
    toggle_maximize_window,
};
