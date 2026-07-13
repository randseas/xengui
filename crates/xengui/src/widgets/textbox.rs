// SPDX-License-Identifier: Apache-2.0
use crate::{
    Background,
    Color,
    EventCtx,
    EventStatus,
    InputEvent,
    Interaction,
    Key,
    KeyState,
    KeyboardEvent,
    LayoutBox,
    LayoutContext,
    PaintContext,
    RectCommand,
    Style,
    StyleBuilder,
    StylePatch,
    TextCommand,
    Widget,
    WidgetContent,
};
use smol_str::SmolStr;
use std::cell::Cell;
use winit::window::CursorIcon;

pub struct TextBox {
    dirty: bool,
    content: String,
    placeholder: SmolStr,
    font: Option<SmolStr>,
    style: Style,

    hover_style: Option<Style>,
    focus_style: Option<Style>,
    disabled_style: Option<Style>,
    computed_style: Style,

    interaction: Interaction,

    cursor_index: usize,
    max_length: Option<usize>,

    #[allow(clippy::type_complexity)]
    on_change: Option<Box<dyn FnMut(&str, &mut EventCtx)>>,
    #[allow(clippy::type_complexity)]
    on_submit: Option<Box<dyn FnMut(&str, &mut EventCtx)>>,

    layout_box: LayoutBox,
    content_size: Cell<(f32, f32)>,
    // Pixel offset of the caret from the text start, cached during measure()
    // since PaintContext has no access to the text pipeline to shape text.
    cursor_offset: Cell<f32>,
}

impl TextBox {
    pub fn new() -> Self {
        let mut interaction = Interaction::new();
        interaction.focusable = true;
        interaction.hover_cursor = Some(CursorIcon::Text);

        Self {
            dirty: true,
            content: String::new(),
            placeholder: SmolStr::new(""),
            font: None,
            style: Style::default(),
            hover_style: None,
            focus_style: None,
            disabled_style: None,
            computed_style: Style::default(),
            interaction,
            cursor_index: 0,
            max_length: None,
            on_change: None,
            on_submit: None,
            layout_box: LayoutBox::default(),
            content_size: Cell::new((0.0, 0.0)),
            cursor_offset: Cell::new(0.0),
        }
    }

    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.content = value.into();
        self.cursor_index = self.content.chars().count();
        self.mark_dirty();
        self
    }

    pub fn text(&self) -> &str {
        &self.content
    }

    pub fn placeholder(mut self, placeholder: impl Into<SmolStr>) -> Self {
        self.placeholder = placeholder.into();
        self.mark_dirty();
        self
    }

    pub fn font(mut self, font: impl Into<SmolStr>) -> Self {
        self.font = Some(font.into());
        self.mark_dirty();
        self
    }

    pub fn max_length(mut self, max_length: usize) -> Self {
        self.max_length = Some(max_length);
        self
    }

    pub fn hover_style(mut self, build: impl FnOnce(StylePatch) -> StylePatch) -> Self {
        self.hover_style = Some(build(StylePatch::new()).build());
        self.mark_dirty();
        self
    }

    pub fn focus_style(mut self, build: impl FnOnce(StylePatch) -> StylePatch) -> Self {
        self.focus_style = Some(build(StylePatch::new()).build());
        self.mark_dirty();
        self
    }

    pub fn disabled_style(mut self, build: impl FnOnce(StylePatch) -> StylePatch) -> Self {
        self.disabled_style = Some(build(StylePatch::new()).build());
        self.mark_dirty();
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.interaction.set_enabled(enabled);
        self.mark_dirty();
        self
    }

    pub fn on_change(mut self, f: impl FnMut(&str, &mut EventCtx) + 'static) -> Self {
        self.on_change = Some(Box::new(f));
        self
    }

    pub fn on_submit(mut self, f: impl FnMut(&str, &mut EventCtx) + 'static) -> Self {
        self.on_submit = Some(Box::new(f));
        self
    }

    fn recompute_style(&mut self) {
        let patch = if !self.interaction.enabled {
            self.disabled_style.as_ref()
        } else if self.interaction.focused {
            self.focus_style.as_ref().or(self.hover_style.as_ref())
        } else if self.interaction.hovered {
            self.hover_style.as_ref()
        } else {
            None
        };

        self.computed_style = match patch {
            Some(patch) => self.style.overlay(patch),
            None => self.style.clone(),
        };
    }

    fn byte_index_for(&self, char_idx: usize) -> usize {
        self.content
            .char_indices()
            .nth(char_idx)
            .map(|(b, _)| b)
            .unwrap_or(self.content.len())
    }

    fn notify_change(&mut self, ctx: &mut EventCtx) {
        if let Some(cb) = self.on_change.as_mut() {
            cb(&self.content, ctx);
        }
        ctx.request_redraw();
    }

    fn submit(&mut self, ctx: &mut EventCtx) {
        if let Some(cb) = self.on_submit.as_mut() {
            cb(&self.content, ctx);
        }
    }

    fn insert_char(&mut self, c: char, ctx: &mut EventCtx) {
        if let Some(max) = self.max_length && self.content.chars().count() >= max {
            return;
        }
        let byte_idx = self.byte_index_for(self.cursor_index);
        self.content.insert(byte_idx, c);
        self.cursor_index += 1;
        self.notify_change(ctx);
    }

    fn delete_before_cursor(&mut self, ctx: &mut EventCtx) {
        if self.cursor_index == 0 {
            return;
        }
        let end = self.byte_index_for(self.cursor_index);
        let start = self.byte_index_for(self.cursor_index - 1);
        self.content.replace_range(start..end, "");
        self.cursor_index -= 1;
        self.notify_change(ctx);
    }

    fn delete_after_cursor(&mut self, ctx: &mut EventCtx) {
        let len = self.content.chars().count();
        if self.cursor_index >= len {
            return;
        }
        let start = self.byte_index_for(self.cursor_index);
        let end = self.byte_index_for(self.cursor_index + 1);
        self.content.replace_range(start..end, "");
        self.notify_change(ctx);
    }

    fn handle_key(&mut self, key: &KeyboardEvent, ctx: &mut EventCtx) {
        match key.key {
            Key::Character(c) => self.insert_char(c, ctx),
            Key::Space => self.insert_char(' ', ctx),
            Key::Backspace => self.delete_before_cursor(ctx),
            Key::Delete => self.delete_after_cursor(ctx),
            Key::ArrowLeft => {
                self.cursor_index = self.cursor_index.saturating_sub(1);
            }
            Key::ArrowRight => {
                let len = self.content.chars().count();
                self.cursor_index = (self.cursor_index + 1).min(len);
            }
            Key::Enter => self.submit(ctx),
            Key::Escape | Key::Tab => ctx.release_focus(),
            _ => {}
        }
        self.dirty = true;
    }
}

impl Default for TextBox {
    fn default() -> Self {
        Self::new()
    }
}

impl StyleBuilder for TextBox {
    fn style_mut(&mut self) -> &mut Style {
        &mut self.style
    }

    fn mark_dirty(&mut self) {
        self.dirty = true;
        self.recompute_style();
    }
}

impl WidgetContent for TextBox {
    fn with_content(self, content: impl Into<SmolStr>) -> Self {
        self.value(content.into().to_string())
    }
}

crate::impl_interaction_builders!(TextBox);

impl Widget for TextBox {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
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

    fn children(&self) -> &[Box<dyn Widget>] {
        &[]
    }

    fn interaction(&self) -> Option<&Interaction> {
        Some(&self.interaction)
    }

    fn interaction_mut(&mut self) -> Option<&mut Interaction> {
        Some(&mut self.interaction)
    }

    fn measure(&self, ctx: &mut LayoutContext) -> (f32, f32) {
        let scale_factor = ctx.scale_factor;
        let style = &self.computed_style;

        let font_size = style.font_size
            .map(|s| s.to_physical(scale_factor))
            .unwrap_or(20.0 * scale_factor);

        let letter_spacing = style.letter_spacing
            .map(|ls| ls.value().to_physical(scale_factor))
            .unwrap_or(0.0);

        let line_height = style.line_height
            .map(|lh| lh.value().to_physical(scale_factor))
            .unwrap_or(0.0);

        let display_text: &str = if self.content.is_empty() {
            &self.placeholder
        } else {
            &self.content
        };

        let (text_w, text_h) = ctx.text.measure(
            display_text,
            self.font.as_deref(),
            font_size,
            style.font_weight.unwrap_or_default(),
            style.font_style.unwrap_or_default(),
            letter_spacing,
            line_height
        );

        self.content_size.set((text_w, text_h));

        let caret_text = &self.content[..self.byte_index_for(self.cursor_index)];
        let (caret_w, _) = ctx.text.measure(
            caret_text,
            self.font.as_deref(),
            font_size,
            style.font_weight.unwrap_or_default(),
            style.font_style.unwrap_or_default(),
            letter_spacing,
            line_height
        );
        self.cursor_offset.set(caret_w);

        let padding = &style.padding.unwrap_or_default();

        (
            text_w + padding.left.value() + padding.right.value(),
            text_h + padding.top.value() + padding.bottom.value(),
        )
    }

    fn layout(&mut self, rect: LayoutBox) {
        self.layout_box = rect;
    }

    fn layout_box(&self) -> &LayoutBox {
        &self.layout_box
    }

    fn paint(&self, ctx: &mut PaintContext) {
        let style = &self.computed_style;

        if let Some(background) = style.background.clone() {
            let border = style.border.as_ref();
            ctx.draw_rect(RectCommand {
                position: (self.layout_box.x, self.layout_box.y),
                size: (self.layout_box.width, self.layout_box.height),
                background: Some(background),
                border_radius: border.map(|b| b.radius),
                border_width: border.map(|b| b.width),
                border_color: border.map(|b| b.color),
            });
        }

        let padding = style.padding.unwrap_or_default();
        let (_, content_h) = self.content_size.get();

        let text_x = self.layout_box.x + padding.left.value();
        let text_y =
            self.layout_box.y +
            padding.top.value() +
            (self.layout_box.height - padding.top.value() - padding.bottom.value() - content_h).max(
                0.0
            ) *
                0.5;

        let is_empty = self.content.is_empty();
        let display_text: SmolStr = if is_empty {
            self.placeholder.clone()
        } else {
            SmolStr::new(&self.content)
        };

        let mut text_style = style.clone();
        if is_empty {
            text_style.color = Some(style.color.unwrap_or(Color::NEUTRAL_400).with_alpha_f32(0.6));
        }

        ctx.draw_text(TextCommand {
            text: display_text,
            position: (text_x, text_y),
            style: text_style,
            font: self.font.clone(),
        });

        if self.interaction.focused {
            let cursor_x = text_x + self.cursor_offset.get();
            let cursor_h = if content_h > 0.0 {
                content_h
            } else {
                style.font_size.map(|s| s.value()).unwrap_or(20.0) * 1.25
            };
            let cursor_y = self.layout_box.y + (self.layout_box.height - cursor_h).max(0.0) * 0.5;

            ctx.draw_rect(RectCommand {
                position: (cursor_x, cursor_y),
                size: (1.5, cursor_h),
                background: Some(Background::Color(style.color.unwrap_or(Color::BLACK))),
                border_radius: None,
                border_width: None,
                border_color: None,
            });
        }
    }

    fn event(&mut self, event: &InputEvent, ctx: &mut EventCtx) -> EventStatus {
        if !self.interaction.enabled {
            return EventStatus::Ignored;
        }

        // Key input bypasses Interaction::handle entirely: the generic handler
        // treats Enter/Space as a click-activation key, which would prevent
        // typing spaces and would fire on_click on every Enter press.
        if let InputEvent::KeyInput { event: key_event, .. } = event {
            if !self.interaction.focused || key_event.state != KeyState::Pressed {
                return EventStatus::Ignored;
            }
            self.handle_key(key_event, ctx);
            self.recompute_style();
            ctx.request_redraw();
            return EventStatus::Handled;
        }

        let status = self.interaction.handle(event, ctx);

        if matches!(event, InputEvent::FocusGained) {
            self.cursor_index = self.content.chars().count();
        }

        if matches!(status, EventStatus::Handled) {
            self.recompute_style();
            self.dirty = true;
        }

        status
    }

    fn content_eq(&self, other: &dyn Widget) -> bool {
        let Some(other) = other.as_any().downcast_ref::<TextBox>() else {
            return false;
        };
        self.content == other.content &&
            self.placeholder == other.placeholder &&
            self.font == other.font &&
            self.cursor_index == other.cursor_index &&
            format!("{:?}", self.style) == format!("{:?}", other.style) &&
            format!("{:?}", self.hover_style) == format!("{:?}", other.hover_style) &&
            format!("{:?}", self.focus_style) == format!("{:?}", other.focus_style) &&
            format!("{:?}", self.disabled_style) == format!("{:?}", other.disabled_style)
    }

    fn after_interaction_transfer(&mut self) {
        self.recompute_style();
    }

    // Always runs regardless of content_eq, so the caret survives a rebuild
    // even on the frame where the text itself changed.
    fn transfer_interaction_state(&mut self, old: &dyn Widget) -> bool {
        let changed = if
            let (Some(new_i), Some(old_i)) = (self.interaction_mut(), old.interaction())
        {
            let changed =
                new_i.hovered != old_i.hovered ||
                new_i.pressed != old_i.pressed ||
                new_i.focused != old_i.focused;
            new_i.transfer_from(old_i);
            changed
        } else {
            false
        };

        if let Some(old_tb) = old.as_any().downcast_ref::<TextBox>() {
            self.cursor_index = old_tb.cursor_index.min(self.content.chars().count());
        }

        changed
    }

    fn transfer_measured_state(&mut self, old: &dyn Widget) {
        if let Some(old) = old.as_any().downcast_ref::<TextBox>() {
            self.content_size.set(old.content_size.get());
            self.cursor_offset.set(old.cursor_offset.get());
        }
    }
}
