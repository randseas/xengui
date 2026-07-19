# xengui-icons

[<img alt="github" src="https://img.shields.io/badge/github-randseas/xengui-00aaaa?logo=github" height="20">](https://github.com/randseas/xengui/tree/main/crates/xengui-icons)
[![Latest version](https://img.shields.io/crates/v/xengui-icons.svg)](https://crates.io/crates/xengui-icons)
[![Downloads](https://img.shields.io/crates/d/xengui-icons.svg)](https://crates.io/crates/xengui-icons)
[![Rust](https://img.shields.io/badge/rust-1.92%2B-blue.svg)](https://www.rust-lang.org)
[![Documentation](https://docs.rs/xengui-icons/badge.svg)](https://docs.rs/xengui-icons)
[![Apache-2.0](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](https://github.com/randseas/xengui/blob/main/crates/xengui-icons/LICENSE)

A lightweight collection of SVG icons for the XenGui ecosystem.

`xengui-icons` provides a curated set of vector icons that integrate seamlessly with XenGui's `Icon` and `Svg` widgets. Icons are embedded at compile time, require no external assets, and can be styled dynamically using your application's theme.

The crate is framework-friendly and simply exposes SVG data, making it usable anywhere an SVG string is accepted.

## Features

- Compile-time embedded SVG icons
- Based on the Lucide icon set
- Zero runtime asset loading
- Lightweight and dependency-free
- Works with `Icon` and `Svg` widgets
- Theme-aware coloring and sizing
- Cross-platform (Windows, Linux, macOS, WebAssembly)

## Example

```rust
use xengui::prelude::*;
use xengui_icons::Lucide;

fn ui() -> impl Widget {
    Icon::new(Lucide::Play)
        .size(24)
        .color(Color::BLUE_500)
}
```

Or use the raw SVG directly:

```rust
use xengui::prelude::*;
use xengui_icons::Lucide;

Svg::new()
    .from_string(Lucide::Play)
    .width(24)
    .height(24);
```

## Installation

`Cargo.toml`

```toml
[dependencies]
xengui-icons = "0.1.0"
```

## License

The library is licensed under the Apache License 2.0.

The bundled icon set is based on the [Lucide](https://lucide.dev) project and is distributed under the ISC License. See the repository for attribution and license details.
