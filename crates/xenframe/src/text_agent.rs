// SPDX-License-Identifier: Apache-2.0
#![cfg(target_arch = "wasm32")]

use wasm_bindgen::JsCast;
use xengui::find_widget_mut;

use crate::{ App, event::XenEvent };

impl App {
    pub(crate) fn create_native_input(&mut self) {
        if
            let Some(document) = web_sys::window().and_then(|w| w.document()) &&
            let Some(body) = document.body() &&
            let Ok(input) = document.create_element("input") &&
            let Ok(input) = input.dyn_into::<web_sys::HtmlInputElement>()
        {
            let _ = body.append_child(&input);
            let _ = input.set_attribute("type", "text");
            let _ = input.set_attribute("autocomplete", "off");
            let _ = input.set_attribute("autocorrect", "off");
            let _ = input.set_attribute("autocapitalize", "off");
            let _ = input.set_attribute("spellcheck", "false");

            let style = input.style();
            let _ = style.set_property("position", "fixed");
            let _ = style.set_property("top", "0");
            let _ = style.set_property("left", "0");
            let _ = style.set_property("opacity", "0");
            let _ = style.set_property("border", "none");
            let _ = style.set_property("outline", "none");
            let _ = style.set_property("width", "1px");
            let _ = style.set_property("height", "1px");
            let _ = style.set_property("font-size", "16px");
            let _ = style.set_property("z-index", "-1");
            let _ = style.set_property("pointer-events", "none");
            let _ = style.set_property("background-color", "transparent");
            let _ = style.set_property("caret-color", "transparent");

            if let Some(proxy) = &self.event_proxy {
                let proxy_clone = proxy.clone();
                let input_clone = input.clone();
                let closure = wasm_bindgen::closure::Closure::<dyn FnMut()>::new(move || {
                    let _ = proxy_clone.send_event(
                        XenEvent::NativeInputChanged(input_clone.value())
                    );
                });
                let _ = input.add_event_listener_with_callback(
                    "input",
                    closure.as_ref().unchecked_ref()
                );
                closure.forget();
            }

            self.native_input = Some(input);
        }
    }

    pub(crate) fn sync_native_input(&mut self, path: &str, focus: bool) {
        let Some(input) = &self.native_input else {
            return;
        };
        let Some(widget) = find_widget_mut(&mut self.root, path) else {
            self.hide_native_input();
            return;
        };
        if widget.native_text_input().is_none() {
            self.hide_native_input();
            return;
        }

        widget.sync_native_input(input);

        if focus {
            let _ = input.focus();
        }
    }

    pub(crate) fn hide_native_input(&self) {
        if let Some(input) = &self.native_input {
            let _ = input.blur();
            let _ = input.set_attribute("value", "null");
            let _ = input.set_attribute(
                "style",
                "position:fixed;top:0;left:0;width:1px;height:1px;opacity:0;border:none;outline:none;font-size:16px;z-index:-1;pointer-events:none;caret-color:transparent;"
            );
        }
    }
}
