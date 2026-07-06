// SPDX-License-Identifier: Apache-2.0
// examples/basic_app/src/main.rs
use xengui::{App, AppConfig, Color, Text};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(target_arch = "wasm32")]
    {
        console_error_panic_hook::set_once();

        let mut app = App::new(AppConfig {
            ..Default::default()
        });

        app.with_font(
            "Inter_Regular",
            include_bytes!("../fonts/Inter_Regular.ttf").to_vec(),
        )
        .with_font(
            "Inter_Medium",
            include_bytes!("../fonts/Inter_Medium.ttf").to_vec(),
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
                .text_color(Color::WHITE.with_alpha(50)),
        ));

        if let Err(e) = app.run() {
            eprintln!("Error running app: {:?}", e);
        }

        Ok(())
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        use xengui::WindowPosition;
        // Start the app
        let mut app = App::new(AppConfig {
            title: "XenGui Basic App".into(),
            width: 640,
            height: 480,
            position: WindowPosition::Center,
            ..Default::default()
        });

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
                .text_color(Color::WHITE.with_alpha(50)),
        ));

        app.run()?;

        Ok(())
    }
}
