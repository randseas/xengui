# xengui

[<img alt="github" src="https://img.shields.io/badge/github-randseas/xengui-00aaaa?logo=github" height="20">](https://github.com/randseas/xengui/tree/main/crates/xengui)
[![Latest version](https://img.shields.io/crates/v/xengui.svg)](https://crates.io/crates/xengui)
[![Downloads](https://img.shields.io/crates/d/xengui.svg)](https://crates.io/crates/xengui)
[![Rust](https://img.shields.io/badge/rust-1.92%2B-blue.svg)](https://www.rust-lang.org)
[![Documentation](https://docs.rs/xengui/badge.svg)](https://docs.rs/xengui)
[![Apache-2.0](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](https://github.com/randseas/xengui/blob/main/crates/xengui/LICENSE)

The core, retained-mode GUI library at the heart of the XenGui ecosystem.

`xengui` provides the widget tree, hooks-based state model, Flexbox/Grid layout engine (via `taffy`), style system, text handling, and interruptible reconciler. It has no GPU or windowing dependencies of its own - rendering is delegated to a [`RenderBackend`] implementation (such as `xengui-wgpu`), and window/event integration is handled by a platform crate (such as `xenframe`), so `xengui` stays fully portable across native and WebAssembly targets.

## Features

- **Retained-mode state** - React-style hooks (`use_state`, `component`) drive re-renders without a virtual-DOM diffing framework bolted on top.
- **Interruptible reconciler** - tree diffing runs as explicit, resumable work units instead of a single blocking pass.
- **Flexbox & Grid layout** - layout system via [`taffy`](https://github.com/DioxusLabs/taffy), including flex direction, wrapping, alignment, gaps, and grid tracks.
- **Declarative styling** - `Style`/`StyleBuilder` API covering colors (including OKLCH), borders, typography, spacing, transitions, and more.
- **Built-in widgets** - `View`, `Label`, `Button`, `Link`, `TextBox`, `Image`, `Svg`, and `ContextMenu`, each with hover/pressed/focus/disabled style variants.
- **Interaction system** - unified handling of hover, click, focus, and keyboard events across widgets.
- **Backend-agnostic rendering** - implement [`RenderBackend`] to target any graphics API.
- **Cross-platform** - native targets (Windows, macOS, Linux) and WebAssembly from a single codebase.

> For a full application quick start (rendering, event loop, and GPU backend together), see the [workspace README](https://github.com/randseas/xengui#quick-start).

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
xengui = "0.2.6"
```

Most applications should also pull in a platform runtime crate (e.g. `xenframe`) and a render backend (e.g. `xengui-wgpu`) - see the [workspace README](https://github.com/randseas/xengui) for a full quick-start example.

## Documentation

Docs are available at: [https://xengui.vercel.app/docs/xengui](https://xengui.vercel.app/docs/xengui)

## License

Apache License 2.0 © 2026 randseas. See [LICENSE](LICENSE) for details.
