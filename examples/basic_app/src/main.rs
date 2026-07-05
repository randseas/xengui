/*
 * Copyright (C) 2026 randseas
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program. If not, see <https://www.gnu.org/licenses/>.
 */
// examples/basic_app/main.rs
use xengui::{props, App, AppConfig, Text, TextProps, WindowPosition};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Start the app
    let mut app = App::new(AppConfig {
        title: "XenGui Basic App".into(),
        width: 640,
        height: 480,
        position: WindowPosition::Center,
        ..Default::default()
    });

    // Basic (fluent)
    app.add_node(Box::new(
        Text::new("text1").text("Hello, world!").scale(20.0),
    ));

    // Advanced (macro)
    let mut text2 = Text::new("text2");
    text2.set_props(props! {
        text: "Hello, world!",
        scale: 20.0,
        position: (0.0, 20.0),
        color: (0.0, 0.5, 0.5, 1.0)
    });
    app.add_node(Box::new(text2));

    app.run().unwrap();

    Ok(())
}
