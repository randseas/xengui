# xen-animation

[<img alt="github" src="https://img.shields.io/badge/github-randseas/xengui-00aaaa?logo=github" height="20">](https://github.com/randseas/xengui/tree/main/crates/xen-animation)
[![Latest version](https://img.shields.io/crates/v/xen_animation.svg)](https://crates.io/crates/xen_animation)
[![Downloads](https://img.shields.io/crates/d/xen_animation.svg)](https://crates.io/crates/xen_animation)
[![Rust](https://img.shields.io/badge/rust-1.92%2B-blue.svg)](https://www.rust-lang.org)
[![Documentation](https://docs.rs/xen-animation/badge.svg)](https://docs.rs/xen-animation)
[![Apache-2.0](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](https://github.com/randseas/xengui/blob/main/crates/xen-animation/LICENSE)

A high-performance, framework-agnostic animation library for Rust.

`xen-animation` provides a centralized animation system with CSS-compatible easing functions, transitions, interpolation, and frame-based animation management. It is designed to be reusable across GUI frameworks, game engines, and other applications.

## Features

- CSS-compatible cubic-bezier easing
- Built-in easing presets (`linear`, `ease-in`, `ease-out`, `ease-in-out`)
- Custom cubic-bezier curves
- Centralized `AnimationManager`
- Generic interpolation traits
- Multiple concurrent animations
- Duration, delay, repeat, reverse, and looping
- Zero rendering dependencies

## Example

```rust
use xen_animation::{AnimationManager, Easing};

// Example coming soon.
```

## Installation

`Cargo.toml`

```toml
[dependencies]
xen-animation = "0.1.0"
```

## License

Apache License 2.0 © 2026 randseas. See [LICENSE](LICENSE) for details.