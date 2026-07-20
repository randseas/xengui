// SPDX-License-Identifier: Apache-2.0
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use std::time::Duration;

#[cfg(not(target_arch = "wasm32"))]
use xenframe::WindowPosition;
use xengui::{ properties::StyleValue, * };
use xenframe::{ App, AppConfig };
use xen_clipboard::Clipboard;

// write debug messages directly into the screen
#[cfg(target_arch = "wasm32")]
fn show_debug_overlay(message: &str) {
    let Some(document) = web_sys::window().and_then(|w| w.document()) else {
        return;
    };
    let Some(body) = document.body() else {
        return;
    };
    let Ok(overlay) = document.create_element("pre") else {
        return;
    };
    let _ = overlay.set_attribute(
        "style",
        "position:fixed;inset:0;margin:0;background:rgba(26,0,0,0.75);color:#ff8080;\
         font:12px/1.5 monospace;padding:16px;white-space:pre-wrap;\
         z-index:2147483647;overflow:auto;"
    );
    overlay.set_text_content(Some(message));
    let _ = body.append_child(&overlay);
}

// write debug messages directly into the screen
#[cfg(target_arch = "wasm32")]
fn install_panic_hook() {
    std::panic::set_hook(
        Box::new(|info| {
            console_error_panic_hook::hook(info);
            show_debug_overlay(&format!("xengui panicked:\n\n{info}"));
        })
    );
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(target_arch = "wasm32")]
    {
        // console_error_panic_hook::set_once();
        install_panic_hook();
        let _ = console_log::init_with_level(log::Level::Info);
        // TEST: overlay
        // show_debug_overlay("xengui: overlay initialized");
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
        position: WindowPosition::Center,

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
                .child(
                    View::new()
                        .flex_direction(FlexDirection::Column)
                        .width(Length::Percent(100.0))
                        .height(Length::Percent(100.0))
                        .align_items(AlignItems::Center)
                        .justify_content(JustifyContent::Center)
                        .child(
                            Label::new()
                                .selectable(true)
                                .label("My XenGui Application")
                                .margin(Edges::only(0, 0, 0, 10))
                                .font_size(20)
                                .color(|theme: &Theme| theme.foreground)
                        )
                        .child(
                            TextBox::new()
                                .value(text.clone())
                                .color(|theme: &Theme| theme.foreground)
                                .placeholder("Enter your name...")
                                .font_size(15)
                                .outline(StyleValue::None)
                                .width(Length::px(150.0))
                                .padding(Edges::all(8))
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
                                .font_size(15)
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
                                        .font_size(15)
                                        .color(|theme: &Theme| theme.foreground)
                                        .background(|theme: &Theme| theme.surface)
                                        .border(|theme: &Theme|
                                            Border::new(1, theme.border, Length::px(8.0))
                                        )
                                        .padding(Edges::only(8, 5, 8, 5))
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
                                        .font_size(15)
                                        .color(|theme: &Theme| theme.foreground)
                                        .background(|theme: &Theme| theme.surface)
                                        .border(|theme: &Theme|
                                            Border::new(1, theme.border, Length::px(8.0))
                                        )
                                        .padding(Edges::only(8, 5, 8, 5))
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
