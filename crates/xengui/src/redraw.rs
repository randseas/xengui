// SPDX-License-Identifier: Apache-2.0

/// Lets `hooks::SetState` ask the host to schedule a repaint without
/// xengui depending on any concrete windowing crate (winit, Bevy, etc.).
pub trait RedrawRequester {
    fn request_redraw(&self);
}
