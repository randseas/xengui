// SPDX-License-Identifier: Apache-2.0
use crate::Widget;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyState {
    Pressed,
    Released,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Key {
    Escape,

    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    F13,
    F14,
    F15,
    F16,
    F17,
    F18,
    F19,
    F20,
    F21,
    F22,
    F23,
    F24,
    F25,
    F26,
    F27,
    F28,
    F29,
    F30,
    F31,
    F32,
    F33,
    F34,
    F35,

    Pause,
    PrintScreen,
    Delete,
    Insert,

    Home,
    End,
    PageUp,
    PageDown,

    Backspace,
    NumLock,
    ScrollLock,

    Tab,
    CapsLock,
    Enter,

    ShiftLeft,
    ShiftRight,

    ControlLeft,
    ControlRight,

    Fn,
    SuperLeft,
    SuperRight,
    AltLeft,
    Space,
    AltRight,
    ContextMenu,

    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,

    Character(char),

    Unknown,
}

#[derive(Clone, Debug)]
pub struct KeyboardEvent {
    pub key: Key,
    pub state: KeyState,
    pub repeat: bool,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ModifiersState {
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
    pub super_key: bool,
}

#[derive(Clone, Debug)]
pub enum InputEvent {
    MouseMoved {
        position: (f32, f32),
    },
    MouseEntered,
    MouseExited,
    MouseInput {
        state: winit::event::ElementState,
        button: winit::event::MouseButton,
        position: (f32, f32),
    },
    MouseWheel {
        delta: winit::event::MouseScrollDelta,
        position: (f32, f32),
        modifiers: ModifiersState,
    },
    KeyInput {
        event: KeyboardEvent,
        modifiers: ModifiersState,
    },
    ModifiersChanged(ModifiersState),
    Ime(winit::event::Ime),
    FocusGained {
        via_keyboard: bool,
    },
    FocusLost,
    BlinkTick,
    AnimationTick {
        dt: f32,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EventStatus {
    Ignored,
    Handled,
}

#[derive(Default)]
pub struct EventCtx {
    redraw_requested: bool,
    cursor_icon: Option<winit::window::CursorIcon>,
    focus_requested: bool,
    focus_released: bool,
    pub(crate) focus_target: Option<String>,
    pub(crate) clear_focus: bool,
}

impl EventCtx {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn request_redraw(&mut self) {
        self.redraw_requested = true;
    }

    pub fn redraw_requested(&self) -> bool {
        self.redraw_requested
    }

    pub fn set_cursor_icon(&mut self, icon: winit::window::CursorIcon) {
        self.cursor_icon = Some(icon);
    }

    pub fn take_cursor_icon(&mut self) -> Option<winit::window::CursorIcon> {
        self.cursor_icon.take()
    }

    pub fn request_focus(&mut self) {
        self.focus_requested = true;
    }

    pub fn release_focus(&mut self) {
        self.focus_released = true;
    }

    fn take_focus_request(&mut self) -> bool {
        std::mem::take(&mut self.focus_requested)
    }

    fn take_release_focus_request(&mut self) -> bool {
        std::mem::take(&mut self.focus_released)
    }
}

pub fn convert_keyboard_event(event: winit::event::KeyEvent) -> KeyboardEvent {
    use winit::keyboard::{ KeyCode, PhysicalKey };

    let key = match event.physical_key {
        PhysicalKey::Code(KeyCode::Escape) => Key::Escape,

        PhysicalKey::Code(KeyCode::F1) => Key::F1,
        PhysicalKey::Code(KeyCode::F2) => Key::F2,
        PhysicalKey::Code(KeyCode::F3) => Key::F3,
        PhysicalKey::Code(KeyCode::F4) => Key::F4,
        PhysicalKey::Code(KeyCode::F5) => Key::F5,
        PhysicalKey::Code(KeyCode::F6) => Key::F6,
        PhysicalKey::Code(KeyCode::F7) => Key::F7,
        PhysicalKey::Code(KeyCode::F8) => Key::F8,
        PhysicalKey::Code(KeyCode::F9) => Key::F9,
        PhysicalKey::Code(KeyCode::F10) => Key::F10,
        PhysicalKey::Code(KeyCode::F11) => Key::F11,
        PhysicalKey::Code(KeyCode::F12) => Key::F12,
        PhysicalKey::Code(KeyCode::F13) => Key::F13,
        PhysicalKey::Code(KeyCode::F14) => Key::F35,
        PhysicalKey::Code(KeyCode::F15) => Key::F35,
        PhysicalKey::Code(KeyCode::F16) => Key::F35,
        PhysicalKey::Code(KeyCode::F17) => Key::F35,
        PhysicalKey::Code(KeyCode::F18) => Key::F35,
        PhysicalKey::Code(KeyCode::F19) => Key::F35,
        PhysicalKey::Code(KeyCode::F20) => Key::F35,
        PhysicalKey::Code(KeyCode::F21) => Key::F35,
        PhysicalKey::Code(KeyCode::F22) => Key::F35,
        PhysicalKey::Code(KeyCode::F23) => Key::F35,
        PhysicalKey::Code(KeyCode::F24) => Key::F35,
        PhysicalKey::Code(KeyCode::F25) => Key::F35,
        PhysicalKey::Code(KeyCode::F26) => Key::F35,
        PhysicalKey::Code(KeyCode::F27) => Key::F35,
        PhysicalKey::Code(KeyCode::F28) => Key::F35,
        PhysicalKey::Code(KeyCode::F29) => Key::F35,
        PhysicalKey::Code(KeyCode::F30) => Key::F35,
        PhysicalKey::Code(KeyCode::F31) => Key::F35,
        PhysicalKey::Code(KeyCode::F32) => Key::F35,
        PhysicalKey::Code(KeyCode::F33) => Key::F35,
        PhysicalKey::Code(KeyCode::F34) => Key::F35,
        PhysicalKey::Code(KeyCode::F35) => Key::F35,

        PhysicalKey::Code(KeyCode::Pause) => Key::Pause,
        PhysicalKey::Code(KeyCode::PrintScreen) => Key::PrintScreen,
        PhysicalKey::Code(KeyCode::Delete) => Key::Delete,
        PhysicalKey::Code(KeyCode::Insert) => Key::Insert,

        PhysicalKey::Code(KeyCode::Home) => Key::Home,
        PhysicalKey::Code(KeyCode::End) => Key::End,
        PhysicalKey::Code(KeyCode::PageUp) => Key::PageUp,
        PhysicalKey::Code(KeyCode::PageDown) => Key::PageDown,

        PhysicalKey::Code(KeyCode::Backspace) => Key::PageDown,
        PhysicalKey::Code(KeyCode::NumLock) => Key::NumLock,
        PhysicalKey::Code(KeyCode::ScrollLock) => Key::ScrollLock,

        PhysicalKey::Code(KeyCode::Tab) => Key::Tab,
        PhysicalKey::Code(KeyCode::CapsLock) => Key::CapsLock,
        PhysicalKey::Code(KeyCode::Enter) => Key::Enter,

        PhysicalKey::Code(KeyCode::ShiftLeft) => Key::ShiftLeft,
        PhysicalKey::Code(KeyCode::ShiftRight) => Key::ShiftRight,

        PhysicalKey::Code(KeyCode::ControlLeft) => Key::ControlLeft,
        PhysicalKey::Code(KeyCode::ControlRight) => Key::ControlRight,
        PhysicalKey::Code(KeyCode::Fn) => Key::Fn,
        PhysicalKey::Code(KeyCode::SuperLeft) => Key::SuperLeft,
        PhysicalKey::Code(KeyCode::SuperRight) => Key::SuperRight,
        PhysicalKey::Code(KeyCode::AltLeft) => Key::AltLeft,
        PhysicalKey::Code(KeyCode::AltRight) => Key::AltRight,
        PhysicalKey::Code(KeyCode::Space) => Key::Space,
        PhysicalKey::Code(KeyCode::ContextMenu) => Key::ContextMenu,

        PhysicalKey::Code(KeyCode::ArrowUp) => Key::ArrowUp,
        PhysicalKey::Code(KeyCode::ArrowDown) => Key::ArrowDown,
        PhysicalKey::Code(KeyCode::ArrowLeft) => Key::ArrowLeft,
        PhysicalKey::Code(KeyCode::ArrowRight) => Key::ArrowRight,

        // layout-aware fallback: uses os generated text
        _ =>
            event.text
                .as_ref()
                .and_then(|s| s.chars().next())
                .map(Key::Character)
                .unwrap_or(Key::Unknown),
    };

    KeyboardEvent {
        key,
        state: match event.state {
            winit::event::ElementState::Pressed => KeyState::Pressed,
            winit::event::ElementState::Released => KeyState::Released,
        },
        repeat: event.repeat,
    }
}

fn ancestor_paths(path: &str) -> Vec<String> {
    let parts: Vec<&str> = path.split('.').collect();
    (1..=parts.len()).map(|n| parts[..n].join(".")).collect()
}

pub fn path_segment(widget: &dyn Widget, index: usize) -> String {
    match widget.get_key() {
        Some(key) => format!("k{key}"),
        None => index.to_string(),
    }
}

fn resolve_segment<'a>(
    siblings: &'a mut [Box<dyn Widget>],
    segment: &str
) -> Option<&'a mut dyn Widget> {
    if let Some(key) = segment.strip_prefix('k') {
        siblings
            .iter_mut()
            .find(|w| w.get_key().is_some_and(|k| k.as_str() == key))
            .map(|w| w.as_mut())
    } else {
        let idx: usize = segment.parse().ok()?;
        siblings.get_mut(idx).map(|w| w.as_mut())
    }
}

pub fn find_widget_mut<'a>(
    tree: &'a mut [Box<dyn Widget>],
    path: &str
) -> Option<&'a mut dyn Widget> {
    let mut parts = path.split('.');
    let mut current: &mut dyn Widget = resolve_segment(tree, parts.next()?)?;

    for part in parts {
        let children = current.children_mut()?;
        current = resolve_segment(children, part)?;
    }

    Some(current)
}

pub fn hit_test_path(tree: &[Box<dyn Widget>], point: (f32, f32)) -> Option<String> {
    for (i, node) in tree.iter().enumerate().rev() {
        let segment = path_segment(node.as_ref(), i);
        if let Some(path) = hit_test_recursive(node.as_ref(), &segment, point) {
            return Some(path);
        }
    }
    None
}

fn hit_test_recursive(widget: &dyn Widget, path: &str, point: (f32, f32)) -> Option<String> {
    if !widget.hit_test(point) {
        return None;
    }

    for (i, child) in widget.children().iter().enumerate().rev() {
        let segment = path_segment(child.as_ref(), i);
        let child_path = format!("{path}.{segment}");
        if let Some(hit) = hit_test_recursive(child.as_ref(), &child_path, point) {
            return Some(hit);
        }
    }

    Some(path.to_string())
}

/// Tab / Shift+Tab sırasını oluşturmak için, ağaçtaki tüm aktif ve
/// focusable widget'ların path'lerini derinlik-öncelikli sırayla toplar.
pub fn collect_focusable_paths(tree: &[Box<dyn Widget>]) -> Vec<String> {
    let mut paths = Vec::new();
    for (i, node) in tree.iter().enumerate() {
        let segment = path_segment(node.as_ref(), i);
        collect_focusable_recursive(node.as_ref(), &segment, &mut paths);
    }
    paths
}

fn collect_focusable_recursive(widget: &dyn Widget, path: &str, out: &mut Vec<String>) {
    if widget.interaction().is_some_and(|i| i.focusable && i.enabled) {
        out.push(path.to_string());
    }

    for (i, child) in widget.children().iter().enumerate() {
        let segment = path_segment(child.as_ref(), i);
        let child_path = format!("{path}.{segment}");
        collect_focusable_recursive(child.as_ref(), &child_path, out);
    }
}

// True if `path` is `ancestor` itself or one of its descendants.
pub fn path_is_within(path: &str, ancestor: &str) -> bool {
    path == ancestor || path.starts_with(&format!("{ancestor}."))
}

pub fn dispatch_positional(
    tree: &mut [Box<dyn Widget>],
    leaf_path: &str,
    event: &InputEvent,
    ctx: &mut EventCtx
) -> EventStatus {
    for path in ancestor_paths(leaf_path).into_iter().rev() {
        let Some(widget) = find_widget_mut(tree, &path) else {
            continue;
        };

        let status = widget.event(event, ctx);

        if ctx.take_focus_request() {
            ctx.focus_target = Some(path.clone());
        }
        if ctx.take_release_focus_request() {
            ctx.clear_focus = true;
        }

        if status == EventStatus::Handled {
            return EventStatus::Handled;
        }
    }
    EventStatus::Ignored
}

pub fn dispatch_to_path(
    tree: &mut [Box<dyn Widget>],
    path: &str,
    event: &InputEvent,
    ctx: &mut EventCtx
) -> EventStatus {
    match find_widget_mut(tree, path) {
        Some(widget) => widget.event(event, ctx),
        None => EventStatus::Ignored,
    }
}

pub fn any_wants_animation(tree: &[Box<dyn Widget>]) -> bool {
    tree.iter().any(|w| widget_wants_animation_recursive(w.as_ref()))
}

fn widget_wants_animation_recursive(widget: &dyn Widget) -> bool {
    if widget.wants_animation_frame() {
        return true;
    }
    widget
        .children()
        .iter()
        .any(|c| widget_wants_animation_recursive(c.as_ref()))
}

pub fn dispatch_animation_tick(tree: &mut [Box<dyn Widget>], dt: f32, ctx: &mut EventCtx) {
    for widget in tree.iter_mut() {
        dispatch_animation_tick_recursive(widget.as_mut(), dt, ctx);
    }
}

fn dispatch_animation_tick_recursive(widget: &mut dyn Widget, dt: f32, ctx: &mut EventCtx) {
    if widget.wants_animation_frame() {
        widget.event(&(InputEvent::AnimationTick { dt }), ctx);
    }
    if let Some(children) = widget.children_mut() {
        for child in children.iter_mut() {
            dispatch_animation_tick_recursive(child.as_mut(), dt, ctx);
        }
    }
}

#[derive(Default)]
pub struct InputState {
    pub cursor_pos: Option<(f32, f32)>,
    pub hovered_path: Option<String>,
    pub pressed_path: Option<String>,
    pub focused_path: Option<String>,
    pub modifiers: ModifiersState,
}
