# xen-animation

[<img alt="github" src="https://img.shields.io/badge/github-randseas/xengui-00aaaa?logo=github" height="20">](https://github.com/randseas/xengui/tree/main/crates/xen-animation)
[![Latest version](https://img.shields.io/crates/v/xen-animation.svg)](https://crates.io/crates/xen-animation)
[![Downloads](https://img.shields.io/crates/d/xen-animation.svg)](https://crates.io/crates/xen-animation)
[![Rust](https://img.shields.io/badge/rust-1.92%2B-blue.svg)](https://www.rust-lang.org)
[![Documentation](https://docs.rs/xen-animation/badge.svg)](https://docs.rs/xen-animation)
[![Apache-2.0](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](https://github.com/randseas/xengui/blob/main/crates/xen-animation/LICENSE)

A high-performance, framework-agnostic animation and transition library for Rust.

`xen-animation` provides a centralized animation system with CSS-compatible easing functions, transitions, interpolation, and frame-based animation management. It is designed to be reusable across GUI frameworks, game engines, real-time applications, and custom renderers.

Callers never own a timer themselves - they only report their current target value every frame through [`AnimationManager::set_target`], and the manager owns the entire lifecycle: starting, easing, retargeting mid-flight, and settling once the transition finishes.

## Features

- CSS-compatible cubic-bezier easing
- Built-in easing presets (`linear`, `ease-in`, `ease-out`, `ease-in-out`)
- Custom cubic-bezier curves
- Centralized `AnimationManager`, generic over any hashable key type
- Multiple concurrent, independently keyed animations
- Duration, delay, and per-property transition overrides
- Zero rendering dependencies

## Example

```rust
use std::time::Duration;
use xen_animation::{AnimationManager, AnimValue, Easing, Transition};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum AnimKey {
    Opacity,
}

fn main() {
    let mut manager: AnimationManager<AnimKey> = AnimationManager::new();
    let transition = Transition::new(Duration::from_millis(300)).easing(Easing::EaseOut);

    // Start animating opacity towards 1.0 using the given transition.
    manager.set_target(AnimKey::Opacity, AnimValue([1.0, 0.0, 0.0, 0.0]), Some(transition));

    // A typical render loop: advance the manager once per frame, then read
    // back the (possibly mid-transition) current value.
    for _ in 0..5 {
        manager.tick(Duration::from_millis(16));

        if let Some(value) = manager.value(AnimKey::Opacity) {
            println!("opacity = {:.3}", value.0[0]);
        }
    }

    println!("still animating: {}", manager.is_animating());
}
```

## Installation

`Cargo.toml`

```toml
[dependencies]
xen-animation = "0.1.2"
```

## License

Apache License 2.0 © 2026 randseas. See [LICENSE](LICENSE) for details.
