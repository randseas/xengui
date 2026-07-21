// SPDX-License-Identifier: Apache-2.0
use crate::{
    AnimKey,
    AnimLayer,
    AnimProperty,
    AnimationManager,
    Background,
    Constraints,
    DEFAULT_SCROLLBAR_HOVER_THICKNESS,
    ElementState,
    EventCtx,
    EventStatus,
    InputEvent,
    Interaction,
    IntoThemed,
    Key,
    KeyState,
    LayoutBox,
    Length,
    MeasureContext,
    MeasureResult,
    ModifiersState,
    MouseButton,
    MouseScrollDelta,
    Overflow,
    PaintContext,
    RectCommand,
    ResolvedScrollbar,
    Style,
    StyleBuilder,
    TriangleCommand,
    Widget,
    WidgetBase,
    WidgetId,
};
use smol_str::SmolStr;
use xen_animation::{ AnimValue, Easing, Transition };
use std::cell::Cell;

// Eased transition applied to scroll position when animating toward a
// wheel/nudge target; drag updates bypass this and snap instantly.
const SCROLL_TRANSITION: Transition = Transition::new(std::time::Duration::from_millis(250)).easing(
    Easing::EaseOut
);
const SCROLLBAR_THICKNESS_TRANSITION: Transition = Transition::new(
    std::time::Duration::from_millis(160)
).easing(Easing::EaseOut);

#[derive(Clone, Copy)]
struct ScrollDrag {
    vertical: bool,
    start_mouse: f32,
    start_offset: f32,
}

#[derive(Clone, Copy)]
enum ArrowDirection {
    Up,
    Down,
    Left,
    Right,
}

fn point_in_rect(point: (f32, f32), rect: (f32, f32, f32, f32)) -> bool {
    let (px, py) = point;
    let (rx, ry, rw, rh) = rect;
    px >= rx && px <= rx + rw && py >= ry && py <= ry + rh
}

// Builds the three points of a small centered arrow triangle within `rect`,
// pointing in `direction`.
fn arrow_triangle(
    rect: (f32, f32, f32, f32),
    direction: ArrowDirection
) -> ((f32, f32), (f32, f32), (f32, f32)) {
    let (x, y, w, h) = rect;
    let cx = x + w * 0.5;
    let cy = y + h * 0.5;
    let margin = w.min(h) * 0.3;
    let half = (w.min(h) * 0.5 - margin).max(1.0);

    match direction {
        ArrowDirection::Up => ((cx, cy - half), (cx - half, cy + half), (cx + half, cy + half)),
        ArrowDirection::Down => ((cx, cy + half), (cx - half, cy - half), (cx + half, cy - half)),
        ArrowDirection::Left => ((cx - half, cy), (cx + half, cy - half), (cx + half, cy + half)),
        ArrowDirection::Right => ((cx + half, cy), (cx - half, cy - half), (cx - half, cy + half)),
    }
}

pub struct View {
    base: WidgetBase,

    anim_id: WidgetId,
    layout_box: LayoutBox,
    children: Vec<Box<dyn Widget>>,
    scroll_offset: Cell<(f32, f32)>,
    scroll_target: Cell<(f32, f32)>,

    content_size: Cell<(f32, f32)>,
    scrollbar_drag: Cell<Option<ScrollDrag>>,
    scroll_step: f32,
    // Whether the pointer is over the scrollbar's own track/thumb/buttons,
    // separate from the widget's general hover state.
    scrollbar_hovered: Cell<bool>,
    // Animated scrollbar thickness, driven by the shared AnimationManager.
    scrollbar_thickness_anim: Cell<f32>,
}

impl View {
    pub fn new() -> Self {
        let mut view = Self {
            base: WidgetBase::new(Interaction::new()),

            anim_id: WidgetId::new_unique(),
            layout_box: LayoutBox::default(),
            children: Vec::new(),

            scroll_offset: Cell::new((0.0, 0.0)),
            scroll_target: Cell::new((0.0, 0.0)),
            content_size: Cell::new((0.0, 0.0)),
            scrollbar_drag: Cell::new(None),
            scroll_step: 96.0,
            scrollbar_hovered: Cell::new(false),
            scrollbar_thickness_anim: Cell::new(0.0),
        };

        view = view
            .selection_background(|theme: &crate::Theme| theme.selection)
            .selection_color(|theme: &crate::Theme| theme.selection_color)
            .caret_color(|theme: &crate::Theme| theme.caret_color)
            .selection_border_color(|theme: &crate::Theme| theme.selection_border_color)
            .selection_border_width(|theme: &crate::Theme| theme.selection_border_width)
            .selection_border_radius(|theme: &crate::Theme| theme.selection_border_radius);

        view.recompute_style();
        view
    }

    /// Stable identity among siblings, kept across rebuilds even when this
    /// widget moves position (reorder, insert, remove). Use for list items
    /// instead of relying on array index.
    pub fn key(mut self, key: impl Into<SmolStr>) -> Self {
        self.base.key = Some(key.into());
        self
    }

    pub fn font(mut self, font: impl Into<SmolStr>) -> Self {
        self.base.style.font = Some(font.into());
        self.mark_dirty();
        self
    }

    pub fn hover_background<M>(mut self, background: impl IntoThemed<Background, M>) -> Self {
        self.base.hover_style.get_or_insert_with(Style::default).background = Some(
            background.resolve_themed()
        );
        self.mark_dirty();
        self
    }

    pub fn pressed_background<M>(mut self, background: impl IntoThemed<Background, M>) -> Self {
        self.base.pressed_style.get_or_insert_with(Style::default).background = Some(
            background.resolve_themed()
        );
        self.mark_dirty();
        self
    }

    pub fn disabled_background<M>(mut self, background: impl IntoThemed<Background, M>) -> Self {
        self.base.disabled_style.get_or_insert_with(Style::default).background = Some(
            background.resolve_themed()
        );
        self.mark_dirty();
        self
    }

    pub fn child(mut self, child: impl Widget + 'static) -> Self {
        self.children.push(Box::new(child));
        self
    }

    /// Bulk variant of `child` for dynamically built lists where each item
    /// is already a boxed trait object (e.g. produced inside a `.map()`).
    pub fn children_vec(mut self, children: Vec<Box<dyn Widget>>) -> Self {
        self.children = children;
        self
    }

    pub fn focusable(mut self, focusable: bool) -> Self {
        self.base.interaction.focusable = focusable;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.base.interaction.set_enabled(enabled);
        self.mark_dirty();
        self
    }

    pub fn scroll_step(mut self, step: f32) -> Self {
        self.scroll_step = step;
        self
    }

    fn recompute_style(&mut self) {
        let patch = if !self.base.interaction.enabled {
            self.base.disabled_style.as_ref()
        } else if self.base.interaction.pressed {
            self.base.pressed_style.as_ref().or(self.base.hover_style.as_ref())
        } else if self.base.interaction.focused {
            self.base.focus_style.as_ref()
        } else if self.base.interaction.hovered {
            self.base.hover_style.as_ref()
        } else {
            None
        };

        let base = self.base.inherited_style.inherit_style(&self.base.style);

        self.base.computed_style = match patch {
            Some(patch) => base.overlay(patch),
            None => base,
        };

        self.base.interaction.hover_cursor = self.base.computed_style.cursor;
    }

    fn resolved_scrollbar(&self) -> ResolvedScrollbar {
        self.base.computed_style.scrollbar.unwrap_or_default().resolve()
    }

    // Overlays scrollbar_hover / scrollbar_pressed on top of the base scrollbar style.
    fn resolved_scrollbar_for_state(&self, hovered: bool, pressed: bool) -> ResolvedScrollbar {
        let base = self.resolved_scrollbar();
        let style = &self.base.computed_style;

        if pressed {
            return match style.scrollbar_pressed.as_ref().or(style.scrollbar_hover.as_ref()) {
                Some(patch) => base.patched(patch, DEFAULT_SCROLLBAR_HOVER_THICKNESS),
                None => ResolvedScrollbar { thickness: DEFAULT_SCROLLBAR_HOVER_THICKNESS, ..base },
            };
        }

        if hovered {
            return match style.scrollbar_hover.as_ref() {
                Some(patch) => base.patched(patch, DEFAULT_SCROLLBAR_HOVER_THICKNESS),
                None => ResolvedScrollbar { thickness: DEFAULT_SCROLLBAR_HOVER_THICKNESS, ..base },
            };
        }

        base
    }

    // Scrollbar style used for this frame: colors/borders switch immediately
    // with hover/pressed state, thickness comes from the animated value.
    fn active_scrollbar(&self) -> ResolvedScrollbar {
        let pressed = self.scrollbar_drag.get().is_some();
        let hovered = self.scrollbar_hovered.get();
        let mut sb = self.resolved_scrollbar_for_state(hovered, pressed);
        sb.thickness = self.current_scrollbar_thickness();
        sb
    }

    fn target_scrollbar_thickness(&self) -> f32 {
        let pressed = self.scrollbar_drag.get().is_some();
        let hovered = self.scrollbar_hovered.get();
        self.resolved_scrollbar_for_state(hovered, pressed).thickness
    }

    fn current_scrollbar_thickness(&self) -> f32 {
        self.scrollbar_thickness_anim.get()
    }

    // Pulls the scrollbar thickness toward its hover/pressed target through
    // the shared AnimationManager, called once per frame from cascade_style.
    fn animate_scrollbar_thickness(&mut self, anim: &mut AnimationManager) {
        let target = self.target_scrollbar_thickness();
        let key = AnimKey {
            widget: self.anim_id,
            layer: AnimLayer::Root,
            property: AnimProperty::ScrollbarThickness,
        };

        anim.set_target(
            key,
            AnimValue([target, 0.0, 0.0, 0.0]),
            Some(SCROLLBAR_THICKNESS_TRANSITION)
        );

        match anim.value(key) {
            Some(v) => {
                self.scrollbar_thickness_anim.set(v.0[0]);
                self.base.dirty = true;
            }
            None => self.scrollbar_thickness_anim.set(target),
        }
    }

    // Pulls scroll_offset toward scroll_target through the shared
    // AnimationManager; a thumb drag snaps instantly instead of easing so
    // it tracks the cursor 1:1.
    fn animate_scroll(&mut self, anim: &mut AnimationManager) {
        let target = self.scroll_target.get();
        let key = AnimKey {
            widget: self.anim_id,
            layer: AnimLayer::Root,
            property: AnimProperty::ScrollOffset,
        };

        let transition = if self.scrollbar_drag.get().is_some() {
            None
        } else {
            Some(SCROLL_TRANSITION)
        };

        anim.set_target(key, AnimValue([target.0, target.1, 0.0, 0.0]), transition);

        match anim.value(key) {
            Some(v) => {
                self.scroll_offset.set((v.0[0], v.0[1]));
                self.base.dirty = true;
            }
            None => self.scroll_offset.set(target),
        }
    }

    // Hit area for scrollbar hover detection; uses the hover thickness so
    // a thin, unhovered bar is still easy to reach with the pointer.
    fn point_in_scrollbar(&self, point: (f32, f32)) -> bool {
        let (has_x, has_y) = self.scrollbar_visibility();
        if !has_x && !has_y {
            return false;
        }

        let b = self.layout_box;
        let t = self.resolved_scrollbar_for_state(true, false).thickness;

        if has_y && point_in_rect(point, (b.x + b.width - t, b.y, t, b.height)) {
            return true;
        }
        if has_x && point_in_rect(point, (b.x, b.y + b.height - t, b.width, t)) {
            return true;
        }
        false
    }

    fn is_scrollable_x(&self) -> bool {
        matches!(self.base.computed_style.overflow_x, Some(Overflow::Scroll | Overflow::Auto))
    }

    fn is_scrollable_y(&self) -> bool {
        matches!(self.base.computed_style.overflow_y, Some(Overflow::Scroll | Overflow::Auto))
    }

    fn clips_x(&self) -> bool {
        matches!(
            self.base.computed_style.overflow_x,
            Some(Overflow::Scroll | Overflow::Auto | Overflow::Hidden)
        )
    }

    fn clips_y(&self) -> bool {
        matches!(
            self.base.computed_style.overflow_y,
            Some(Overflow::Scroll | Overflow::Auto | Overflow::Hidden)
        )
    }

    fn max_scroll_x(&self) -> f32 {
        (self.content_size.get().0 - self.layout_box.width).max(0.0)
    }

    fn max_scroll_y(&self) -> f32 {
        (self.content_size.get().1 - self.layout_box.height).max(0.0)
    }

    fn clamp_offset(&self, offset: (f32, f32)) -> (f32, f32) {
        (offset.0.clamp(0.0, self.max_scroll_x()), offset.1.clamp(0.0, self.max_scroll_y()))
    }

    // Whether each axis actually has a visible scrollbar right now (enabled
    // for that axis and there is something to scroll to).
    fn scrollbar_visibility(&self) -> (bool, bool) {
        (
            self.is_scrollable_x() &&
                (self.base.computed_style.overflow_x == Some(Overflow::Scroll) ||
                    self.max_scroll_x() > 0.0),
            self.is_scrollable_y() &&
                (self.base.computed_style.overflow_y == Some(Overflow::Scroll) ||
                    self.max_scroll_y() > 0.0),
        )
    }

    fn vertical_track_bounds(&self) -> Option<(f32, f32)> {
        let (has_x, has_y) = self.scrollbar_visibility();
        if !has_y {
            return None;
        }
        let b = self.layout_box;
        let t = self.active_scrollbar().thickness;
        let full_h = if has_x { b.height - t } else { b.height };
        Some((b.y + t, (full_h - 2.0 * t).max(0.0)))
    }

    fn horizontal_track_bounds(&self) -> Option<(f32, f32)> {
        let (has_x, has_y) = self.scrollbar_visibility();
        if !has_x {
            return None;
        }
        let b = self.layout_box;
        let t = self.active_scrollbar().thickness;
        let full_w = if has_y { b.width - t } else { b.width };
        Some((b.x + t, (full_w - 2.0 * t).max(0.0)))
    }

    fn vertical_thumb_rect(&self) -> Option<(f32, f32, f32, f32)> {
        let (track_y, track_h) = self.vertical_track_bounds()?;
        let b = self.layout_box;
        let sb = self.active_scrollbar();
        let content_h = self.content_size.get().1.max(b.height);

        let thumb_h = ((track_h * b.height) / content_h).max(sb.min_thumb_length).min(track_h);
        let max_offset = self.max_scroll_y();
        let progress = if max_offset > 0.0 { self.scroll_offset.get().1 / max_offset } else { 0.0 };
        let thumb_y = track_y + progress * (track_h - thumb_h);

        Some((b.x + b.width - sb.thickness, thumb_y, sb.thickness, thumb_h))
    }

    fn horizontal_thumb_rect(&self) -> Option<(f32, f32, f32, f32)> {
        let (track_x, track_w) = self.horizontal_track_bounds()?;
        let b = self.layout_box;
        let sb = self.active_scrollbar();
        let content_w = self.content_size.get().0.max(b.width);

        let thumb_w = ((track_w * b.width) / content_w).max(sb.min_thumb_length).min(track_w);
        let max_offset = self.max_scroll_x();
        let progress = if max_offset > 0.0 { self.scroll_offset.get().0 / max_offset } else { 0.0 };
        let thumb_x = track_x + progress * (track_w - thumb_w);

        Some((thumb_x, b.y + b.height - sb.thickness, thumb_w, sb.thickness))
    }

    #[allow(clippy::type_complexity)]
    fn vertical_buttons(&self) -> Option<((f32, f32, f32, f32), (f32, f32, f32, f32))> {
        let (_, has_y) = self.scrollbar_visibility();
        if !has_y {
            return None;
        }
        let b = self.layout_box;
        let t = self.active_scrollbar().thickness;
        let (has_x, _) = self.scrollbar_visibility();
        let bottom = if has_x { b.y + b.height - t } else { b.y + b.height };
        Some(((b.x + b.width - t, b.y, t, t), (b.x + b.width - t, bottom - t, t, t)))
    }

    #[allow(clippy::type_complexity)]
    fn horizontal_buttons(&self) -> Option<((f32, f32, f32, f32), (f32, f32, f32, f32))> {
        let (has_x, _) = self.scrollbar_visibility();
        if !has_x {
            return None;
        }
        let b = self.layout_box;
        let t = self.active_scrollbar().thickness;
        let (_, has_y) = self.scrollbar_visibility();
        let right = if has_y { b.x + b.width - t } else { b.x + b.width };
        Some(((b.x, b.y + b.height - t, t, t), (right - t, b.y + b.height - t, t, t)))
    }

    fn start_scroll_animation(&mut self, target: (f32, f32), ctx: &mut EventCtx) {
        self.scroll_target.set(target);
        self.base.dirty = true;
        ctx.request_redraw();
    }

    fn nudge(&mut self, dx: f32, dy: f32, ctx: &mut EventCtx) {
        let current = self.scroll_target.get();
        let next = self.clamp_offset((current.0 + dx, current.1 + dy));
        if next != current {
            self.start_scroll_animation(next, ctx);
        }
    }

    fn handle_page_key(&mut self, key: Key, modifiers: ModifiersState, ctx: &mut EventCtx) -> bool {
        if modifiers.shift {
            if !self.is_scrollable_x() {
                return false;
            }

            let dx: f32 = match key {
                Key::PageUp => -self.layout_box.width,
                Key::PageDown => self.layout_box.width,
                _ => {
                    return false;
                }
            };

            let current = self.scroll_target.get();
            let next = self.clamp_offset((current.0 + dx, current.1));
            if next == current {
                return false;
            }

            self.start_scroll_animation(next, ctx);
            return true;
        }

        if !self.is_scrollable_y() {
            return false;
        }

        let dy: f32 = match key {
            Key::PageUp => -self.layout_box.height,
            Key::PageDown => self.layout_box.height,
            _ => {
                return false;
            }
        };

        let current = self.scroll_target.get();
        let next = self.clamp_offset((current.0, current.1 + dy));
        if next == current {
            return false;
        }

        self.start_scroll_animation(next, ctx);
        true
    }

    fn handle_key(&mut self, key: Key, modifiers: ModifiersState, ctx: &mut EventCtx) -> bool {
        self.handle_page_key(key, modifiers, ctx)
    }

    fn handle_wheel(
        &mut self,
        delta: MouseScrollDelta,
        position: (f32, f32),
        modifiers: ModifiersState,
        ctx: &mut EventCtx,
        scroll_step: f32
    ) -> bool {
        if !self.hit_test(position) || (!self.is_scrollable_x() && !self.is_scrollable_y()) {
            return false;
        }

        let (raw_dx, raw_dy) = match delta {
            MouseScrollDelta::LineDelta(x, y) => (x * scroll_step, y * scroll_step),
            MouseScrollDelta::PixelDelta(x, y) => (x as f32, y as f32),
        };

        let (raw_dx, raw_dy) = if modifiers.shift { (raw_dy, raw_dx) } else { (raw_dx, raw_dy) };

        let (dx, dy) = if self.is_scrollable_y() {
            (raw_dx, raw_dy)
        } else {
            (raw_dx + raw_dy, 0.0)
        };
        let dx = if self.is_scrollable_x() { dx } else { 0.0 };
        let dy = if self.is_scrollable_y() { dy } else { 0.0 };

        if dx == 0.0 && dy == 0.0 {
            return false;
        }

        let current = self.scroll_target.get();
        let next = self.clamp_offset((current.0 - dx, current.1 - dy));

        // A no-op scroll (already at the edge, or this widget inherited a
        // scrollable style from an ancestor without actually overflowing)
        // must report unhandled so the event bubbles up to a real scrollable parent.
        if next == current {
            return false;
        }

        self.start_scroll_animation(next, ctx);
        true
    }

    fn handle_scrollbar_mouse(
        &mut self,
        state: ElementState,
        button: MouseButton,
        position: (f32, f32),
        ctx: &mut EventCtx
    ) -> bool {
        if button != MouseButton::Left {
            return false;
        }

        match state {
            ElementState::Pressed => {
                let target = self.scroll_target.get();

                if let Some((up, down)) = self.vertical_buttons() {
                    if point_in_rect(position, up) {
                        if target.1 > 0.0 {
                            self.nudge(0.0, -self.scroll_step, ctx);
                        }
                        return true;
                    }
                    if point_in_rect(position, down) {
                        if target.1 < self.max_scroll_y() {
                            self.nudge(0.0, self.scroll_step, ctx);
                        }
                        return true;
                    }
                }
                if let Some((left, right)) = self.horizontal_buttons() {
                    if point_in_rect(position, left) {
                        if target.0 > 0.0 {
                            self.nudge(-self.scroll_step, 0.0, ctx);
                        }
                        return true;
                    }
                    if point_in_rect(position, right) {
                        if target.0 < self.max_scroll_x() {
                            self.nudge(self.scroll_step, 0.0, ctx);
                        }
                        return true;
                    }
                }
                if let Some(thumb) = self.vertical_thumb_rect() && point_in_rect(position, thumb) {
                    self.scrollbar_drag.set(
                        Some(ScrollDrag {
                            vertical: true,
                            start_mouse: position.1,
                            start_offset: self.scroll_offset.get().1,
                        })
                    );
                    return true;
                }
                if let Some(thumb) = self.horizontal_thumb_rect() && point_in_rect(position, thumb) {
                    self.scrollbar_drag.set(
                        Some(ScrollDrag {
                            vertical: false,
                            start_mouse: position.0,
                            start_offset: self.scroll_offset.get().0,
                        })
                    );
                    return true;
                }

                // Clicking an empty stretch of track jumps the thumb straight
                // to that point instead of requiring a drag or repeated nudges.
                if let Some((track_y, track_h)) = self.vertical_track_bounds() {
                    let t = self.active_scrollbar().thickness;
                    let b = self.layout_box;
                    if point_in_rect(position, (b.x + b.width - t, track_y, t, track_h)) {
                        if let Some(target_y) = self.vertical_track_offset_for(position.1) {
                            let next = self.clamp_offset((target.0, target_y));
                            if next != target {
                                self.start_scroll_animation(next, ctx);
                            }
                        }
                        return true;
                    }
                }
                if let Some((track_x, track_w)) = self.horizontal_track_bounds() {
                    let t = self.active_scrollbar().thickness;
                    let b = self.layout_box;
                    if point_in_rect(position, (track_x, b.y + b.height - t, track_w, t)) {
                        if let Some(target_x) = self.horizontal_track_offset_for(position.0) {
                            let next = self.clamp_offset((target_x, target.1));
                            if next != target {
                                self.start_scroll_animation(next, ctx);
                            }
                        }
                        return true;
                    }
                }

                false
            }
            ElementState::Released => {
                if self.scrollbar_drag.get().is_some() {
                    self.scrollbar_drag.set(None);
                    ctx.request_redraw();
                    true
                } else {
                    false
                }
            }
        }
    }

    fn vertical_track_offset_for(&self, mouse_y: f32) -> Option<f32> {
        let (track_y, track_h) = self.vertical_track_bounds()?;
        let b = self.layout_box;
        let sb = self.active_scrollbar();
        let content_h = self.content_size.get().1.max(b.height);
        let thumb_h = ((track_h * b.height) / content_h).max(sb.min_thumb_length).min(track_h);
        let travel = (track_h - thumb_h).max(1.0);
        let progress = ((mouse_y - track_y - thumb_h * 0.5) / travel).clamp(0.0, 1.0);
        Some(progress * self.max_scroll_y())
    }

    fn horizontal_track_offset_for(&self, mouse_x: f32) -> Option<f32> {
        let (track_x, track_w) = self.horizontal_track_bounds()?;
        let b = self.layout_box;
        let sb = self.active_scrollbar();
        let content_w = self.content_size.get().0.max(b.width);
        let thumb_w = ((track_w * b.width) / content_w).max(sb.min_thumb_length).min(track_w);
        let travel = (track_w - thumb_w).max(1.0);
        let progress = ((mouse_x - track_x - thumb_w * 0.5) / travel).clamp(0.0, 1.0);
        Some(progress * self.max_scroll_x())
    }

    fn handle_scrollbar_drag(&mut self, position: (f32, f32), ctx: &mut EventCtx) -> bool {
        let Some(drag) = self.scrollbar_drag.get() else {
            return false;
        };

        let t = self.active_scrollbar().thickness;

        let (track_len, content_len, viewport_len, max_offset) = if drag.vertical {
            let (_, track) = self.vertical_track_bounds().unwrap_or((0.0, 0.0));
            (
                track,
                self.content_size.get().1.max(self.layout_box.height),
                self.layout_box.height,
                self.max_scroll_y(),
            )
        } else {
            let (_, track) = self.horizontal_track_bounds().unwrap_or((0.0, 0.0));
            (
                track,
                self.content_size.get().0.max(self.layout_box.width),
                self.layout_box.width,
                self.max_scroll_x(),
            )
        };

        let thumb_len = ((track_len * viewport_len) / content_len).max(t * 2.0).min(track_len);
        let travel = (track_len - thumb_len).max(1.0);

        let mouse_pos = if drag.vertical { position.1 } else { position.0 };
        let delta_offset = (mouse_pos - drag.start_mouse) * (max_offset / travel);

        let current = self.scroll_offset.get();
        let next = if drag.vertical {
            self.clamp_offset((current.0, drag.start_offset + delta_offset))
        } else {
            self.clamp_offset((drag.start_offset + delta_offset, current.1))
        };

        if next != current {
            // Thumb drag tracks the cursor 1:1 - no easing here, only wheel
            // and button nudges go through the animated path.
            self.scroll_offset.set(next);
            self.scroll_target.set(next);
            self.base.dirty = true;
            ctx.request_redraw();
        }

        true
    }
}

impl Default for View {
    fn default() -> Self {
        Self::new()
    }
}

impl StyleBuilder for View {
    fn style_mut(&mut self) -> &mut Style {
        &mut self.base.style
    }

    fn mark_dirty(&mut self) {
        self.base.dirty = true;
        self.recompute_style();
    }
}

crate::impl_interaction_builders!(base View);
crate::impl_themed_style_builders!(base View; hover_style => hover_style, pressed_style => pressed_style, disabled_style => disabled_style, focus_style => focus_style);

impl Widget for View {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn debug_name(&self) -> &'static str {
        "Widget#View"
    }

    fn get_key(&self) -> Option<&SmolStr> {
        self.base.key.as_ref()
    }

    fn is_dirty(&self) -> bool {
        self.base.dirty
    }

    fn set_dirty(&mut self, dirty: bool) {
        self.base.dirty = dirty;
    }

    fn style(&self) -> &Style {
        &self.base.style
    }

    fn style_mut(&mut self) -> &mut Style {
        &mut self.base.style
    }

    fn computed_style(&self) -> &Style {
        &self.base.computed_style
    }

    fn children(&self) -> &[Box<dyn Widget>] {
        &self.children
    }

    fn children_mut(&mut self) -> Option<&mut Vec<Box<dyn Widget>>> {
        Some(&mut self.children)
    }

    fn interaction(&self) -> Option<&Interaction> {
        Some(&self.base.interaction)
    }

    fn interaction_mut(&mut self) -> Option<&mut Interaction> {
        Some(&mut self.base.interaction)
    }

    fn scroll_offset(&self) -> (f32, f32) {
        self.scroll_offset.get()
    }

    fn set_content_size(&mut self, size: (f32, f32)) {
        self.content_size.set(size);
        // Re-clamp in case the scrollable range shrank (e.g. children were
        // removed, or the viewport was resized).
        self.scroll_offset.set(self.clamp_offset(self.scroll_offset.get()));
        self.scroll_target.set(self.clamp_offset(self.scroll_target.get()));
    }

    fn clip_children(&self) -> Option<(f32, f32, f32, f32)> {
        if self.clips_x() || self.clips_y() {
            let b = &self.layout_box;
            Some((b.x, b.y, b.width, b.height))
        } else {
            None
        }
    }

    fn measure(&self, _ctx: &mut MeasureContext, _constraints: Constraints) -> MeasureResult {
        MeasureResult::new(0.0, 0.0)
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
            self.get_key(),
            self.layout_box.x,
            self.layout_box.y,
            self.is_dirty()
        );

        self.paint_box(ctx);
        self.paint_outline(ctx);
    }

    fn paint_overlay(&self, ctx: &mut PaintContext) {
        let sb = self.active_scrollbar();
        let b = self.layout_box;
        let t = sb.thickness;

        let thumb_border_width = (sb.thumb_border_width > 0.0).then(||
            Length::px(sb.thumb_border_width)
        );
        let track_border_width = (sb.track_border_width > 0.0).then(||
            Length::px(sb.track_border_width)
        );

        if let Some((x, y, w, h)) = self.vertical_thumb_rect() {
            if sb.track_color.a() > 0.0 || track_border_width.is_some() {
                ctx.draw_rect(RectCommand {
                    position: (b.x + b.width - t, b.y),
                    size: (t, b.height),
                    background: Some(Background::Color(sb.track_color)),
                    border_radius: None,
                    border_width: track_border_width,
                    border_color: Some(sb.track_border_color),
                    clip_rect: None,
                });
            }
            ctx.draw_rect(RectCommand {
                position: (x, y),
                size: (w, h),
                background: Some(Background::Color(sb.thumb_color)),
                border_radius: Some(Length::px(sb.thumb_radius)),
                border_width: thumb_border_width,
                border_color: Some(sb.thumb_border_color),
                clip_rect: None,
            });
        }

        if let Some((x, y, w, h)) = self.horizontal_thumb_rect() {
            if sb.track_color.a() > 0.0 || track_border_width.is_some() {
                ctx.draw_rect(RectCommand {
                    position: (b.x, b.y + b.height - t),
                    size: (b.width, t),
                    background: Some(Background::Color(sb.track_color)),
                    border_radius: None,
                    border_width: track_border_width,
                    border_color: Some(sb.track_border_color),
                    clip_rect: None,
                });
            }
            ctx.draw_rect(RectCommand {
                position: (x, y),
                size: (w, h),
                background: Some(Background::Color(sb.thumb_color)),
                border_radius: Some(Length::px(sb.thumb_radius)),
                border_width: thumb_border_width,
                border_color: Some(sb.thumb_border_color),
                clip_rect: None,
            });
        }

        if let Some((up, down)) = self.vertical_buttons() {
            let target = self.scroll_target.get();

            for (rect, dir, disabled) in [
                (up, ArrowDirection::Up, target.1 <= 0.0),
                (down, ArrowDirection::Down, target.1 >= self.max_scroll_y()),
            ] {
                let dim = if disabled { 0.35 } else { 1.0 };
                let (p0, p1, p2) = arrow_triangle(rect, dir);
                ctx.draw_triangle(TriangleCommand {
                    p0,
                    p1,
                    p2,
                    color: sb.arrow_color.with_alpha_f32(sb.arrow_color.a() * dim),
                    clip_rect: None,
                });
            }
        }

        if let Some((left, right)) = self.horizontal_buttons() {
            let target = self.scroll_target.get();

            for (rect, dir, disabled) in [
                (left, ArrowDirection::Left, target.0 <= 0.0),
                (right, ArrowDirection::Right, target.0 >= self.max_scroll_x()),
            ] {
                let dim = if disabled { 0.35 } else { 1.0 };
                let (p0, p1, p2) = arrow_triangle(rect, dir);
                ctx.draw_triangle(TriangleCommand {
                    p0,
                    p1,
                    p2,
                    color: sb.arrow_color.with_alpha_f32(sb.arrow_color.a() * dim),
                    clip_rect: None,
                });
            }
        }
    }

    fn event(&mut self, event: &InputEvent, ctx: &mut EventCtx) -> EventStatus {
        if let InputEvent::MouseMoved { position } = event {
            if self.scrollbar_drag.get().is_some() && self.handle_scrollbar_drag(*position, ctx) {
                return EventStatus::Handled;
            }

            let now_hovered = self.point_in_scrollbar(*position);
            if now_hovered != self.scrollbar_hovered.get() {
                self.scrollbar_hovered.set(now_hovered);
                self.base.dirty = true;
                ctx.request_redraw();
            }
        }

        if matches!(event, InputEvent::MouseExited) && self.scrollbar_hovered.get() {
            self.scrollbar_hovered.set(false);
            self.base.dirty = true;
            ctx.request_redraw();
        }

        if
            let InputEvent::MouseWheel { delta, position, modifiers } = event &&
            self.handle_wheel(*delta, *position, *modifiers, ctx, self.scroll_step)
        {
            return EventStatus::Handled;
        }

        if
            let InputEvent::KeyInput { event: key_event, modifiers } = event &&
            key_event.state == KeyState::Pressed &&
            self.handle_key(key_event.key, *modifiers, ctx)
        {
            return EventStatus::Handled;
        }

        if
            let InputEvent::MouseInput { state, button, position } = event &&
            self.handle_scrollbar_mouse(*state, *button, *position, ctx)
        {
            return EventStatus::Handled;
        }

        if !self.base.interaction.is_active() {
            return EventStatus::Ignored;
        }

        let before_style = self.base.computed_style.clone();
        let before_focus_visible = self.base.interaction.focus_visible;

        let status = self.base.interaction.handle(event, ctx);

        if matches!(status, EventStatus::Handled) {
            self.recompute_style();

            if
                self.base.computed_style != before_style ||
                self.base.interaction.focus_visible != before_focus_visible
            {
                self.base.dirty = true;
                ctx.request_redraw();
            }
        }

        status
    }

    fn content_eq(&self, other: &dyn Widget) -> bool {
        let Some(other) = other.as_any().downcast_ref::<View>() else {
            return false;
        };

        self.base.style == other.base.style &&
            self.base.hover_style == other.base.hover_style &&
            self.base.pressed_style == other.base.pressed_style &&
            self.base.disabled_style == other.base.disabled_style &&
            self.base.focus_style == other.base.focus_style
    }

    fn cascade_style(&mut self, parent: &Style, anim: &mut AnimationManager) {
        self.base.inherited_style = parent.clone();
        self.recompute_style();

        if crate::animate_computed_style(self.anim_id, &mut self.base.computed_style, anim) {
            self.base.dirty = true;
        }

        self.animate_scroll(anim);
        self.animate_scrollbar_thickness(anim);

        for child in self.children.iter_mut() {
            child.cascade_style(&self.base.computed_style, anim);
        }
    }

    fn after_interaction_transfer(&mut self) {
        self.recompute_style();
    }

    fn transfer_measured_state(&mut self, old: &dyn Widget) {
        if let Some(old) = old.as_any().downcast_ref::<View>() {
            self.scroll_offset.set(old.scroll_offset.get());
            self.scroll_target.set(old.scroll_target.get());
            self.content_size.set(old.content_size.get());
            self.scrollbar_hovered.set(old.scrollbar_hovered.get());
            self.scrollbar_thickness_anim.set(old.scrollbar_thickness_anim.get());
            self.anim_id = old.anim_id;
        }
    }

    fn anim_id(&self) -> WidgetId {
        self.anim_id
    }
}
