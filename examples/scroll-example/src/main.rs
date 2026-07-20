// SPDX-License-Identifier: Apache-2.0
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use xenframe::{ App, AppConfig };
#[cfg(not(target_arch = "wasm32"))]
use xenframe::WindowPosition;

use xengui::*;

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
        title: "XenGui - Scroll Example".into(),
        #[cfg(not(target_arch = "wasm32"))]
        width: 900,
        #[cfg(not(target_arch = "wasm32"))]
        height: 700,
        #[cfg(not(target_arch = "wasm32"))]
        position: WindowPosition::Center,
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
                        .label("Scroll example")
                        .font_size(14)
                        .color(Color::NEUTRAL_500)
                        .margin(Edges::only(0, 0, 0, 10))
                )
                .child(
                    View::new()
                        .flex_direction(FlexDirection::Row)
                        .margin(Edges::only(0, 0, 12, 0))
                        .child(
                            Button::new()
                                .label("button1")
                                .font_size(13)
                                .color(Color::NEUTRAL_500)
                                .background(Color::NEUTRAL_100)
                                .border(Border::new(1, Color::NEUTRAL_200, Length::px(6.0)))
                                .padding(Edges::only(10, 4, 10, 6))
                                .hover_style(|s, _theme: &Theme|
                                    s
                                        .background(Color::NEUTRAL_200)
                                        .border(Border::new(1, Color::NEUTRAL_300, Length::px(6.0)))
                                )
                        )
                        .child(
                            Button::new()
                                .label("button2")
                                .font_size(13)
                                .color(Color::NEUTRAL_500)
                                .background(Color::NEUTRAL_100)
                                .border(Border::new(1, Color::NEUTRAL_200, Length::px(6.0)))
                                .padding(Edges::only(10, 4, 10, 6))
                                .hover_style(|s, _theme: &Theme|
                                    s
                                        .background(Color::NEUTRAL_200)
                                        .border(Border::new(1, Color::NEUTRAL_300, Length::px(6.0)))
                                )
                        )
                )
                .child(
                    View::new()
                        .flex_direction(FlexDirection::Column)
                        .overflow_x(Overflow::Auto)
                        .overflow_y(Overflow::Auto)
                        .scrollbar_track_color(Color::NEUTRAL_100)
                        .scrollbar_thumb_color(Color::NEUTRAL_400)
                        .scrollbar_arrow_color(Color::NEUTRAL_400)
                        .child({
                            let label = "label";
                            let mut children: Vec<Box<dyn Widget>> = Vec::with_capacity(50);

                            for i in 0..50 {
                                children.push(
                                    Box::new(
                                        Label::new()
                                            .key(i.to_string())
                                            .label(label)
                                            .color(Color::NEUTRAL_500)
                                    )
                                );
                            }

                            View::new().flex_direction(FlexDirection::Column).children_vec(children)
                        })
                )
        )
    });

    if let Err(e) = app.run() {
        eprintln!("Error running app: {:?}", e);
    }

    Ok(())
}
