use std::sync::Arc;
use web_time::Instant;
use winit::{
    event::WindowEvent,
    event_loop::{ ActiveEventLoop, ControlFlow },
    window::{ WindowAttributes, WindowId },
};
use xengui::{
    ElementState,
    EventCtx,
    EventStatus,
    InputEvent,
    Key,
    KeyState,
    ModifiersState,
    MouseButton,
    Theme,
    any_wants_animation,
    clear_text_selection_recursive,
    dispatch_animation_tick,
    dispatch_positional,
    dispatch_to_path,
    find_widget_mut,
    hit_test_path,
    hooks::{ self, set_redraw_handle },
    path_is_within,
    select_all_text_recursive,
    update_global_text_selection,
};
use crate::{
    App,
    event::XenEvent,
    keyboard::{ convert_ime_event, convert_keyboard_event },
    mouse::{ convert_element_state, convert_mouse_button, convert_scroll_delta },
    window::Fullscreen,
};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;

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
            use crate::WindowPosition;

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
                use winit::dpi::PhysicalPosition;

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
            use crate::WindowPosition;

            if let Some(actual_theme) = window.theme() {
                self.config.theme = Some(actual_theme);
            } else {
                self.config.theme = Some(winit::window::Theme::Dark);
            }

            if
                let WindowPosition::Center = self.config.position &&
                let Some(monitor) = window.current_monitor()
            {
                use winit::dpi::PhysicalPosition;

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
            // Reads the browser's actual color-scheme preference on startup
            // instead of hardcoding Dark.
            let prefers_dark = web_sys
                ::window()
                .and_then(|w| w.match_media("(prefers-color-scheme: dark)").ok().flatten())
                .map(|mql| mql.matches())
                .unwrap_or(false);

            self.config.theme = Some(
                if prefers_dark {
                    winit::window::Theme::Dark
                } else {
                    winit::window::Theme::Light
                }
            );

            use winit::{ platform::web::WindowExtWebSys, window::Window };

            // Local helper: pull the current browser viewport size and push it
            // into the winit window. This is what actually causes winit to
            // enqueue a `WindowEvent::Resized` (when the size changed).
            fn sync_canvas_to_viewport(window: &Window) {
                if let Some(web_window) = web_sys::window() {
                    // visualViewport shrinks when the on-screen keyboard opens;
                    // window.innerHeight doesn't reliably do that on mobile browsers
                    let (inner_w, inner_h) = if let Some(vv) = web_window.visual_viewport() {
                        (Some(vv.width()), Some(vv.height()))
                    } else {
                        (
                            web_window
                                .inner_width()
                                .ok()
                                .and_then(|val| val.as_f64()),
                            web_window
                                .inner_height()
                                .ok()
                                .and_then(|val| val.as_f64()),
                        )
                    };

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

                    // visualViewport fires its own resize event on keyboard open/close,
                    // which the window's resize event doesn't always trigger
                    if let Some(vv) = web_window.visual_viewport() {
                        let window_for_vv = window.clone();
                        let vv_closure = wasm_bindgen::closure::Closure::<dyn FnMut()>::new(
                            move || {
                                sync_canvas_to_viewport(&window_for_vv);
                            }
                        );
                        let _ = vv.add_event_listener_with_callback(
                            "resize",
                            vv_closure.as_ref().unchecked_ref()
                        );
                        vv_closure.forget();
                    }
                }
            }
        }

        // Applies dark_theme/light_theme selection now that the real OS
        // appearance is known, before the window is ever shown.
        if self.sync_active_theme_with_system() {
            self.schedule_render();
            while self.pump_reconciliation() {}
        }

        self.window = Some(window.clone());

        set_redraw_handle(std::rc::Rc::new(crate::redraw::WinitRedraw(window.clone())));

        // Target dependent Graphics Pipeline initialization wrapper
        #[cfg(not(target_arch = "wasm32"))]
        {
            let user_fonts = std::mem::take(&mut self.config.fonts);
            let size = window.inner_size();
            match xengui_wgpu::WgpuWindowRenderer::new(window, size.width, size.height, user_fonts) {
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
            use crate::overlay::show_fatal_overlay;

            if let Some(proxy) = &self.event_proxy {
                let window_clone = window.clone();
                let proxy_clone = proxy.clone();
                let user_fonts = std::mem::take(&mut self.config.fonts);
                let size = window_clone.inner_size();

                wasm_bindgen_futures::spawn_local(async move {
                    log::info!("renderer init: adapter/device request starting");
                    let t0 = web_time::Instant::now();
                    match
                        xengui_wgpu::WgpuWindowRenderer::new(
                            window_clone,
                            size.width,
                            size.height,
                            user_fonts
                        ).await
                    {
                        Ok(renderer) => {
                            log::info!("renderer init: succeeded in {:?}", t0.elapsed());
                            let _ = proxy_clone.send_event(
                                XenEvent::RendererReady(Box::new(renderer))
                            );
                        }
                        Err(e) => {
                            let message = format!("xengui: renderer init failed\n\n{e}");
                            log::error!("{message}");
                            show_fatal_overlay(&message);
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
                            event.prevent_default();
                            let _ = proxy_clone2.send_event(XenEvent::CancelSelection);
                            return;
                        }

                        if event.key() == "Control" {
                            event.prevent_default();
                        }

                        // winit's web backend deliberately skips preventDefault for
                        // modifier combos (to avoid blocking browser shortcuts like
                        // Ctrl+R), so the browser's own select-all/cut/copy/paste
                        // fires alongside ours and interferes with focus/selection.
                        // Stop the browser default only for the combos widgets
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

                // Live-updates active_theme whenever the browser's
                // color-scheme preference flips, mirroring native's
                // WindowEvent::ThemeChanged.
                if
                    let Some(web_window) = web_sys::window() &&
                    let Ok(Some(mql)) = web_window.match_media("(prefers-color-scheme: dark)")
                {
                    let proxy_clone = proxy.clone();
                    let theme_closure: wasm_bindgen::closure::Closure<
                        dyn FnMut(web_sys::MediaQueryListEvent)
                    > = wasm_bindgen::closure::Closure::new(
                        move |event: web_sys::MediaQueryListEvent| {
                            let theme = if event.matches() {
                                winit::window::Theme::Dark
                            } else {
                                winit::window::Theme::Light
                            };

                            let _ = proxy_clone.send_event(XenEvent::SystemThemeChanged(theme));
                        }
                    );
                    let _ = mql.add_event_listener_with_callback(
                        "change",
                        theme_closure.as_ref().unchecked_ref()
                    );
                    theme_closure.forget();
                }

                // create invisible input for mobile devices
                #[cfg(target_arch = "wasm32")]
                self.create_native_input();
            }
            log::info!("application resumed, gpu context ready");
        }
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: XenEvent) {
        match event {
            XenEvent::RendererReady(renderer) => {
                self.renderer = Some(*renderer);
                log::info!("web gpu context successfully attached to event loop");
                if let Some(window) = &self.window {
                    let size = window.inner_size();
                    let theme = crate::window::system_theme(self.config.theme);
                    let scale_factor = window.scale_factor() as f32;
                    if let Some(renderer) = &mut self.renderer {
                        // Forces a real configure+render with the window's
                        // actual current size, in case it changed between
                        // renderer init (async) and this point.
                        renderer.resize(
                            &mut self.root,
                            theme,
                            scale_factor,
                            size.width,
                            size.height
                        );
                    }
                    window.request_redraw();
                }
            }
            XenEvent::CancelSelection => {
                self.cancel_text_selection();
            }
            XenEvent::SystemThemeChanged(new_theme) => {
                self.config.theme = Some(new_theme);
                log::info!("browser color-scheme changed: {:?}", new_theme);

                let system_switched = self.sync_active_theme_with_system();
                let active_is_auto = self.config.themes
                    .get(self.config.active_theme)
                    .is_some_and(Theme::is_auto);

                if system_switched || active_is_auto {
                    self.schedule_render();
                    while self.pump_reconciliation() {}
                } else {
                    for node in &mut self.root {
                        node.set_dirty(true);
                    }
                }
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            XenEvent::NativeInputChanged(text) => {
                if let Some(path) = self.input.focused_path.clone() {
                    let mut ctx = EventCtx::new();
                    if let Some(widget) = find_widget_mut(&mut self.root, &path) {
                        widget.set_native_text_value(&text, &mut ctx);
                    }
                    self.apply_event_ctx(ctx);
                    if let Some(window) = &self.window {
                        window.request_redraw();
                    }
                }
            }
        }
    }

    fn window_event(&mut self, _event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => _event_loop.exit(),
            WindowEvent::RedrawRequested => {
                log::info!("RedrawRequested fired, is_visible={}", self.is_visible);
                if hooks::take_dirty() {
                    self.schedule_render();
                }
                if let Some(renderer) = &mut self.renderer {
                    let theme = crate::window::system_theme(self.config.theme);
                    let scale_factor = self.window
                        .as_ref()
                        .map_or(1.0, |w| w.scale_factor() as f32);
                    renderer.render_frame(&mut self.root, theme, scale_factor);
                    if !self.is_visible && let Some(window) = &self.window {
                        window.set_visible(true);
                        self.is_visible = true;
                    }
                }

                // Re-syncs every frame instead of only on focus/resize, so
                // scrolling, drag-reflow, or any other layout drift never
                // leaves the native input stuck at a stale position.
                #[cfg(target_arch = "wasm32")]
                if let Some(path) = self.input.focused_path.clone() {
                    self.sync_native_input(&path, false);
                }
            }
            WindowEvent::Resized(new_size) => {
                if let Some(renderer) = &mut self.renderer {
                    let theme = crate::window::system_theme(self.config.theme);
                    let scale_factor = self.window
                        .as_ref()
                        .map_or(1.0, |w| w.scale_factor() as f32);
                    renderer.resize(
                        &mut self.root,
                        theme,
                        scale_factor,
                        new_size.width,
                        new_size.height
                    );
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

                let system_switched = self.sync_active_theme_with_system();
                let active_is_auto = self.config.themes
                    .get(self.config.active_theme)
                    .is_some_and(Theme::is_auto);

                if system_switched || active_is_auto {
                    // Auto colors, or a full active_theme swap, were baked in
                    // at the last render pass, so rebuild the tree instead of
                    // just repainting it.
                    self.schedule_render();
                    while self.pump_reconciliation() {}
                } else {
                    for node in &mut self.root {
                        node.set_dirty(true);
                    }
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
                    update_global_text_selection(&mut self.root, anchor, point);
                    // Label/Link selection is driven outside the widget event
                    // system, so it needs an explicit redraw request here.
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
                    clear_text_selection_recursive(&mut self.root);
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

                    // A click outside the focused widget's own subtree releases focus.
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
                            #[cfg(target_arch = "wasm32")]
                            self.hide_native_input();
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
                            state: convert_element_state(state),
                            button: convert_mouse_button(button),
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
                            delta: convert_scroll_delta(delta),
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

                let mut status = EventStatus::Ignored;

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
                    status == EventStatus::Ignored &&
                    keyboard_event.state == KeyState::Pressed &&
                    !keyboard_event.repeat &&
                    (self.input.modifiers.ctrl || self.input.modifiers.super_key)
                {
                    match keyboard_event.key {
                        Key::Character('a' | 'A') => {
                            select_all_text_recursive(&mut self.root);
                            if let Some(window) = &self.window {
                                window.request_redraw();
                            }
                        }
                        Key::Character('c' | 'C') => self.copy_selected_text(),
                        _ => {}
                    }
                }

                // Directional focus navigation, only when the focused widget
                // itself didn't consume the arrow key (e.g. TextBox uses
                // arrows for caret movement instead).
                if
                    status == EventStatus::Ignored &&
                    keyboard_event.state == KeyState::Pressed &&
                    !keyboard_event.repeat &&
                    !self.input.modifiers.ctrl &&
                    !self.input.modifiers.super_key &&
                    !self.input.modifiers.alt
                {
                    match keyboard_event.key {
                        Key::ArrowDown | Key::ArrowRight => self.advance_focus(false),
                        Key::ArrowUp | Key::ArrowLeft => self.advance_focus(true),
                        _ => {}
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
                        &InputEvent::Ime(convert_ime_event(ime_event)),
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
                            state: ElementState::Released,
                            button: MouseButton::Left,
                            position: point,
                        }),
                        &mut ctx
                    );
                    self.apply_event_ctx(ctx);
                }

                // Losing window focus never delivers a real MouseExited either,
                // so the hovered widget would otherwise stay stuck in its
                // hover-styled state until the next CursorMoved.
                if let Some(path) = self.input.hovered_path.take() {
                    let mut ctx = EventCtx::new();
                    dispatch_to_path(&mut self.root, &path, &InputEvent::MouseExited, &mut ctx);
                    self.apply_event_ctx(ctx);
                }
                self.input.cursor_pos = None;
            }
            _ => (),
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if let Some((deadline, point, path)) = self.pending_long_press.clone() {
            if Instant::now() >= deadline {
                self.pending_long_press = None;
                self.trigger_long_press_select(&path, point);
            } else {
                event_loop.set_control_flow(ControlFlow::Poll);
            }
        }

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

        if let Some(renderer) = &self.renderer && renderer.is_animating() {
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
