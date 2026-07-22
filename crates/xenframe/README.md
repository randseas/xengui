# xenframe

[<img alt="github" src="https://img.shields.io/badge/github-randseas/xengui-00aaaa?logo=github" height="20">](https://github.com/randseas/xengui/tree/main/crates/xenframe)
[![Latest version](https://img.shields.io/crates/v/xenframe.svg)](https://crates.io/crates/xenframe)
[![Downloads](https://img.shields.io/crates/d/xenframe.svg)](https://crates.io/crates/xenframe)
[![Rust](https://img.shields.io/badge/rust-1.92%2B-blue.svg)](https://www.rust-lang.org)
[![Documentation](https://docs.rs/xenframe/badge.svg)](https://docs.rs/xenframe)
[![Apache-2.0](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](https://github.com/randseas/xengui/blob/main/crates/xenframe/LICENSE)

The platform runtime for XenGui: window creation, the event loop, and input integration.

`xenframe` wraps [`winit`](https://github.com/rust-windowing/winit) and wires it up to `xengui`, translating OS/browser events (mouse, keyboard, IME, touch, focus, theme changes) into `xengui`'s own platform-agnostic `InputEvent` type, and driving `xengui-wgpu`'s renderer every frame. On WebAssembly targets it additionally manages canvas resizing and a hidden native `<input>` element used to bring up the on-screen keyboard on mobile browsers. `xengui` itself stays platform-agnostic; `xenframe` is what turns it into a runnable native or web application.

## Features

- Window creation and configuration (title, size, position, fullscreen, resizability)
- Full `winit` event loop integration, including an interruptible reconciliation pump
- Mouse, keyboard, touch, and IME event translation
- Multi-click / word-select / long-press gesture handling
- Clipboard integration via `xen-clipboard`
- Cursor icon mapping
- System and app theme (light/dark/auto) synchronization
- Blinking caret scheduling via `ControlFlow::WaitUntil`, with zero CPU cost when nothing is focused
- WebAssembly support: canvas resize animation, hidden mobile text input, and browser event bridging

> For a full application quick start (rendering, event loop, and GPU backend together), see the [workspace README](https://github.com/randseas/xengui#quick-start).

## Example

```rust
use xengui::*;
use xenframe::{App, AppConfig, WindowPosition};

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
        Box::new(
            View::new()
                .width(Length::Percent(100.0))
                .height(Length::Percent(100.0))
                .background(Color::WHITE)
        )
    });

    app.run()?;

    Ok(())
}
```

## Installation

`Cargo.toml`

```toml
[dependencies]
xenframe = "0.1.0"
```

## Documentation

Docs are available at: [https://xengui.vercel.app/docs/xenframe](https://xengui.vercel.app/docs/xenframe)

## License

Apache License 2.0 © 2026 randseas. See [LICENSE](LICENSE) for details.
