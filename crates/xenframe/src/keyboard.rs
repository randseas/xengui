// SPDX-License-Identifier: Apache-2.0
use xengui::{ ImeEvent, Key, KeyState, KeyboardEvent };

/// Converts a winit key event into xengui's platform-agnostic representation.
pub fn convert_keyboard_event(event: winit::event::KeyEvent) -> KeyboardEvent {
    use winit::keyboard::{ KeyCode, PhysicalKey, Key as WinitKey };

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
        PhysicalKey::Code(KeyCode::F14) => Key::F14,
        PhysicalKey::Code(KeyCode::F15) => Key::F15,
        PhysicalKey::Code(KeyCode::F16) => Key::F16,
        PhysicalKey::Code(KeyCode::F17) => Key::F17,
        PhysicalKey::Code(KeyCode::F18) => Key::F18,
        PhysicalKey::Code(KeyCode::F19) => Key::F19,
        PhysicalKey::Code(KeyCode::F20) => Key::F20,
        PhysicalKey::Code(KeyCode::F21) => Key::F21,
        PhysicalKey::Code(KeyCode::F22) => Key::F22,
        PhysicalKey::Code(KeyCode::F23) => Key::F23,
        PhysicalKey::Code(KeyCode::F24) => Key::F24,
        PhysicalKey::Code(KeyCode::F25) => Key::F25,
        PhysicalKey::Code(KeyCode::F26) => Key::F26,
        PhysicalKey::Code(KeyCode::F27) => Key::F27,
        PhysicalKey::Code(KeyCode::F28) => Key::F28,
        PhysicalKey::Code(KeyCode::F29) => Key::F29,
        PhysicalKey::Code(KeyCode::F30) => Key::F30,
        PhysicalKey::Code(KeyCode::F31) => Key::F31,
        PhysicalKey::Code(KeyCode::F32) => Key::F32,
        PhysicalKey::Code(KeyCode::F33) => Key::F33,
        PhysicalKey::Code(KeyCode::F34) => Key::F34,
        PhysicalKey::Code(KeyCode::F35) => Key::F35,

        PhysicalKey::Code(KeyCode::Pause) => Key::Pause,
        PhysicalKey::Code(KeyCode::PrintScreen) => Key::PrintScreen,
        PhysicalKey::Code(KeyCode::Delete) => Key::Delete,
        PhysicalKey::Code(KeyCode::Insert) => Key::Insert,

        PhysicalKey::Code(KeyCode::Home) => Key::Home,
        PhysicalKey::Code(KeyCode::End) => Key::End,
        PhysicalKey::Code(KeyCode::PageUp) => Key::PageUp,
        PhysicalKey::Code(KeyCode::PageDown) => Key::PageDown,

        PhysicalKey::Code(KeyCode::Backspace) => Key::Backspace,
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
        // logical_key stays correct even when ctrl is held, unlike `text`
        _ =>
            match &event.logical_key {
                WinitKey::Character(s) =>
                    s.chars().next().map(Key::Character).unwrap_or(Key::Unknown),
                _ =>
                    event.text
                        .as_ref()
                        .and_then(|s| s.chars().next())
                        .map(Key::Character)
                        .unwrap_or(Key::Unknown),
            }
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

/// Converts winit's IME composition event into xengui's own representation.
pub fn convert_ime_event(ime: winit::event::Ime) -> ImeEvent {
    match ime {
        winit::event::Ime::Enabled => ImeEvent::Enabled,
        winit::event::Ime::Preedit(text, range) => ImeEvent::Preedit(text, range),
        winit::event::Ime::Commit(text) => ImeEvent::Commit(text),
        winit::event::Ime::Disabled => ImeEvent::Disabled,
    }
}
