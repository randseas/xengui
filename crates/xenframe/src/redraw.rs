// SPDX-License-Identifier: Apache-2.0
use std::sync::Arc;
use winit::window::Window;
use xengui::RedrawRequester;

pub struct WinitRedraw(pub Arc<Window>);

impl RedrawRequester for WinitRedraw {
    fn request_redraw(&self) {
        self.0.request_redraw();
    }
}
