use crate::properties::DEFAULT_CURSOR_ICON;
// SPDX-License-Identifier: Apache-2.0
use crate::{
    Background,
    Constraints,
    EventCtx,
    EventStatus,
    InputEvent,
    Interaction,
    Key,
    KeyState,
    LayoutBox,
    MeasureContext,
    MeasureResult,
    PaintContext,
    Style,
    StyleBuilder,
    StylePatch,
    TextCommand,
    Widget,
    WidgetContent,
    properties::DEFAULT_FONT_SIZE,
    properties::DEFAULT_LINK_COLOR,
    TextDecoration,
};
use smol_str::SmolStr;
use std::cell::Cell;
use winit::event::{ ElementState, MouseButton };

pub struct Link {
    key: Option<SmolStr>,

    dirty: bool,
    style: Style,
    inherited_style: Style,
    computed_style: Style,

    hover_style: Option<Style>,
    pressed_style: Option<Style>,
    disabled_style: Option<Style>,

    interaction: Interaction,
    selectable: bool,

    content: SmolStr,
    layout_box: LayoutBox,
    content_size: Cell<(f32, f32)>,
    measured_max_width: Cell<Option<f32>>,

    href: Option<SmolStr>,
    target_blank: bool,
}

impl Link {
    pub fn new() -> Self {
        let mut interaction = Interaction::new();
        interaction.focusable = true;
        interaction.hover_cursor = Some(DEFAULT_CURSOR_ICON);

        let mut link = Self {
            key: None,

            dirty: true,
            style: Style::default(),
            inherited_style: Style::default(),
            computed_style: Style::default(),

            hover_style: None,
            pressed_style: None,
            disabled_style: None,

            interaction,
            selectable: false,

            content: SmolStr::new(""),
            layout_box: LayoutBox::default(),
            content_size: Cell::new((0.0, 0.0)),
            measured_max_width: Cell::new(None),

            href: None,
            target_blank: false,
        };

        link.recompute_style();
        link
    }

    /// Stable identity among siblings, kept across rebuilds even when this
    /// widget moves position (reorder, insert, remove). Use for list items
    /// instead of relying on array index.
    pub fn key(mut self, key: impl Into<SmolStr>) -> Self {
        self.key = Some(key.into());
        self
    }

    // Builder methods
    pub fn label(mut self, content: impl Into<SmolStr>) -> Self {
        self.content = content.into();
        self.mark_dirty();
        self
    }

    pub fn font(mut self, font: impl Into<SmolStr>) -> Self {
        self.style.font = Some(font.into());
        self.mark_dirty();
        self
    }

    pub fn selectable(mut self, selectable: bool) -> Self {
        self.selectable = selectable;
        self.mark_dirty();
        self
    }

    /// Clicking or activating the link opens this URL through the system
    /// browser (native targets) or `window.open` (wasm32).
    pub fn href(mut self, href: impl Into<SmolStr>) -> Self {
        self.href = Some(href.into());
        self.mark_dirty();
        self
    }

    /// Only affects wasm32 - opens the href in a new tab instead of the
    /// current one.
    pub fn target_blank(mut self, value: bool) -> Self {
        self.target_blank = value;
        self
    }

    /// Full style overlay to be applied during hover state - includes every field of Style
    /// such as background, border, color, font_size, padding, margin, etc.
    /// Only the fields you provide will overwrite the base style.
    ///
    /// ```ignore
    /// Link::new()
    ///     .label("Read more")
    ///     .hover_style(|s| s
    ///         .text_decoration(TextDecoration::Underline)
    ///     )
    /// ```
    pub fn hover_style(mut self, build: impl FnOnce(StylePatch) -> StylePatch) -> Self {
        self.hover_style = Some(build(StylePatch::new()).build());
        self.mark_dirty();
        self
    }

    /// Full style overlay to be applied during pressed state - includes every field of Style
    /// such as background, border, color, font_size, padding, margin, etc.
    /// Only the fields you provide will overwrite the base style.
    ///
    /// ```ignore
    /// Link::new()
    ///     .label("Read more")
    ///     .pressed_style(|s| s
    ///         .color(Color::NEUTRAL_600)
    ///     )
    /// ```
    pub fn pressed_style(mut self, build: impl FnOnce(StylePatch) -> StylePatch) -> Self {
        self.pressed_style = Some(build(StylePatch::new()).build());
        self.mark_dirty();
        self
    }

    /// Full style overlay to be applied during disabled state - includes every field of Style
    /// such as background, border, color, font_size, padding, margin, etc.
    /// Only the fields you provide will overwrite the base style.
    ///
    /// ```ignore
    /// Link::new()
    ///     .label("Read more")
    ///     .disabled_style(|s| s
    ///         .color(Color::NEUTRAL_400)
    ///     )
    /// ```
    pub fn disabled_style(mut self, build: impl FnOnce(StylePatch) -> StylePatch) -> Self {
        self.disabled_style = Some(build(StylePatch::new()).build());
        self.mark_dirty();
        self
    }

    pub fn hover_background<B: Into<Background>>(mut self, background: B) -> Self {
        self.hover_style.get_or_insert_with(Style::default).background = Some(background.into());
        self.mark_dirty();
        self
    }

    pub fn pressed_background<B: Into<Background>>(mut self, background: B) -> Self {
        self.pressed_style.get_or_insert_with(Style::default).background = Some(background.into());
        self.mark_dirty();
        self
    }

    pub fn disabled_background<B: Into<Background>>(mut self, background: B) -> Self {
        self.disabled_style.get_or_insert_with(Style::default).background = Some(background.into());
        self.mark_dirty();
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.interaction.set_enabled(enabled);
        self.mark_dirty();
        self
    }

    fn recompute_style(&mut self) {
        let patch = if !self.interaction.enabled {
            self.disabled_style.as_ref()
        } else if self.interaction.pressed {
            self.pressed_style.as_ref().or(self.hover_style.as_ref())
        } else if self.interaction.hovered {
            self.hover_style.as_ref()
        } else {
            None
        };

        let base = self.inherited_style.inherit_style(&self.style);

        self.computed_style = match patch {
            Some(patch) => base.overlay(patch),
            None => base,
        };

        self.interaction.hover_cursor = self.computed_style.cursor
            .map(crate::Cursor::to_winit)
            .or(Some(DEFAULT_CURSOR_ICON));
    }

    fn open_href(&self) {
        let Some(href) = &self.href else {
            return;
        };

        let url = normalize_url(href);

        #[cfg(target_arch = "wasm32")]
        {
            if let Some(window) = web_sys::window() {
                let target = if self.target_blank { "_blank" } else { "_self" };
                let _ = window.open_with_url_and_target(&url, target);
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            open_native(&url);
        }
    }
}

impl Default for Link {
    fn default() -> Self {
        Self::new()
    }
}

impl StyleBuilder for Link {
    fn style_mut(&mut self) -> &mut Style {
        &mut self.style
    }

    fn mark_dirty(&mut self) {
        self.dirty = true;
        self.recompute_style();
    }
}

impl WidgetContent for Link {
    fn with_content(self, content: impl Into<SmolStr>) -> Self {
        self.label(content)
    }
}

crate::impl_interaction_builders!(Link);

impl Widget for Link {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn debug_name(&self) -> &'static str {
        "Widget#Link"
    }

    fn get_key(&self) -> Option<&SmolStr> {
        self.key.as_ref()
    }

    fn is_dirty(&self) -> bool {
        self.dirty
    }

    fn set_dirty(&mut self, dirty: bool) {
        self.dirty = dirty;
    }

    fn style(&self) -> &Style {
        &self.style
    }

    fn style_mut(&mut self) -> &mut Style {
        &mut self.style
    }

    fn computed_style(&self) -> &Style {
        &self.computed_style
    }

    fn children(&self) -> &[Box<dyn Widget>] {
        &[]
    }

    fn interaction(&self) -> Option<&Interaction> {
        Some(&self.interaction)
    }

    fn interaction_mut(&mut self) -> Option<&mut Interaction> {
        Some(&mut self.interaction)
    }

    fn layout(&mut self, rect: LayoutBox) {
        self.layout_box = rect;
    }

    fn layout_box(&self) -> &LayoutBox {
        &self.layout_box
    }

    fn measure(&self, ctx: &mut MeasureContext, constraints: Constraints) -> MeasureResult {
        let scale_factor = ctx.scale_factor;
        let style = &self.computed_style;

        let font_size = style.font_size
            .map(|s| s.to_physical(scale_factor))
            .unwrap_or(DEFAULT_FONT_SIZE.to_physical(scale_factor));

        let letter_spacing = style.letter_spacing
            .map(|ls| ls.value().to_physical(scale_factor))
            .unwrap_or(0.0);

        let line_height = style.line_height
            .map(|lh| lh.value().to_physical(scale_factor))
            .unwrap_or(0.0);

        self.measured_max_width.set(constraints.max_width);

        let result = ctx.text.measure(
            &self.content,
            style.font.as_deref(),
            font_size,
            style.font_weight.unwrap_or_default(),
            style.font_style.unwrap_or_default(),
            letter_spacing,
            line_height,
            constraints.max_width
        );

        self.content_size.set((result.width, result.height));

        let padding = style.padding.unwrap_or_default();
        let width = result.width + padding.left.value() + padding.right.value();
        let height = result.height + padding.top.value() + padding.bottom.value();
        let (width, height) = constraints.constrain_size(width, height);

        MeasureResult {
            width,
            height,
            baseline: result.baseline,
        }
    }

    fn paint(&self, ctx: &mut PaintContext) {
        let style = &self.computed_style;

        log::trace!(
            "paint -> '{}' x={} y={} dirty={:?}",
            self.content,
            self.layout_box.x,
            self.layout_box.y,
            self.is_dirty()
        );

        self.paint_box(ctx);
        self.paint_outline(ctx);
        self.paint_focus(ctx);

        let padding = style.padding.unwrap_or_default();

        let text_x = self.layout_box.x + padding.left.value();
        let text_y = self.layout_box.y + padding.top.value();

        let mut text_style = style.clone();
        text_style.font_size.get_or_insert(DEFAULT_FONT_SIZE);
        text_style.color.get_or_insert(DEFAULT_LINK_COLOR);
        if self.interaction.hovered {
            text_style.text_decoration.get_or_insert(TextDecoration::UNDERLINE);
        }

        ctx.draw_text(TextCommand {
            text: self.content.clone(),
            position: (text_x, text_y),
            style: text_style,
            max_width: self.measured_max_width.get(),
            clip_rect: None,
        });
    }

    fn event(&mut self, event: &InputEvent, ctx: &mut EventCtx) -> EventStatus {
        if !self.interaction.is_active() {
            return EventStatus::Ignored;
        }

        let is_click = match event {
            InputEvent::MouseInput {
                state: ElementState::Released,
                button: MouseButton::Left,
                ..
            } => self.interaction.pressed && self.interaction.hovered,
            InputEvent::KeyInput { event: key_event, .. } =>
                self.interaction.focused &&
                    !key_event.repeat &&
                    key_event.state == KeyState::Pressed &&
                    matches!(key_event.key, Key::Enter | Key::Space),
            _ => false,
        };

        let status = self.interaction.handle(event, ctx);

        if is_click {
            self.open_href();
        }

        if matches!(status, EventStatus::Handled) {
            self.recompute_style();
            self.dirty = true;
        }

        status
    }

    fn content_eq(&self, other: &dyn Widget) -> bool {
        let Some(other) = other.as_any().downcast_ref::<Link>() else {
            return false;
        };

        self.content == other.content &&
            self.style == other.style &&
            self.hover_style == other.hover_style &&
            self.pressed_style == other.pressed_style &&
            self.disabled_style == other.disabled_style &&
            self.selectable == other.selectable &&
            self.href == other.href &&
            self.target_blank == other.target_blank
    }

    fn cascade_style(&mut self, parent: &Style) {
        self.inherited_style = parent.clone();
        self.recompute_style();
    }

    fn after_interaction_transfer(&mut self) {
        self.recompute_style();
    }

    fn transfer_measured_state(&mut self, old: &dyn Widget) {
        if let Some(old) = old.as_any().downcast_ref::<Link>() {
            self.content_size.set(old.content_size.get());
            self.measured_max_width.set(old.measured_max_width.get());
        }
    }
}

fn normalize_url(href: &str) -> String {
    if href.starts_with("http://") || href.starts_with("https://") {
        href.to_string()
    } else {
        format!("https://{href}")
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn open_native(url: &str) {
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("cmd").args(["/C", "start", "", url]).spawn();
    }
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open").arg(url).spawn();
    }
    #[cfg(
        any(target_os = "linux", target_os = "freebsd", target_os = "openbsd", target_os = "netbsd")
    )]
    {
        let _ = std::process::Command::new("xdg-open").arg(url).spawn();
    }
}
