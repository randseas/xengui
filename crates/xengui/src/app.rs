// SPDX-License-Identifier: Apache-2.0
// crates/xengui/src/app.rs
use crate::{DebugText, VNode, XenRenderer};
use std::collections::VecDeque;
use std::sync::Arc;
use winit::{
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    monitor::{MonitorHandle, VideoModeHandle},
    window::{Window, WindowAttributes, WindowId},
};
#[cfg(not(target_arch = "wasm32"))]
use winit::dpi::PhysicalPosition;

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
    pub debug_mode: bool,
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
            debug_mode: false,
        }
    }
}

pub enum XenEvent {
    RendererReady(XenRenderer),
}

pub struct App {
    // GPU Yönetimi
    renderer: Option<XenRenderer>,
    window: Option<Arc<Window>>,

    // Uygulama Verisi
    log_history: VecDeque<String>,
    config: AppConfig, // Builder ile gelen ayarlar
    v_domtree: Vec<Box<dyn VNode>>,
    is_visible: bool,

    #[cfg(target_arch = "wasm32")]
    pub event_proxy: Option<winit::event_loop::EventLoopProxy<XenEvent>>,
}

impl App {
    pub fn new(config: AppConfig) -> Self {
        let mut log_history = VecDeque::new();
        log_history.push_back("[INFO] System Initialized.".to_string());
        Self {
            renderer: None,
            window: None,
            config,
            log_history,
            v_domtree: vec![Box::new(DebugText::new("".into()))],
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

    pub fn add_node(&mut self, node: Box<dyn VNode>) {
        self.v_domtree.push(node);
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

    /// Logs a message to the internal history and triggers a redraw.
    fn log(&mut self, msg: String) {
        if !self.config.debug_mode {
            return;
        }
        self.log_history.push_back(msg); // O(1) ekleme
        if self.log_history.len() > 50 {
            self.log_history.pop_front(); // O(1) silme, performans darboğazı giderildi
        }
        let mut needs_redraw = false;
        for node in self.v_domtree.iter_mut() {
            if node.key() == "debug_text" {
                node.set_dirty(true);
                needs_redraw = true;
            }
        }
        if needs_redraw {
            if let Some(w) = &self.window {
                w.request_redraw();
            }
        }
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

        // Consolidate all window properties into a single attribute profile
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

        // Apply web-only configuration specs (Auto-maximizing layout strategy)
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

            if let WindowPosition::Center = self.config.position {
                if let Some(monitor) = window.current_monitor() {
                    let monitor_size = monitor.size();
                    let window_size = window.outer_size();
                    window.set_outer_position(PhysicalPosition::new(
                        (monitor_size.width as i32 - window_size.width as i32) / 2,
                        (monitor_size.height as i32 - window_size.height as i32) / 2,
                    ));
                }
            }
        }

        // Web optimization: Ensure canvas automatically tracks viewport bounding box dimensions
        #[cfg(target_arch = "wasm32")]
        {
            self.config.theme = Some(winit::window::Theme::Dark);

            use winit::platform::web::WindowExtWebSys;
            if let Some(_canvas) = window.canvas() {
                if let Some(web_window) = web_sys::window() {
                    let inner_w = web_window.inner_width().ok().and_then(|val| val.as_f64());
                    let inner_h = web_window.inner_height().ok().and_then(|val| val.as_f64());

                    if let (Some(w_val), Some(h_val)) = (inner_w, inner_h) {
                        // Fix: Explicitly specify target type token <u32> to satisfy compiler constraints
                        let phys_size = winit::dpi::LogicalSize::new(w_val, h_val)
                            .to_physical::<u32>(window.scale_factor());
                        let _ = window.request_inner_size(phys_size);
                    }
                }
            }
        }

        self.window = Some(window.clone());

        // Target dependent Graphics Pipeline initialization wrapper
        #[cfg(not(target_arch = "wasm32"))]
        {
            match XenRenderer::new(window) {
                Ok(renderer) => {
                    self.renderer = Some(renderer);
                    self.log("[INFO] Application Resumed: GPU Context Ready.".to_string());
                }
                Err(e) => {
                    eprintln!("[CRITICAL] Cannot start GPU pipeline: {}", e);
                    std::process::exit(1);
                }
            }
        }

        #[cfg(target_arch = "wasm32")]
        {
            if let Some(proxy) = &self.event_proxy {
                let window_clone = window.clone();
                let proxy_clone = proxy.clone();

                wasm_bindgen_futures::spawn_local(async move {
                    if let Ok(renderer) = XenRenderer::new(window_clone).await {
                        let _ = proxy_clone.send_event(XenEvent::RendererReady(renderer));
                    }
                });
            }
            self.log(
                "[INFO] Application Resumed on Web target. Async GPU compilation started."
                    .to_string(),
            );
        }
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: XenEvent) {
        match event {
            XenEvent::RendererReady(renderer) => {
                self.renderer = Some(renderer);
                self.log("[INFO] Web GPU Context successfully attached to Event Loop.".to_string());
                if let Some(w) = &self.window {
                    w.request_redraw();
                }
            }
        }
    }

    fn window_event(&mut self, _event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                #[cfg(not(target_arch = "wasm32"))]
                std::process::exit(0);
                #[cfg(target_arch = "wasm32")]
                _event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                if let Some(renderer) = &mut self.renderer {
                    renderer.render_frame(
                        &mut self.v_domtree,
                        &self.config.theme,
                        self.config.debug_mode,
                    );
                    if !self.is_visible {
                        if let Some(window) = &self.window {
                            window.set_visible(true);
                            self.is_visible = true;
                        }
                    }
                }
            }
            WindowEvent::Resized(new_size) => {
                if let Some(renderer) = &mut self.renderer {
                    renderer.resize(
                        &mut self.v_domtree,
                        &self.config.theme,
                        self.config.debug_mode,
                        new_size,
                    );
                    if !self.is_visible {
                        if let Some(window) = &self.window {
                            window.set_visible(true);
                            self.is_visible = true;
                        }
                    }
                }
            }
            WindowEvent::ThemeChanged(new_theme) => {
                self.config.theme = Some(new_theme);
                /* Debug */
                println!("[INFO] Theme changed: {:?}", new_theme);
                self.log(format!("[INFO] Theme changed: {:?}", new_theme));
                for node in &mut self.v_domtree {
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
