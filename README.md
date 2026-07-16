# XenGui: a retained-mode GUI library in pure Rust

[<img alt="github" src="https://img.shields.io/badge/github-randseas/xengui-00aaaa?logo=github" height="20">](https://github.com/randseas/xengui)
[![Latest version](https://img.shields.io/crates/v/xengui.svg)](https://crates.io/crates/xengui)
[![Downloads](https://img.shields.io/crates/d/xengui.svg)](https://crates.io/crates/xengui)
[![Rust](https://img.shields.io/badge/rust-1.92%2B-blue.svg)](https://www.rust-lang.org)
[![Documentation](https://docs.rs/xengui/badge.svg)](https://docs.rs/xengui)
[![Apache-2.0](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](https://github.com/randseas/xengui/blob/main/LICENSE)

<!-- ![CI/CD](https://github.com/randseas/xengui/actions/workflows/ci_check.yml/badge.svg) -->

<p align="start" style="margin-top: -.5rem">
  <a href="https://xengui.vercel.app">
    <img src="https://raw.githubusercontent.com/randseas/xengui/main/assets/XenGui_logo.svg" alt="XenGui logo" width="350"/>
  </a>
</p>

<div align="start" style="margin-top: -1.5rem; text-decoration: underline; text-decoration-color: #4daafc;">

### [Live web demo](https://xengui.vercel.app)

</div>

---

XenGui (pronounced `/ˈzɛn.ɡuː.aɪ/` | `Zen-goo-eye`) is a retained-mode rendering GUI implementation in pure **Rust**, built on the `wgpu` graphics API and `winit` window management. It combines a hooks-based retained-mode model with a Flexbox/Grid layout engine (powered by `taffy`) and a batched wgpu rendering pipeline, running natively on Windows, macOS, and Linux, as well as in the browser via WebAssembly.

> [!IMPORTANT]
> XenGui is currently an early development release. APIs are still evolving and may change without notice between versions. Use with caution in production projects and expect breaking changes until a stable `1.0.0` release.

## Features

- **Retained-mode state** - React-style hooks (`use_state`, `component`) drive re-renders without a virtual DOM diffing framework bolted on top.
- **Flexbox & Grid layout** - Layout system via [`taffy`](https://github.com/DioxusLabs/taffy), including flex direction, wrapping, alignment, gaps, and grid tracks.
- **GPU-accelerated rendering** - Rects, text, and images are batched and drawn through dedicated `wgpu` pipelines.
- **Declarative styling** - `Style`/`StyleBuilder` API covering colors (including OKLCH), borders, typography, spacing, and more.
- **Built-in widgets** - `View`, `Label`, `Button`, and `Image`, each with hover/pressed/disabled style variants.
- **Interaction system** - Unified handling of hover, click, focus, and keyboard events across widgets.
- **Cross-platform** - Native targets (Windows, macOS, Linux) and WebAssembly from a single codebase.

## Example

```rust
View::new()
    .display(Display::Flex)
    .flex_direction(FlexDirection::Column)
    .align_items(AlignItems::Center)
    .justify_content(JustifyContent::Center)
    .width(Length::Percent(100.0))
    .height(Length::Percent(100.0))
    .background(Color::WHITE)
    .child(
        Label::new()
            .label(format!("Count: {counter}"))
            .font_size(20)
            .color(Color::NEUTRAL_700)
        )
    .child(
        Button::new()
            .label("Increment")
            .padding(Edges::symmetric(12, 8))
            .background(Color::NEUTRAL_200)
            .on_click(move |_ctx| set_counter.update(|v| *v += 1))
    );
```

## Installation

`Cargo.toml`

```toml
[dependencies]
xengui = "0.2.4"
```

## Sections

- [Features](#features)
- [Example](#example)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [Demo](#demo)

## Quick Start

```rust
use xengui::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = AppConfig {
        title: "My XenGui App".into(),
        width: 640,
        height: 480,
        position: WindowPosition::Center,
        ..Default::default()
    };

    let mut app = App::new(config);

    app.render(|| {
        let (counter, set_counter) = use_state::<i32>(0);

        Box::new(
            View::new()
                .display(Display::Flex)
                .flex_direction(FlexDirection::Column)
                .align_items(AlignItems::Center)
                .justify_content(JustifyContent::Center)
                .width(Length::Percent(100.0))
                .height(Length::Percent(100.0))
                .background(Color::WHITE)
                .child(
                    Label::new()
                        .label(format!("Count: {counter}"))
                        .font_size(20)
                        .color(Color::NEUTRAL_700)
                )
                .child(
                    Button::new()
                        .label("Increment")
                        .padding(Edges::symmetric(12, 8))
                        .background(Color::NEUTRAL_200)
                        .on_click(move |_ctx| set_counter.update(|v| *v += 1))
                )
        )
    });

    if let Err(e) = app.run() {
        eprintln!("Error running app: {:?}", e);
    }

    Ok(())
}
```

Run it with:

```bash
cargo run
```

For WebAssembly targets, build and serve with [Trunk](https://github.com/trunk-rs/trunk):

```bash
trunk serve
```

## Demo

Demo: [https://xengui.vercel.app](https://xengui.vercel.app)

## License

Apache License 2.0 © 2026 randseas. See [LICENSE](LICENSE) for details.
