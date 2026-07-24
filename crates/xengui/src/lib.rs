// SPDX-License-Identifier: Apache-2.0
pub mod layout;
pub mod macros;
pub mod paint;
pub mod svg_compat;
pub mod style;
pub mod widgets;
pub mod input;
pub mod interaction;
pub mod text;
pub mod hooks;
pub mod widget;
pub mod widget_base;
pub mod animation;
pub mod constants;
pub mod dispatcher;
pub mod reconciler;
pub mod redraw;
pub mod types;

pub use layout::*;
pub use macros::WidgetContent;
pub use paint::*;
pub use style::*;
pub use style::system_theme::SystemTheme;
pub use widget::{ Widget, scaled_layout_box };
pub use widget_base::WidgetBase;
pub use animation::{
    AnimKey,
    AnimLayer,
    AnimProperty,
    AnimValue,
    AnimationManager,
    Easing,
    Transition,
    TransitionOverrides,
    TransitionProperty,
    animate_computed_style,
};
pub use input::*;
pub use interaction::*;
pub use text::*;
pub use hooks::{ component, use_state, ComponentId, ComponentKey, SetState };

pub use input::{
    InputEvent,
    Key,
    KeyState,
    find_widget_mut,
    any_wants_animation,
    dispatch_animation_tick,
};
pub use constants::*;
pub use dispatcher::Dispatcher;
pub use style::{
    current_theme,
    set_active_theme,
    set_active_theme_by_name,
    BoxShadow,
    Color,
    Cursor,
    IntoThemed,
    Length,
    Border,
    Edges,
    Style,
    StyleBuilder,
    Theme,
    ThemeMode,
    FlexDirection,
    FlexWrap,
    FontStyle,
    FontWeight,
    Overflow,
};
pub use widgets::{
    image_source_from_bytes,
    Button,
    IconPosition,
    ContextMenu,
    ContextMenuHandle,
    ContextMenuItem,
    Image,
    ImageSource,
    Label,
    ObjectFit,
    Svg,
    SvgCircleBuilder,
    SvgGroupBuilder,
    SvgLineBuilder,
    SvgPathBuilder,
    SvgRectBuilder,
    View,
    TextBox,
};
pub use redraw::RedrawRequester;
pub use xen_svg::{ SvgColor, SvgDocument, SvgElement, Transform2D };
pub use svg_compat::IntoSvgColor;
pub use types::*;

#[cfg(not(target_arch = "wasm32"))]
pub use widgets::image_source_from_path;
