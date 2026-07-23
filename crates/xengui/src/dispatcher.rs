// SPDX-License-Identifier: Apache-2.0
use crate::{
    collect_focusable_paths,
    dispatch_positional,
    dispatch_to_path,
    hit_test_path,
    ElementState,
    EventCtx,
    InputEvent,
    InputState,
    KeyboardEvent,
    ModifiersState,
    MouseButton,
    Widget,
};

/// High-level pointer/keyboard/focus dispatcher built on top of the
/// low-level primitives in `input.rs`. Platform crates (e.g. xenframe)
/// can own a single `Dispatcher` instead of re-implementing hover,
/// pointer-capture and focus bookkeeping themselves.
#[derive(Default)]
pub struct Dispatcher {
    pub state: InputState,
}

impl Dispatcher {
    pub fn new() -> Self {
        Self::default()
    }

    /// Updates hover state for the widget under `point` and forwards the
    /// move event to whichever widget currently has pointer capture.
    pub fn pointer_moved(&mut self, tree: &mut [Box<dyn Widget>], point: (f32, f32)) -> EventCtx {
        let mut ctx = EventCtx::new();
        self.state.cursor_pos = Some(point);

        let new_hover = hit_test_path(tree, point);
        if new_hover != self.state.hovered_path {
            if let Some(old) = self.state.hovered_path.take() {
                dispatch_to_path(tree, &old, &InputEvent::MouseExited, &mut ctx);
            }
            if let Some(new) = &new_hover {
                dispatch_to_path(tree, new, &InputEvent::MouseEntered, &mut ctx);
            }
            self.state.hovered_path = new_hover.clone();
        }

        // While a button is held, movement stays captured by the pressed
        // widget even after the cursor leaves its bounds.
        let move_target = self.state.pressed_path.clone().or(new_hover);
        if let Some(path) = &move_target {
            dispatch_positional(
                tree,
                path,
                &(InputEvent::MouseMoved { position: point }),
                &mut ctx
            );
        }

        ctx
    }

    /// Dispatches a mouse button press/release, tracking pointer capture.
    pub fn pointer_input(
        &mut self,
        tree: &mut [Box<dyn Widget>],
        point: (f32, f32),
        input_state: ElementState,
        button: MouseButton
    ) -> EventCtx {
        let mut ctx = EventCtx::new();

        let path = if input_state == ElementState::Released {
            self.state.pressed_path.clone()
        } else {
            self.state.hovered_path.clone().or_else(|| hit_test_path(tree, point))
        };

        if input_state == ElementState::Pressed {
            self.state.pressed_path = path.clone();
        }

        if let Some(path) = &path {
            dispatch_positional(
                tree,
                path,
                &(InputEvent::MouseInput { state: input_state, button, position: point }),
                &mut ctx
            );
        }

        if input_state == ElementState::Released {
            self.state.pressed_path = None;
        }

        ctx
    }

    /// Dispatches a keyboard event to the currently focused widget.
    pub fn key_input(
        &mut self,
        tree: &mut [Box<dyn Widget>],
        event: KeyboardEvent,
        modifiers: ModifiersState
    ) -> EventCtx {
        let mut ctx = EventCtx::new();
        if let Some(path) = self.state.focused_path.clone() {
            dispatch_positional(
                tree,
                &path,
                &(InputEvent::KeyInput { event, modifiers }),
                &mut ctx
            );
        }
        ctx
    }

    /// Moves keyboard focus to the next (or previous) focusable widget,
    /// wrapping at the boundaries.
    pub fn advance_focus(&mut self, tree: &mut [Box<dyn Widget>], backward: bool) -> EventCtx {
        let mut ctx = EventCtx::new();
        let focusable = collect_focusable_paths(tree);
        if focusable.is_empty() {
            return ctx;
        }

        let current_index = self.state.focused_path
            .as_ref()
            .and_then(|p| focusable.iter().position(|f| f == p));

        let next_index = match (current_index, backward) {
            (None, false) => 0,
            (None, true) => focusable.len() - 1,
            (Some(i), false) => (i + 1) % focusable.len(),
            (Some(i), true) => (i + focusable.len() - 1) % focusable.len(),
        };

        if let Some(old) = self.state.focused_path.take() {
            dispatch_to_path(tree, &old, &InputEvent::FocusLost, &mut ctx);
        }

        let new_path = focusable[next_index].clone();
        dispatch_to_path(
            tree,
            &new_path,
            &(InputEvent::FocusGained { via_keyboard: true }),
            &mut ctx
        );
        self.state.focused_path = Some(new_path);

        ctx
    }
}
