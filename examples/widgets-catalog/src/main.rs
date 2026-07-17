// SPDX-License-Identifier: Apache-2.0
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use xengui::{ widgets::Link, * };

fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(target_arch = "wasm32")]
    {
        console_error_panic_hook::set_once();
        let _ = console_log::init_with_level(log::Level::Info);
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = env_logger::Builder
            ::new()
            .filter_module("xengui", log::LevelFilter::Info)
            .filter_level(log::LevelFilter::Warn)
            .format_timestamp(None)
            .try_init();
    }

    let config = AppConfig {
        #[cfg(not(target_arch = "wasm32"))]
        title: "XenGui - Widgets Catalog".into(),
        #[cfg(not(target_arch = "wasm32"))]
        width: 900,
        #[cfg(not(target_arch = "wasm32"))]
        height: 700,
        #[cfg(not(target_arch = "wasm32"))]
        position: xengui::WindowPosition::Center,
        ..Default::default()
    };

    let mut app = App::new(config);

    app.with_font(
        "Noto_Sans",
        include_bytes!(
            concat!(env!("CARGO_MANIFEST_DIR"), "/fonts/NotoSans-VariableFont.ttf")
        ).to_vec()
    );

    app.render(|| {
        Box::new(
            View::new()
                .font("Noto_Sans")
                .display(Display::Flex)
                .flex_direction(FlexDirection::Column)
                .width(Length::Percent(100.0))
                .height(Length::Percent(100.0))
                .background(Color::WHITE)
                .padding(Edges::all(15))
                .child(
                    Label::new()
                        .label("Widgets Catalog")
                        .color(Color::NEUTRAL_500)
                        .margin(Edges::only(0, 0, 0, 10))
                )
                .child(
                    View::new()
                        .flex_direction(FlexDirection::Column)
                        .overflow_x(Overflow::Auto)
                        .overflow_y(Overflow::Auto)
                        .scrollbar_track_color(Color::NEUTRAL_100)
                        .scrollbar_thumb_color(Color::NEUTRAL_400)
                        .scrollbar_arrow_color(Color::NEUTRAL_400)
                        .child(
                            Button::new()
                                .label("button1")
                                .font_size(13)
                                .color(Color::NEUTRAL_500)
                                .background(Color::NEUTRAL_100)
                                .border(Border::new(1, Color::NEUTRAL_200, Length::px(6.0)))
                                .padding(Edges::only(10, 4, 10, 6))
                                .hover_style(|s|
                                    s
                                        .background(Color::NEUTRAL_200)
                                        .border(Border::new(1, Color::NEUTRAL_300, Length::px(6.0)))
                                )
                        )
                        .child(Label::new().label("label1").color(Color::NEUTRAL_500))
                        .child(
                            Link::new()
                                .label("https://github.com/randseas")
                                .href("https://github.com/randseas")
                        )
                )
        )
    });

    if let Err(e) = app.run() {
        eprintln!("Error running app: {:?}", e);
    }

    Ok(())
}
