// SPDX-License-Identifier: Apache-2.0
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use std::time::Duration;

use xengui::{ properties::StyleValue, * };
use xen_clipboard::Clipboard;

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
        title: "XenGui App".into(),
        #[cfg(not(target_arch = "wasm32"))]
        width: 640,
        #[cfg(not(target_arch = "wasm32"))]
        height: 480,
        #[cfg(not(target_arch = "wasm32"))]
        position: xengui::WindowPosition::Center,

        themes: vec![
            Theme::new("pearl_light")
                .mode(ThemeMode::Light)
                .primary(Color::VIOLET_500)
                .accent(Color::VIOLET_400)

                .background(Color::NEUTRAL_50)
                .surface(Color::NEUTRAL_200)

                .foreground(Color::NEUTRAL_800)
                .foreground_muted(Color::NEUTRAL_400)

                .border(Color::NEUTRAL_800)
                .border_hover(Color::NEUTRAL_700)

                .hover(Color::NEUTRAL_800)
                .pressed(Color::NEUTRAL_700)
                .disabled(Color::NEUTRAL_600)

                .selection(Color::VIOLET_500.with_alpha(80))
                .selection_color(Color::VIOLET_800)
                .caret_color(Color::VIOLET_500),

            Theme::new("pearl_dark")
                .mode(ThemeMode::Dark)
                .primary(Color::VIOLET_500)
                .accent(Color::VIOLET_400)

                .background(Color::NEUTRAL_950)
                .surface(Color::NEUTRAL_900)

                .foreground(Color::NEUTRAL_100)
                .foreground_muted(Color::NEUTRAL_400)

                .border(Color::NEUTRAL_800)
                .border_hover(Color::NEUTRAL_700)

                .hover(Color::NEUTRAL_800)
                .pressed(Color::NEUTRAL_700)
                .disabled(Color::NEUTRAL_600)

                .selection(Color::VIOLET_500.with_alpha(80))
                .selection_color(Color::VIOLET_200)
                .caret_color(Color::VIOLET_500)
        ],
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
        let clipboard = Clipboard::default();
        let (text, set_text) = use_state(String::from("Ferris"));
        let (counter, set_counter) = use_state::<i32>(12);

        let inc: SetState<i32> = set_counter.clone();
        let dec: SetState<i32> = set_counter.clone();

        Box::new(
            View::new()
                .font("Noto_Sans")
                .display(Display::Flex)
                .flex_direction(FlexDirection::Column)
                .justify_content(JustifyContent::Start)
                .align_items(AlignItems::Start)
                .width(Length::Percent(100.0))
                .height(Length::Percent(100.0))
                .background(|theme: &Theme| theme.background)
                .selection_background(|theme: &Theme| theme.selection)
                .selection_color(|theme: &Theme| theme.selection_color)
                .caret_color(|theme: &Theme| theme.caret_color)
                .child(
                    View::new()
                        .flex_direction(FlexDirection::Column)
                        .width(Length::Percent(100.0))
                        .height(Length::Percent(100.0))
                        .align_items(AlignItems::Center)
                        .justify_content(JustifyContent::Center)
                        .child(
                            Label::new()
                                .label("My XenGui Application")
                                .margin(Edges::only(0, 0, 0, 8))
                                .font_size(20)
                                .color(|theme: &Theme| theme.foreground)
                        )
                        .child(
                            TextBox::new()
                                .enabled(false)
                                .value(text.clone())
                                .color(|theme: &Theme| theme.foreground)
                                .placeholder("Enter your name...")
                                .font_size(16)
                                .outline(StyleValue::None)
                                .focus_outline(StyleValue::Default)
                                .width(Length::px(150.0))
                                .padding(Edges::only(10, 7, 10, 8))
                                .transition_all(
                                    Transition::new(Duration::from_millis(200)).easing(
                                        Easing::EaseInOut
                                    )
                                )
                                .background(|theme: &Theme| theme.surface)
                                .border(|theme: &Theme|
                                    Border::new(1, theme.border, Length::px(8.0))
                                )
                                .hover_style(|s, theme|
                                    s.border(Border::new(1, theme.border_hover, Length::px(8.0)))
                                )
                                .focus_style(|s, theme|
                                    s.border(Border::new(2, theme.primary, Length::px(8.0)))
                                )
                                .on_change(move |value, _ctx| set_text.set(value.to_string()))
                                .on_submit(move |value, _ctx| {
                                    clipboard.set_text(value.to_string(), move |result| {
                                        match result {
                                            Ok(_) => log::info!("clipboard -> copied"),
                                            Err(err) => log::error!("clipboard -> failed: {err}"),
                                        }
                                    });
                                })
                        )
                        .child(
                            Label::new()
                                .label(format!("Hello {text}, age {counter}"))
                                .font_size(16)
                                .color(|theme: &Theme| theme.foreground)
                                .margin(Edges::only(0, 6, 0, 0))
                        )
                        .child(
                            View::new()
                                .flex_direction(FlexDirection::Row)
                                .gap(4, 0)
                                .child(
                                    Button::new()
                                        .label("Increment")
                                        .font_size(16)
                                        .color(|theme: &Theme| theme.foreground)
                                        .background(|theme: &Theme| theme.surface)
                                        .border(|theme: &Theme|
                                            Border::new(1, theme.border, Length::px(8.0))
                                        )
                                        .padding(Edges::only(9, 4, 9, 7))
                                        .margin(Edges::only(0, 10, 0, 0))
                                        .transition_all(
                                            Transition::new(Duration::from_millis(200)).easing(
                                                Easing::EaseInOut
                                            )
                                        )
                                        .on_click(move |_ctx|
                                            inc.update(|v| {
                                                *v += 1;
                                            })
                                        )
                                        .hover_style(|s, theme|
                                            s
                                                .background(theme.hover)
                                                .border(
                                                    Border::new(1, theme.border, Length::px(8.0))
                                                )
                                                .color(theme.foreground)
                                        )
                                        .pressed_style(|s, theme|
                                            s
                                                .background(theme.pressed)
                                                .border(
                                                    Border::new(1, theme.pressed, Length::px(8.0))
                                                )
                                                .color(theme.foreground)
                                        )
                                )
                                .child(
                                    Button::new()
                                        .label("Decrement")
                                        .font_size(16)
                                        .color(|theme: &Theme| theme.foreground)
                                        .background(|theme: &Theme| theme.surface)
                                        .border(|theme: &Theme|
                                            Border::new(1, theme.border, Length::px(8.0))
                                        )
                                        .padding(Edges::only(9, 4, 9, 7))
                                        .margin(Edges::only(0, 10, 0, 0))
                                        .transition_all(
                                            Transition::new(Duration::from_millis(200)).easing(
                                                Easing::EaseInOut
                                            )
                                        )
                                        .on_click(move |_ctx|
                                            dec.update(|v| {
                                                *v -= 1;
                                            })
                                        )
                                        .hover_style(|s, theme|
                                            s
                                                .background(theme.hover)
                                                .border(
                                                    Border::new(1, theme.border, Length::px(8.0))
                                                )
                                                .color(theme.foreground)
                                        )
                                        .pressed_style(|s, theme|
                                            s
                                                .background(theme.pressed)
                                                .border(
                                                    Border::new(1, theme.pressed, Length::px(8.0))
                                                )
                                                .color(theme.foreground)
                                        )
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
