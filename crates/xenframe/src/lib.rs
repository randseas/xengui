// SPDX-License-Identifier: Apache-2.0

//! Platform runtime for xengui: window creation, the winit event loop,
//! clipboard/cursor/IME integration, and (on wasm) canvas + hidden-input
//! handling. `xengui` itself is platform-agnostic; everything here
//! translates OS/browser events into `xengui::InputEvent` and drives
//! `xengui::XenRenderer`.

pub mod app;
pub mod window;
pub mod handler;

// pub mod renderer;
// pub mod input;
// pub mod clipboard;
pub mod cursor;
pub mod keyboard;
pub mod mouse;
pub mod text_agent;
// pub mod focus;
// pub mod ime;
// pub mod platform;
pub mod config;
pub mod event;
pub mod redraw;

#[cfg(target_arch = "wasm32")]
pub mod web;
pub mod overlay;

pub use app::{ App };
pub use config::*;
pub use window::WindowPosition;
