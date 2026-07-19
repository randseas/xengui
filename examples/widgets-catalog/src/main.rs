// SPDX-License-Identifier: Apache-2.0
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use std::time::Duration;

// hides console window on windows subsystem
use xengui::{ properties::StyleValue, widgets::Link, * };

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

        themes: vec![
            Theme::light(),
            Theme::dark(),
            Theme::new("ocean").primary(Color::CYAN_500).background(Color::SKY_950)
        ],
        active_theme: 1,

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
        let (text, set_text) = use_state(String::from("Ferris"));

        Box::new(
            View::new()
                .font("Noto_Sans")
                .font_size(14)
                .display(Display::Flex)
                .flex_direction(FlexDirection::Column)
                .width(Length::Percent(100.0))
                .height(Length::Percent(100.0))
                .background(|theme: &Theme| (
                    if theme.is_dark() {
                        Color::NEUTRAL_950
                    } else {
                        Color::WHITE
                    }
                ))
                .padding(Edges::all(15))
                .child(
                    Label::new()
                        .label("Widgets Catalog")
                        .font_size(Length::px(18.0))
                        .color(Color::NEUTRAL_500)
                        .margin(Edges::only(0, 0, 0, 6))
                )
                .child(
                    View::new()
                        .flex_direction(FlexDirection::Column)
                        .overflow_x(Overflow::Auto)
                        .overflow_y(Overflow::Auto)
                        .scrollbar_track_color(Color::NEUTRAL_100)
                        .scrollbar_thumb_color(Color::NEUTRAL_400)
                        .scrollbar_arrow_color(Color::NEUTRAL_400)
                        .child(Label::new().label("label1").color(Color::NEUTRAL_500))
                        .child(
                            Link::new()
                                .label("https://github.com/randseas")
                                .href("https://github.com/randseas")
                        )
                        .child(
                            TextBox::new()
                                .value(text.clone())
                                .color(Color::NEUTRAL_500)
                                .placeholder("textBox1...")
                                .font_size(14)
                                .outline(StyleValue::None)
                                .min_width(Length::px(150.0))
                                .max_width(Length::px(170.0))
                                .transition_all(
                                    Transition::new(Duration::from_millis(200)).easing(
                                        Easing::EaseInOut
                                    )
                                )
                                .padding(Edges::only(10, 7, 10, 8))
                                .background(Color::WHITE)
                                .border(Border::new(1, Color::NEUTRAL_300, Length::px(8.0)))
                                .hover_style(|s, _theme: &Theme|
                                    s.border(Border::new(1, Color::NEUTRAL_400, Length::px(8.0)))
                                )
                                .focus_style(|s, _theme: &Theme|
                                    s.border(Border::new(2, Color::BLUE_500, Length::px(8.0)))
                                )
                                .on_change(move |value, _ctx| set_text.set(value.to_string()))
                        )
                        .child(
                            Button::new()
                                .label("button1")
                                .font_size(14)
                                .color(Color::NEUTRAL_500)
                                .background(Color::NEUTRAL_100)
                                .border(Border::new(1, Color::NEUTRAL_200, Length::px(8.0)))
                                .padding(Edges::only(9, 5, 9, 6))
                                .transition_all(
                                    Transition::new(Duration::from_millis(200)).easing(
                                        Easing::EaseInOut
                                    )
                                )
                                .hover_style(|s, _theme: &Theme|
                                    s
                                        .background(Color::NEUTRAL_200)
                                        .border(Border::new(1, Color::NEUTRAL_300, Length::px(8.0)))
                                        .color(Color::NEUTRAL_600)
                                )
                                .pressed_style(|s, _theme: &Theme|
                                    s
                                        .background(Color::NEUTRAL_200)
                                        .border(Border::new(1, Color::NEUTRAL_400, Length::px(8.0)))
                                        .color(Color::NEUTRAL_700)
                                )
                                .disabled_style(|s, _theme: &Theme|
                                    s.background(Color::NEUTRAL_100).color(Color::NEUTRAL_400)
                                )
                        )
                        .child(
                            Button::new()
                                .label("button1")
                                .font_size(14)
                                .background(Color::BLUE_500)
                                .border(Border::new(1, Color::BLUE_500, Length::px(8.0)))
                                .padding(Edges::only(9, 5, 9, 6))
                                .transition_all(
                                    Transition::new(Duration::from_millis(200)).easing(
                                        Easing::EaseInOut
                                    )
                                )
                                .hover_style(|s, _theme: &Theme|
                                    s
                                        .background(Color::BLUE_600)
                                        .border(Border::new(1, Color::BLUE_600, Length::px(8.0)))
                                )
                                .pressed_style(|s, _theme: &Theme|
                                    s
                                        .background(Color::BLUE_700)
                                        .scale(0.98)
                                        .content_scale(1.0)
                                        .border(Border::new(1, Color::BLUE_700, Length::px(8.0)))
                                )
                        )
                        .child(
                            Button::new()
                                .label("Sign in with GitHub")
                                .font_size(15)
                                .background(Color::NEUTRAL_800)
                                .padding(Edges::only(12, 8, 12, 8))
                                .border(Border::new(1, Color::NEUTRAL_700, Length::px(10.0)))
                                .transition_all(
                                    Transition::new(Duration::from_millis(200)).easing(
                                        Easing::EaseInOut
                                    )
                                )
                                .transition_transform(
                                    Transition::new(Duration::from_millis(200)).easing(
                                        Easing::EaseInOut
                                    )
                                )
                                .hover_style(|s, _theme: &Theme|
                                    s
                                        .background(Color::NEUTRAL_900)
                                        .border(
                                            Border::new(1, Color::NEUTRAL_800, Length::px(10.0))
                                        )
                                )
                                .pressed_style(|s, _theme: &Theme|
                                    s
                                        .background(Color::NEUTRAL_900)
                                        .scale(0.98)
                                        .content_scale(1.0)
                                        .border(
                                            Border::new(1, Color::NEUTRAL_800, Length::px(10.0))
                                        )
                                )
                        )
                        .child(
                            Button::new()
                                .label("Animated Button")
                                .font_size(15)
                                .background(Color::BLUE_500)
                                .padding(Edges::only(12, 8, 12, 8))
                                .border(Border::new(1, Color::BLUE_500, Length::px(8.0)))
                                .transition_all(Transition::new(Duration::from_millis(200)))
                                .hover_style(|s, _theme: &Theme|
                                    s.border(Border::new(1, Color::BLUE_500, Length::px(20.0)))
                                )
                        )
                        .child(
                            Button::new()
                                .label("disabled_button1")
                                .enabled(false)
                                .font_size(13)
                                .color(Color::NEUTRAL_500)
                                .background(Color::NEUTRAL_100)
                                .border(Border::new(1, Color::NEUTRAL_200, Length::px(8.0)))
                                .padding(Edges::only(9, 5, 9, 6))
                                .hover_style(|s, _theme: &Theme|
                                    s
                                        .background(Color::NEUTRAL_200)
                                        .border(Border::new(1, Color::NEUTRAL_300, Length::px(8.0)))
                                        .color(Color::NEUTRAL_600)
                                )
                                .pressed_style(|s, _theme: &Theme|
                                    s
                                        .background(Color::NEUTRAL_200)
                                        .border(Border::new(1, Color::NEUTRAL_400, Length::px(8.0)))
                                        .color(Color::NEUTRAL_700)
                                )
                                .disabled_style(|s, _theme: &Theme|
                                    s.background(Color::NEUTRAL_100).color(Color::NEUTRAL_400)
                                )
                        )
                        .child(
                            Image::new()
                                .bytes(
                                    include_bytes!(
                                        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/ferris.png")
                                    )
                                )
                                .object_fit(ObjectFit::Fill)
                                .width(160)
                                .height(105)
                        )
                )
        )
    });

    if let Err(e) = app.run() {
        eprintln!("Error running app: {:?}", e);
    }

    Ok(())
}
