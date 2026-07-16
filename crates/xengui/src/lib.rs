// SPDX-License-Identifier: Apache-2.0
pub mod app;
pub mod layout;
pub mod macros;
pub mod paint;
pub mod renderer;
pub mod style;
pub mod widgets;
pub mod input;
pub mod interaction;
pub mod text;
pub mod hooks;
pub mod widget;
mod reconciler;

pub use app::WindowPosition;
pub use layout::*;
pub use macros::WidgetContent;
pub use paint::*;
pub use renderer::XenRenderer;
pub use style::*;
pub use widget::Widget;
pub use input::*;
pub use interaction::*;
pub use text::*;
pub use hooks::{ component, use_state, ComponentId, ComponentKey, SetState };

pub use app::{ App, AppConfig };
pub use input::{ InputEvent, Key, KeyState };
pub use style::{
    Color,
    Length,
    Border,
    Edges,
    Style,
    StyleBuilder,
    FlexDirection,
    FlexWrap,
    FontStyle,
    FontWeight,
};
pub use widgets::{ image_source_from_bytes, Button, Image, ImageSource, Label, ObjectFit, View, TextBox };

#[cfg(not(target_arch = "wasm32"))]
pub use widgets::image_source_from_path;
