# XenGui: a reactive GUI library in pure Rust

[<img alt="github" src="https://img.shields.io/badge/github-randseas/xengui-00aaaa?logo=github" height="20">](https://github.com/randseas/xengui)
[![Latest version](https://img.shields.io/crates/v/xengui.svg)](https://crates.io/crates/xengui)
[![Apache-2.0](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](https://github.com/randseas/xengui/blob/main/LICENSE)
![CI/CD](https://github.com/randseas/xengui/actions/workflows/ci_check.yml/badge.svg)

<p align="start" style="margin-top: -.5rem">
  <a href="https://xengui.vercel.app">
    <img src="https://raw.githubusercontent.com/randseas/xengui/main/assets/XenGui_logo.svg" alt="XenGui logo" width="350"/>
  </a>
</p>

<div align="start" style="margin-top: -1.5rem; text-decoration: underline; text-decoration-color: #4daafc;">

### [Live web demo](https://xengui.vercel.app)

</div>

---

XenGui (pronounced `/ˈzɛn.ɡuː.aɪ/` | `Zen-goo-eye`) is a reactive rendering GUI implementation in pure **Rust** utilizing the `wgpu` graphics API and `winit` window management. The system utilizes a strictly decoupled state-render pipeline bound by a virtual node (`VNode`) trait abstraction.

## Example

```rust
app.add_node(Box::new(
    Text::new("title")
        .text("XenGui")
        .font_size(24)
        .text_color(Color::TEAL),
    ));

app.add_node(Box::new(
    Text::new("text2")
        .text("Hello, world!")
        .font_size(20.0)
        .text_color(Color::WHITE),
    ));

app.add_node(Box::new(
    Text::new("text3")
        .text(format!("Platform: {PLATFORM}"))
        .font_size(20.0)
        .text_color(Color::LIGHT_GRAY),
    ));
```

## Installation

`Cargo.toml`

```toml
[dependencies]
xengui = "0.2.0"
```

## Sections:

- [Example](#example)
- [Demo](#demo)
- [Quick start](#quickstart)
- [Documentation](#documentation)

## Quick Start

Quick start write here.

## Demo

Demo is available at: [https://xengui.vercel.app](https://xengui.vercel.app)

## Documentation

Docs are available at: [https://xengui.vercel.app/docs](https://xengui.vercel.app/docs)

## Inspiration

This project is inspired by [Dear ImGui](https://github.com//imgui) and [egui](https://github.com/emilk/egui).

## License

Apache License 2.0 © 2026 randseas. See [LICENSE](LICENSE) for details.
