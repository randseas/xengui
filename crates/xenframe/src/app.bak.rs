// SPDX-License-Identifier: Apache-2.0
use crate::{
    EventCtx,
    InputEvent,
    InputState,
    Key,
    KeyState,
    ModifiersState,
    Theme,
    Widget,
    XenRenderer,
    any_wants_animation,
    collect_focusable_paths,
    convert_keyboard_event,
    dispatch_animation_tick,
    dispatch_positional,
    dispatch_to_path,
    find_widget_mut,
    hit_test_path,
    hooks::set_redraw_handle,
    path_is_within,
    TOUCH_LONG_PRESS_DURATION,
    TOUCH_LONG_PRESS_MOVE_TOLERANCE_DP,
};
use std::sync::Arc;
use web_time::Instant;

#[cfg(not(target_arch = "wasm32"))]
use winit::dpi::PhysicalPosition;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;

use winit::{
    event::WindowEvent,
    event_loop::{ ActiveEventLoop, ControlFlow, EventLoop },
    monitor::{ MonitorHandle, VideoModeHandle },
    window::{ Window, WindowAttributes, WindowId },
};
