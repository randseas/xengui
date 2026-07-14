# xen-clipboard

[<img alt="github" src="https://img.shields.io/badge/github-randseas/xengui-00aaaa?logo=github" height="20">](https://github.com/randseas/xengui/tree/main/crates/xen-clipboard)
[![Latest version](https://img.shields.io/crates/v/xen_clipboard.svg)](https://crates.io/crates/xen_clipboard)
[![Downloads](https://img.shields.io/crates/d/xen_clipboard.svg)](https://crates.io/crates/xen_clipboard)
[![Rust](https://img.shields.io/badge/rust-1.92%2B-blue.svg)](https://www.rust-lang.org)
[![Documentation](https://docs.rs/xen-clipboard/badge.svg)](https://docs.rs/xen-clipboard)
[![Apache-2.0](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](https://github.com/randseas/xengui/blob/main/crates/xen-clipboard/LICENSE)

Cross-platform clipboard library in rust.

## Supported Platforms

- Windows
- WebAssembly

## Planned Platforms

- Linux (Wayland/X11)
- macOS
- Android
- iOS

## Example

```rust
use xen_clipboard::Clipboard;

fn main() {
    let clipboard = Clipboard::new();

    // Write text to the clipboard.
    clipboard.set_text("Hello, Xen Clipboard!").unwrap();

    // Read text from the clipboard.
    clipboard.get_text(|result| match result {
        Ok(Some(text)) => println!("Clipboard: {text}"),
        Ok(None) => println!("Clipboard is empty."),
        Err(err) => eprintln!("Failed to read clipboard: {err}"),
    });

    // Check whether the clipboard contains text.
    clipboard.has_text(|result| match result {
        Ok(has_text) => println!("Has text: {has_text}"),
        Err(err) => eprintln!("Failed to query clipboard: {err}"),
    });
}
```

## Installation

`Cargo.toml`

```toml
[dependencies]
xen-clipboard = "0.1.3-alpha.1"
```

## Documentation

Docs are available at: [https://xengui.vercel.app/docs/xen-clipboard](https://xengui.vercel.app/docs/xen-clipboard)

## License

Apache License 2.0 © 2026 randseas. See [LICENSE](LICENSE) for details.
