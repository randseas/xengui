// SPDX-License-Identifier: Apache-2.0
use crate::Widget;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyState {
    Pressed,
    Released,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Key {
    Tab,
    Enter,
    Space,
    Escape,

    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,

    Backspace,
    Delete,

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
    },
    KeyInput {
        event: KeyboardEvent,
        modifiers: ModifiersState,
    },
    ModifiersChanged(ModifiersState),
    Ime(winit::event::Ime),
    FocusGained,
    FocusLost,
    BlinkTick,
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
        PhysicalKey::Code(KeyCode::Tab) => Key::Tab,
        PhysicalKey::Code(KeyCode::Enter) => Key::Enter,
        PhysicalKey::Code(KeyCode::Space) => Key::Space,
        PhysicalKey::Code(KeyCode::Escape) => Key::Escape,

        PhysicalKey::Code(KeyCode::ArrowUp) => Key::ArrowUp,
        PhysicalKey::Code(KeyCode::ArrowDown) => Key::ArrowDown,
        PhysicalKey::Code(KeyCode::ArrowLeft) => Key::ArrowLeft,
        PhysicalKey::Code(KeyCode::ArrowRight) => Key::ArrowRight,

        PhysicalKey::Code(KeyCode::Backspace) => Key::Backspace,
        PhysicalKey::Code(KeyCode::Delete) => Key::Delete,

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

#[derive(Default)]
pub struct InputState {
    pub cursor_pos: Option<(f32, f32)>,
    pub hovered_path: Option<String>,
    pub pressed_path: Option<String>,
    pub focused_path: Option<String>,
    pub modifiers: ModifiersState,
}
