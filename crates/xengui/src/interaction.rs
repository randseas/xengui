// SPDX-License-Identifier: Apache-2.0
use crate::{
    Cursor,
    ElementState,
    EventCtx,
    EventStatus,
    InputEvent,
    Key,
    KeyState,
    KeyboardEvent,
    MouseButton,
};

type Callback = Box<dyn FnMut(&mut EventCtx)>;
type HoverCallback = Box<dyn FnMut(bool, &mut EventCtx)>;
type MouseInputCallback = Box<dyn FnMut(ElementState, MouseButton, &mut EventCtx)>;
type KeyCallback = Box<dyn FnMut(&KeyboardEvent, &mut EventCtx)>;

pub struct Interaction {
    pub enabled: bool,

    pub focusable: bool,
    pub hover_cursor: Option<Cursor>,

    pub hovered: bool,
    pub pressed: bool,
    pub focused: bool,
    pub focus_visible: bool,

    pub on_mouse_enter: Option<Callback>,
    pub on_mouse_leave: Option<Callback>,

    pub on_hover: Option<HoverCallback>,

    pub on_mouse_input: Option<MouseInputCallback>,
    pub on_key: Option<KeyCallback>,
    pub on_click: Option<Callback>,
}

impl Interaction {
    pub fn new() -> Self {
        Self {
            enabled: true,
            focusable: false,
            hover_cursor: None,
            hovered: false,
            pressed: false,
            focused: false,
            focus_visible: false,
            on_mouse_enter: None,
            on_mouse_leave: None,
            on_hover: None,
            on_mouse_input: None,
            on_key: None,
            on_click: None,
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.hovered = false;
            self.pressed = false;
        }
    }

    pub fn is_active(&self) -> bool {
        self.focusable ||
            self.hover_cursor.is_some() ||
            self.on_mouse_enter.is_some() ||
            self.on_mouse_leave.is_some() ||
            self.on_hover.is_some() ||
            self.on_mouse_input.is_some() ||
            self.on_key.is_some() ||
            self.on_click.is_some()
    }

    pub fn transfer_from(&mut self, old: &Interaction) {
        self.hovered = old.hovered;
        self.pressed = old.pressed;
        self.focused = old.focused;
        self.focus_visible = old.focus_visible;
    }

    fn is_activation_key(key: Key) -> bool {
        matches!(key, Key::Enter | Key::Space)
    }

    pub fn handle(&mut self, event: &InputEvent, ctx: &mut EventCtx) -> EventStatus {
        if !self.enabled {
            return EventStatus::Ignored;
        }

        match event {
            InputEvent::MouseEntered => {
                self.hovered = true;
                if let Some(icon) = self.hover_cursor {
                    ctx.set_cursor_icon(icon);
                }
                if let Some(cb) = self.on_mouse_enter.as_mut() {
                    cb(ctx);
                }
                if let Some(cb) = self.on_hover.as_mut() {
                    cb(true, ctx);
                }
                EventStatus::Handled
            }

            InputEvent::MouseExited => {
                let was_pressed = self.pressed;
                self.hovered = false;
                self.pressed = false;
                if self.hover_cursor.is_some() && !was_pressed {
                    ctx.set_cursor_icon(Cursor::Default);
                }
                if let Some(cb) = self.on_mouse_leave.as_mut() {
                    cb(ctx);
                }
                if let Some(cb) = self.on_hover.as_mut() {
                    cb(false, ctx);
                }
                EventStatus::Handled
            }

            InputEvent::MouseInput { state, button, .. } => {
                if let Some(cb) = self.on_mouse_input.as_mut() {
                    cb(*state, *button, ctx);
                }

                if *button == MouseButton::Left {
                    match state {
                        ElementState::Pressed => {
                            self.pressed = true;
                            // a pointer press dismisses the ring, even if the widget already has keyboard focus.
                            self.focus_visible = false;
                            if let Some(icon) = self.hover_cursor {
                                ctx.set_cursor_icon(icon);
                            }
                            if self.focusable {
                                ctx.request_focus();
                            }
                        }
                        ElementState::Released => {
                            let was_click = self.pressed && self.hovered;
                            self.pressed = false;
                            if was_click && let Some(cb) = self.on_click.as_mut() {
                                cb(ctx);
                            }
                        }
                    }
                }
                EventStatus::Handled
            }

            InputEvent::KeyInput { event: key_event, .. } => {
                let mut consumed = self.on_key.is_some();

                if let Some(cb) = self.on_key.as_mut() {
                    cb(key_event, ctx);
                }

                if
                    self.focused &&
                    key_event.key == Key::Escape &&
                    key_event.state == KeyState::Pressed
                {
                    ctx.release_focus();
                    consumed = true;
                }

                if self.focused && Self::is_activation_key(key_event.key) {
                    match key_event.state {
                        KeyState::Pressed if !key_event.repeat => {
                            self.pressed = true;
                            if let Some(cb) = self.on_click.as_mut() {
                                cb(ctx);
                            }
                        }
                        KeyState::Released => {
                            self.pressed = false;
                        }
                        _ => {}
                    }
                    consumed = true;
                }

                if consumed {
                    EventStatus::Handled
                } else {
                    EventStatus::Ignored
                }
            }
            
            InputEvent::FocusGained { via_keyboard } => {
                self.focused = true;
                self.focus_visible = *via_keyboard;
                if let Some(icon) = self.hover_cursor {
                    ctx.set_cursor_icon(icon);
                }
                EventStatus::Handled
            }

            InputEvent::FocusLost => {
                self.focused = false;
                self.pressed = false;
                self.focus_visible = false;
                EventStatus::Handled
            }

            _ => EventStatus::Ignored,
        }
    }
}

impl Default for Interaction {
    fn default() -> Self {
        Self::new()
    }
}

#[macro_export]
macro_rules! impl_interaction_builders {
    (base $ty:ty) => {
        impl $ty {
            pub fn on_click(mut self, f: impl FnMut(&mut $crate::EventCtx) + 'static) -> Self {
                self.base.interaction.on_click = Some(Box::new(f));
                self
            }

            pub fn on_hover(
                mut self,
                f: impl FnMut(bool, &mut $crate::EventCtx) + 'static,
            ) -> Self {
                self.base.interaction.on_hover = Some(Box::new(f));
                self
            }

            pub fn on_mouse_enter(
                mut self,
                f: impl FnMut(&mut $crate::EventCtx) + 'static,
            ) -> Self {
                self.base.interaction.on_mouse_enter = Some(Box::new(f));
                self
            }

            pub fn on_mouse_leave(
                mut self,
                f: impl FnMut(&mut $crate::EventCtx) + 'static,
            ) -> Self {
                self.base.interaction.on_mouse_leave = Some(Box::new(f));
                self
            }

            pub fn on_mouse_input(
                mut self,
                f: impl FnMut(
                    $crate::ElementState,
                    $crate::MouseButton,
                    &mut $crate::EventCtx,
                ) + 'static,
            ) -> Self {
                self.base.interaction.on_mouse_input = Some(Box::new(f));
                self
            }

            pub fn on_key(
                mut self,
                f: impl FnMut(&$crate::KeyboardEvent, &mut $crate::EventCtx) + 'static,
            ) -> Self {
                self.base.interaction.on_key = Some(Box::new(f));
                self
            }
        }
    };
}