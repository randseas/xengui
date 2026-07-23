// SPDX-License-Identifier: Apache-2.0
//! wgpu render backend for xengui. Implements `xengui::RenderBackend` so
//! xengui's core stays free of any GPU-API dependency.
//!
//! `WgpuWindowRenderer` owns a wgpu device/surface for native windowed
//! apps (used by `xenframe`). A host that already owns its own wgpu
//! device and render target (e.g. a Bevy render node) should build
//! `WgpuPipelines` once and call `begin_frame` directly instead.

mod pipelines;
mod backend;
mod window_renderer;
mod window_chrome;

pub use backend::{ WgpuFrame, WgpuPipelines };
pub use window_renderer::WgpuWindowRenderer;
pub use window_chrome::{ WindowChrome, WindowShadow };