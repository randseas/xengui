#[component]
fn PageName() -> impl Widget {
    let count = use_state(|| 0);

    fn increment() {
        count.set(count + 1)
    };

    xui! {
        <View>
          <Label style={ color: "#ffffff" }>Count: {count.get()}</Label>
          <Button onClick={increment}>Tıkla!</Button>
        </View>
    }
}