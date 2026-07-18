// SPDX-License-Identifier: Apache-2.0
use crate::{
    AnimationManager,
    Background,
    Color,
    Constraints,
    EventCtx,
    EventStatus,
    InputEvent,
    Interaction,
    IntoThemed,
    Key,
    KeyState,
    LayoutBox,
    MeasureContext,
    MeasureResult,
    PaintContext,
    RectCommand,
    Style,
    StyleBuilder,
    TextCommand,
    TextDecoration,
    Widget,
    WidgetContent,
    properties::{
        DEFAULT_CURSOR_ICON,
        DEFAULT_FONT_SIZE,
        DEFAULT_LINK_COLOR,
        DEFAULT_POINTER_CURSOR_ICON,
    },
};
use smol_str::SmolStr;
use std::cell::{ Cell, RefCell };
use winit::{ event::{ ElementState, MouseButton }, window::CursorIcon };
use web_time::Instant;

const MULTI_CLICK_INTERVAL: std::time::Duration = std::time::Duration::from_millis(400);
const MULTI_CLICK_DISTANCE: f32 = 4.0;

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

    char_offsets: RefCell<Vec<f32>>,
    selection_anchor: Cell<Option<usize>>,
    selection_cursor: Cell<Option<usize>>,
    dragging: Cell<bool>,
    moved_during_press: Cell<bool>,

    click_count: Cell<u8>,
    last_click_time: Cell<Option<Instant>>,
    last_click_pos: Cell<(f32, f32)>,
    href: Option<SmolStr>,
    target_blank: bool,
}

impl Link {
    pub fn new() -> Self {
        let mut interaction = Interaction::new();
        interaction.focusable = true;
        interaction.hover_cursor = Some(DEFAULT_POINTER_CURSOR_ICON);

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

            char_offsets: RefCell::new(Vec::new()),
            selection_anchor: Cell::new(None),
            selection_cursor: Cell::new(None),
            dragging: Cell::new(false),
            moved_during_press: Cell::new(false),

            click_count: Cell::new(0),
            last_click_time: Cell::new(None),
            last_click_pos: Cell::new((0.0, 0.0)),
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

    pub fn hover_background<M>(mut self, background: impl IntoThemed<Background, M>) -> Self {
        self.hover_style.get_or_insert_with(Style::default).background = Some(
            background.resolve_themed()
        );
        self.mark_dirty();
        self
    }

    pub fn pressed_background<M>(mut self, background: impl IntoThemed<Background, M>) -> Self {
        self.pressed_style.get_or_insert_with(Style::default).background = Some(
            background.resolve_themed()
        );
        self.mark_dirty();
        self
    }

    pub fn disabled_background<M>(mut self, background: impl IntoThemed<Background, M>) -> Self {
        self.disabled_style.get_or_insert_with(Style::default).background = Some(
            background.resolve_themed()
        );
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
            .or(
                Some(
                    if self.selectable {
                        CursorIcon::Text
                    } else if self.href.is_some() {
                        DEFAULT_POINTER_CURSOR_ICON
                    } else {
                        DEFAULT_CURSOR_ICON
                    }
                )
            );
    }

    fn open_href(&self, _force_new_tab: bool) {
        let Some(href) = &self.href else {
            return;
        };

        let url = normalize_url(href);

        #[cfg(target_arch = "wasm32")]
        {
            if let Some(window) = web_sys::window() {
                let target = if self.target_blank || _force_new_tab { "_blank" } else { "_self" };
                let _ = window.open_with_url_and_target(&url, target);
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            open_native(&url);
        }
    }

    fn index_for_offset(&self, local_x: f32) -> usize {
        let offsets = self.char_offsets.borrow();
        if offsets.len() <= 1 {
            return 0;
        }
        let mut best = 0;
        let mut best_dist = f32::MAX;
        for (i, &off) in offsets.iter().enumerate() {
            let dist = (off - local_x).abs();
            if dist < best_dist {
                best_dist = dist;
                best = i;
            }
        }
        best
    }

    fn char_class(c: char) -> u8 {
        if c.is_whitespace() { 0 } else if c.is_alphanumeric() || c == '_' { 1 } else { 2 }
    }

    fn word_bounds_at(&self, idx: usize) -> (usize, usize) {
        let chars: Vec<char> = self.content.chars().collect();
        if chars.is_empty() {
            return (0, 0);
        }
        let probe = idx.min(chars.len() - 1);
        let class = Self::char_class(chars[probe]);

        let mut start = probe;
        while start > 0 && Self::char_class(chars[start - 1]) == class {
            start -= 1;
        }
        let mut end = probe + 1;
        while end < chars.len() && Self::char_class(chars[end]) == class {
            end += 1;
        }
        (start, end)
    }

    fn byte_index_for(&self, char_idx: usize) -> usize {
        self.content
            .char_indices()
            .nth(char_idx)
            .map(|(b, _)| b)
            .unwrap_or(self.content.len())
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
crate::impl_themed_style_builders!(Link; hover_style => hover_style, pressed_style => pressed_style, disabled_style => disabled_style);

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

        if self.selectable {
            *self.char_offsets.borrow_mut() = ctx.text.character_offsets(
                &self.content,
                style.font.as_deref(),
                font_size,
                style.font_weight.unwrap_or_default(),
                style.font_style.unwrap_or_default(),
                letter_spacing,
                line_height
            );
        }

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

        let padding = style.padding.unwrap_or_default();

        let text_x = self.layout_box.x + padding.left.value();
        let text_y = self.layout_box.y + padding.top.value();

        let mut text_style = style.clone();
        text_style.font_size.get_or_insert(DEFAULT_FONT_SIZE);
        text_style.color.get_or_insert(DEFAULT_LINK_COLOR);

        if self.interaction.hovered {
            text_style.text_decoration.get_or_insert(TextDecoration::UNDERLINE);
        }

        let selection = if self.selectable { self.text_selection() } else { None };

        if let Some((start, end)) = selection {
            let offsets = self.char_offsets.borrow();
            if let (Some(&start_x), Some(&end_x)) = (offsets.get(start), offsets.get(end)) {
                let (_, content_h) = self.content_size.get();
                ctx.draw_rect(RectCommand {
                    position: (text_x + start_x, text_y),
                    size: (end_x - start_x, content_h.max(1.0)),
                    background: Some(
                        Background::Color(
                            style.selection_background.unwrap_or(Color::rgba(90, 140, 230, 100))
                        )
                    ),
                    border_radius: None,
                    border_width: None,
                    border_color: None,
                    clip_rect: None,
                });
            }
        }

        let split = selection.zip(style.selection_color);

        if let Some(((start, end), sel_fg)) = split {
            let start_b = self.byte_index_for(start);
            let end_b = self.byte_index_for(end);
            let offsets = self.char_offsets.borrow();
            let start_x = offsets.get(start).copied().unwrap_or(0.0);
            let end_x = offsets.get(end).copied().unwrap_or(0.0);
            drop(offsets);

            if start_b > 0 {
                ctx.draw_text(TextCommand {
                    text: SmolStr::new(&self.content[..start_b]),
                    position: (text_x, text_y),
                    style: text_style.clone(),
                    max_width: None,
                    clip_rect: None,
                });
            }
            if end_b > start_b {
                let mut sel_style = text_style.clone();
                sel_style.color = Some(sel_fg);
                ctx.draw_text(TextCommand {
                    text: SmolStr::new(&self.content[start_b..end_b]),
                    position: (text_x + start_x, text_y),
                    style: sel_style,
                    max_width: None,
                    clip_rect: None,
                });
            }
            if end_b < self.content.len() {
                ctx.draw_text(TextCommand {
                    text: SmolStr::new(&self.content[end_b..]),
                    position: (text_x + end_x, text_y),
                    style: text_style,
                    max_width: None,
                    clip_rect: None,
                });
            }
        } else {
            ctx.draw_text(TextCommand {
                text: self.content.clone(),
                position: (text_x, text_y),
                style: text_style,
                max_width: self.measured_max_width.get(),
                clip_rect: None,
            });
        }
    }

    fn event(&mut self, event: &InputEvent, ctx: &mut EventCtx) -> EventStatus {
        if !self.interaction.is_active() {
            return EventStatus::Ignored;
        }

        // Middle-click opens the link in a new tab, matching browser convention.
        if
            let InputEvent::MouseInput {
                state: ElementState::Released,
                button: MouseButton::Middle,
                ..
            } = event &&
            self.interaction.hovered &&
            self.href.is_some()
        {
            self.open_href(true);
            return EventStatus::Handled;
        }

        if self.selectable {
            if
                let InputEvent::MouseInput {
                    state: ElementState::Pressed,
                    button: MouseButton::Left,
                    position,
                } = event
            {
                let padding_left = self.computed_style.padding.unwrap_or_default().left.value();
                let local_x = position.0 - self.layout_box.x - padding_left;
                let idx = self.index_for_offset(local_x);

                let now = Instant::now();
                let (last_x, last_y) = self.last_click_pos.get();
                let same_spot =
                    (position.0 - last_x).abs() < MULTI_CLICK_DISTANCE &&
                    (position.1 - last_y).abs() < MULTI_CLICK_DISTANCE;
                let is_repeat =
                    same_spot &&
                    self.last_click_time
                        .get()
                        .is_some_and(|t| now.duration_since(t) < MULTI_CLICK_INTERVAL);
                let click_count = if is_repeat { (self.click_count.get() + 1).min(3) } else { 1 };
                self.click_count.set(click_count);
                self.last_click_time.set(Some(now));
                self.last_click_pos.set(*position);
                self.moved_during_press.set(false);

                match click_count {
                    1 => {
                        self.selection_anchor.set(Some(idx));
                        self.selection_cursor.set(Some(idx));
                        self.dragging.set(true);
                    }
                    2 => {
                        let (start, end) = self.word_bounds_at(idx);
                        self.selection_anchor.set(Some(start));
                        self.selection_cursor.set(Some(end));
                        self.dragging.set(false);
                        ctx.suppress_text_drag();
                    }
                    _ => {
                        let len = self.content.chars().count();
                        self.selection_anchor.set(Some(0));
                        self.selection_cursor.set(Some(len));
                        self.dragging.set(false);
                        ctx.suppress_text_drag();
                    }
                }
            }

            if let InputEvent::MouseMoved { position } = event && self.dragging.get() {
                let padding_left = self.computed_style.padding.unwrap_or_default().left.value();
                let local_x = position.0 - self.layout_box.x - padding_left;
                let idx = self.index_for_offset(local_x);
                if self.selection_anchor.get() != Some(idx) {
                    self.moved_during_press.set(true);
                }
                return EventStatus::Handled;
            }

            if
                matches!(event, InputEvent::MouseInput {
                    state: ElementState::Released,
                    button: MouseButton::Left,
                    ..
                })
            {
                self.dragging.set(false);
            }
        }

        let is_click = match event {
            InputEvent::MouseInput {
                state: ElementState::Released,
                button: MouseButton::Left,
                ..
            } =>
                self.interaction.pressed &&
                    self.interaction.hovered &&
                    !self.moved_during_press.get() &&
                    self.click_count.get() <= 1,
            InputEvent::KeyInput { event: key_event, .. } =>
                self.interaction.focused &&
                    !key_event.repeat &&
                    key_event.state == KeyState::Pressed &&
                    matches!(key_event.key, Key::Enter | Key::Space),
            _ => false,
        };

        let status = self.interaction.handle(event, ctx);

        if is_click {
            self.open_href(false);
        }

        if matches!(status, EventStatus::Handled) {
            self.recompute_style();
            self.dirty = true;
        }

        status
    }

    fn selectable_text(&self) -> Option<&str> {
        self.selectable.then_some(self.content.as_str())
    }

    fn text_selection(&self) -> Option<(usize, usize)> {
        let anchor = self.selection_anchor.get()?;
        let cursor = self.selection_cursor.get()?;
        (anchor != cursor).then(|| (anchor.min(cursor), anchor.max(cursor)))
    }

    fn set_text_selection(&mut self, range: Option<(usize, usize)>) {
        let (anchor, cursor) = range.map_or((None, None), |(s, e)| (Some(s), Some(e)));
        self.selection_anchor.set(anchor);
        self.selection_cursor.set(cursor);
        self.dirty = true;
    }

    fn cancel_text_selection(&mut self) {
        self.selection_anchor.set(None);
        self.selection_cursor.set(None);
        self.dragging.set(false);
        self.dirty = true;
    }

    fn text_index_at(&self, point: (f32, f32)) -> usize {
        let padding_left = self.computed_style.padding.unwrap_or_default().left.value();
        let local_x = point.0 - self.layout_box.x - padding_left;
        self.index_for_offset(local_x)
    }

    fn select_all_text(&mut self) {
        if !self.selectable {
            return;
        }
        self.selection_anchor.set(Some(0));
        self.selection_cursor.set(Some(self.content.chars().count()));
        self.dirty = true;
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

    fn cascade_style(&mut self, parent: &Style, _anim: &mut AnimationManager) {
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
            self.char_offsets.replace(old.char_offsets.borrow().clone());
            self.selection_anchor.set(old.selection_anchor.get());
            self.selection_cursor.set(old.selection_cursor.get());
            self.click_count.set(old.click_count.get());
            self.last_click_time.set(old.last_click_time.get());
            self.last_click_pos.set(old.last_click_pos.get());
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
