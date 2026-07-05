// SPDX-License-Identifier: Apache-2.0
// crates/xengui/src/app.rs
use crate::{DebugText, VNode, XenRenderer};
use std::collections::VecDeque;
use std::sync::Arc;
use winit::{
    dpi::PhysicalPosition,
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
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub theme: Option<winit::window::Theme>,
    pub resizable: bool,
    pub fullscreen: Option<Fullscreen>,
    pub position: WindowPosition,
    pub debug_mode: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            title: "XenGui App".to_string(),
            width: 800,
            height: 600,
            theme: None,
            resizable: true,
            fullscreen: None,
            position: WindowPosition::Center,
            debug_mode: false,
        }
    }
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
        }
    }

    pub fn with_title(&mut self, title: &str) -> &mut Self {
        self.config.title = title.to_string();
        self
    }

    pub fn add_node(&mut self, node: Box<dyn VNode>) {
        self.v_domtree.push(node);
    }

    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Burada winit::event_loop oluşturulacak
        // Kendi içindeki 'resumed' event'inde XenRenderer::new() çağrılacak
        let event_loop = EventLoop::new()?;
        event_loop.set_control_flow(ControlFlow::Wait);
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
}

impl winit::application::ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // State Kaybı Çözümü: Mobil cihazlarda app öne geldiğinde window zaten varsa yeniden yaratma
        if self.window.is_some() {
            return;
        }
        let winit_fullscreen = self.config.fullscreen.as_ref().map(|f| match f {
            Fullscreen::Borderless(monitor) => {
                winit::window::Fullscreen::Borderless(monitor.clone())
            }
            Fullscreen::Exclusive(video_mode) => {
                winit::window::Fullscreen::Exclusive(video_mode.clone())
            }
        });
        let mut attr = WindowAttributes::default()
            .with_title(&self.config.title)
            .with_inner_size(winit::dpi::LogicalSize::new(
                self.config.width as f64,
                self.config.height as f64,
            ))
            .with_visible(false)
            .with_transparent(true)
            .with_resizable(self.config.resizable)
            .with_blur(true)
            .with_theme(None)
            .with_fullscreen(winit_fullscreen);

        if let WindowPosition::Fixed(x, y) = self.config.position {
            attr = attr.with_position(PhysicalPosition::new(x, y));
        }

        // todo!("if required, you've must add .unwrap() after .expect() in case.");
        let window = Arc::new(
            event_loop
                .create_window(attr)
                .expect("Critical Error: Could not create window."),
        );
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
                    (monitor_size.width - window_size.width) as i32 / 2,
                    (monitor_size.height - window_size.height) as i32 / 2,
                ));
            }
        }
        self.window = Some(window.clone());
        // Create XenRenderer
        match XenRenderer::new(window) {
            Ok(renderer) => {
                self.renderer = Some(renderer);
                self.log("[INFO] Application Resumed: GPU Context Ready.".to_string());
            }
            Err(e) => {
                eprintln!("[CRITICAL] Cannot start GPU: {}", e);
                std::process::exit(1); // Sistem desteklemiyorsa güvenli çıkış yap
            }
        }
    }

    fn window_event(&mut self, _event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => std::process::exit(0),
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
