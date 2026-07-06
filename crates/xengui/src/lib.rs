// SPDX-License-Identifier: Apache-2.0
// crates/xengui/src/lib.rs
pub mod app;
pub mod components;
pub mod context;
pub mod core;
pub mod renderer;
pub mod paint;
pub mod style;

pub use app::App;
pub use app::AppConfig;
pub use app::WindowPosition;
pub use components::text::Text;
pub use context::RenderContext;
pub use core::VNode;
pub use renderer::XenRenderer;
pub use style::{
    Color,
    Edges,
    Length,
    Style,
};
pub use paint::*;