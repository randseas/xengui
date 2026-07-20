// SPDX-License-Identifier: Apache-2.0
pub mod layout;
pub mod macros;
pub mod paint;
pub mod svg_compat;
pub mod renderer;
pub mod style;
pub mod widgets;
pub mod input;
pub mod interaction;
pub mod text;
pub mod hooks;
pub mod widget;
pub mod animation;
pub mod constants;
pub mod reconciler;

pub use layout::*;
pub use macros::WidgetContent;
pub use paint::*;
pub use renderer::XenRenderer;
pub use style::*;
pub use widget::{ Widget, scaled_layout_box };
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

pub use app::{ App, AppConfig, AppThemeMode };
pub use input::{
    InputEvent,
    Key,
    KeyState,
    find_widget_mut,
    any_wants_animation,
    dispatch_animation_tick,
};
pub use constants::*;
pub use style::{
    current_theme,
    set_active_theme,
    set_active_theme_by_name,
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
pub use xen_svg::{ SvgColor, SvgDocument, SvgElement, Transform2D };
pub use svg_compat::IntoSvgColor;

#[cfg(not(target_arch = "wasm32"))]
pub use widgets::image_source_from_path;
