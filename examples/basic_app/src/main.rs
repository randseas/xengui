// SPDX-License-Identifier: Apache-2.0
// examples/basic_app/main.rs
use xengui::{props, App, AppConfig, Text, TextProps, WindowPosition};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Start the app
    let mut app = App::new(AppConfig {
        title: "XenGui Basic App".into(),
        width: 640,
        height: 480,
        position: WindowPosition::Center,
        ..Default::default()
    });

    // Basic (fluent)
    app.add_node(Box::new(
        Text::new("text1").text("Hello, world!").scale(20.0),
    ));

    // Advanced (macro)
    let mut text2 = Text::new("text2");
    text2.set_props(props! {
        text: "Hello, world!",
        scale: 20.0,
        position: (0.0, 20.0),
        color: (0.0, 0.5, 0.5, 1.0)
    });
    app.add_node(Box::new(text2));

    app.run().unwrap();

    Ok(())
}
