// SPDX-License-Identifier: Apache-2.0
//! Event & Input altyapısı.
//!
//! Mimari özet:
//! - Hit-testing, `renderer.rs::paint_recursive` ile AYNI path konvansiyonunu
//!   ("0.1.2") kullanır. Bu sayede `RenderCache` ve input dispatch aynı
//!   "adres uzayını" paylaşır.
//! - Pozisyonel event'ler (mouse) DOM'daki gibi "bubble" edilir: en derindeki
//!   (topmost) widget'tan başlayıp köke doğru sırayla `Widget::event()`
//!   çağrılır; bir widget `EventStatus::Handled` dönerse üst widget'lara
//!   propagate edilmez.
//! - Klavye/IME event'leri hit-test değil, `focused_path` üzerinden dispatch
//!   edilir.
//! - Focus/cursor-icon/redraw talepleri widget'tan `EventCtx` üzerinden
//!   toplanır; gerçek state mutasyonu (App::input) dispatch bittikten SONRA
//!   çağıran taraf (App) içinde uygulanır — widget'lar App'i bilmez.

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

/// Bir widget ağacı üzerinde gezinen pozisyonel/klavye olayları.
///
/// Winit'in `ElementState`, `MouseButton`, `KeyEvent`, `ModifiersState`,
/// `MouseScrollDelta` ve `Ime` tipleri doğrudan kullanılıyor; platform
/// event'lerini yeniden modellemenin bu aşamada katma değeri yok.
#[derive(Clone, Debug)]
pub enum InputEvent {
    MouseMoved {
        position: (f32, f32),
    },
    /// Sadece hit-test edilen widget'a (bubble YOK) gönderilir.
    MouseEntered,
    /// Sadece hit-test edilen widget'a (bubble YOK) gönderilir.
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
    /// Sadece hit-test edilen widget'a (bubble YOK) gönderilir.
    FocusGained,
    /// Sadece hit-test edilen widget'a (bubble YOK) gönderilir.
    FocusLost,
}

/// `Widget::event()` bir event'i tükettiyse `Handled`, aksi halde `Ignored`
/// döner. `Handled`, bubble zincirinin orada durması demektir (DOM'daki
/// `stopPropagation` gibi).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EventStatus {
    Ignored,
    Handled,
}

/// `Widget::event()` çağrısı sırasında widget'ın dış dünyaya (App) ilettiği
/// talepler. Widget kendi path'ini bilmediği için focus talebi path bazlı
/// değil, "bu çağrıda beni focusla" bayrağı olarak modellenir; path'i
/// dispatcher (bkz. `dispatch_positional`) doldurur.
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

    /// Bu frame'in sonunda yeniden çizim tetiklenmesini ister (ör. hover
    /// rengi değişti).
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

    /// Bu çağrıyı alan widget klavye focus'unu talep eder (ör. TextInput
    /// tıklandığında).
    pub fn request_focus(&mut self) {
        self.focus_requested = true;
    }

    /// Bu çağrıyı alan widget focus'u bırakır (ör. Escape'e basıldı).
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
    use winit::keyboard::{KeyCode, PhysicalKey};

    let key = match event.physical_key {
        PhysicalKey::Code(KeyCode::Tab) => Key::Tab,
        PhysicalKey::Code(KeyCode::Enter) => Key::Enter,
        PhysicalKey::Code(KeyCode::Space) => Key::Space,
        PhysicalKey::Code(KeyCode::Escape) => Key::Escape,

        PhysicalKey::Code(KeyCode::ArrowUp) => Key::ArrowUp,
        PhysicalKey::Code(KeyCode::ArrowDown) => Key::ArrowDown,
        PhysicalKey::Code(KeyCode::ArrowLeft) => Key::ArrowLeft,
        PhysicalKey::Code(KeyCode::ArrowRight) => Key::ArrowRight,

        _ => Key::Unknown,
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

/// Verilen `path`'in ("0.1.2") kökten başlayarak tüm ata path'lerini
/// döner: `["0", "0.1", "0.1.2"]`.
fn ancestor_paths(path: &str) -> Vec<String> {
    let parts: Vec<&str> = path.split('.').collect();
    (1..=parts.len()).map(|n| parts[..n].join(".")).collect()
}

/// `path` ile adreslenen widget'a mutable erişim sağlar. Path,
/// `renderer.rs::paint_recursive`'in ürettiği ile aynı formattadır.
pub fn find_widget_mut<'a>(
    tree: &'a mut [Box<dyn Widget>],
    path: &str,
) -> Option<&'a mut dyn Widget> {
    let mut parts = path.split('.');
    let root_idx: usize = parts.next()?.parse().ok()?;
    let mut current: &mut dyn Widget = tree.get_mut(root_idx)?.as_mut();

    for part in parts {
        let idx: usize = part.parse().ok()?;
        let children = current.children_mut()?;
        current = children.get_mut(idx)?.as_mut();
    }

    Some(current)
}

/// Verilen ekran koordinatındaki EN ÜSTTEKİ (topmost) widget'ın path'ini
/// bulur. Painter's-algorithm gereği son çizilen widget en üsttedir, bu
/// yüzden hem root listesi hem de her seviyedeki children TERS sırada
/// gezilir (`renderer.rs::paint_recursive`'in çizim sırasının tersi).
pub fn hit_test_path(tree: &[Box<dyn Widget>], point: (f32, f32)) -> Option<String> {
    for (i, node) in tree.iter().enumerate().rev() {
        if let Some(path) = hit_test_recursive(node.as_ref(), &i.to_string(), point) {
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
        let child_path = format!("{path}.{i}");
        if let Some(hit) = hit_test_recursive(child.as_ref(), &child_path, point) {
            return Some(hit);
        }
    }

    // Hiçbir çocuk isabet etmedi ama kendisi isabet etti -> kendisi topmost.
    Some(path.to_string())
}

/// Pozisyonel bir event'i `leaf_path`'ten köke doğru "bubble" ederek
/// dispatch eder. İlk `Handled` dönen widget'ta durur.
///
/// Ayrıca, dispatch sırasında widget'ın `ctx.request_focus()` /
/// `ctx.release_focus()` çağırıp çağırmadığını kontrol edip
/// `ctx.focus_target` / `ctx.clear_focus` alanlarını doldurur — App bu
/// alanları okuyup gerçek focus state'ini günceller.
pub fn dispatch_positional(
    tree: &mut [Box<dyn Widget>],
    leaf_path: &str,
    event: &InputEvent,
    ctx: &mut EventCtx,
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

/// Bubble YAPMADAN tek bir widget'a event gönderir. `MouseEntered`,
/// `MouseExited`, `FocusGained`, `FocusLost` gibi "bu widget'a özel" event'ler
/// için kullanılır.
pub fn dispatch_to_path(
    tree: &mut [Box<dyn Widget>],
    path: &str,
    event: &InputEvent,
    ctx: &mut EventCtx,
) -> EventStatus {
    match find_widget_mut(tree, path) {
        Some(widget) => widget.event(event, ctx),
        None => EventStatus::Ignored,
    }
}

/// `App` tarafından tutulan, frame'ler arası kalıcı input state'i: imleç
/// pozisyonu, hover/pressed/focus path'leri, modifier tuşları.
#[derive(Default)]
pub struct InputState {
    pub cursor_pos: Option<(f32, f32)>,
    pub hovered_path: Option<String>,
    pub pressed_path: Option<String>,
    pub focused_path: Option<String>,
    pub modifiers: ModifiersState,
}
