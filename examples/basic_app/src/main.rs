// SPDX-License-Identifier: Apache-2.0
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
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

    app.with_font(
        "Inter_Regular",
        include_bytes!("..\\fonts\\Inter_Regular.ttf").to_vec(),
    );

    let mut inner = View::new()
        .flex_direction(FlexDirection::Column)
        .width(Length::Percent(100.0))
        .height(Length::Percent(100.0))
        .align_items(AlignItems::Start)
        .justify_content(JustifyContent::Start);

    for size in [14, 16, 18, 20, 24] {
        inner = inner.child(
            Label::new()
                .label("The quick brown fox jumps over the lazy dog.")
                .font("Inter_Regular")
                .font_size(size)
                .color(Color::BLACK),
        );
    }

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
            .child(inner),
    );

    app.add_node(root);

    if let Err(e) = app.run() {
        eprintln!("Error running app: {:?}", e);
    }

    Ok(())
}
