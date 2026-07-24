// SPDX-License-Identifier: Apache-2.0
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hides console window on windows subsystem
use std::time::Duration;
use xenframe::{ App, AppConfig };

#[cfg(not(target_arch = "wasm32"))]
use xenframe::WindowPosition;
use xengui::{ properties::StyleValue, * };

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
        title: "Settings".into(),

        #[cfg(not(target_arch = "wasm32"))]
        width: 900,
        #[cfg(not(target_arch = "wasm32"))]
        height: 700,

        #[cfg(not(target_arch = "wasm32"))]
        position: WindowPosition::Center,
        #[cfg(not(target_arch = "wasm32"))]
        decorations: false,

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
        let (text, set_text) = use_state(String::from(""));

        Box::new(
            View::new()
                .font("Noto_Sans")
                .display(Display::Flex)
                .flex_direction(FlexDirection::Column)
                .width(Length::Percent(100.0))
                .height(Length::Percent(100.0))
                .background(|theme: &Theme| theme.background)
                .padding(Edges::all(15))
                .child(
                    Label::new()
                        .label("Settings")
                        .font_size(Length::px(18.0))
                        .color(|theme: &Theme| theme.foreground)
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
                        .gap(0, 4)
                        .child(
                            Label::new()
                                .label("label1")
                                .color(|theme: &Theme| theme.foreground)
                        )
                        .child(
                            TextBox::new()
                                .value(text.clone())
                                .color(|theme: &Theme| theme.foreground)
                                .placeholder("Search in settings...")
                                .font_size(14)
                                .outline(StyleValue::None)
                                .min_width(Length::px(180.0))
                                .transition_all(
                                    Transition::new(Duration::from_millis(200)).easing(
                                        Easing::EaseInOut
                                    )
                                )
                                .padding(Edges::all(8))
                                .background(|theme: &Theme| theme.surface)
                                .border(|theme: &Theme|
                                    Border::new(1, theme.border, Length::px(8.0))
                                )
                                .hover_style(|s, theme: &Theme|
                                    s.border(Border::new(1, theme.border_hover, Length::px(8.0)))
                                )
                                .focus_style(|s, theme: &Theme|
                                    s.border(Border::new(2, theme.primary, Length::px(8.0)))
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
                            View::new()
                                .color(Color::NEUTRAL_400)
                                .child(
                                    Svg::from_string(
                                        r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-wand-sparkles-icon lucide-wand-sparkles"><path d="m21.64 3.64-1.28-1.28a1.21 1.21 0 0 0-1.72 0L2.36 18.64a1.21 1.21 0 0 0 0 1.72l1.28 1.28a1.2 1.2 0 0 0 1.72 0L21.64 5.36a1.2 1.2 0 0 0 0-1.72"/><path d="m14 7 3 3"/><path d="M5 6v4"/><path d="M19 14v4"/><path d="M10 2v2"/><path d="M7 8H3"/><path d="M21 16h-4"/><path d="M11 3H9"/></svg>"#
                                    )
                                        .width(24)
                                        .height(24)
                                )
                        )
                        .child(
                            View::new()
                                .color(Color::NEUTRAL_400)
                                .child(
                                    Svg::from_string(
                                        r#" <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-qr-code-icon lucide-qr-code"><rect width="5" height="5" x="3" y="3" rx="1"/><rect width="5" height="5" x="16" y="3" rx="1"/><rect width="5" height="5" x="3" y="16" rx="1"/><path d="M21 16h-3a2 2 0 0 0-2 2v3"/><path d="M21 21v.01"/><path d="M12 7v3a2 2 0 0 1-2 2H7"/><path d="M3 12h.01"/><path d="M12 3h.01"/><path d="M12 16v.01"/><path d="M16 12h1"/><path d="M21 12v.01"/><path d="M12 21v-1"/></svg>"#
                                    )
                                        .width(24)
                                        .height(24)
                                )
                        )
                )
        )
    });

    if let Err(e) = app.run() {
        eprintln!("Error running app: {:?}", e);
    }

    Ok(())
}
