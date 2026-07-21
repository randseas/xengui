use std::{ sync::Arc };
use web_time::Instant;

use winit::event_loop::{ ControlFlow, EventLoop };
use winit::window::Window;
use xengui::{
    ElementState,
    EventCtx,
    InputEvent,
    InputState,
    MouseButton,
    TOUCH_LONG_PRESS_DURATION,
    TOUCH_LONG_PRESS_MOVE_TOLERANCE_DP,
    Widget,
    clear_text_selection_recursive,
    collect_focusable_paths,
    collect_selected_text_recursive,
    dispatch_positional,
    dispatch_to_path,
    hit_test_path,
    hooks,
    path_is_within,
    reconciler,
    style,
    update_global_text_selection,
};
use xengui_wgpu::WgpuWindowRenderer;

use crate::AppThemeMode;
use crate::config::AppConfig;
use crate::cursor::to_winit_cursor;
use crate::event::XenEvent;

pub struct App {
    pub(crate) renderer: Option<WgpuWindowRenderer>,
    pub(crate) window: Option<Arc<Window>>,

    pub(crate) config: AppConfig,
    pub(crate) root: Vec<Box<dyn Widget>>,
    pub(crate) is_visible: bool,
    pub(crate) input: InputState,

    pub(crate) component: Option<std::rc::Rc<dyn Fn() -> Box<dyn Widget>>>,
    pub(crate) next_blink: Option<Instant>,
    pub(crate) next_animation: Option<Instant>,
    pub(crate) reconcile_work: Option<reconciler::WorkLoop>,
    pub(crate) clipboard: xen_clipboard::Clipboard,
    pub(crate) pending_long_press: Option<(Instant, (f32, f32), String)>,

    #[cfg(target_arch = "wasm32")]
    pub(crate) text_agent: Option<crate::text_agent::TextAgent>,
    #[cfg(target_arch = "wasm32")]
    pub(crate) event_proxy: Option<winit::event_loop::EventLoopProxy<XenEvent>>,
}

impl App {
    pub fn new(config: AppConfig) -> Self {
        log::info!(target: "xengui", "app initialized");
        Self {
            renderer: None,
            window: None,

            config,
            root: Vec::new(),
            is_visible: false,
            input: InputState::default(),

            component: None,
            next_blink: None,
            next_animation: None,
            reconcile_work: None,
            clipboard: xen_clipboard::Clipboard::new(),
            pending_long_press: None,

            #[cfg(target_arch = "wasm32")]
            text_agent: None,
            #[cfg(target_arch = "wasm32")]
            event_proxy: None,
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn with_title(&mut self, title: &str) -> &mut Self {
        self.config.title = title.to_string();
        self
    }

    pub fn with_font(&mut self, name: &str, font_data: Vec<u8>) -> &mut Self {
        self.config.fonts.push((name.to_string(), font_data));
        self
    }

    pub fn add_node(&mut self, node: Box<dyn Widget>) {
        self.root.push(node);
    }

    pub fn render(&mut self, builder: impl (Fn() -> Box<dyn Widget>) + 'static) {
        self.component = Some(std::rc::Rc::new(builder));
        self.schedule_render();
        // No event loop is running yet at this point, so drive the first
        // reconciliation to completion instead of leaving unfinished work
        // with nothing pumping it.
        while self.pump_reconciliation() {}
    }

    // Runs the render-phase closure and starts (or restarts, interrupting
    // whatever was in flight) an interruptible reconciliation against the
    // still-on-screen tree. Does not touch `self.root` directly - that
    // only happens once `pump_reconciliation` reports completion.
    pub(crate) fn schedule_render(&mut self) {
        let Some(builder) = self.component.clone() else {
            return;
        };

        self.apply_pending_theme_switch();

        let system_is_dark = matches!(self.config.theme, Some(winit::window::Theme::Dark));
        let active = self.config.themes.get(self.config.active_theme).cloned().unwrap_or_default();
        style::theme::set_current_theme(active.resolved_for_system(system_is_dark));

        hooks::begin_render();
        let new_root = hooks::component("root", || builder());
        hooks::end_render();

        hooks::take_dirty();

        let new_tree = vec![new_root];
        self.reconcile_work = Some(reconciler::WorkLoop::new(new_tree, &self.root));
    }

    // Advances any in-progress reconciliation by one time slice. Returns
    // true if there is still more work left to do.
    pub(crate) fn pump_reconciliation(&mut self) -> bool {
        let Some(work) = self.reconcile_work.as_mut() else {
            return false;
        };

        // Budget kept well under a 16.6ms frame so input handling and
        // painting on the same thread never starve.
        const SLICE: std::time::Duration = std::time::Duration::from_millis(5);
        let deadline = Instant::now() + SLICE;

        match work.perform_work(deadline) {
            reconciler::WorkLoopStatus::Yielded => true,
            reconciler::WorkLoopStatus::Complete(tree) => {
                self.root = tree;
                self.reconcile_work = None;
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
                false
            }
        }
    }
}

impl App {
    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let event_loop: EventLoop<XenEvent> = EventLoop::<XenEvent>::with_user_event().build()?;
        event_loop.set_control_flow(ControlFlow::Wait);

        #[cfg(target_arch = "wasm32")]
        {
            self.event_proxy = Some(event_loop.create_proxy());
        }

        event_loop.run_app(self)?;
        Ok(())
    }

    #[cfg(target_arch = "wasm32")]
    pub fn set_renderer(&mut self, renderer: WgpuWindowRenderer) {
        self.renderer = Some(renderer);
    }

    pub(crate) fn apply_event_ctx(&mut self, mut ctx: EventCtx) {
        if let Some(new_focus) = ctx.focus_target.take() {
            if self.input.focused_path.as_deref() != Some(new_focus.as_str()) {
                if let Some(old) = self.input.focused_path.take() {
                    let mut sub_ctx = EventCtx::new();
                    dispatch_to_path(&mut self.root, &old, &InputEvent::FocusLost, &mut sub_ctx);
                }

                let mut sub_ctx = EventCtx::new();

                dispatch_to_path(
                    &mut self.root,
                    &new_focus,
                    &(InputEvent::FocusGained { via_keyboard: false }),
                    &mut sub_ctx
                );

                #[cfg(target_arch = "wasm32")]
                self.sync_native_input(&new_focus, true);
                self.input.focused_path = Some(new_focus);
                self.next_blink = None;
            } else {
                #[cfg(target_arch = "wasm32")]
                self.sync_native_input(&new_focus, true);
            }
        } else if ctx.clear_focus && let Some(old) = self.input.focused_path.take() {
            let mut sub_ctx = EventCtx::new();
            dispatch_to_path(&mut self.root, &old, &InputEvent::FocusLost, &mut sub_ctx);
            #[cfg(target_arch = "wasm32")]
            self.hide_native_input();
        }

        if let Some(icon) = ctx.take_cursor_icon() && let Some(window) = &self.window {
            window.set_cursor(to_winit_cursor(icon));
        }

        if ctx.redraw_requested() && let Some(window) = &self.window {
            window.request_redraw();
        }
    }

    // Tab / Shift+Tab moves to the next (or previous, if backward=true) focusable
    // widget, wrapping to the beginning/end at the boundaries.
    pub(crate) fn advance_focus(&mut self, backward: bool) {
        let focusable = collect_focusable_paths(&self.root);
        if focusable.is_empty() {
            return;
        }

        let current_index = self.input.focused_path
            .as_ref()
            .and_then(|p| focusable.iter().position(|f| f == p));

        let next_index = match (current_index, backward) {
            (None, false) => 0,
            (None, true) => focusable.len() - 1,
            (Some(i), false) => (i + 1) % focusable.len(),
            (Some(i), true) => (i + focusable.len() - 1) % focusable.len(),
        };

        if let Some(old) = self.input.focused_path.take() {
            let mut ctx = EventCtx::new();
            dispatch_to_path(&mut self.root, &old, &InputEvent::FocusLost, &mut ctx);
            #[cfg(target_arch = "wasm32")]
            self.hide_native_input();
            self.apply_event_ctx(ctx);
        }

        let new_path = focusable[next_index].clone();
        let mut ctx = EventCtx::new();
        dispatch_to_path(
            &mut self.root,
            &new_path,
            &(InputEvent::FocusGained { via_keyboard: true }),
            &mut ctx
        );
        self.input.focused_path = Some(new_path.to_string());
        self.next_blink = None;
        self.apply_event_ctx(ctx);
    }

    pub(crate) fn copy_selected_text(&self) {
        let mut text = String::new();
        collect_selected_text_recursive(&self.root, &mut text);
        if !text.is_empty() {
            self.clipboard.set_text(text, |_| {});
        }
    }

    pub(crate) fn cancel_text_selection(&mut self) {
        clear_text_selection_recursive(&mut self.root);
        self.input.text_drag_anchor = None;
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }

    // Simulates a same-spot double tap so the target widget's own
    // multi-click word-selection logic (already used for mouse) takes over.
    pub(crate) fn trigger_long_press_select(&mut self, path: &str, point: (f32, f32)) {
        for state in [ElementState::Pressed, ElementState::Released] {
            let mut ctx = EventCtx::new();
            dispatch_positional(
                &mut self.root,
                path,
                &(InputEvent::MouseInput { state, button: MouseButton::Left, position: point }),
                &mut ctx
            );
            if ctx.take_suppress_text_drag() {
                self.input.text_drag_anchor = None;
            }
            self.apply_event_ctx(ctx);
        }

        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }

    // Maps a touch point onto the existing mouse-event pipeline so widgets
    // don't need any touch-specific handling: Started acts like hover-in +
    // press, Moved updates position, Ended acts like release + hover-out,
    // Cancelled just clears state without firing a click.
    pub(crate) fn handle_touch(&mut self, touch: winit::event::Touch) {
        use winit::event::TouchPhase;

        let point = (touch.location.x as f32, touch.location.y as f32);

        match touch.phase {
            TouchPhase::Started => {
                clear_text_selection_recursive(&mut self.root);
                self.input.cursor_pos = Some(point);
                let path = hit_test_path(&self.root, point);

                // Same out-of-subtree focus release as WindowEvent::MouseInput;
                // touch presses don't go through that handler so this is repeated here.
                if let Some(focused) = self.input.focused_path.clone() {
                    let stays_focused = path
                        .as_deref()
                        .is_some_and(|p| path_is_within(p, &focused));
                    if !stays_focused {
                        let mut ctx = EventCtx::new();
                        dispatch_to_path(
                            &mut self.root,
                            &focused,
                            &InputEvent::FocusLost,
                            &mut ctx
                        );
                        self.input.focused_path = None;
                        self.apply_event_ctx(ctx);
                    }
                }

                if let Some(old) = self.input.hovered_path.take() {
                    let mut ctx = EventCtx::new();
                    dispatch_to_path(&mut self.root, &old, &InputEvent::MouseExited, &mut ctx);
                    self.apply_event_ctx(ctx);
                }

                if let Some(new) = &path {
                    let mut ctx = EventCtx::new();
                    dispatch_to_path(&mut self.root, new, &InputEvent::MouseEntered, &mut ctx);
                    self.apply_event_ctx(ctx);
                }
                self.input.hovered_path = path.clone();

                let mut suppress_drag = false;

                if let Some(path) = &path {
                    self.input.pressed_path = Some(path.clone());
                    let mut ctx = EventCtx::new();
                    dispatch_positional(
                        &mut self.root,
                        path,
                        &(InputEvent::MouseInput {
                            state: ElementState::Pressed,
                            button: MouseButton::Left,
                            position: point,
                        }),
                        &mut ctx
                    );
                    suppress_drag = ctx.take_suppress_text_drag();
                    self.apply_event_ctx(ctx);
                }

                // A double/triple tap already resolved its own word/line
                // selection - letting a drag anchor survive here would let
                // the next Moved event immediately overwrite it.
                self.input.text_drag_anchor = if suppress_drag { None } else { Some(point) };
                self.pending_long_press = path.map(|p| (
                    Instant::now() + TOUCH_LONG_PRESS_DURATION,
                    point,
                    p,
                ));
            }

            TouchPhase::Moved => {
                self.input.cursor_pos = Some(point);
                if let Some(path) = self.input.hovered_path.clone() {
                    let mut ctx = EventCtx::new();
                    dispatch_positional(
                        &mut self.root,
                        &path,
                        &(InputEvent::MouseMoved { position: point }),
                        &mut ctx
                    );
                    self.apply_event_ctx(ctx);
                }

                if let Some(anchor) = self.input.text_drag_anchor {
                    update_global_text_selection(&mut self.root, anchor, point);
                }

                if let Some((_, start_point, _)) = self.pending_long_press {
                    let scale_factor = self.window
                        .as_ref()
                        .map_or(1.0, |w| w.scale_factor() as f32);
                    let moved = (point.0 - start_point.0).abs() + (point.1 - start_point.1).abs();
                    if moved > TOUCH_LONG_PRESS_MOVE_TOLERANCE_DP * scale_factor {
                        self.pending_long_press = None;
                    }
                }
            }

            TouchPhase::Ended => {
                if let Some(path) = self.input.hovered_path.clone() {
                    let mut ctx = EventCtx::new();
                    dispatch_positional(
                        &mut self.root,
                        &path,
                        &(InputEvent::MouseInput {
                            state: ElementState::Released,
                            button: MouseButton::Left,
                            position: point,
                        }),
                        &mut ctx
                    );
                    self.apply_event_ctx(ctx);
                }

                if let Some(old) = self.input.hovered_path.take() {
                    let mut ctx = EventCtx::new();
                    dispatch_to_path(&mut self.root, &old, &InputEvent::MouseExited, &mut ctx);
                    self.apply_event_ctx(ctx);
                }

                self.input.pressed_path = None;
                self.input.cursor_pos = None;
                self.input.text_drag_anchor = None;
                self.pending_long_press = None;
            }

            TouchPhase::Cancelled => {
                if let Some(old) = self.input.hovered_path.take() {
                    let mut ctx = EventCtx::new();
                    dispatch_to_path(&mut self.root, &old, &InputEvent::MouseExited, &mut ctx);
                    self.apply_event_ctx(ctx);
                }
                self.input.pressed_path = None;
                self.input.cursor_pos = None;
                self.input.text_drag_anchor = None;
                self.pending_long_press = None;
            }
        }
    }

    // Applies a theme switch requested via `set_active_theme`/
    // `set_active_theme_by_name` since the last render pass.
    fn apply_pending_theme_switch(&mut self) {
        let Some(switch) = style::theme::take_theme_switch() else {
            return;
        };
        match switch {
            style::theme::ThemeSwitch::Index(index) => {
                if index < self.config.themes.len() {
                    self.config.active_theme = index;
                }
            }
            style::theme::ThemeSwitch::Name(name) => {
                if let Some(index) = self.config.themes.iter().position(|t| t.name() == name) {
                    self.config.active_theme = index;
                }
            }
        }
    }

    // Keeps `active_theme` synced with the OS appearance while `theme_mode`
    // is `System`; returns whether the index actually changed.
    pub(crate) fn sync_active_theme_with_system(&mut self) -> bool {
        if self.config.theme_mode != AppThemeMode::System {
            return false;
        }

        let is_dark = matches!(self.config.theme, Some(winit::window::Theme::Dark));
        let target = if is_dark { self.config.dark_theme } else { self.config.light_theme };

        if target < self.config.themes.len() && target != self.config.active_theme {
            self.config.active_theme = target;
            true
        } else {
            false
        }
    }
}
