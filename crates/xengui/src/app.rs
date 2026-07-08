// SPDX-License-Identifier: Apache-2.0
use crate::{Widget, XenRenderer};
use std::sync::Arc;

#[cfg(not(target_arch = "wasm32"))]
use winit::dpi::PhysicalPosition;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;

use winit::{
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    monitor::{MonitorHandle, VideoModeHandle},
    window::{Window, WindowAttributes, WindowId},
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

    pub debug: bool,
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

            debug: false,
            fonts: Vec::new(),
        }
    }
}

pub enum XenEvent {
    RendererReady(XenRenderer),
}

pub struct App {
    renderer: Option<XenRenderer>,
    window: Option<Arc<Window>>,

    config: AppConfig,
    root: Vec<Box<dyn Widget>>,
    is_visible: bool,

    #[cfg(target_arch = "wasm32")]
    pub event_proxy: Option<winit::event_loop::EventLoopProxy<XenEvent>>,
}

impl App {
    pub fn new(config: AppConfig) -> Self {
        if config.debug {
            log::info!("[INFO] XenGui Initialized");
        }
        Self {
            renderer: None,
            window: None,
            config,
            root: Vec::new(),
            is_visible: false,
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
        let winit_fullscreen = self.config.fullscreen.as_ref().map(|f| match f {
            Fullscreen::Borderless(monitor) => {
                winit::window::Fullscreen::Borderless(monitor.clone())
            }
            Fullscreen::Exclusive(video_mode) => {
                winit::window::Fullscreen::Exclusive(video_mode.clone())
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
                .with_inner_size(winit::dpi::LogicalSize::new(
                    self.config.width as f64,
                    self.config.height as f64,
                ))
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
            attributes = attributes.with_append(true).with_prevent_default(false);
        }

        // Instantiate the official application window instance
        let window = Arc::new(
            event_loop
                .create_window(attributes)
                .expect("Critical Error: Could not create window context."),
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

            if let WindowPosition::Center = self.config.position
                && let Some(monitor) = window.current_monitor()
            {
                let monitor_size = monitor.size();
                let window_size = window.outer_size();
                window.set_outer_position(PhysicalPosition::new(
                    (monitor_size.width as i32 - window_size.width as i32) / 2,
                    (monitor_size.height as i32 - window_size.height as i32) / 2,
                ));
            }
        }

        // Web optimization: Ensure canvas automatically tracks viewport bounding box dimensions
        #[cfg(target_arch = "wasm32")]
        {
            self.config.theme = Some(winit::window::Theme::Dark);

            use winit::platform::web::WindowExtWebSys;

            // Local helper: pull the current browser viewport size and push it
            // into the winit window. This is what actually causes winit to
            // enqueue a `WindowEvent::Resized` (when the size changed).
            fn sync_canvas_to_viewport(window: &Window) {
                if let Some(web_window) = web_sys::window() {
                    let inner_w = web_window.inner_width().ok().and_then(|val| val.as_f64());
                    let inner_h = web_window.inner_height().ok().and_then(|val| val.as_f64());

                    if let (Some(w_val), Some(h_val)) = (inner_w, inner_h) {
                        let phys_size = winit::dpi::LogicalSize::new(w_val, h_val)
                            .to_physical::<u32>(window.scale_factor());
                        let _ = window.request_inner_size(phys_size);
                    }
                }
            }

            if window.canvas().is_some() {
                // Set the initial size once, as before.
                sync_canvas_to_viewport(&window);

                // IMPORTANT: the one-shot call above only sets the size at
                // startup. Nothing was previously re-syncing the canvas when
                // the *browser window* was resized afterwards, so the canvas'
                // own box size never changed, winit's internal ResizeObserver
                // never fired, and `WindowEvent::Resized` never reached
                // `window_event()` -> layout was never recalculated.
                //
                // Fix: listen for the DOM "resize" event on `window` for the
                // lifetime of the app, and re-sync on every occurrence.
                if let Some(web_window) = web_sys::window() {
                    let window_for_closure = window.clone();
                    let closure = wasm_bindgen::closure::Closure::<dyn FnMut()>::new(move || {
                        sync_canvas_to_viewport(&window_for_closure);
                    });
                    let _ = web_window.add_event_listener_with_callback(
                        "resize",
                        closure.as_ref().unchecked_ref(),
                    );
                    // Leak the closure so it stays alive for the whole
                    // program (equivalent to the app's own lifetime here).
                    closure.forget();
                }
            }
        }

        self.window = Some(window.clone());

        // Target dependent Graphics Pipeline initialization wrapper
        #[cfg(not(target_arch = "wasm32"))]
        {
            let user_fonts = std::mem::take(&mut self.config.fonts);
            match XenRenderer::new(window, user_fonts, self.config.debug) {
                Ok(renderer) => {
                    self.renderer = Some(renderer);
                    if self.config.debug {
                        log::info!("[INFO] Application Resumed: GPU Context Ready.");
                    }
                }
                Err(e) => {
                    if self.config.debug {
                        log::info!("[CRITICAL] Cannot start GPU pipeline: {}", e);
                    }
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
                let debug = std::mem::take(&mut self.config.debug);

                wasm_bindgen_futures::spawn_local(async move {
                    if let Ok(renderer) = XenRenderer::new(window_clone, user_fonts, debug).await {
                        let _ = proxy_clone.send_event(XenEvent::RendererReady(renderer));
                    }
                });
            }
            if self.config.debug {
                log::info!("[INFO] Application Resumed on Web target.");
            }
        }
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: XenEvent) {
        match event {
            XenEvent::RendererReady(renderer) => {
                self.renderer = Some(renderer);
                if self.config.debug {
                    log::info!("[INFO] Web GPU Context successfully attached to Event Loop.");
                }
                if let Some(w) = &self.window {
                    w.request_redraw();
                }
            }
        }
    }

    fn window_event(&mut self, _event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => _event_loop.exit(),
            WindowEvent::RedrawRequested => {
                if let Some(renderer) = &mut self.renderer {
                    renderer.render_frame(&mut self.root, &self.config.theme);
                    if !self.is_visible
                        && let Some(window) = &self.window
                    {
                        window.set_visible(true);
                        self.is_visible = true;
                    }
                }
            }
            WindowEvent::Resized(new_size) => {
                if let Some(renderer) = &mut self.renderer {
                    renderer.resize(&mut self.root, &self.config.theme, new_size);
                    if !self.is_visible
                        && let Some(window) = &self.window
                    {
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
                if self.config.debug {
                    log::info!("[INFO] Theme changed: {:?}", new_theme);
                }
                for node in &mut self.root {
                    node.set_dirty(true);
                }
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            _ => (),
        }
    }
}