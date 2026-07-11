// SPDX-License-Identifier: Apache-2.0
use xengui::*;

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

    let root = Box::new(
        View::new()
            .display(Display::Flex)
            .flex_direction(FlexDirection::Column)
            .justify_content(JustifyContent::Start)
            .align_items(AlignItems::Start)
            .width(Length::Percent(100.0))
            .height(Length::Percent(100.0))
            .background(Color::WHITE)
            .padding(Edges::all(15))
            .child(
                View::new()
                    .flex_direction(FlexDirection::Column)
                    .width(Length::Percent(100.0))
                    .height(Length::Percent(100.0))
                    .align_items(AlignItems::Center)
                    .justify_content(JustifyContent::Center)
                    .child(
                        Label::new()
                            .label("Hello, world!")
                            .font_size(16)
                            .color(Color::BLACK),
                    )
                    .child(
                        Label::new()
                            .label("Hello, world!")
                            .font_size(20)
                            .color(Color::BLACK),
                    )
                    .child(
                        Label::new()
                            .label("Hello, world!")
                            .font_size(24)
                            .color(Color::BLACK),
                    )
                    .child(
                        Label::new()
                            .label("Hello, world!")
                            .font_size(28)
                            .color(Color::BLACK),
                    )
                    .child(
                        Label::new()
                            .label("Hello, world!")
                            .font_size(32)
                            .color(Color::BLACK),
                    )
                    .child(
                        Label::new()
                            .label("Hello, world!")
                            .font_size(36)
                            .color(Color::BLACK),
                    )
                    .child(
                        Label::new()
                            .label("Hello, world!")
                            .font_size(40)
                            .color(Color::BLACK),
                    )
                    .child(
                        Label::new()
                            .label("Hello, world!")
                            .font_size(44)
                            .color(Color::BLACK),
                    )
                    .child(
                        Label::new()
                            .label("Hello, world!")
                            .font_size(48)
                            .color(Color::BLACK),
                    )
                    .child(
                        Label::new()
                            .label("Hello, world!")
                            .font_size(52)
                            .color(Color::BLACK),
                    )
                    .child(
                        Label::new()
                            .label("Hello, world!")
                            .font_size(56)
                            .color(Color::BLACK),
                    )
                    .child(
                        Label::new()
                            .label("Hello, world!")
                            .font_size(60)
                            .color(Color::BLACK),
                    ),
            ),
    );

    app.add_node(root);

    if let Err(e) = app.run() {
        eprintln!("Error running app: {:?}", e);
    }

    Ok(())
}
