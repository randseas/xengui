// SPDX-License-Identifier: Apache-2.0
use crate::{
    EventCtx,
    InputEvent,
    InputState,
    Key,
    KeyState,
    ModifiersState,
    Widget,
    XenRenderer,
    any_wants_animation,
    collect_focusable_paths,
    convert_keyboard_event,
    dispatch_animation_tick,
    dispatch_positional,
    dispatch_to_path,
    find_widget_mut,
    hit_test_path,
    hooks::set_redraw_handle,
    path_is_within,
};
use std::sync::Arc;
use web_time::Instant;

#[cfg(not(target_arch = "wasm32"))]
use winit::dpi::PhysicalPosition;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;

use winit::{
    event::WindowEvent,
    event_loop::{ ActiveEventLoop, ControlFlow, EventLoop },
    monitor::{ MonitorHandle, VideoModeHandle },
    window::{ Window, WindowAttributes, WindowId },
};

pub enum WindowPosition {
    Center,
    Default,
    Fixed(i32, i32),
}

pub enum Fullscreen {
    Exclusive(VideoModeHandle),
    /// Providing `None` to `Borderless` will fullscreen on the current monitor.
    Borderless(Option<MonitorHandle>),
}

pub struct AppConfig {
    #[cfg(not(target_arch = "wasm32"))]
    pub title: String,

    #[cfg(not(target_arch = "wasm32"))]
    pub width: u32,
    #[cfg(not(target_arch = "wasm32"))]
    pub height: u32,

    pub theme: Option<winit::window::Theme>,

    #[cfg(not(target_arch = "wasm32"))]
    pub resizable: bool,

    pub fullscreen: Option<Fullscreen>,

    #[cfg(not(target_arch = "wasm32"))]
    pub position: WindowPosition,

    pub fonts: Vec<(String, Vec<u8>)>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            #[cfg(not(target_arch = "wasm32"))]
            title: "XenGui App".to_string(),

            #[cfg(not(target_arch = "wasm32"))]
            width: 800,
            #[cfg(not(target_arch = "wasm32"))]
            height: 600,

            theme: None,

            #[cfg(not(target_arch = "wasm32"))]
            resizable: true,

            fullscreen: None,

            #[cfg(not(target_arch = "wasm32"))]
            position: WindowPosition::Center,

            fonts: Vec::new(),
        }
    }
}

pub enum XenEvent {
    RendererReady(Box<XenRenderer>),
    CancelSelection,
}

pub struct App {
    renderer: Option<XenRenderer>,
    window: Option<Arc<Window>>,

    config: AppConfig,
    root: Vec<Box<dyn Widget>>,
    is_visible: bool,
    input: InputState,

    component: Option<std::rc::Rc<dyn Fn() -> Box<dyn Widget>>>,
    next_blink: Option<Instant>,
    next_animation: Option<Instant>,
    reconcile_work: Option<crate::reconciler::WorkLoop>,
    clipboard: xen_clipboard::Clipboard,

    #[cfg(target_arch = "wasm32")]
    pub event_proxy: Option<winit::event_loop::EventLoopProxy<XenEvent>>,
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
    fn schedule_render(&mut self) {
        let Some(builder) = self.component.clone() else {
            return;
        };

        crate::hooks::begin_render();
        let new_root = crate::hooks::component("root", || builder());
        crate::hooks::end_render();

        crate::hooks::take_dirty();

        let new_tree = vec![new_root];
        self.reconcile_work = Some(crate::reconciler::WorkLoop::new(new_tree, &self.root));
    }

    // Advances any in-progress reconciliation by one time slice. Returns
    // true if there is still more work left to do.
    fn pump_reconciliation(&mut self) -> bool {
        let Some(work) = self.reconcile_work.as_mut() else {
            return false;
        };

        // Budget kept well under a 16.6ms frame so input handling and
        // painting on the same thread never starve.
        const SLICE: std::time::Duration = std::time::Duration::from_millis(5);
        let deadline = Instant::now() + SLICE;

        match work.perform_work(deadline) {
            crate::reconciler::WorkLoopStatus::Yielded => true,
            crate::reconciler::WorkLoopStatus::Complete(tree) => {
                self.root = tree;
                self.reconcile_work = None;
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
                false
            }
        }
    }

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
    pub fn set_renderer(&mut self, renderer: XenRenderer) {
        self.renderer = Some(renderer);
    }
}

impl winit::application::ApplicationHandler<XenEvent> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // State Loss Prevention: Avoid recreation if the window already exists
        if self.window.is_some() {
            return;
        }

        // Parse custom fullscreen configurations into winit primitives
        let winit_fullscreen = self.config.fullscreen.as_ref().map(|f| {
            match f {
                Fullscreen::Borderless(monitor) => {
                    winit::window::Fullscreen::Borderless(monitor.clone())
                }
                Fullscreen::Exclusive(video_mode) => {
                    winit::window::Fullscreen::Exclusive(video_mode.clone())
                }
            }
        });

        let mut attributes = WindowAttributes::default()
            .with_visible(false)
            .with_theme(None)
            .with_fullscreen(winit_fullscreen);

        #[cfg(not(target_arch = "wasm32"))]
        {
            attributes = attributes
                .with_title(&self.config.title)
                .with_inner_size(
                    winit::dpi::LogicalSize::new(
                        self.config.width as f64,
                        self.config.height as f64
                    )
                )
                .with_resizable(self.config.resizable)
                .with_transparent(true)
                .with_blur(true);

            if let WindowPosition::Fixed(x, y) = self.config.position {
                attributes = attributes.with_position(PhysicalPosition::new(x, y));
            }
        }

        #[cfg(target_arch = "wasm32")]
        {
            use winit::platform::web::WindowAttributesExtWebSys;
            attributes = attributes.with_append(true).with_prevent_default(true);
        }

        // Instantiate the official application window instance
        let window = Arc::new(
            event_loop
                .create_window(attributes)
                .expect("Critical Error: Could not create window context.")
        );

        // Synchronize system window theme preferences
        if let Some(actual_theme) = window.theme() {
            self.config.theme = Some(actual_theme);
        } else {
            self.config.theme = Some(winit::window::Theme::Dark);
        }

        // Calculate layout mechanics only on desktop targets
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Some(actual_theme) = window.theme() {
                self.config.theme = Some(actual_theme);
            } else {
                self.config.theme = Some(winit::window::Theme::Dark);
            }

            if
                let WindowPosition::Center = self.config.position &&
                let Some(monitor) = window.current_monitor()
            {
                let monitor_size = monitor.size();
                let window_size = window.outer_size();
                window.set_outer_position(
                    PhysicalPosition::new(
                        ((monitor_size.width as i32) - (window_size.width as i32)) / 2,
                        ((monitor_size.height as i32) - (window_size.height as i32)) / 2
                    )
                );
            }
        }

        #[cfg(target_arch = "wasm32")]
        {
            self.config.theme = Some(winit::window::Theme::Dark);

            use winit::platform::web::WindowExtWebSys;

            // Local helper: pull the current browser viewport size and push it
            // into the winit window. This is what actually causes winit to
            // enqueue a `WindowEvent::Resized` (when the size changed).
            fn sync_canvas_to_viewport(window: &Window) {
                if let Some(web_window) = web_sys::window() {
                    let inner_w = web_window
                        .inner_width()
                        .ok()
                        .and_then(|val| val.as_f64());
                    let inner_h = web_window
                        .inner_height()
                        .ok()
                        .and_then(|val| val.as_f64());

                    if let (Some(w_val), Some(h_val)) = (inner_w, inner_h) {
                        let phys_size = winit::dpi::LogicalSize
                            ::new(w_val, h_val)
                            .to_physical::<u32>(window.scale_factor());
                        let _ = window.request_inner_size(phys_size);
                    }
                }
            }

            if window.canvas().is_some() {
                sync_canvas_to_viewport(&window);

                if let Some(web_window) = web_sys::window() {
                    let window_for_closure = window.clone();
                    let closure = wasm_bindgen::closure::Closure::<dyn FnMut()>::new(move || {
                        sync_canvas_to_viewport(&window_for_closure);
                    });
                    let _ = web_window.add_event_listener_with_callback(
                        "resize",
                        closure.as_ref().unchecked_ref()
                    );
                    closure.forget();
                }
            }
        }

        self.window = Some(window.clone());

        set_redraw_handle(window.clone());

        // Target dependent Graphics Pipeline initialization wrapper
        #[cfg(not(target_arch = "wasm32"))]
        {
            let user_fonts = std::mem::take(&mut self.config.fonts);
            match XenRenderer::new(window, user_fonts) {
                Ok(renderer) => {
                    self.renderer = Some(renderer);

                    log::info!("application resumed, gpu context ready");
                }
                Err(e) => {
                    log::info!("cannot start gpu pipeline: {}", e);

                    std::process::exit(1);
                }
            }
        }

        #[cfg(target_arch = "wasm32")]
        {
            if let Some(proxy) = &self.event_proxy {
                let window_clone = window.clone();
                let proxy_clone = proxy.clone();
                let user_fonts = std::mem::take(&mut self.config.fonts);

                wasm_bindgen_futures::spawn_local(async move {
                    match XenRenderer::new(window_clone, user_fonts).await {
                        Ok(renderer) => {
                            let _ = proxy_clone.send_event(
                                XenEvent::RendererReady(Box::new(renderer))
                            );
                        }
                        Err(e) => {
                            log::error!("wasm32(web) XenRenderer init failed: {e}");
                        }
                    }
                });

                if let Some(document) = web_sys::window().and_then(|w| w.document()) {
                    let proxy_clone = proxy.clone();
                    // Escape's keydown never reaches the page while the browser
                    // is exiting fullscreen or pointer lock (spec-mandated, not
                    // overridable) - fullscreenchange is the only signal we get.
                    let fs_closure = wasm_bindgen::closure::Closure::<dyn FnMut()>::new(move || {
                        let _ = proxy_clone.send_event(XenEvent::CancelSelection);
                    });
                    let _ = document.add_event_listener_with_callback_and_bool(
                        "keydown",
                        fs_closure.as_ref().unchecked_ref(),
                        true // capture phase: runs before winit's canvas listener,
                        // so a stopPropagation() there can't swallow this
                    );
                    fs_closure.forget();

                    // Direct DOM listener as a safety net: on some
                    // browsers/focus states winit's own canvas keydown
                    // listener doesn't reliably surface Escape, so it's
                    // also caught here independently of winit's pipeline.
                    let proxy_clone2 = proxy.clone();
                    let key_closure = wasm_bindgen::closure::Closure::<
                        dyn FnMut(web_sys::KeyboardEvent)
                    >::new(move |event: web_sys::KeyboardEvent| {
                        if event.key() == "Escape" {
                            let _ = proxy_clone2.send_event(XenEvent::CancelSelection);
                            return;
                        }

                        // winit's web backend deliberately skips preventDefault for
                        // modifier combos (to avoid blocking browser shortcuts like
                        // Ctrl+R), so the browser's own select-all/cut/copy/paste
                        // fires alongside ours and interferes with focus/selection.
                        // Stop the browser default only for the combos TextBox
                        // already implements itself.
                        let cmd = event.ctrl_key() || event.meta_key();
                        if cmd {
                            match event.key().as_str() {
                                "a" | "A" | "x" | "X" | "c" | "C" | "v" | "V" => {
                                    event.prevent_default();
                                }
                                _ => {}
                            }
                        }
                    });
                    let _ = document.add_event_listener_with_callback_and_bool(
                        "keydown",
                        key_closure.as_ref().unchecked_ref(),
                        true // capture phase: runs before winit's canvas listener,
                        // so a stopPropagation() there can't swallow this
                    );
                    key_closure.forget();
                }
            }
            log::info!("application resumed, gpu context ready");
        }
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: XenEvent) {
        match event {
            XenEvent::RendererReady(renderer) => {
                self.renderer = Some(*renderer);
                log::info!("web gpu context successfully attached to event loop");
                if let Some(w) = &self.window {
                    w.request_redraw();
                }
            }
            XenEvent::CancelSelection => {
                self.cancel_text_selection();
            }
        }
    }

    fn window_event(&mut self, _event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => _event_loop.exit(),
            WindowEvent::RedrawRequested => {
                if crate::hooks::take_dirty() {
                    self.schedule_render();
                }
                if let Some(renderer) = &mut self.renderer {
                    renderer.render_frame(&mut self.root, &self.config.theme);
                    if !self.is_visible && let Some(window) = &self.window {
                        window.set_visible(true);
                        self.is_visible = true;
                    }
                }
            }
            WindowEvent::Resized(new_size) => {
                if let Some(renderer) = &mut self.renderer {
                    renderer.resize(&mut self.root, &self.config.theme, new_size);
                    if !self.is_visible && let Some(window) = &self.window {
                        window.set_visible(true);
                        self.is_visible = true;
                    }
                }
            }
            WindowEvent::ScaleFactorChanged { .. } => {
                for node in &mut self.root {
                    node.set_dirty(true);
                }
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            WindowEvent::ThemeChanged(new_theme) => {
                self.config.theme = Some(new_theme);
                log::info!("theme changed: {:?}", new_theme);
                for node in &mut self.root {
                    node.set_dirty(true);
                }
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                let point = (position.x as f32, position.y as f32);
                self.input.cursor_pos = Some(point);

                let new_hover = hit_test_path(&self.root, point);
                if new_hover != self.input.hovered_path {
                    if let Some(old) = self.input.hovered_path.take() {
                        let mut ctx = EventCtx::new();
                        dispatch_to_path(&mut self.root, &old, &InputEvent::MouseExited, &mut ctx);
                        self.apply_event_ctx(ctx);
                    }
                    if let Some(new) = &new_hover {
                        let mut ctx = EventCtx::new();
                        dispatch_to_path(&mut self.root, new, &InputEvent::MouseEntered, &mut ctx);
                        self.apply_event_ctx(ctx);
                    }
                    self.input.hovered_path = new_hover.clone();
                }

                // While a button is held, movement is captured by the widget that was
                // pressed, so drags (e.g. a scrollbar thumb) keep tracking the cursor
                // even after it leaves that widget's bounds.
                let move_target = self.input.pressed_path.clone().or(new_hover);

                if let Some(path) = &move_target {
                    let mut ctx = EventCtx::new();
                    dispatch_positional(
                        &mut self.root,
                        path,
                        &(InputEvent::MouseMoved { position: point }),
                        &mut ctx
                    );
                    self.apply_event_ctx(ctx);
                }

                if let Some(anchor) = self.input.text_drag_anchor {
                    crate::update_global_text_selection(&mut self.root, anchor, point);
                    if let Some(window) = &self.window {
                        window.request_redraw();
                    }
                }
            }
            WindowEvent::CursorLeft { .. } => {
                self.input.cursor_pos = None;
                if let Some(old) = self.input.hovered_path.take() {
                    let mut ctx = EventCtx::new();
                    dispatch_to_path(&mut self.root, &old, &InputEvent::MouseExited, &mut ctx);
                    self.apply_event_ctx(ctx);
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if state == winit::event::ElementState::Pressed {
                    crate::clear_text_selection_recursive(&mut self.root);
                    if let Some(window) = &self.window {
                        window.request_redraw();
                    }
                }

                let Some(point) = self.input.cursor_pos else {
                    return;
                };

                if
                    state == winit::event::ElementState::Pressed &&
                    button == winit::event::MouseButton::Left
                {
                    self.input.text_drag_anchor = Some(point);
                }

                // On release, target the widget that was actually pressed (mouse
                // capture) rather than re-hit-testing the current cursor position -
                // the cursor may have left that widget's bounds during a drag.
                let path = if state == winit::event::ElementState::Released {
                    self.input.pressed_path.clone()
                } else {
                    self.input.hovered_path.clone().or_else(|| hit_test_path(&self.root, point))
                };

                if state == winit::event::ElementState::Pressed {
                    self.input.pressed_path = path.clone();

                    // Odaklı widget'ın kendi alt ağacı dışına yapılan bir tıklama focus'u bırakır.
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
                }

                if let Some(path) = &path {
                    let mut ctx = EventCtx::new();
                    dispatch_positional(
                        &mut self.root,
                        path,
                        &(InputEvent::MouseInput {
                            state,
                            button,
                            position: point,
                        }),
                        &mut ctx
                    );
                    if ctx.take_suppress_text_drag() {
                        self.input.text_drag_anchor = None;
                    }
                    self.apply_event_ctx(ctx);
                }

                if state == winit::event::ElementState::Released {
                    self.input.pressed_path = None;
                    self.input.text_drag_anchor = None;
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let Some(point) = self.input.cursor_pos else {
                    return;
                };
                let path = self.input.hovered_path
                    .clone()
                    .or_else(|| hit_test_path(&self.root, point));

                if let Some(path) = &path {
                    let mut ctx = EventCtx::new();
                    dispatch_positional(
                        &mut self.root,
                        path,
                        &(InputEvent::MouseWheel {
                            delta,
                            position: point,
                            modifiers: self.input.modifiers,
                        }),
                        &mut ctx
                    );
                    self.apply_event_ctx(ctx);
                }
            }
            WindowEvent::Touch(touch) => {
                self.handle_touch(touch);
            }
            WindowEvent::KeyboardInput { event, .. } => {
                let keyboard_event = convert_keyboard_event(event);

                // Mirrors modifier state from the key event itself, since
                // ModifiersChanged can arrive out of order relative to the
                // following KeyboardInput on some platforms.
                match keyboard_event.key {
                    Key::ControlLeft | Key::ControlRight => {
                        self.input.modifiers.ctrl = keyboard_event.state == KeyState::Pressed;
                    }
                    Key::ShiftLeft | Key::ShiftRight => {
                        self.input.modifiers.shift = keyboard_event.state == KeyState::Pressed;
                    }
                    Key::AltLeft | Key::AltRight => {
                        self.input.modifiers.alt = keyboard_event.state == KeyState::Pressed;
                    }
                    Key::SuperLeft | Key::SuperRight => {
                        self.input.modifiers.super_key = keyboard_event.state == KeyState::Pressed;
                    }
                    _ => {}
                }

                if
                    keyboard_event.key == Key::Tab &&
                    keyboard_event.state == KeyState::Pressed &&
                    !keyboard_event.repeat
                {
                    self.advance_focus(self.input.modifiers.shift);
                    return;
                }

                let mut status = crate::EventStatus::Ignored;

                if let Some(path) = self.input.focused_path.clone() {
                    let mut ctx = EventCtx::new();

                    status = dispatch_positional(
                        &mut self.root,
                        &path,
                        &(InputEvent::KeyInput {
                            event: keyboard_event.clone(),
                            modifiers: self.input.modifiers,
                        }),
                        &mut ctx
                    );
                    self.apply_event_ctx(ctx);
                }

                // Page-wide select-all / copy, only when no focused widget
                // already consumed the shortcut for itself (e.g. TextBox).
                if
                    status == crate::EventStatus::Ignored &&
                    keyboard_event.state == KeyState::Pressed &&
                    !keyboard_event.repeat &&
                    (self.input.modifiers.ctrl || self.input.modifiers.super_key)
                {
                    match keyboard_event.key {
                        Key::Character('a' | 'A') => {
                            crate::select_all_text_recursive(&mut self.root);
                            if let Some(window) = &self.window {
                                window.request_redraw();
                            }
                        }
                        Key::Character('c' | 'C') => self.copy_selected_text(),
                        _ => {}
                    }
                }

                // Deselect selection when pressed esc
                if
                    keyboard_event.key == Key::Escape &&
                    keyboard_event.state == KeyState::Pressed &&
                    !keyboard_event.repeat
                {
                    crate::clear_text_selection_recursive(&mut self.root);
                    self.input.text_drag_anchor = None;
                    if let Some(window) = &self.window {
                        window.request_redraw();
                    }
                }
            }
            WindowEvent::ModifiersChanged(new_mods) => {
                let mods = new_mods.state();

                self.input.modifiers = ModifiersState {
                    ctrl: mods.control_key(),
                    shift: mods.shift_key(),
                    alt: mods.alt_key(),
                    super_key: mods.super_key(),
                };
            }
            WindowEvent::Ime(ime_event) => {
                if let Some(path) = self.input.focused_path.clone() {
                    let mut ctx = EventCtx::new();
                    dispatch_positional(
                        &mut self.root,
                        &path,
                        &InputEvent::Ime(ime_event),
                        &mut ctx
                    );
                    self.apply_event_ctx(ctx);
                }
            }
            WindowEvent::Focused(has_focus) if !has_focus => {
                // Window losing focus mid-drag (e.g. alt-tab while holding the
                // scrollbar thumb) never delivers a real mouse-up, so synthesize
                // one to the captured widget before clearing capture - otherwise
                // it's left thinking the button is still held.
                if let Some(path) = self.input.pressed_path.take() {
                    let point = self.input.cursor_pos.unwrap_or((0.0, 0.0));
                    let mut ctx = EventCtx::new();
                    dispatch_positional(
                        &mut self.root,
                        &path,
                        &(InputEvent::MouseInput {
                            state: winit::event::ElementState::Released,
                            button: winit::event::MouseButton::Left,
                            position: point,
                        }),
                        &mut ctx
                    );
                    self.apply_event_ctx(ctx);
                }
            }
            _ => (),
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if self.reconcile_work.is_some() {
            let still_pending = self.pump_reconciliation();
            if still_pending {
                event_loop.set_control_flow(ControlFlow::Poll);
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
                return;
            }
        }

        if let Some(renderer) = &self.renderer && renderer.anim.is_animating() {
            event_loop.set_control_flow(ControlFlow::Poll);
            if let Some(window) = &self.window {
                window.request_redraw();
            }
            return;
        }

        // Tree-wide animations (e.g. smooth scrolling) run independently of
        // keyboard focus, so this is checked before the blink logic below.
        if any_wants_animation(&self.root) {
            let now = Instant::now();
            let dt = now.duration_since(self.next_animation.unwrap_or(now)).as_secs_f32().min(0.05);

            let mut ctx = EventCtx::new();
            dispatch_animation_tick(&mut self.root, dt, &mut ctx);
            self.apply_event_ctx(ctx);

            self.next_animation = Some(now);
            event_loop.set_control_flow(ControlFlow::Poll);
            return;
        }
        self.next_animation = None;

        let Some(focused) = self.input.focused_path.clone() else {
            self.next_blink = None;
            event_loop.set_control_flow(ControlFlow::Wait);
            return;
        };

        let interval = find_widget_mut(&mut self.root, &focused).and_then(|w| w.blink_interval());

        let Some(interval) = interval else {
            self.next_blink = None;
            event_loop.set_control_flow(ControlFlow::Wait);
            return;
        };

        let now = Instant::now();
        let deadline = *self.next_blink.get_or_insert(now + interval);

        if now >= deadline {
            let mut ctx = EventCtx::new();
            dispatch_to_path(&mut self.root, &focused, &InputEvent::BlinkTick, &mut ctx);
            self.apply_event_ctx(ctx);
            self.next_blink = Some(now + interval);
        }

        event_loop.set_control_flow(ControlFlow::WaitUntil(self.next_blink.unwrap()));
    }
}

impl App {
    fn apply_event_ctx(&mut self, mut ctx: EventCtx) {
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
                self.input.focused_path = Some(new_focus);
                self.next_blink = None;
            }
        } else if ctx.clear_focus && let Some(old) = self.input.focused_path.take() {
            let mut sub_ctx = EventCtx::new();
            dispatch_to_path(&mut self.root, &old, &InputEvent::FocusLost, &mut sub_ctx);
        }

        if let Some(icon) = ctx.take_cursor_icon() && let Some(window) = &self.window {
            window.set_cursor(icon);
        }

        if ctx.redraw_requested() && let Some(window) = &self.window {
            window.request_redraw();
        }
    }

    // Tab / Shift+Tab ile bir sonraki (backward=true ise bir önceki) focusable
    // widget'a geçer, uçlarda başa/sona sarar.
    fn advance_focus(&mut self, backward: bool) {
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

    fn copy_selected_text(&self) {
        let mut text = String::new();
        crate::collect_selected_text_recursive(&self.root, &mut text);
        if !text.is_empty() {
            self.clipboard.set_text(text, |_| {});
        }
    }

    fn cancel_text_selection(&mut self) {
        crate::clear_text_selection_recursive(&mut self.root);
        self.input.text_drag_anchor = None;
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }

    // Maps a touch point onto the existing mouse-event pipeline so widgets
    // don't need any touch-specific handling: Started acts like hover-in +
    // press, Moved updates position, Ended acts like release + hover-out,
    // Cancelled just clears state without firing a click.
    fn handle_touch(&mut self, touch: winit::event::Touch) {
        use winit::event::{ ElementState, MouseButton, TouchPhase };

        let point = (touch.location.x as f32, touch.location.y as f32);

        match touch.phase {
            TouchPhase::Started => {
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
                    self.apply_event_ctx(ctx);
                }
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
            }

            TouchPhase::Cancelled => {
                if let Some(old) = self.input.hovered_path.take() {
                    let mut ctx = EventCtx::new();
                    dispatch_to_path(&mut self.root, &old, &InputEvent::MouseExited, &mut ctx);
                    self.apply_event_ctx(ctx);
                }
                self.input.pressed_path = None;
                self.input.cursor_pos = None;
            }
        }
    }
}
