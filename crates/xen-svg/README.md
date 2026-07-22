# xen-svg

[<img alt="github" src="https://img.shields.io/badge/github-randseas/xengui-00aaaa?logo=github" height="20">](https://github.com/randseas/xengui/tree/main/crates/xen-svg)
[![Latest version](https://img.shields.io/crates/v/xen-svg.svg)](https://crates.io/crates/xen-svg)
[![Downloads](https://img.shields.io/crates/d/xen-svg.svg)](https://crates.io/crates/xen-svg)
[![Rust](https://img.shields.io/badge/rust-1.92%2B-blue.svg)](https://www.rust-lang.org)
[![Documentation](https://docs.rs/xen-svg/badge.svg)](https://docs.rs/xen-svg)
[![Apache-2.0](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](https://github.com/randseas/xengui/blob/main/crates/xen-svg/LICENSE)

A lightweight, platform-agnostic SVG parser and tessellation library for Rust.

`xen-svg` provides a small SVG document model, parsers for common SVG attributes (including `path`, `viewBox`, and `transform`), and a tessellator that converts vector graphics into triangles ready for rendering.

The library has **no rendering, GPU, or windowing dependencies**, making it suitable for GUI frameworks, game engines, embedded systems, and custom rendering pipelines.

## Features

- Parse SVG documents into a lightweight element tree
- SVG path (`d`) parser
- `viewBox` parsing
- `transform` parsing
- Triangle tessellation for rendering
- Lightweight SVG color model
- Renderer-agnostic design
- Cross-platform (Windows, Linux, macOS, WebAssembly)

## Example

```rust
use xen_svg::{parse_svg, tessellate_document};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let svg = r#"
        <svg viewBox="0 0 24 24">
            <path d="M12 2L2 22h20Z"/>
        </svg>
    "#;

    let document = parse_svg(svg)?;
    let triangles = tessellate_document(&document);

    println!("Generated {} triangles.", triangles.len());

    Ok(())
}
```

The generated triangle list can then be uploaded to any graphics API such as WGPU, OpenGL, Vulkan, DirectX, Metal, or a custom software renderer.

## Installation

`Cargo.toml`

```toml
[dependencies]
xen-svg = "0.1.1"
```

## License

Licensed under the Apache License 2.0.
