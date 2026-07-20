// SPDX-License-Identifier: Apache-2.0

/// Converts winit's mouse button press/release state into xengui's own.
pub fn convert_element_state(state: winit::event::ElementState) -> xengui::ElementState {
    match state {
        winit::event::ElementState::Pressed => xengui::ElementState::Pressed,
        winit::event::ElementState::Released => xengui::ElementState::Released,
    }
}

/// Converts a winit mouse button into xengui's own.
pub fn convert_mouse_button(button: winit::event::MouseButton) -> xengui::MouseButton {
    match button {
        winit::event::MouseButton::Left => xengui::MouseButton::Left,
        winit::event::MouseButton::Right => xengui::MouseButton::Right,
        winit::event::MouseButton::Middle => xengui::MouseButton::Middle,
        winit::event::MouseButton::Back => xengui::MouseButton::Back,
        winit::event::MouseButton::Forward => xengui::MouseButton::Forward,
        winit::event::MouseButton::Other(code) => xengui::MouseButton::Other(code),
    }
}

/// Converts a winit scroll delta into xengui's own.
pub fn convert_scroll_delta(delta: winit::event::MouseScrollDelta) -> xengui::MouseScrollDelta {
    match delta {
        winit::event::MouseScrollDelta::LineDelta(x, y) =>
            xengui::MouseScrollDelta::LineDelta(x, y),
        winit::event::MouseScrollDelta::PixelDelta(pos) =>
            xengui::MouseScrollDelta::PixelDelta(pos.x, pos.y),
    }
}
