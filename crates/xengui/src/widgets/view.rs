// SPDX-License-Identifier: Apache-2.0
use crate::{
    Background,
    Color,
    Constraints,
    EventCtx,
    EventStatus,
    InputEvent,
    Interaction,
    LayoutBox,
    Length,
    MeasureContext,
    MeasureResult,
    Overflow,
    PaintContext,
    RectCommand,
    Style,
    StyleBuilder,
    StylePatch,
    Widget,
};
use smol_str::SmolStr;
use std::cell::Cell;
use winit::event::{ ElementState, MouseButton, MouseScrollDelta };

const DEFAULT_SCROLLBAR_THICKNESS: f32 = 8.0;

#[derive(Clone, Copy)]
struct ScrollDrag {
    vertical: bool,
    start_mouse: f32,
    start_offset: f32,
}

fn point_in_rect(point: (f32, f32), rect: (f32, f32, f32, f32)) -> bool {
    let (px, py) = point;
    let (rx, ry, rw, rh) = rect;
    px >= rx && px <= rx + rw && py >= ry && py <= ry + rh
}

pub struct View {
    key: Option<SmolStr>,

    dirty: bool,
    style: Style,
    inherited_style: Style,
    computed_style: Style,

    hover_style: Option<Style>,
    pressed_style: Option<Style>,
    disabled_style: Option<Style>,

    layout_box: LayoutBox,
    children: Vec<Box<dyn Widget>>,
    interaction: Interaction,

    scroll_offset: Cell<(f32, f32)>,
    content_size: Cell<(f32, f32)>,
    scrollbar_thickness: f32,
    scrollbar_thumb_color: Option<Color>,
    scrollbar_track_color: Option<Color>,
    scrollbar_drag: Cell<Option<ScrollDrag>>,
    scrollbar_button_color: Option<Color>,
    scroll_step: f32,
}

impl View {
    pub fn new() -> Self {
        let mut view = Self {
            key: None,

            dirty: true,
            style: Style::default(),
            inherited_style: Style::default(),
            computed_style: Style::default(),

            hover_style: None,
            pressed_style: None,
            disabled_style: None,

            layout_box: LayoutBox::default(),
            children: Vec::new(),
            interaction: Interaction::new(),

            scroll_offset: Cell::new((0.0, 0.0)),
            content_size: Cell::new((0.0, 0.0)),
            scrollbar_thickness: DEFAULT_SCROLLBAR_THICKNESS,
            scrollbar_thumb_color: None,
            scrollbar_track_color: None,
            scrollbar_drag: Cell::new(None),
            scrollbar_button_color: None,
            scroll_step: 32.0,
        };

        view.recompute_style();
        view
    }

    /// Stable identity among siblings, kept across rebuilds even when this
    /// widget moves position (reorder, insert, remove). Use for list items
    /// instead of relying on array index.
    pub fn key(mut self, key: impl Into<SmolStr>) -> Self {
        self.key = Some(key.into());
        self
    }

    pub fn font(mut self, font: impl Into<SmolStr>) -> Self {
        self.style.font = Some(font.into());
        self.mark_dirty();
        self
    }

    /// Full style overlay to be applied during hover state - includes every field of Style
    /// such as background, border, color, font_size, padding, margin, etc.
    /// Only the fields you provide will overwrite the base style.
    pub fn hover_style(mut self, build: impl FnOnce(StylePatch) -> StylePatch) -> Self {
        self.hover_style = Some(build(StylePatch::new()).build());
        self.mark_dirty();
        self
    }

    /// Full style overlay to be applied during pressed state - includes every field of Style
    /// such as background, border, color, font_size, padding, margin, etc.
    /// Only the fields you provide will overwrite the base style.
    pub fn pressed_style(mut self, build: impl FnOnce(StylePatch) -> StylePatch) -> Self {
        self.pressed_style = Some(build(StylePatch::new()).build());
        self.mark_dirty();
        self
    }

    /// Full style overlay to be applied during disabled state - includes every field of Style
    /// such as background, border, color, font_size, padding, margin, etc.
    /// Only the fields you provide will overwrite the base style.
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
        self.interaction.focusable = focusable;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.interaction.set_enabled(enabled);
        self.mark_dirty();
        self
    }

    pub fn scrollbar_thickness(mut self, thickness: f32) -> Self {
        self.scrollbar_thickness = thickness;
        self.mark_dirty();
        self
    }

    pub fn scrollbar_thumb_color(mut self, color: Color) -> Self {
        self.scrollbar_thumb_color = Some(color);
        self.mark_dirty();
        self
    }

    pub fn scrollbar_track_color(mut self, color: Color) -> Self {
        self.scrollbar_track_color = Some(color);
        self.mark_dirty();
        self
    }

    pub fn scrollbar_button_color(mut self, color: Color) -> Self {
        self.scrollbar_button_color = Some(color);
        self.mark_dirty();
        self
    }

    pub fn scroll_step(mut self, step: f32) -> Self {
        self.scroll_step = step;
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

        let base = self.inherited_style.overlay(&self.style);

        self.computed_style = match patch {
            Some(patch) => base.overlay(patch),
            None => base,
        };
    }

    fn is_scrollable_x(&self) -> bool {
        matches!(self.computed_style.overflow_x, Some(Overflow::Scroll))
    }

    fn is_scrollable_y(&self) -> bool {
        matches!(self.computed_style.overflow_y, Some(Overflow::Scroll))
    }

    fn clips_x(&self) -> bool {
        matches!(self.computed_style.overflow_x, Some(Overflow::Scroll | Overflow::Hidden))
    }

    fn clips_y(&self) -> bool {
        matches!(self.computed_style.overflow_y, Some(Overflow::Scroll | Overflow::Hidden))
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
            self.is_scrollable_x() && self.max_scroll_x() > 0.0,
            self.is_scrollable_y() && self.max_scroll_y() > 0.0,
        )
    }

    fn vertical_track_bounds(&self) -> Option<(f32, f32)> {
        // Returns (track start y, track length) between the up/down buttons
        let (has_x, has_y) = self.scrollbar_visibility();
        if !has_y {
            return None;
        }
        let b = self.layout_box;
        let t = self.scrollbar_thickness;
        let full_h = if has_x { b.height - t } else { b.height };
        Some((b.y + t, (full_h - 2.0 * t).max(0.0)))
    }

    fn horizontal_track_bounds(&self) -> Option<(f32, f32)> {
        let (has_x, has_y) = self.scrollbar_visibility();
        if !has_x {
            return None;
        }
        let b = self.layout_box;
        let t = self.scrollbar_thickness;
        let full_w = if has_y { b.width - t } else { b.width };
        Some((b.x + t, (full_w - 2.0 * t).max(0.0)))
    }

    fn vertical_thumb_rect(&self) -> Option<(f32, f32, f32, f32)> {
        let (track_y, track_h) = self.vertical_track_bounds()?;
        let b = self.layout_box;
        let t = self.scrollbar_thickness;
        let content_h = self.content_size.get().1.max(b.height);

        let thumb_h = ((track_h * b.height) / content_h).max(t * 1.5).min(track_h);
        let max_offset = self.max_scroll_y();
        let progress = if max_offset > 0.0 { self.scroll_offset.get().1 / max_offset } else { 0.0 };
        let thumb_y = track_y + progress * (track_h - thumb_h);

        Some((b.x + b.width - t, thumb_y, t, thumb_h))
    }

    fn horizontal_thumb_rect(&self) -> Option<(f32, f32, f32, f32)> {
        let (track_x, track_w) = self.horizontal_track_bounds()?;
        let b = self.layout_box;
        let t = self.scrollbar_thickness;
        let content_w = self.content_size.get().0.max(b.width);

        let thumb_w = ((track_w * b.width) / content_w).max(t * 1.5).min(track_w);
        let max_offset = self.max_scroll_x();
        let progress = if max_offset > 0.0 { self.scroll_offset.get().0 / max_offset } else { 0.0 };
        let thumb_x = track_x + progress * (track_w - thumb_w);

        Some((thumb_x, b.y + b.height - t, thumb_w, t))
    }

    #[allow(clippy::type_complexity)]
    fn vertical_buttons(&self) -> Option<((f32, f32, f32, f32), (f32, f32, f32, f32))> {
        // (up button rect, down button rect)
        let (_, has_y) = self.scrollbar_visibility();
        if !has_y {
            return None;
        }
        let b = self.layout_box;
        let t = self.scrollbar_thickness;
        let (has_x, _) = self.scrollbar_visibility();
        let bottom = if has_x { b.y + b.height - t } else { b.y + b.height };
        Some(((b.x + b.width - t, b.y, t, t), (b.x + b.width - t, bottom - t, t, t)))
    }

    #[allow(clippy::type_complexity)]
    fn horizontal_buttons(&self) -> Option<((f32, f32, f32, f32), (f32, f32, f32, f32))> {
        // (left button rect, right button rect)
        let (has_x, _) = self.scrollbar_visibility();
        if !has_x {
            return None;
        }
        let b = self.layout_box;
        let t = self.scrollbar_thickness;
        let (_, has_y) = self.scrollbar_visibility();
        let right = if has_y { b.x + b.width - t } else { b.x + b.width };
        Some(((b.x, b.y + b.height - t, t, t), (right - t, b.y + b.height - t, t, t)))
    }

    fn nudge(&mut self, dx: f32, dy: f32, ctx: &mut EventCtx) {
        let current = self.scroll_offset.get();
        let next = self.clamp_offset((current.0 + dx, current.1 + dy));
        if next != current {
            self.scroll_offset.set(next);
            self.dirty = true;
            ctx.request_redraw();
        }
    }

    fn handle_wheel(
        &mut self,
        delta: MouseScrollDelta,
        position: (f32, f32),
        ctx: &mut EventCtx
    ) -> bool {
        if !self.hit_test(position) || (!self.is_scrollable_x() && !self.is_scrollable_y()) {
            return false;
        }

        // Trackpads report precise pixel deltas; wheel mice report line
        // counts, converted here to a comfortable pixel step per line.
        const LINE_HEIGHT: f32 = 40.0;
        let (raw_dx, raw_dy) = match delta {
            MouseScrollDelta::LineDelta(x, y) => (x * LINE_HEIGHT, y * LINE_HEIGHT),
            MouseScrollDelta::PixelDelta(pos) => (pos.x as f32, pos.y as f32),
        };

        // A vertical-only wheel scrolls horizontally when there is no
        // vertical overflow, matching common desktop conventions.
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

        let current = self.scroll_offset.get();
        // Flip the signs below if scroll direction feels inverted on your platform.
        let next = self.clamp_offset((current.0 - dx, current.1 - dy));

        if next != current {
            self.scroll_offset.set(next);
            self.dirty = true;
            ctx.request_redraw();
        }

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
                if let Some((up, down)) = self.vertical_buttons() {
                    if point_in_rect(position, up) {
                        self.nudge(0.0, -self.scroll_step, ctx);
                        return true;
                    }
                    if point_in_rect(position, down) {
                        self.nudge(0.0, self.scroll_step, ctx);
                        return true;
                    }
                }
                if let Some((left, right)) = self.horizontal_buttons() {
                    if point_in_rect(position, left) {
                        self.nudge(-self.scroll_step, 0.0, ctx);
                        return true;
                    }
                    if point_in_rect(position, right) {
                        self.nudge(self.scroll_step, 0.0, ctx);
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

    fn handle_scrollbar_drag(&mut self, position: (f32, f32), ctx: &mut EventCtx) -> bool {
        let Some(drag) = self.scrollbar_drag.get() else {
            return false;
        };

        let t = self.scrollbar_thickness;

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
            self.scroll_offset.set(next);
            self.dirty = true;
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
        &mut self.style
    }

    fn mark_dirty(&mut self) {
        self.dirty = true;
        self.recompute_style();
    }
}

crate::impl_interaction_builders!(View);

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
        &self.children
    }

    fn children_mut(&mut self) -> Option<&mut Vec<Box<dyn Widget>>> {
        Some(&mut self.children)
    }

    fn interaction(&self) -> Option<&Interaction> {
        Some(&self.interaction)
    }

    fn interaction_mut(&mut self) -> Option<&mut Interaction> {
        Some(&mut self.interaction)
    }

    fn scroll_offset(&self) -> (f32, f32) {
        self.scroll_offset.get()
    }

    fn set_content_size(&mut self, size: (f32, f32)) {
        self.content_size.set(size);
        // Re-clamp in case the scrollable range shrank (e.g. children were
        // removed, or the viewport was resized).
        let clamped = self.clamp_offset(self.scroll_offset.get());
        self.scroll_offset.set(clamped);
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
        self.paint_focus(ctx);
    }

    fn paint_overlay(&self, ctx: &mut PaintContext) {
        let track_color = self.scrollbar_track_color.unwrap_or(Color::TRANSPARENT);
        let thumb_color = self.scrollbar_thumb_color.unwrap_or(Color::NEUTRAL_400.with_alpha(160));
        let b = self.layout_box;
        let t = self.scrollbar_thickness;

        if let Some((x, y, w, h)) = self.vertical_thumb_rect() {
            if track_color.a() > 0.0 {
                ctx.draw_rect(RectCommand {
                    position: (b.x + b.width - t, b.y),
                    size: (t, b.height),
                    background: Some(Background::Color(track_color)),
                    border_radius: None,
                    border_width: None,
                    border_color: None,
                    clip_rect: None,
                });
            }
            ctx.draw_rect(RectCommand {
                position: (x, y),
                size: (w, h),
                background: Some(Background::Color(thumb_color)),
                border_radius: Some(Length::px(t * 0.5)),
                border_width: None,
                border_color: None,
                clip_rect: None,
            });
        }

        if let Some((x, y, w, h)) = self.horizontal_thumb_rect() {
            if track_color.a() > 0.0 {
                ctx.draw_rect(RectCommand {
                    position: (b.x, b.y + b.height - t),
                    size: (b.width, t),
                    background: Some(Background::Color(track_color)),
                    border_radius: None,
                    border_width: None,
                    border_color: None,
                    clip_rect: None,
                });
            }
            ctx.draw_rect(RectCommand {
                position: (x, y),
                size: (w, h),
                background: Some(Background::Color(thumb_color)),
                border_radius: Some(Length::px(t * 0.5)),
                border_width: None,
                border_color: None,
                clip_rect: None,
            });
        }

        if let Some((up, down)) = self.vertical_buttons() {
            let btn_color = self.scrollbar_button_color.unwrap_or(thumb_color);
            for (x, y, w, h) in [up, down] {
                ctx.draw_rect(RectCommand {
                    position: (x, y),
                    size: (w, h),
                    background: Some(Background::Color(btn_color)),
                    border_radius: Some(Length::px(2.0)),
                    border_width: None,
                    border_color: None,
                    clip_rect: None,
                });
            }
        }

        if let Some((left, right)) = self.horizontal_buttons() {
            let btn_color = self.scrollbar_button_color.unwrap_or(thumb_color);
            for (x, y, w, h) in [left, right] {
                ctx.draw_rect(RectCommand {
                    position: (x, y),
                    size: (w, h),
                    background: Some(Background::Color(btn_color)),
                    border_radius: Some(Length::px(2.0)),
                    border_width: None,
                    border_color: None,
                    clip_rect: None,
                });
            }
        }
    }

    fn event(&mut self, event: &InputEvent, ctx: &mut EventCtx) -> EventStatus {
        if
            let InputEvent::MouseWheel { delta, position } = event &&
            self.handle_wheel(*delta, *position, ctx)
        {
            return EventStatus::Handled;
        }

        if
            let InputEvent::MouseInput { state, button, position } = event &&
            self.handle_scrollbar_mouse(*state, *button, *position, ctx)
        {
            return EventStatus::Handled;
        }

        if
            let InputEvent::MouseMoved { position } = event &&
            self.handle_scrollbar_drag(*position, ctx)
        {
            return EventStatus::Handled;
        }

        if !self.interaction.is_active() {
            return EventStatus::Ignored;
        }

        let status = self.interaction.handle(event, ctx);

        if matches!(status, EventStatus::Handled) {
            self.recompute_style();
            self.dirty = true;
        }

        status
    }

    fn content_eq(&self, other: &dyn Widget) -> bool {
        let Some(other) = other.as_any().downcast_ref::<View>() else {
            return false;
        };

        self.style == other.style &&
            self.hover_style == other.hover_style &&
            self.pressed_style == other.pressed_style &&
            self.disabled_style == other.disabled_style
    }

    fn cascade_style(&mut self, parent: &Style) {
        self.inherited_style = parent.clone();
        self.recompute_style();
    }

    fn after_interaction_transfer(&mut self) {
        self.recompute_style();
    }

    fn transfer_measured_state(&mut self, old: &dyn Widget) {
        if let Some(old) = old.as_any().downcast_ref::<View>() {
            self.scroll_offset.set(old.scroll_offset.get());
            self.content_size.set(old.content_size.get());
        }
    }
}
