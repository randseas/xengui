// SPDX-License-Identifier: Apache-2.0
use crate::{ EventCtx, EventStatus, InputEvent, Key, KeyState, KeyboardEvent };
use winit::{ event::{ ElementState, MouseButton }, window::CursorIcon };

type Callback = Box<dyn FnMut(&mut EventCtx)>;
type HoverCallback = Box<dyn FnMut(bool, &mut EventCtx)>;
type MouseInputCallback = Box<dyn FnMut(ElementState, MouseButton, &mut EventCtx)>;
type KeyCallback = Box<dyn FnMut(&KeyboardEvent, &mut EventCtx)>;

pub struct Interaction {
    pub enabled: bool,

    pub focusable: bool,
    pub hover_cursor: Option<CursorIcon>,

    pub hovered: bool,
    pub pressed: bool,
    pub focused: bool,

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
                ctx.request_redraw();
                EventStatus::Handled
            }

            InputEvent::MouseExited => {
                self.hovered = false;
                self.pressed = false;
                if self.hover_cursor.is_some() {
                    ctx.set_cursor_icon(CursorIcon::Default);
                }
                if let Some(cb) = self.on_mouse_leave.as_mut() {
                    cb(ctx);
                }
                if let Some(cb) = self.on_hover.as_mut() {
                    cb(false, ctx);
                }
                ctx.request_redraw();
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
                ctx.request_redraw();
                EventStatus::Handled
            }

            InputEvent::KeyInput { event: key_event, .. } => {
                if let Some(cb) = self.on_key.as_mut() {
                    cb(key_event, ctx);
                }

                if self.focused && Self::is_activation_key(key_event.key) {
                    match key_event.state {
                        KeyState::Pressed if !key_event.repeat => {
                            self.pressed = true;
                            if let Some(cb) = self.on_click.as_mut() {
                                cb(ctx);
                            }
                            ctx.request_redraw();
                        }
                        KeyState::Released => {
                            self.pressed = false;
                            ctx.request_redraw();
                        }
                        _ => {}
                    }
                }

                EventStatus::Handled
            }

            InputEvent::FocusGained => {
                self.focused = true;
                ctx.request_redraw();
                EventStatus::Handled
            }

            InputEvent::FocusLost => {
                self.focused = false;
                self.pressed = false;
                ctx.request_redraw();
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
    ($ty:ty) => {
        impl $ty {
            pub fn on_click(mut self, f: impl FnMut(&mut $crate::EventCtx) + 'static) -> Self {
                self.interaction.on_click = Some(Box::new(f));
                self
            }

            pub fn on_hover(
                mut self,
                f: impl FnMut(bool, &mut $crate::EventCtx) + 'static,
            ) -> Self {
                self.interaction.on_hover = Some(Box::new(f));
                self
            }

            pub fn on_mouse_enter(
                mut self,
                f: impl FnMut(&mut $crate::EventCtx) + 'static,
            ) -> Self {
                self.interaction.on_mouse_enter = Some(Box::new(f));
                self
            }

            pub fn on_mouse_leave(
                mut self,
                f: impl FnMut(&mut $crate::EventCtx) + 'static,
            ) -> Self {
                self.interaction.on_mouse_leave = Some(Box::new(f));
                self
            }

            pub fn on_mouse_input(
                mut self,
                f: impl FnMut(
                    ::winit::event::ElementState,
                    ::winit::event::MouseButton,
                    &mut $crate::EventCtx,
                ) + 'static,
            ) -> Self {
                self.interaction.on_mouse_input = Some(Box::new(f));
                self
            }

            pub fn on_key(
                mut self,
                f: impl FnMut(&$crate::KeyboardEvent, &mut $crate::EventCtx) + 'static,
            ) -> Self {
                self.interaction.on_key = Some(Box::new(f));
                self
            }
        }
    };
}
