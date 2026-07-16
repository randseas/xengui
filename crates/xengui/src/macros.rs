// SPDX-License-Identifier: Apache-2.0

pub trait WidgetContent: Sized {
    fn with_content(self, content: impl Into<smol_str::SmolStr>) -> Self;
}

impl WidgetContent for crate::Label {
    fn with_content(self, content: impl Into<smol_str::SmolStr>) -> Self {
        self.label(content)
    }
}

/// ```ignore
/// view! {
///     View {
///         width: 400,
///         height: 300,
///         padding: 20,
///         background: Color::BLACK,
///         Label("Hello"),
///         View { flex_direction: FlexDirection::Row, Label("World") }
///     }
/// }
/// ```
#[macro_export]
macro_rules! view {
    (
        $widget:ident { $($rest:tt)* }
    ) => {
        $crate::view_props!( $widget::new() ; $($rest)* )
    };
    ($widget:ident($content:expr)) => {
        $crate::WidgetContent::with_content($widget::new(), $content)
    };
}

#[macro_export]
macro_rules! view_props {
    ($acc:expr;) => { $acc };

    // prop: (a, b, ...) - for builder methods taking multiple positional
    // arguments (e.g. `gap: (4, 0)` -> `.gap(4, 0)`), distinct from a
    // single expr that happens to be a parenthesized tuple.
    ($acc:expr; $key:ident: ($($val:expr),+ $(,)?)) => {
        $acc.$key($($val),+)
    };
    (
        $acc:expr;
        $key:ident: ($($val:expr),+ $(,)?),
        $($rest:tt)*
    ) => {
        $crate::view_props!( $acc.$key($($val),+) ; $($rest)* )
    };

    // Child { ... }
    (
        $acc:expr;
        $widget:ident { $($inner:tt)* }
    ) => {
        {
        let mut __parent = $acc;
        let __child = $crate::view_props!( $widget::new() ; $($inner)* );
        __parent = __parent.child(__child);
        __parent
        }
    };
    // Child { ... }
    (
        $acc:expr;
        $widget:ident { $($inner:tt)* },
        $($rest:tt)*
    ) => {
        $crate::view_props!({
            let mut __parent = $acc;
            let __child = $crate::view_props!( $widget::new() ; $($inner)* );
            __parent = __parent.child(__child);
            __parent
        } ; $($rest)*)
    };

    // Child(expr)
    ($acc:expr; $widget:ident($content:expr)) => {
        {
        let mut __parent = $acc;
        let __child = $crate::WidgetContent::with_content($widget::new(), $content);
        __parent = __parent.child(__child);
        __parent
        }
    };
    // Child(expr)
    (
        $acc:expr;
        $widget:ident($content:expr),
        $($rest:tt)*
    ) => {
        $crate::view_props!({
            let mut __parent = $acc;
            let __child = $crate::WidgetContent::with_content($widget::new(), $content);
            __parent = __parent.child(__child);
            __parent
        } ; $($rest)*)
    };
}
