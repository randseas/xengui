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

#[macro_export]
macro_rules! impl_themed_style_builders {
    ($ty:ty; $($method:ident => $field:ident),+ $(,)?) => {
        impl $ty {
            $(
                pub fn $method(
                    mut self,
                    build: impl FnOnce($crate::StylePatch, &$crate::Theme) -> $crate::StylePatch
                ) -> Self {
                    let theme = $crate::current_theme();
                    self.$field = Some(build($crate::StylePatch::new(), &theme).build());
                    self.mark_dirty();
                    self
                }
            )+
        }
    };
    (base $ty:ty; $($method:ident => $field:ident),+ $(,)?) => {
        impl $ty {
            $(
                pub fn $method(
                    mut self,
                    build: impl FnOnce($crate::StylePatch, &$crate::Theme) -> $crate::StylePatch
                ) -> Self {
                    let theme = $crate::current_theme();
                    self.base.$field = Some(build($crate::StylePatch::new(), &theme).build());
                    self.mark_dirty();
                    self
                }
            )+
        }
    };
}

#[macro_export]
macro_rules! impl_common_style_builders {
    (base $ty:ty) => {
        impl $ty {
            /// Stable identity among siblings, kept across rebuilds even when this
            /// widget moves position (reorder, insert, remove). Use for list items
            /// instead of relying on array index.
            pub fn key(mut self, key: impl Into<smol_str::SmolStr>) -> Self {
                self.base.key = Some(key.into());
                self
            }

            pub fn font(mut self, font: impl Into<smol_str::SmolStr>) -> Self {
                self.base.style.font = Some(font.into());
                self.mark_dirty();
                self
            }

            pub fn hover_background<M>(
                mut self,
                background: impl $crate::IntoThemed<$crate::Background, M>,
            ) -> Self {
                self.base.hover_style.get_or_insert_with($crate::Style::default).background =
                    Some(background.resolve_themed());
                self.mark_dirty();
                self
            }

            pub fn pressed_background<M>(
                mut self,
                background: impl $crate::IntoThemed<$crate::Background, M>,
            ) -> Self {
                self.base.pressed_style.get_or_insert_with($crate::Style::default).background =
                    Some(background.resolve_themed());
                self.mark_dirty();
                self
            }

            pub fn disabled_background<M>(
                mut self,
                background: impl $crate::IntoThemed<$crate::Background, M>,
            ) -> Self {
                self.base.disabled_style.get_or_insert_with($crate::Style::default).background =
                    Some(background.resolve_themed());
                self.mark_dirty();
                self
            }

            pub fn enabled(mut self, enabled: bool) -> Self {
                self.base.interaction.set_enabled(enabled);
                self.mark_dirty();
                self
            }
        }
    };
}

/// Generates the `Widget` trait accessor methods every `base: WidgetBase` +
/// `layout_box: LayoutBox` widget needs (identity, dirty flag, style
/// pointers, interaction, layout box). Widget-specific behavior
/// (`debug_name`, `children`, `measure`, `paint`, `event`, ...) is still
/// implemented by hand next to this macro call.
#[macro_export]
macro_rules! impl_widget_boilerplate {
    () => {
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }

        fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
            self
        }

        fn get_key(&self) -> Option<&smol_str::SmolStr> {
            self.base.key.as_ref()
        }

        fn is_dirty(&self) -> bool {
            self.base.dirty
        }

        fn set_dirty(&mut self, dirty: bool) {
            self.base.dirty = dirty;
        }

        fn style(&self) -> &$crate::Style {
            &self.base.style
        }

        fn style_mut(&mut self) -> &mut $crate::Style {
            &mut self.base.style
        }

        fn computed_style(&self) -> &$crate::Style {
            &self.base.computed_style
        }

        fn interaction(&self) -> Option<&$crate::Interaction> {
            Some(&self.base.interaction)
        }

        fn interaction_mut(&mut self) -> Option<&mut $crate::Interaction> {
            Some(&mut self.base.interaction)
        }

        fn layout(&mut self, rect: $crate::LayoutBox) {
            self.layout_box = rect;
        }

        fn layout_box(&self) -> &$crate::LayoutBox {
            &self.layout_box
        }
    };
}
