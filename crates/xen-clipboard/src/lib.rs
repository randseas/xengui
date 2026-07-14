// SPDX-License-Identifier: Apache-2.0
mod clipboard;
mod error;
mod platform;

pub use clipboard::Clipboard;
pub use error::ClipboardError;

#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "android")]
mod android;

#[cfg(target_os = "ios")]
mod ios;

#[cfg(target_arch = "wasm32")]
mod wasm;

pub(crate) use clipboard::ClipboardBackend;
