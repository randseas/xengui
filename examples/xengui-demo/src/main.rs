// SPDX-License-Identifier: Apache-2.0
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use std::time::Duration;

#[cfg(not(target_arch = "wasm32"))]
use xenframe::WindowPosition;
use xengui::{ Display::Flex, FlexDirection::Column, properties::StyleValue, widgets::Link, * };
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
        "position:fixed;inset:0;margin:0;background:rgba(0,0,0,0);color:#ff8080;\
         font:12px/1.5 monospace;padding:16px;white-space:pre-wrap;\
         z-index:2147483647;overflow:auto;pointer-events:none;"
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
        title: "XenGui App".into(),
        reload_shortcut: true,

        #[cfg(not(target_arch = "wasm32"))]
        width: 640,
        #[cfg(not(target_arch = "wasm32"))]
        height: 480,
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
        xen_router::Router
            ::new()
            .route("/", |_|
                Box::new(
                    View::new()
                        .font("Noto_Sans")
                        .display(Flex)
                        .flex_direction(Column)
                        .gap(0, 4)
                        .child(Label::new().label("Current page: / (home)"))
                        .child(xen_router::router_link("/").label("Home"))
                        .child(xen_router::router_link("/docs").label("Docs"))
                        .child(xen_router::router_link("/docs/xenframe").label("Docs (xenframe)"))
                        .child(xen_router::router_link("/users/42").label("Users :42"))
                        .child(
                            xen_router::router_link("/test/string_test").label("Test :string_test")
                        )
                )
            )
            .route("/docs", |_|
                Box::new(
                    View::new()
                        .font("Noto_Sans")
                        .display(Flex)
                        .flex_direction(Column)
                        .gap(0, 4)
                        .child(Label::new().label("Current page: /docs (docs)"))
                        .child(xen_router::router_link("/").label("Home"))
                        .child(xen_router::router_link("/docs").label("Docs"))
                        .child(xen_router::router_link("/docs/xenframe").label("Docs (xenframe)"))
                        .child(xen_router::router_link("/users/42").label("Users :42"))
                        .child(
                            xen_router::router_link("/test/string_test").label("Test :string_test")
                        )
                )
            )
            .route("/docs/xenframe", |_|
                Box::new(
                    View::new()
                        .font("Noto_Sans")
                        .display(Flex)
                        .flex_direction(Column)
                        .gap(0, 4)
                        .child(Label::new().label("Current page: /docs/xenframe (xenframe docs)"))
                        .child(xen_router::router_link("/").label("Home"))
                        .child(xen_router::router_link("/docs").label("Docs"))
                        .child(xen_router::router_link("/docs/xenframe").label("Docs (xenframe)"))
                        .child(xen_router::router_link("/users/42").label("Users :42"))
                        .child(
                            xen_router::router_link("/test/string_test").label("Test :string_test")
                        )
                )
            )
            .route("/users/:id", |p| {
                Box::new(
                    View::new()
                        .font("Noto_Sans")
                        .display(Flex)
                        .flex_direction(Column)
                        .gap(0, 4)
                        .child(Label::new().label("Current page: /users/:id (users)"))
                        .child(Label::new().label(format!("User: {}", p.get("id").unwrap_or(""))))
                        .child(xen_router::router_link("/").label("Home"))
                        .child(xen_router::router_link("/docs").label("Docs"))
                        .child(xen_router::router_link("/docs/xenframe").label("Docs (xenframe)"))
                        .child(xen_router::router_link("/users/42").label("Users :42"))
                        .child(
                            xen_router::router_link("/test/string_test").label("Test :string_test")
                        )
                )
            })
            .route("/test/:string", |p| {
                Box::new(
                    View::new()
                        .font("Noto_Sans")
                        .display(Flex)
                        .flex_direction(Column)
                        .gap(0, 4)
                        .child(Label::new().label("Current page: /test/:string (test)"))
                        .child(
                            Label::new().label(
                                format!("Test string: {}", p.get("string").unwrap_or(""))
                            )
                        )
                        .child(xen_router::router_link("/").label("Home"))
                        .child(xen_router::router_link("/docs").label("Docs"))
                        .child(xen_router::router_link("/docs/xenframe").label("Docs (xenframe)"))
                        .child(xen_router::router_link("/users/42").label("Users :42"))
                        .child(
                            xen_router::router_link("/test/string_test").label("Test :string_test")
                        )
                )
            })
            .not_found(|| Box::new(Label::new().label("404")))
            .build()
    });

    /*app.render(|| {
        let clipboard = Clipboard::default();
        let (text, set_text) = use_state(String::from("Ferris"));
        let (counter, set_counter) = use_state::<i32>(12);

        let inc: SetState<i32> = set_counter.clone();
        let dec: SetState<i32> = set_counter.clone();

        Box::new(
            ContextMenu::new()
                .item(
                    ContextMenuItem::new("Back")
                        .shortcut("Ctrl+B")
                        .enabled(false)
                        .on_click(|_ctx| {
                            log::info!("context menu -> back");
                        })
                )
                .item(
                    ContextMenuItem::new("Forward")
                        .shortcut("Ctrl+F")
                        .enabled(true)
                        .on_click(|_ctx| {
                            log::info!("context menu -> forward");
                        })
                )
                .item(
                    ContextMenuItem::new("Reload")
                        .shortcut("Ctrl+R")
                        .on_click(|_ctx| {
                            log::info!("context menu -> reload");
                            xenframe::request_reload();
                        })
                )
                .divider()
                .item(
                    ContextMenuItem::new("New")
                        .shortcut("Ctrl+N")
                        .submenu_item(ContextMenuItem::new("Text file"))
                        .submenu_item(ContextMenuItem::new("HTML file"))
                        .submenu_divider()
                        .submenu_item(ContextMenuItem::new("JS file"))
                        .submenu_item(ContextMenuItem::new("Rust file"))
                        .on_click(|_ctx| {
                            log::info!("context menu -> new file");
                        })
                )
                .divider()
                .item(
                    ContextMenuItem::new("Inspect").on_click(|_ctx| {
                        log::info!("context menu -> inspect");
                    })
                )
                .font("Noto_Sans")
                .item_border(Border::new(0, Color::TRANSPARENT, 6))
                .padding(6.0)
                .font_size(13)
                .menu_min_width(240.0)
                .menu_background(|theme: &Theme| theme.surface)
                .border(|theme: &Theme| Border::new(1, theme.border, Length::px(10.0)))
                .child(
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
                                            s.border(
                                                Border::new(1, theme.border_hover, Length::px(8.0))
                                            )
                                        )
                                        .focus_style(|s, theme|
                                            s.border(Border::new(2, theme.primary, Length::px(8.0)))
                                        )
                                        .on_change(move |value, _ctx|
                                            set_text.set(value.to_string())
                                        )
                                        .on_submit(move |value, _ctx| {
                                            clipboard.set_text(value.to_string(), move |result| {
                                                match result {
                                                    Ok(_) => log::info!("clipboard -> copied"),
                                                    Err(err) =>
                                                        log::error!("clipboard -> failed: {err}"),
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
                                                    Transition::new(
                                                        Duration::from_millis(200)
                                                    ).easing(Easing::EaseInOut)
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
                                                            Border::new(
                                                                1,
                                                                theme.border,
                                                                Length::px(8.0)
                                                            )
                                                        )
                                                        .color(theme.foreground)
                                                )
                                                .pressed_style(|s, theme|
                                                    s
                                                        .background(theme.pressed)
                                                        .border(
                                                            Border::new(
                                                                1,
                                                                theme.pressed,
                                                                Length::px(8.0)
                                                            )
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
                                                    Transition::new(
                                                        Duration::from_millis(200)
                                                    ).easing(Easing::EaseInOut)
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
                                                            Border::new(
                                                                1,
                                                                theme.border,
                                                                Length::px(8.0)
                                                            )
                                                        )
                                                        .color(theme.foreground)
                                                )
                                                .pressed_style(|s, theme|
                                                    s
                                                        .background(theme.pressed)
                                                        .border(
                                                            Border::new(
                                                                1,
                                                                theme.pressed,
                                                                Length::px(8.0)
                                                            )
                                                        )
                                                        .color(theme.foreground)
                                                )
                                        )
                                )
                                .child(
                                    Image::new()
                                        .bytes(
                                            include_bytes!(
                                                concat!(
                                                    env!("CARGO_MANIFEST_DIR"),
                                                    "/assets/ferris.png"
                                                )
                                            )
                                        )
                                        .object_fit(ObjectFit::Fill)
                                        .width(160)
                                        .height(105)
                                )
                        )
                )
        )
    });*/

    if let Err(e) = app.run() {
        eprintln!("Error running app: {:?}", e);
    }

    Ok(())
}
