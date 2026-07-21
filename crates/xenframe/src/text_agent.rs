// SPDX-License-Identifier: Apache-2.0
#![cfg(target_arch = "wasm32")]

//! Hidden native `<input>` element that brings up the on-screen keyboard on
//! mobile browsers and mirrors a focused widget's text into it, since a
//! `<canvas>` alone has no way to request a mobile keyboard by itself.

use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use winit::event_loop::EventLoopProxy;
use xengui::{ find_widget_mut, Widget };

use crate::{ App, event::XenEvent };

/// Owns the hidden `<input>` element used to drive mobile keyboard input.
pub struct TextAgent {
    input: web_sys::HtmlInputElement,
}

impl TextAgent {
    /// Creates and styles the hidden input, then wires up the listener
    /// that forwards its value to `XenEvent::NativeInputChanged`.
    pub(crate) fn attach(proxy: EventLoopProxy<XenEvent>) -> Option<Self> {
        let document = web_sys::window()?.document()?;
        let body = document.body()?;
        let input = document
            .create_element("input")
            .ok()?
            .dyn_into::<web_sys::HtmlInputElement>()
            .ok()?;

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
        
        let on_input = {
            let input = input.clone();
            move |event: web_sys::InputEvent| {
                // Gboard leaves stray invisible characters behind after a
                // suggestion commits unless focus is fully reset between
                // words; forcing a blur/refocus here clears that state.
                // Skipped for deletions, since refocusing mid-gesture
                // cancels the OS's own held-backspace repeat.
                let is_delete = event.input_type().starts_with("delete");
                if !event.is_composing() && !is_delete {
                    let _ = input.blur();
                    let _ = focus_without_scroll(&input);
                }
                let _ = proxy.send_event(XenEvent::NativeInputChanged(input.value()));
            }
        };
        let closure = wasm_bindgen::closure::Closure::<dyn FnMut(web_sys::InputEvent)>::new(
            on_input
        );
        let _ = input.add_event_listener_with_callback("input", closure.as_ref().unchecked_ref());
        closure.forget();

        Some(Self { input })
    }

    /// Copies a widget's text-input snapshot onto the hidden input so
    /// mobile keyboards get the correct value/placeholder/read-only state.
    pub(crate) fn mirror(&self, widget: &dyn Widget) {
        widget.sync_native_input(&self.input);
    }

    /// Focuses the hidden input without letting Safari scroll the page -
    /// a plain `.focus()` on a fixed-position element otherwise drags the
    /// canvas out of view the moment the on-screen keyboard opens.
    pub(crate) fn focus(&self) {
        let _ = focus_without_scroll(&self.input);
    }

    /// Resets the input's value and pushes it fully offscreen/inert again;
    /// used once focus leaves every native-text-backed widget.
    pub(crate) fn hide(&self) {
        let _ = self.input.blur();
        let _ = self.input.remove_attribute("value");
        let _ = self.input.remove_attribute("placeholder");
        let _ = self.input.set_attribute(
            "style",
            "position:fixed;top:0;left:0;width:1px;height:1px;opacity:0;border:none;outline:none;font-size:16px;z-index:-1;pointer-events:none;caret-color:transparent;"
        );
    }
}

impl Drop for TextAgent {
    fn drop(&mut self) {
        self.input.remove();
    }
}

/// Focuses `input` with `preventScroll`, so gaining focus never scrolls
/// the page - the default browser behavior Safari applies most eagerly.
fn focus_without_scroll(input: &web_sys::HtmlInputElement) -> Result<(), JsValue> {
    let opts = web_sys::FocusOptions::new();
    opts.set_prevent_scroll(true);
    input.focus_with_options(&opts)
}

impl App {
    /// Creates the mobile text agent, if the wasm event proxy is ready.
    pub(crate) fn create_native_input(&mut self) {
        if let Some(proxy) = &self.event_proxy {
            self.text_agent = TextAgent::attach(proxy.clone());
        }
    }

    /// Mirrors the widget at `path` onto the text agent's input and
    /// optionally focuses it; hides the agent if `path` no longer points
    /// to a native-text-backed widget.
    pub(crate) fn sync_native_input(&mut self, path: &str, focus: bool) {
        let Some(agent) = &self.text_agent else {
            return;
        };
        let Some(widget) = find_widget_mut(&mut self.root, path) else {
            agent.hide();
            return;
        };
        if widget.native_text_input().is_none() {
            agent.hide();
            return;
        }

        agent.mirror(widget);
        if focus {
            self.suppress_next_focus_loss = true;
            agent.focus();
        }
    }

    pub(crate) fn hide_native_input(&self) {
        if let Some(agent) = &self.text_agent {
            agent.hide();
        }
    }
}
