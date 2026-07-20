use crate::properties::{ DEFAULT_FONT_SIZE, DEFAULT_LINE_HEIGHT_RATIO };
use crate::widget::NativeTextInputSnapshot;
// SPDX-License-Identifier: Apache-2.0
use crate::{
    AnimationManager,
    Background,
    Color,
    Constraints,
    Edges,
    EventCtx,
    EventStatus,
    InputEvent,
    Interaction,
    Key,
    KeyState,
    KeyboardEvent,
    LayoutBox,
    Length,
    MeasureContext,
    MeasureResult,
    ModifiersState,
    PaintContext,
    RectCommand,
    Size,
    Style,
    StyleBuilder,
    TextCommand,
    Widget,
    WidgetContent,
    WidgetId,
    MULTI_CLICK_INTERVAL,
    MULTI_CLICK_DISTANCE_DP,
};
use smol_str::SmolStr;
use std::cell::{ Cell, RefCell };
use std::sync::{ Arc, Mutex };
use web_time::Instant;
use winit::event::{ ElementState, MouseButton };
use winit::window::CursorIcon;
use xen_clipboard::Clipboard;

pub struct TextBox {
    key: Option<SmolStr>,
    anim_id: WidgetId,

    dirty: bool,
    content: String,
    placeholder: SmolStr,
    style: Style,

    hover_style: Option<Style>,
    focus_style: Option<Style>,
    disabled_style: Option<Style>,
    inherited_style: Style,
    computed_style: Style,

    interaction: Interaction,

    cursor_index: usize,
    max_length: Option<usize>,

    // Selection anchor; the selected range spans [anchor, cursor_index)
    // in whichever order is smaller-to-larger. `None` means no selection.
    selection_anchor: Option<usize>,

    undo_stack: Vec<(String, usize, Option<usize>)>,
    redo_stack: Vec<(String, usize, Option<usize>)>,

    dragging: bool,
    drag_word_selection: bool,
    drag_word_anchor: usize,
    // Tracks the real OS-level left-button-held state, independent of
    // hover/Interaction - MouseExited/MouseEntered never touch this.
    mouse_button_held: Cell<bool>,

    click_count: Cell<u8>,
    last_click_time: Cell<Option<Instant>>,
    last_click_pos: Cell<(f32, f32)>,
    // Set right before a mouse-press asks for focus, so the FocusGained
    // that follows keeps the caret where the user clicked instead of
    // jumping to the end of the content (the default for e.g. Tab focus).
    focus_via_pointer: Cell<bool>,
    current_modifiers: Cell<ModifiersState>,

    clipboard: Clipboard,
    // Clipboard reads are callback-based (required for WASM, where they're
    // promise-based), so a completed read is parked here and applied on
    // the next event this widget receives.
    pending_paste: Arc<Mutex<Option<String>>>,

    #[allow(clippy::type_complexity)]
    on_change: Option<Box<dyn FnMut(&str, &mut EventCtx)>>,
    #[allow(clippy::type_complexity)]
    on_submit: Option<Box<dyn FnMut(&str, &mut EventCtx)>>,

    read_only: bool,

    layout_box: LayoutBox,
    content_size: Cell<(f32, f32)>,
    // Pixel offset of the caret from the text start, cached during measure()
    // since PaintContext has no access to the text pipeline to shape text.
    cursor_offset: Cell<f32>,
    // Pixel offset of every character boundary (index 0..=char_count),
    // cached during measure() and reused for caret placement, mouse
    // hit-testing, and selection-highlight geometry.
    char_offsets: RefCell<Vec<f32>>,
    caret_visible: Cell<bool>,
    // Horizontal pixel offset applied when the intrinsic text width exceeds
    // the visible content area, so the caret always stays on-screen.
    scroll_offset: Cell<f32>,
    scale_factor: Cell<f32>,
}

impl TextBox {
    pub fn new() -> Self {
        let mut interaction = Interaction::new();
        interaction.focusable = true;
        interaction.hover_cursor = Some(CursorIcon::Text);

        let style = Style {
            padding: Some(Edges::symmetric(8.0, 6.0)),
            min_size: Some(Size { width: Some(Length::px(120.0)), height: None }),
            ..Default::default()
        };

        Self {
            key: None,
            anim_id: WidgetId::new_unique(),

            dirty: true,
            content: String::new(),
            placeholder: SmolStr::new(""),
            style,
            hover_style: None,
            focus_style: None,
            disabled_style: None,
            inherited_style: Style::default(),
            computed_style: Style::default(),
            interaction,
            cursor_index: 0,
            max_length: None,
            selection_anchor: None,

            undo_stack: Vec::new(),
            redo_stack: Vec::new(),

            dragging: false,
            drag_word_selection: false,
            drag_word_anchor: 0,
            mouse_button_held: Cell::new(false),

            click_count: Cell::new(0),
            last_click_time: Cell::new(None),
            last_click_pos: Cell::new((0.0, 0.0)),
            focus_via_pointer: Cell::new(false),
            current_modifiers: Cell::new(ModifiersState::default()),
            clipboard: Clipboard::new(),
            pending_paste: Arc::new(Mutex::new(None)),
            on_change: None,
            on_submit: None,

            read_only: false,

            layout_box: LayoutBox::default(),
            content_size: Cell::new((0.0, 0.0)),
            cursor_offset: Cell::new(0.0),
            char_offsets: RefCell::new(Vec::new()),
            caret_visible: Cell::new(true),
            scroll_offset: Cell::new(0.0),
            scale_factor: Cell::new(1.0),
        }
    }

    /// Stable identity among siblings, kept across rebuilds even when this
    /// widget moves position (reorder, insert, remove). Use for list items
    /// instead of relying on array index.
    pub fn key(mut self, key: impl Into<SmolStr>) -> Self {
        self.key = Some(key.into());
        self
    }

    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.content = value.into();
        self.cursor_index = self.content.chars().count();
        self.selection_anchor = None;
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
        self.style.font = Some(font.into());
        self.mark_dirty();
        self
    }

    pub fn max_length(mut self, max_length: usize) -> Self {
        self.max_length = Some(max_length);
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.interaction.set_enabled(enabled);
        self.mark_dirty();
        self
    }

    pub fn read_only(mut self, value: bool) -> Self {
        self.read_only = value;
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
            self.focus_style.as_ref()
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
            .or(Some(CursorIcon::Text));
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

    // Returns the selection as an ordered (start, end) char-index pair,
    // or None when nothing is selected.
    fn selection_range(&self) -> Option<(usize, usize)> {
        let anchor = self.selection_anchor?;
        if anchor == self.cursor_index {
            return None;
        }
        Some((anchor.min(self.cursor_index), anchor.max(self.cursor_index)))
    }

    fn selected_text(&self) -> Option<String> {
        let (start, end) = self.selection_range()?;
        let start_b = self.byte_index_for(start);
        let end_b = self.byte_index_for(end);
        Some(self.content[start_b..end_b].to_string())
    }

    // Removes the selected text (if any), moves the cursor to where it
    // started, and clears the selection. Does not notify on_change -
    // callers combine this with their own edit before notifying once.
    fn delete_selection(&mut self) -> bool {
        let Some((start, end)) = self.selection_range() else {
            return false;
        };
        let start_b = self.byte_index_for(start);
        let end_b = self.byte_index_for(end);
        self.content.replace_range(start_b..end_b, "");
        self.cursor_index = start;
        self.selection_anchor = None;
        true
    }

    // Pushes a snapshot before a mutating edit; any new edit invalidates redo history.
    fn push_undo_snapshot(&mut self) {
        self.undo_stack.push((self.content.clone(), self.cursor_index, self.selection_anchor));
        self.redo_stack.clear();
    }

    fn undo(&mut self, ctx: &mut EventCtx) {
        // return if read_only
        if self.read_only {
            return;
        }

        let Some((content, cursor_index, selection_anchor)) = self.undo_stack.pop() else {
            return;
        };
        self.redo_stack.push((self.content.clone(), self.cursor_index, self.selection_anchor));
        self.content = content;
        self.cursor_index = cursor_index;
        self.selection_anchor = selection_anchor;
        self.notify_change(ctx);
    }

    fn redo(&mut self, ctx: &mut EventCtx) {
        // return if read_only
        if self.read_only {
            return;
        }

        let Some((content, cursor_index, selection_anchor)) = self.redo_stack.pop() else {
            return;
        };
        self.undo_stack.push((self.content.clone(), self.cursor_index, self.selection_anchor));
        self.content = content;
        self.cursor_index = cursor_index;
        self.selection_anchor = selection_anchor;
        self.notify_change(ctx);
    }

    fn move_cursor_to(&mut self, target: usize, extend_selection: bool) {
        if extend_selection {
            if self.selection_anchor.is_none() {
                self.selection_anchor = Some(self.cursor_index);
            }
        } else {
            self.selection_anchor = None;
        }
        self.cursor_index = target;
    }

    fn char_class(c: char) -> u8 {
        if c.is_whitespace() { 0 } else if c.is_alphanumeric() || c == '_' { 1 } else { 2 }
    }

    // Word (or punctuation/whitespace run) boundaries around `idx`, used
    // for double-click word selection.
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

    fn word_left(&self, from: usize) -> usize {
        let chars: Vec<char> = self.content.chars().collect();
        let mut i = from;
        while i > 0 && Self::char_class(chars[i - 1]) == 0 {
            i -= 1;
        }
        if i == 0 {
            return 0;
        }
        let class = Self::char_class(chars[i - 1]);
        while i > 0 && Self::char_class(chars[i - 1]) == class {
            i -= 1;
        }
        i
    }

    fn word_right(&self, from: usize) -> usize {
        let chars: Vec<char> = self.content.chars().collect();
        let len = chars.len();
        let mut i = from;
        while i < len && Self::char_class(chars[i]) == 0 {
            i += 1;
        }
        if i == len {
            return len;
        }
        let class = Self::char_class(chars[i]);
        while i < len && Self::char_class(chars[i]) == class {
            i += 1;
        }
        i
    }

    // Maps a pixel offset (relative to the text start) to the nearest
    // character boundary, using the offsets cached by the last measure().
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

    fn insert_char(&mut self, c: char, ctx: &mut EventCtx) {
        // return if read_only
        if self.read_only {
            return;
        }

        let can_insert = match self.max_length {
            Some(max) => self.selection_range().is_some() || self.content.chars().count() < max,
            None => true,
        };
        if can_insert {
            self.push_undo_snapshot();
        }

        let had_selection = self.delete_selection();

        if let Some(max) = self.max_length && self.content.chars().count() >= max {
            if had_selection {
                self.notify_change(ctx);
            }
            return;
        }

        let byte_idx = self.byte_index_for(self.cursor_index);
        self.content.insert(byte_idx, c);
        self.cursor_index += 1;
        self.notify_change(ctx);
    }

    // Inserts arbitrary text (used by paste), stripping control characters
    // and truncating to whatever room max_length leaves.
    fn insert_text(&mut self, text: &str, ctx: &mut EventCtx) {
        // return if read_only
        if self.read_only {
            return;
        }

        let filtered: String = text
            .chars()
            .filter(|c| !c.is_control())
            .collect();
        if filtered.is_empty() {
            return;
        }

        self.push_undo_snapshot();
        let had_selection = self.delete_selection();

        let to_insert: String = if let Some(max) = self.max_length {
            let available = max.saturating_sub(self.content.chars().count());
            filtered.chars().take(available).collect()
        } else {
            filtered
        };

        if to_insert.is_empty() {
            if had_selection {
                self.notify_change(ctx);
            } else {
                self.undo_stack.pop();
            }
            return;
        }

        let byte_idx = self.byte_index_for(self.cursor_index);
        let inserted_chars = to_insert.chars().count();
        self.content.insert_str(byte_idx, &to_insert);
        self.cursor_index += inserted_chars;
        self.notify_change(ctx);
    }

    fn delete_before_cursor(&mut self, ctx: &mut EventCtx) {
        // return if read_only
        if self.read_only {
            return;
        }

        if self.selection_range().is_some() {
            self.push_undo_snapshot();
            self.delete_selection();
            self.notify_change(ctx);
            return;
        }
        if self.cursor_index == 0 {
            return;
        }
        self.push_undo_snapshot();
        let end = self.byte_index_for(self.cursor_index);
        let start = self.byte_index_for(self.cursor_index - 1);
        self.content.replace_range(start..end, "");
        self.cursor_index -= 1;
        self.notify_change(ctx);
    }

    fn delete_word_before_cursor(&mut self, ctx: &mut EventCtx) {
        if self.read_only {
            return;
        }

        if self.selection_range().is_some() {
            self.push_undo_snapshot();
            self.delete_selection();
            self.notify_change(ctx);
            return;
        }

        if self.cursor_index == 0 {
            return;
        }

        let target = self.word_left(self.cursor_index);

        self.push_undo_snapshot();

        let start = self.byte_index_for(target);
        let end = self.byte_index_for(self.cursor_index);

        self.content.replace_range(start..end, "");
        self.cursor_index = target;

        self.notify_change(ctx);
    }

    fn delete_after_cursor(&mut self, ctx: &mut EventCtx) {
        // return if read_only
        if self.read_only {
            return;
        }

        if self.selection_range().is_some() {
            self.push_undo_snapshot();
            self.delete_selection();
            self.notify_change(ctx);
            return;
        }
        let len = self.content.chars().count();
        if self.cursor_index >= len {
            return;
        }
        self.push_undo_snapshot();
        let start = self.byte_index_for(self.cursor_index);
        let end = self.byte_index_for(self.cursor_index + 1);
        self.content.replace_range(start..end, "");
        self.notify_change(ctx);
    }

    fn delete_word_after_cursor(&mut self, ctx: &mut EventCtx) {
        if self.read_only {
            return;
        }

        if self.selection_range().is_some() {
            self.push_undo_snapshot();
            self.delete_selection();
            self.notify_change(ctx);
            return;
        }

        let len = self.content.chars().count();

        if self.cursor_index >= len {
            return;
        }

        let target = self.word_right(self.cursor_index);

        self.push_undo_snapshot();

        let start = self.byte_index_for(self.cursor_index);
        let end = self.byte_index_for(target);

        self.content.replace_range(start..end, "");

        self.notify_change(ctx);
    }

    fn copy_selection(&self) {
        if let Some(text) = self.selected_text() {
            self.clipboard.set_text(text, |_| {});
        }
    }

    fn cut_selection(&mut self, ctx: &mut EventCtx) {
        // return if read_only
        if self.read_only {
            return;
        }

        let Some(text) = self.selected_text() else {
            return;
        };
        self.clipboard.set_text(text, |_| {});
        self.push_undo_snapshot();
        self.delete_selection();
        self.notify_change(ctx);
    }

    // Kicks off an async clipboard read; the result lands in `pending_paste`
    // and is applied by poll_clipboard_paste (called on the next event).
    fn paste_from_clipboard(&mut self) {
        let pending = Arc::clone(&self.pending_paste);
        self.clipboard.get_text(move |result| {
            if
                let Ok(Some(text)) = result &&
                !text.is_empty() &&
                let Ok(mut guard) = pending.lock()
            {
                *guard = Some(text);
            }
        });
    }

    fn poll_clipboard_paste(&mut self, ctx: &mut EventCtx) {
        let text = self.pending_paste
            .lock()
            .ok()
            .and_then(|mut guard| guard.take());
        if let Some(text) = text {
            self.insert_text(&text, ctx);
        }
    }

    fn handle_key(&mut self, key: &KeyboardEvent, modifiers: ModifiersState, ctx: &mut EventCtx) {
        self.caret_visible.set(true);
        let cmd = modifiers.ctrl || modifiers.super_key;

        if cmd {
            match key.key {
                Key::Character('a' | 'A') => {
                    self.selection_anchor = Some(0);
                    self.cursor_index = self.content.chars().count();
                    self.dirty = true;
                    return;
                }
                Key::Character('c' | 'C') => {
                    self.copy_selection();
                    return;
                }
                Key::Character('x' | 'X') => {
                    self.cut_selection(ctx);
                    self.dirty = true;
                    return;
                }
                Key::Character('v' | 'V') => {
                    self.paste_from_clipboard();
                    // Applies immediately on backends that resolve synchronously.
                    self.poll_clipboard_paste(ctx);
                    self.dirty = true;
                    return;
                }
                Key::Character('z' | 'Z') => {
                    if modifiers.shift {
                        self.redo(ctx);
                    } else {
                        self.undo(ctx);
                    }
                    self.dirty = true;
                    return;
                }
                Key::Character('y' | 'Y') => {
                    self.redo(ctx);
                    self.dirty = true;
                    return;
                }
                Key::ArrowLeft => {
                    let target = self.word_left(self.cursor_index);
                    self.move_cursor_to(target, modifiers.shift);
                    self.dirty = true;
                    return;
                }
                Key::ArrowRight => {
                    let target = self.word_right(self.cursor_index);
                    self.move_cursor_to(target, modifiers.shift);
                    self.dirty = true;
                    return;
                }
                Key::Backspace => {
                    self.delete_word_before_cursor(ctx);
                    self.dirty = true;
                    return;
                }
                Key::Delete => {
                    self.delete_word_after_cursor(ctx);
                    self.dirty = true;
                    return;
                }
                _ => {}
            }
        }

        match key.key {
            Key::Character(c) => self.insert_char(c, ctx),
            Key::Space => self.insert_char(' ', ctx),
            Key::Backspace => self.delete_before_cursor(ctx),
            Key::Delete => self.delete_after_cursor(ctx),
            Key::ArrowLeft => {
                if let Some((start, _)) = self.selection_range() && !modifiers.shift {
                    self.cursor_index = start;
                    self.selection_anchor = None;
                } else {
                    let target = self.cursor_index.saturating_sub(1);
                    self.move_cursor_to(target, modifiers.shift);
                }
            }
            Key::ArrowRight => {
                let len = self.content.chars().count();
                if let Some((_, end)) = self.selection_range() && !modifiers.shift {
                    self.cursor_index = end;
                    self.selection_anchor = None;
                } else {
                    let target = (self.cursor_index + 1).min(len);
                    self.move_cursor_to(target, modifiers.shift);
                }
            }
            Key::Home => self.move_cursor_to(0, modifiers.shift),
            Key::End => {
                let len = self.content.chars().count();
                self.move_cursor_to(len, modifiers.shift);
            }
            Key::Enter => self.submit(ctx),
            Key::Escape => {
                if self.selection_anchor.is_some() {
                    self.selection_anchor = None;
                } else {
                    ctx.release_focus();
                }
            }
            Key::Tab => ctx.release_focus(),
            _ => {}
        }
        self.dirty = true;
    }

    fn handle_mouse_press(&mut self, position: (f32, f32)) {
        let padding_left = self.computed_style.padding
            .unwrap_or_default()
            .left.to_physical(self.scale_factor.get());
        let local_x = position.0 - self.layout_box.x - padding_left + self.scroll_offset.get();
        let click_index = self.index_for_offset(local_x);

        let now = Instant::now();
        let (last_x, last_y) = self.last_click_pos.get();
        let click_distance = MULTI_CLICK_DISTANCE_DP * self.scale_factor.get();
        let same_spot =
            (position.0 - last_x).abs() < click_distance &&
            (position.1 - last_y).abs() < click_distance;
        let is_repeat =
            same_spot &&
            self.last_click_time
                .get()
                .is_some_and(|t| now.duration_since(t) < MULTI_CLICK_INTERVAL);

        let click_count = if is_repeat { (self.click_count.get() + 1).min(3) } else { 1 };
        self.click_count.set(click_count);
        self.last_click_time.set(Some(now));
        self.last_click_pos.set(position);
        self.focus_via_pointer.set(true);
        self.caret_visible.set(true);

        let shift_held = self.current_modifiers.get().shift;

        match click_count {
            1 if shift_held => {
                if self.selection_anchor.is_none() {
                    self.selection_anchor = Some(self.cursor_index);
                }
                self.cursor_index = click_index;
                self.dragging = true;
            }
            1 => {
                self.cursor_index = click_index;
                self.selection_anchor = None;

                self.drag_word_selection = false;
                self.dragging = true;
            }
            2 => {
                let (start, end) = self.word_bounds_at(click_index);

                self.selection_anchor = Some(start);
                self.cursor_index = end;

                self.drag_word_anchor = click_index;
                self.drag_word_selection = true;
                self.dragging = true;
            }
            _ => {
                let len = self.content.chars().count();
                self.selection_anchor = if len > 0 { Some(0) } else { None };
                self.drag_word_selection = false;
                self.cursor_index = len;
                self.dragging = false;
            }
        }
    }

    fn handle_mouse_drag(&mut self, position: (f32, f32)) {
        let padding_left = self.computed_style.padding
            .unwrap_or_default()
            .left.to_physical(self.scale_factor.get());
        let local_x = position.0 - self.layout_box.x - padding_left + self.scroll_offset.get();
        let idx = self.index_for_offset(local_x);

        if self.drag_word_selection {
            let (anchor_start, anchor_end) = self.word_bounds_at(self.drag_word_anchor);
            let (word_start, word_end) = self.word_bounds_at(idx);

            if idx >= self.drag_word_anchor {
                self.selection_anchor = Some(anchor_start);
                self.cursor_index = word_end;
            } else {
                self.selection_anchor = Some(anchor_end);
                self.cursor_index = word_start;
            }

            return;
        }

        if self.selection_anchor.is_none() && idx != self.cursor_index {
            self.selection_anchor = Some(self.cursor_index);
        }

        self.cursor_index = idx;
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
crate::impl_themed_style_builders!(TextBox; hover_style => hover_style, focus_style => focus_style, disabled_style => disabled_style);

impl Widget for TextBox {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn get_key(&self) -> Option<&SmolStr> {
        self.key.as_ref()
    }

    fn debug_name(&self) -> &'static str {
        "Widget#TextBox"
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

    fn measure(&self, ctx: &mut MeasureContext, constraints: Constraints) -> MeasureResult {
        let scale_factor = ctx.scale_factor;
        self.scale_factor.set(scale_factor);
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

        let display_text: &str = if self.content.is_empty() {
            &self.placeholder
        } else {
            &self.content
        };

        let result = ctx.text.measure(
            display_text,
            style.font.as_deref(),
            font_size,
            style.font_weight.unwrap_or_default(),
            style.font_style.unwrap_or_default(),
            letter_spacing,
            line_height,
            constraints.max_width
        );

        self.content_size.set((result.width, result.height));

        // Placeholder acts as a width floor even after content is typed, so the
        // box doesn't shrink below it once the field becomes non-empty.
        let placeholder_w = if self.placeholder.is_empty() {
            0.0
        } else {
            ctx.text.measure(
                &self.placeholder,
                style.font.as_deref(),
                font_size,
                style.font_weight.unwrap_or_default(),
                style.font_style.unwrap_or_default(),
                letter_spacing,
                line_height,
                None
            ).width
        };

        let text_w = result.width.max(placeholder_w);

        // Cumulative pixel offset for every character boundary, reused for
        // the caret, mouse hit-testing, and selection-highlight geometry.
        let char_count = self.content.chars().count();
        let mut offsets = Vec::with_capacity(char_count + 1);
        offsets.push(0.0);
        for i in 1..=char_count {
            let end_byte = self.byte_index_for(i);
            let result = ctx.text.measure(
                &self.content[..end_byte],
                style.font.as_deref(),
                font_size,
                style.font_weight.unwrap_or_default(),
                style.font_style.unwrap_or_default(),
                letter_spacing,
                line_height,
                None
            );
            offsets.push(result.width);
        }

        self.cursor_offset.set(*offsets.get(self.cursor_index.min(char_count)).unwrap_or(&0.0));
        *self.char_offsets.borrow_mut() = offsets;

        let padding = &style.padding.unwrap_or_default();
        let width =
            text_w +
            padding.left.to_physical(scale_factor) +
            padding.right.to_physical(scale_factor);
        let height =
            result.height +
            padding.top.to_physical(scale_factor) +
            padding.bottom.to_physical(scale_factor);
        let (width, height) = constraints.constrain_size(width, height);

        MeasureResult::new(width, height)
    }

    fn layout(&mut self, rect: LayoutBox) {
        self.layout_box = rect;
    }

    fn layout_box(&self) -> &LayoutBox {
        &self.layout_box
    }

    fn paint(&self, ctx: &mut PaintContext) {
        log::trace!(
            "paint -> '{:?}' x={} y={} dirty={:?}",
            self.content,
            self.layout_box.x,
            self.layout_box.y,
            self.is_dirty()
        );

        let style = &self.computed_style;

        self.paint_box(ctx);
        self.paint_outline(ctx);

        let padding = style.padding.unwrap_or_default();
        let sf = ctx.scale_factor;
        let (_, content_h) = self.content_size.get();

        let content_left = self.layout_box.x + padding.left.to_physical(sf);
        let content_width = (
            self.layout_box.width -
            padding.left.to_physical(sf) -
            padding.right.to_physical(sf)
        ).max(0.0);

        // Scrolls only as far as needed to keep the caret inside the visible
        // content area, and never scrolls past the point where empty space
        // would appear on the right (e.g. after deleting trailing text).
        let total_width = self.content_size.get().0;
        let cursor_offset = self.cursor_offset.get();
        let mut scroll = self.scroll_offset.get();
        if cursor_offset - scroll > content_width {
            scroll = cursor_offset - content_width;
        }
        if cursor_offset < scroll {
            scroll = cursor_offset;
        }
        let max_scroll = (total_width - content_width).max(0.0);
        scroll = scroll.clamp(0.0, max_scroll);
        self.scroll_offset.set(scroll);

        let text_x = content_left - scroll;
        let text_y =
            self.layout_box.y +
            padding.top.to_physical(sf) +
            (
                self.layout_box.height -
                padding.top.to_physical(sf) -
                padding.bottom.to_physical(sf) -
                content_h
            ).max(0.0) *
                0.5;

        let line_h = if content_h > 0.0 {
            content_h
        } else {
            style.font_size.map(|s| s.value()).unwrap_or(DEFAULT_FONT_SIZE.value()) *
                DEFAULT_LINE_HEIGHT_RATIO
        };
        let line_y = (self.layout_box.y + (self.layout_box.height - line_h).max(0.0) * 0.5).round();

        let text_clip = Some((
            content_left,
            self.layout_box.y,
            content_width,
            self.layout_box.height,
        ));

        let active_selection = self.interaction.focused.then(|| self.selection_range()).flatten();
        let mut sel_bounds: Option<(f32, f32)> = None;

        if let Some((start, end)) = active_selection {
            let offsets = self.char_offsets.borrow();
            if let (Some(&start_x), Some(&end_x)) = (offsets.get(start), offsets.get(end)) {
                let content_right = content_left + content_width;
                let sel_left = (text_x + start_x).max(content_left);
                let sel_right = (text_x + end_x).min(content_right);

                if sel_right > sel_left {
                    let sel_bg = style.selection_background.unwrap_or(
                        Color::rgba(90, 140, 230, 100)
                    );
                    ctx.draw_rect(RectCommand {
                        position: (sel_left, line_y),
                        size: (sel_right - sel_left, line_h),
                        background: Some(Background::Color(sel_bg)),
                        border_radius: style.selection_border_radius,
                        border_width: style.selection_border_width,
                        border_color: style.selection_border_color,
                        clip_rect: None,
                    });
                    sel_bounds = Some((sel_left, sel_right));
                }
            }
        }

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

        // Normal-colored text is skipped under the selection rect instead of
        // being drawn full-width and overlaid - drawing both would blend two
        // layers of glyphs on top of each other in the selected range.
        if let Some((sel_left, sel_right)) = sel_bounds {
            let content_right = content_left + content_width;
            if sel_left > content_left {
                ctx.draw_text(TextCommand {
                    text: display_text.clone(),
                    position: (text_x, text_y),
                    style: text_style.clone(),
                    max_width: None,
                    clip_rect: Some((
                        content_left,
                        self.layout_box.y,
                        sel_left - content_left,
                        self.layout_box.height,
                    )),
                });
            }
            if sel_right < content_right {
                ctx.draw_text(TextCommand {
                    text: display_text.clone(),
                    position: (text_x, text_y),
                    style: text_style.clone(),
                    max_width: None,
                    clip_rect: Some((
                        sel_right,
                        self.layout_box.y,
                        content_right - sel_right,
                        self.layout_box.height,
                    )),
                });
            }
        } else {
            ctx.draw_text(TextCommand {
                text: display_text.clone(),
                position: (text_x, text_y),
                style: text_style.clone(),
                max_width: None,
                clip_rect: text_clip,
            });
        }

        // Draws the exact same shaped text a second time, colored for selection
        // and scissored to the selection rect - reshaping only the substring
        // would produce different kerning and jitter glyph positions during drag.
        if let (Some((sel_left, sel_right)), Some(sel_fg)) = (sel_bounds, style.selection_color) {
            let mut sel_style = text_style;
            sel_style.color = Some(sel_fg);
            ctx.draw_text(TextCommand {
                text: display_text,
                position: (text_x, text_y),
                style: sel_style,
                max_width: None,
                clip_rect: Some((sel_left, line_y, sel_right - sel_left, line_h)),
            });
        }

        if self.interaction.focused && self.caret_visible.get() {
            let cursor_x = (text_x + self.cursor_offset.get()).round();
            let caret_color = style.caret_color.unwrap_or(style.color.unwrap_or(Color::BLACK));

            ctx.draw_rect(RectCommand {
                position: (cursor_x, line_y),
                size: (2.0, line_h),
                background: Some(Background::Color(caret_color)),
                border_radius: None,
                border_width: None,
                border_color: None,
                clip_rect: None,
            });
        }
    }

    fn event(&mut self, event: &InputEvent, ctx: &mut EventCtx) -> EventStatus {
        if !self.interaction.enabled {
            return EventStatus::Ignored;
        }

        // Applies any clipboard read that finished since the last event
        // (always relevant on WASM, where the read is asynchronous).
        self.poll_clipboard_paste(ctx);

        if let InputEvent::ModifiersChanged(modifiers) = event {
            self.current_modifiers.set(*modifiers);
            return EventStatus::Ignored;
        }

        if matches!(event, InputEvent::BlinkTick) {
            self.caret_visible.set(!self.caret_visible.get());
            self.dirty = true;
            ctx.request_redraw();
            return EventStatus::Handled;
        }

        // Key input bypasses Interaction::handle entirely: the generic handler
        // treats Enter/Space as a click-activation key, which would prevent
        // typing spaces and would fire on_click on every Enter press.
        if let InputEvent::KeyInput { event: key_event, modifiers } = event {
            if !self.interaction.focused || key_event.state != KeyState::Pressed {
                return EventStatus::Ignored;
            }
            self.handle_key(key_event, *modifiers, ctx);
            self.recompute_style();
            ctx.request_redraw();
            return EventStatus::Handled;
        }

        if let InputEvent::MouseInput { state, button, position } = event {
            let status = self.interaction.handle(event, ctx);

            if *button == MouseButton::Left {
                match state {
                    ElementState::Pressed => {
                        self.mouse_button_held.set(true);
                        self.handle_mouse_press(*position);
                    }
                    ElementState::Released => {
                        self.mouse_button_held.set(false);
                        self.dragging = false;
                        self.drag_word_selection = false;
                    }
                }
                self.recompute_style();
                self.dirty = true;
                ctx.request_redraw();
                return EventStatus::Handled;
            }

            return status;
        }

        // MouseMoved isn't handled by Interaction at all, so drag-selection
        // has to be driven from here directly.
        if let InputEvent::MouseMoved { position } = event {
            if self.dragging {
                self.handle_mouse_drag(*position);
                self.dirty = true;
                ctx.request_redraw();
                return EventStatus::Handled;
            }
            return EventStatus::Ignored;
        }

        // Interaction::handle() clears `pressed` on MouseExited, so the
        // real button-held state must be captured before calling it.
        let status = self.interaction.handle(event, ctx);

        if matches!(event, InputEvent::MouseExited) && !self.mouse_button_held.get() {
            self.dragging = false;
            self.drag_word_selection = false;
        }

        if matches!(event, InputEvent::FocusGained { .. }) {
            if self.focus_via_pointer.get() {
                self.focus_via_pointer.set(false);
            } else {
                self.cursor_index = self.content.chars().count();
                self.selection_anchor = None;
            }
            self.caret_visible.set(true);
        }

        if matches!(event, InputEvent::FocusLost) {
            self.selection_anchor = None;
            self.dragging = false;
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
            self.cursor_index == other.cursor_index &&
            self.selection_anchor == other.selection_anchor &&
            self.style == other.style &&
            self.hover_style == other.hover_style &&
            self.focus_style == other.focus_style &&
            self.disabled_style == other.disabled_style
    }

    fn cascade_style(&mut self, parent: &Style, anim: &mut AnimationManager) {
        self.inherited_style = parent.clone();
        self.recompute_style();
        if crate::animate_computed_style(self.anim_id, &mut self.computed_style, anim) {
            self.dirty = true;
        }
    }

    fn after_interaction_transfer(&mut self) {
        self.recompute_style();
    }

    fn transfer_interaction_state(&mut self, old: &dyn Widget) {
        if let (Some(new_i), Some(old_i)) = (self.interaction_mut(), old.interaction()) {
            new_i.transfer_from(old_i);
        }

        if let Some(old_tb) = old.as_any().downcast_ref::<TextBox>() {
            let new_len = self.content.chars().count();
            self.cursor_index = old_tb.cursor_index.min(new_len);
            self.selection_anchor = old_tb.selection_anchor.map(|a| a.min(new_len));
            self.dragging = old_tb.dragging;
            self.click_count.set(old_tb.click_count.get());
            self.last_click_time.set(old_tb.last_click_time.get());
            self.last_click_pos.set(old_tb.last_click_pos.get());
            self.focus_via_pointer.set(old_tb.focus_via_pointer.get());
            self.current_modifiers.set(old_tb.current_modifiers.get());
            self.caret_visible.set(old_tb.caret_visible.get());
            self.pending_paste = old_tb.pending_paste.clone();
            self.undo_stack = old_tb.undo_stack.clone();
            self.redo_stack = old_tb.redo_stack.clone();
        }
    }

    fn transfer_measured_state(&mut self, old: &dyn Widget) {
        if let Some(old) = old.as_any().downcast_ref::<TextBox>() {
            self.content_size.set(old.content_size.get());
            self.cursor_offset.set(old.cursor_offset.get());
            self.char_offsets.replace(old.char_offsets.borrow().clone());
            self.scroll_offset.set(old.scroll_offset.get());
            self.scale_factor.set(old.scale_factor.get());
            self.anim_id = old.anim_id;
        }
    }

    fn cancel_text_selection(&mut self) {
        self.selection_anchor = None;
        self.dragging = false;
        self.drag_word_selection = false;
        self.dirty = true;
    }

    fn blink_interval(&self) -> Option<std::time::Duration> {
        self.interaction.focused.then_some(std::time::Duration::from_millis(530))
    }

    fn anim_id(&self) -> WidgetId {
        self.anim_id
    }

    fn native_text_input(&self) -> Option<NativeTextInputSnapshot> {
        Some(NativeTextInputSnapshot {
            value: self.content.clone(),
            placeholder: self.placeholder.to_string(),
            max_length: self.max_length,
            read_only: self.read_only,
        })
    }

    fn set_native_text_value(&mut self, value: &str, ctx: &mut EventCtx) {
        if self.read_only {
            return;
        }
        self.push_undo_snapshot();
        self.content = value.to_string();
        self.cursor_index = self.content.chars().count();
        self.selection_anchor = None;
        self.dirty = true;
        self.notify_change(ctx);
    }

    #[cfg(target_arch = "wasm32")]
    fn sync_native_input(
        &self,
        input: &web_sys::HtmlInputElement,
        scale_factor: f32,
        canvas_offset: (f32, f32)
    ) {
        input.set_value(&self.content);
        let _ = input.set_attribute("placeholder", &self.placeholder);
        let _ = input.set_attribute("type", "text");
        input.set_read_only(self.read_only);

        // layout_box() is in physical pixels; CSS needs logical pixels.
        let b = self.layout_box;
        let (logical_x, logical_y, logical_w, logical_h) = (
            b.x / scale_factor,
            b.y / scale_factor,
            b.width / scale_factor,
            b.height / scale_factor,
        );

        let final_x = canvas_offset.0 + logical_x;
        let final_y = canvas_offset.1 + logical_y;

        let radius = self.computed_style.border.map(|b| b.radius.value()).unwrap_or(0.0);

        let _ = input.set_attribute(
            "style",
            &format!(
                "position:fixed;left:{final_x}px;top:{final_y}px;\
                 width:{logical_w}px;height:{logical_h}px;\
                 margin:0;padding:0;box-sizing:border-box;\
                 border-radius:{radius}px;\
                 opacity:0;border:none;outline:none;background:transparent;\
                 cursor:text;font-size:16px;z-index:2147483647;pointer-events:none;caret-color:transparent;"
            )
        );
    }
}
