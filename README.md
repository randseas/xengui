# XenGui: an reactive GUI in pure Rust

[<img alt="github" src="https://img.shields.io/badge/github-randseas/xengui-00aaaa?logo=github" height="20">](https://github.com/randseas/xengui)
[![Apache-2.0](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](https://github.com/randseas/xengui/blob/main/LICENSE)
![WASM Library Check](https://github.com/randseas/xengui/actions/workflows/ci_check.yml/badge.svg)

<p align="start" style="margin-top: -.5rem">
  <a href="https://xengui.vercel.app">
    <img src="https://raw.githubusercontent.com/randseas/xengui/main/assets/XenGui_logo.svg" alt="XenGui logo" width="350"/>
  </a>
</p>

<div align="start" style="margin-top: -1.5rem;text-decoration: underline; text-decoration-color: #4daafc;">

### [Live web demo](https://xengui.vercel.app/demo)

</div>

---

XenGui (pronounced `/ˈzɛn.ɡuː.aɪ/` | `Zen-goo-eye`) is a reactive rendering GUI implementation in pure `Rust` utilizing the `wgpu` graphics API and `winit` window management. The system utilizes a strictly decoupled state-render pipeline bound by a virtual node (`VNode`) trait abstraction.

## Example

```rust
// Virtual nodes are mounted directly to the DOM tree
app.add_node(Box::new());
app.run()?;
```

## Installation

```toml
[dependencies]
xengui = "0.2.0"
```

## Sections:

- [Example](#example)
- [Quick start](#quickstart)
- [Demo](#demo)

## Inspiration

This project is heavily inspired by [egui](https://github.com/emilk/egui), a fantastic immediate mode GUI library. While XenGui is built from the ground up with its own architecture, the design philosophy and the ergonomics of `egui` served as a significant inspiration.

## License
Apache License 2.0 © 2026 randseas. See [LICENSE](LICENSE) for details.
