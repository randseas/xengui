// SPDX-License-Identifier: Apache-2.0
use xengui::{App, AppConfig, Color, StyleBuilder, Text};

#[cfg(target_arch = "wasm32")]
const PLATFORM: &str = "wasm32 (web)";

#[cfg(all(not(target_arch = "wasm32"), target_os = "windows"))]
const PLATFORM: &str = "windows";

#[cfg(all(not(target_arch = "wasm32"), target_os = "linux"))]
const PLATFORM: &str = "linux";

#[cfg(all(not(target_arch = "wasm32"), target_os = "macos"))]
const PLATFORM: &str = "macos";

#[cfg(all(
    not(target_arch = "wasm32"),
    not(target_os = "windows"),
    not(target_os = "linux"),
    not(target_os = "macos")
))]
const PLATFORM: &str = "unknown";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    let config = AppConfig {
        #[cfg(not(target_arch = "wasm32"))]
        title: "XenGui App".into(),

        #[cfg(not(target_arch = "wasm32"))]
        width: 640,

        #[cfg(not(target_arch = "wasm32"))]
        height: 480,

        #[cfg(not(target_arch = "wasm32"))]
        position: xengui::WindowPosition::Center,

        ..Default::default()
    };

    let mut app = App::new(config);

    app.with_font(
        "Inter_Regular",
        include_bytes!("../fonts/Inter_Regular.ttf").to_vec(),
    )
    .with_font(
        "Inter_Medium",
        include_bytes!("../fonts/Inter_Medium.ttf").to_vec(),
    )
    .with_font(
        "Inter_SemiBold",
        include_bytes!("../fonts/Inter_SemiBold.ttf").to_vec(),
    );

    app.add_node(Box::new(
        Text::new("title")
            .text("XenGui")
            .font("Inter_SemiBold")
            .font_size(24)
            .color(Color::TEAL)
            .background(Color::YELLOW),
    ));

    app.add_node(Box::new(
        Text::new("text2")
            .text("Hello, world!")
            .font("Inter_Regular")
            .font_size(16)
            .color(Color::WHITE)
            .background(Color::ORANGE),
    ));

    app.add_node(Box::new(
        Text::new("text3")
            .text(format!("Platform: {PLATFORM}"))
            .font("Inter_Regular")
            .font_size(16)
            .color(Color::WHITE)
            .background(Color::RED),
    ));

    #[cfg(target_arch = "wasm32")]
    {
        if let Err(e) = app.run() {
            eprintln!("Error running app: {:?}", e);
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        app.run()?;
    }

    Ok(())
}
