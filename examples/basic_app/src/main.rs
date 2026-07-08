// SPDX-License-Identifier: Apache-2.0
use xengui::{
    AlignItems, App, AppConfig, Border, Color, Display, Edges, FlexDirection,
    JustifyContent, Length, StyleBuilder, Text, widgets::View,
};

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
        include_bytes!("../fonts/Inter_Regular.ttf").to_vec(),
    )
    .with_font(
        "Inter_Medium",
        include_bytes!("../fonts/Inter_Medium.ttf").to_vec(),
    )
    .with_font(
        "Inter_SemiBold",
        include_bytes!("../fonts/Inter_SemiBold.ttf").to_vec(),
    );

    let root = Box::new(
        View::new()
            .display(Display::Flex)
            .flex_direction(FlexDirection::Column)
            .justify_content(JustifyContent::Start)
            .align_items(AlignItems::Center)
            //.width(Length::Percent(1.0)) // 1.0 = %100
            //.height(Length::Percent(1.0)) // 1.0 = %100
            .padding(Edges::all(20.0))
            .background(Color::DARK_GRAY)
            .child(
                // Header section
                View::new()
                    .display(Display::Flex)
                    .flex_direction(FlexDirection::Row)
                    .justify_content(JustifyContent::Center)
                    .align_items(AlignItems::Center)
                    .size(760.0, 60.0)
                    .margin(Edges::all(20.0))
                    .background(Color::LIGHT_GRAY)
                    .border(Border::new(
                        Length::pixels(2.0),
                        Color::RED,
                        Length::pixels(16.0),
                    ))
                    .child(Text::new().text("Dashboard Header").color(Color::BLACK)),
            )
            .child(
                // Content area
                // Header section (Navbar with flex layout)
                View::new()
                    .display(Display::Flex)
                    .flex_direction(FlexDirection::Row)
                    .justify_content(JustifyContent::Start)
                    .align_items(AlignItems::Center)
                    .size(760.0, 60.0)
                    .margin(Edges::all(20.0))
                    .background(Color::LIGHT_GRAY)
                    .child(
                        // Logo (none / size defined by content)
                        View::new()
                            .padding(Edges::all(10.0))
                            .child(Text::new().text("Logo").color(Color::BLUE)),
                    )
                    .child(
                        // Arama Çubuğu (grow)
                        View::new()
                            .flex_grow(1.0)
                            .margin(Edges::all(5.0))
                            .background(Color::DARK_GRAY) // visual separation for input
                            .align_items(AlignItems::Center)
                            .justify_content(JustifyContent::Center)
                            .child(Text::new().text("Arama Çubuğu")),
                    )
                    .child(
                        // Profil (none / size defined by content)
                        View::new()
                            .padding(Edges::all(10.0))
                            .child(Text::new().text("Profil").color(Color::BLUE)),
                    ),
            ),
    );

    app.add_node(root);

    #[cfg(target_arch = "wasm32")]
    {
        if let Err(e) = app.run() {
            eprintln!("Error running app: {:?}", e);
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        app.run()?;
    }

    Ok(())
}
