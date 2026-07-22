# xengui-wgpu

[<img alt="github" src="https://img.shields.io/badge/github-randseas/xengui-00aaaa?logo=github" height="20">](https://github.com/randseas/xengui/tree/main/crates/xengui-wgpu)
[![Latest version](https://img.shields.io/crates/v/xengui-wgpu.svg)](https://crates.io/crates/xengui-wgpu)
[![Downloads](https://img.shields.io/crates/d/xengui-wgpu.svg)](https://crates.io/crates/xengui-wgpu)
[![Rust](https://img.shields.io/badge/rust-1.92%2B-blue.svg)](https://www.rust-lang.org)
[![Documentation](https://docs.rs/xengui-wgpu/badge.svg)](https://docs.rs/xengui-wgpu)
[![Apache-2.0](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](https://github.com/randseas/xengui/blob/main/crates/xengui-wgpu/LICENSE)

The [`wgpu`](https://github.com/gfx-rs/wgpu) render backend for XenGui.

`xengui-wgpu` implements `xengui`'s `RenderBackend` trait on top of `wgpu`, so `xengui`'s core (layout, widgets, reconciler, `FrameRenderer`) never depends on a concrete graphics API. It owns dedicated batched pipelines for rects (with SDF-based rounding, borders, and anti-aliasing), triangles, images, and text (via [`glyphon`](https://github.com/grovesNL/glyphon)), and interleaves draw calls in true paint order across all four.

`WgpuWindowRenderer` owns a full wgpu device/surface for native windowed apps and is the integration point used by `xenframe`. A host that already owns its own wgpu device and render target (e.g. a game engine's render graph) can instead build `WgpuPipelines` once and call `begin_frame` directly against its own encoder and view.

## Features

- Full `RenderBackend` implementation for `xengui`
- Batched, scissor-clipped pipelines for rects, triangles, images, and text
- SDF-based rounded-rect rendering with borders and anti-aliasing
- Text shaping and glyph atlas management via `glyphon`
- Shape caching to avoid re-shaping unchanged text runs every frame
- User-supplied font loading, with WASM fallback-font support
- Native (Windows, macOS, Linux) and WebAssembly targets

> For a full application quick start (rendering, event loop, and GPU backend together), see the [workspace README](https://github.com/randseas/xengui#quick-start).

## Example

```rust
use std::sync::Arc;
use xengui_wgpu::WgpuWindowRenderer;

let renderer = WgpuWindowRenderer::new(window, width, height, user_fonts)?;

// Each frame:
renderer.render_frame(&mut widget_tree, theme, scale_factor);
```

## Installation

`Cargo.toml`

```toml
[dependencies]
xengui-wgpu = "0.1.0"
```

## Documentation

Docs are available at: [https://xengui.vercel.app/docs/xengui-wgpu](https://xengui.vercel.app/docs/xengui-wgpu)

## License

Apache License 2.0 © 2026 randseas. See [LICENSE](LICENSE) for details.
