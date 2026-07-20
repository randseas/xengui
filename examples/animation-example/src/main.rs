// SPDX-License-Identifier: Apache-2.0
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use std::time::Duration;

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
        title: "XenGui - Animation Example".into(),
        #[cfg(not(target_arch = "wasm32"))]
        width: 700,
        #[cfg(not(target_arch = "wasm32"))]
        height: 500,
        #[cfg(not(target_arch = "wasm32"))]
        position: WindowPosition::Center,
        ..Default::default()
    };

    let mut app = App::new(config);

    app.render(|| {
        Box::new(
            View::new()
                .display(Display::Flex)
                .flex_direction(FlexDirection::Column)
                .align_items(AlignItems::Center)
                .justify_content(JustifyContent::Center)
                .width(Length::Percent(100.0))
                .height(Length::Percent(100.0))
                .gap(20, 20)
                .background(Color::NEUTRAL_50)
                .child(
                    Label::new()
                        .label("Independent color / transform transitions")
                        .font_size(16)
                        .color(Color::NEUTRAL_600)
                )
                .child(
                    // Color fades slowly, scale reacts fast - each group
                    // keeps its own duration/easing via the per-group builders.
                    Button::new()
                        .label("Hover & Press Me")
                        .font_size(15)
                        .background(Color::BLUE_500)
                        .padding(Edges::only(16, 10, 16, 10))
                        .border(Border::new(1, Color::BLUE_500, Length::px(10.0)))
                        .transition_colors(
                            Transition::new(Duration::from_millis(500)).easing(Easing::EaseInOut)
                        )
                        .transition_transform(
                            Transition::new(Duration::from_millis(150)).easing(Easing::EaseOut)
                        )
                        .hover_style(|s, _theme: &Theme|
                            s
                                .background(Color::BLUE_600)
                                .border(Border::new(1, Color::BLUE_600, Length::px(10.0)))
                        )
                        .pressed_style(|s, _theme: &Theme|
                            s
                                .background(Color::BLUE_800)
                                .scale(0.9)
                                .content_scale(1.0)
                                .border(Border::new(1, Color::BLUE_800, Length::px(10.0)))
                        )
                )
                .child(
                    Button::new()
                        .label("Border Grow")
                        .font_size(15)
                        .background(Color::NEUTRAL_800)
                        .padding(Edges::only(16, 10, 16, 10))
                        .border(Border::new(1, Color::NEUTRAL_700, Length::px(8.0)))
                        .transition_all(Transition::new(Duration::from_millis(250)))
                        .hover_style(|s, _theme: &Theme|
                            s.border(Border::new(1, Color::NEUTRAL_700, Length::px(24.0)))
                        )
                )
        )
    });

    if let Err(e) = app.run() {
        eprintln!("Error running app: {:?}", e);
    }

    Ok(())
}
