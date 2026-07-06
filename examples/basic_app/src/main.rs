// SPDX-License-Identifier: Apache-2.0
// examples/basic_app/src/main.rs
use xengui::{props, App, AppConfig, Text, TextProps};

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
        );

        // Basic (fluent)
        app.add_node(Box::new(
            Text::new("text1")
                .font("Inter_SemiBold")
                .text("XenGui App")
                .scale(32.0),
        ));

        // Advanced (macro)
        let mut text2 = Text::new("text2");
        text2.set_props(props! {
            text: "Hello, world!",
            scale: 20.0,
            position: (0.0, 32.0),
            color: (0.0, 0.5, 0.5, 1.0)
        });
        app.add_node(Box::new(text2));

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

        // Basic (fluent)
        app.add_node(Box::new(
            Text::new("text1")
                .font("Inter_SemiBold")
                .text("XenGui App")
                .scale(32.0),
        ));

        // Advanced (macro)
        let mut text2 = Text::new("text2");
        text2.set_props(props! {
            text: "Hello, world!",
            scale: 20.0,
            position: (0.0, 32.0),
            color: (0.0, 0.5, 0.5, 1.0)
        });
        app.add_node(Box::new(text2));

        app.run()?;

        Ok(())
    }
}
