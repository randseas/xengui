use xengui::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = AppConfig {
        title: "My XenGui App".into(),
        width: 640,
        height: 480,
        position: WindowPosition::Center,
        ..Default::default()
    };

    let mut app = App::new(config);

    app.render(|| {
        let (counter, set_counter) = use_state::<i32>(0);

        Box::new(
            View::new()
                .display(Display::Flex)
                .flex_direction(FlexDirection::Column)
                .align_items(AlignItems::Center)
                .justify_content(JustifyContent::Center)
                .width(Length::Percent(100.0))
                .height(Length::Percent(100.0))
                .background(Color::WHITE)
                .child(
                    Label::new()
                        .label(format!("Count: {counter}"))
                        .font_size(20)
                        .color(Color::NEUTRAL_700)
                )
                .child(
                    Button::new()
                        .label("Increment")
                        .padding(Edges::symmetric(12, 8))
                        .background(Color::NEUTRAL_200)
                        .on_click(move |_ctx| set_counter.set(counter + 1))
                )
        )
    });

    app.run()?;
    Ok(())
}
