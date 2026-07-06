// SPDX-License-Identifier: Apache-2.0
use xengui::{App, AppConfig, Color, Text};

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
            .font("Inter_Medium")
            .font_size(24)
            .position((0.0, 0.0))
            .text_color(Color::TEAL),
    ));

    app.add_node(Box::new(
        Text::new("text2")
            .font("Inter_Regular")
            .text("Hello, world!")
            .font_size(20.0)
            .position((0.0, 24.0))
            .text_color(Color::WHITE),
    ));

    app.add_node(Box::new(
        Text::new("text3")
            .font("Inter_Regular")
            .text(format!("Platform: {PLATFORM}"))
            .font_size(20.0)
            .position((0.0, 44.0))
            .text_color(Color::LIGHT_GRAY),
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
