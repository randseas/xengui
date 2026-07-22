// SPDX-License-Identifier: Apache-2.0
use crate::{
    AnimKey,
    AnimLayer,
    AnimProperty,
    AnimValue,
    AnimationManager,
    Background,
    Border,
    Color,
    Constraints,
    Easing,
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
    MouseButton,
    PaintContext,
    RectCommand,
    Style,
    StyleBuilder,
    TextCommand,
    Transition,
    Widget,
    WidgetBase,
    WidgetId,
    properties::{ DEFAULT_FONT_SIZE, DEFAULT_LINE_HEIGHT_RATIO },
};
use smol_str::SmolStr;
use std::cell::Cell;
use std::time::Duration;

pub struct ContextMenuItem {
    label: SmolStr,
    #[allow(clippy::type_complexity)]
    on_click: Option<Box<dyn FnMut(&mut EventCtx)>>,
    enabled: bool,
}

impl ContextMenuItem {
    pub fn new(label: impl Into<SmolStr>) -> Self {
        Self { label: label.into(), on_click: None, enabled: true }
    }

    pub fn on_click(mut self, f: impl FnMut(&mut EventCtx) + 'static) -> Self {
        self.on_click = Some(Box::new(f));
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

enum ContextMenuEntry {
    Item(ContextMenuItem),
    Divider,
}

const ITEM_HEIGHT: f32 = 30.0;
const ITEM_PADDING_X: f32 = 12.0;
const MENU_PADDING: f32 = 4.0;
const MENU_WIDTH: f32 = 160.0;
const DIVIDER_HEIGHT: f32 = 9.0;
const DIVIDER_LINE_THICKNESS: f32 = 1.0;

const OPACITY_TRANSITION: Transition = Transition::new(Duration::from_millis(150)).easing(
    Easing::EaseInOut
);

fn faded_background(bg: Background, opacity: f32) -> Background {
    match bg {
        Background::Color(c) => Background::Color(c.with_alpha_f32(c.a() * opacity)),
    }
}

/// A page-wide right-click popup menu. Wraps arbitrary content (like
/// `View`) and should sit near the root of the tree so `paint_overlay`
/// runs after every other widget's own paint pass, guaranteeing the
/// popup always renders on top regardless of sibling paint order.
///
/// Colors left unset (`menu_background`, `item_hover_background`,
/// `item_text_color` never called) automatically follow the active
/// theme via `current_theme()` instead of a fixed light/dark value.
pub struct ContextMenu {
    base: WidgetBase,
    anim_id: WidgetId,

    children: Vec<Box<dyn Widget>>,
    entries: Vec<ContextMenuEntry>,

    open: Cell<bool>,
    opacity_anim: Cell<f32>,
    menu_pos: Cell<(f32, f32)>,
    menu_size: Cell<(f32, f32)>,
    hovered_index: Cell<Option<usize>>,

    item_hover_background: Option<Background>,
    item_text_color: Option<Color>,
    divider_color: Option<Color>,

    background: Option<Background>,
    border: Option<Border>,
    menu_padding: Option<f32>,

    layout_box: LayoutBox,
}

impl ContextMenu {
    pub fn new() -> Self {
        let mut menu = Self {
            base: WidgetBase::new(Interaction::new()),
            anim_id: WidgetId::new_unique(),
            children: Vec::new(),
            entries: Vec::new(),
            open: Cell::new(false),
            opacity_anim: Cell::new(0.0),
            menu_pos: Cell::new((0.0, 0.0)),
            menu_size: Cell::new((0.0, 0.0)),
            hovered_index: Cell::new(None),
            item_hover_background: None,
            item_text_color: None,
            divider_color: None,
            background: None,
            border: None,
            menu_padding: None,
            layout_box: LayoutBox::default(),
        };
        menu.base.style.size = Some(
            crate::Size::new(Length::percent(100.0), Length::percent(100.0))
        );
        menu.recompute_style();
        menu
    }

    pub fn key(mut self, key: impl Into<SmolStr>) -> Self {
        self.base.key = Some(key.into());
        self
    }

    pub fn child(mut self, child: impl Widget + 'static) -> Self {
        self.children.push(Box::new(child));
        self
    }

    pub fn children_vec(mut self, children: Vec<Box<dyn Widget>>) -> Self {
        self.children = children;
        self
    }

    pub fn item(mut self, item: ContextMenuItem) -> Self {
        self.entries.push(ContextMenuEntry::Item(item));
        self
    }

    pub fn divider(mut self) -> Self {
        self.entries.push(ContextMenuEntry::Divider);
        self
    }

    pub fn menu_background<M>(mut self, background: impl IntoThemed<Background, M>) -> Self {
        self.background = Some(background.resolve_themed());
        self.base.dirty = true;
        self
    }

    pub fn border<M>(mut self, border: impl IntoThemed<Border, M>) -> Self {
        self.border = Some(border.resolve_themed());
        self.base.dirty = true;
        self
    }

    /// Overrides the menu's own inner padding; falls back to a sensible
    /// default when never called.
    pub fn padding(mut self, value: f32) -> Self {
        self.menu_padding = Some(value);
        self
    }

    pub fn item_hover_background<M>(mut self, background: impl IntoThemed<Background, M>) -> Self {
        self.item_hover_background = Some(background.resolve_themed());
        self
    }

    pub fn item_text_color(mut self, color: Color) -> Self {
        self.item_text_color = Some(color);
        self
    }

    pub fn divider_color(mut self, color: Color) -> Self {
        self.divider_color = Some(color);
        self
    }

    fn recompute_style(&mut self) {
        self.base.computed_style = self.base.inherited_style.inherit_style(&self.base.style);
    }

    fn effective_padding(&self) -> f32 {
        self.menu_padding.unwrap_or(MENU_PADDING)
    }

    fn total_menu_height(&self) -> f32 {
        let sum: f32 = self.entries
            .iter()
            .map(|e| {
                match e {
                    ContextMenuEntry::Item(_) => ITEM_HEIGHT,
                    ContextMenuEntry::Divider => DIVIDER_HEIGHT,
                }
            })
            .sum();
        sum + self.effective_padding() * 2.0
    }

    fn close(&self, ctx: &mut EventCtx) {
        if self.open.get() {
            self.open.set(false);
            self.hovered_index.set(None);
            ctx.request_redraw();
        }
    }

    fn open_at(&self, position: (f32, f32), ctx: &mut EventCtx) {
        let height = self.total_menu_height();

        let max_x = (self.layout_box.x + self.layout_box.width - MENU_WIDTH).max(self.layout_box.x);
        let max_y = (self.layout_box.y + self.layout_box.height - height).max(self.layout_box.y);
        let x = position.0.min(max_x).max(self.layout_box.x);
        let y = position.1.min(max_y).max(self.layout_box.y);

        self.menu_pos.set((x, y));
        self.menu_size.set((MENU_WIDTH, height));
        self.open.set(true);
        self.hovered_index.set(None);
        ctx.request_redraw();
    }

    // Rect (or, for a divider, the thin line's rect) for the entry at
    // `index` within `entries`.
    fn entry_rect(&self, index: usize) -> (f32, f32, f32, f32) {
        let (mx, my) = self.menu_pos.get();
        let (mw, _) = self.menu_size.get();
        let padding = self.effective_padding();

        let y_offset: f32 = self.entries[..index]
            .iter()
            .map(|e| {
                match e {
                    ContextMenuEntry::Item(_) => ITEM_HEIGHT,
                    ContextMenuEntry::Divider => DIVIDER_HEIGHT,
                }
            })
            .sum();

        match self.entries[index] {
            ContextMenuEntry::Item(_) =>
                (mx + padding, my + padding + y_offset, mw - padding * 2.0, ITEM_HEIGHT),
            ContextMenuEntry::Divider => {
                let line_y = my + padding + y_offset + DIVIDER_HEIGHT * 0.5;
                (mx + padding, line_y, mw - padding * 2.0, DIVIDER_LINE_THICKNESS)
            }
        }
    }

    // Entry index under `point`, or `None` if outside the menu or over a
    // divider (dividers aren't clickable).
    fn index_at(&self, point: (f32, f32)) -> Option<usize> {
        let (mx, my) = self.menu_pos.get();
        let (mw, mh) = self.menu_size.get();
        if point.0 < mx || point.0 > mx + mw || point.1 < my || point.1 > my + mh {
            return None;
        }
        (0..self.entries.len()).find(|&i| {
            if !matches!(self.entries[i], ContextMenuEntry::Item(_)) {
                return false;
            }
            let (x, y, w, h) = self.entry_rect(i);
            point.0 >= x && point.0 <= x + w && point.1 >= y && point.1 <= y + h
        })
    }

    // Pulls opacity toward 1 (open) or 0 (closed) through the shared
    // AnimationManager, called once per frame from cascade_style.
    fn animate_opacity(&mut self, anim: &mut AnimationManager) {
        let target = if self.open.get() { 1.0 } else { 0.0 };
        let key = AnimKey {
            widget: self.anim_id,
            layer: AnimLayer::Root,
            property: AnimProperty::Opacity,
        };

        anim.set_target(key, AnimValue([target, 0.0, 0.0, 0.0]), Some(OPACITY_TRANSITION));

        match anim.value(key) {
            Some(v) => {
                self.opacity_anim.set(v.0[0]);
                self.base.dirty = true;
            }
            None => self.opacity_anim.set(target),
        }
    }
}

impl Default for ContextMenu {
    fn default() -> Self {
        Self::new()
    }
}

impl StyleBuilder for ContextMenu {
    fn style_mut(&mut self) -> &mut Style {
        &mut self.base.style
    }

    fn mark_dirty(&mut self) {
        self.base.dirty = true;
        self.recompute_style();
    }
}

impl Widget for ContextMenu {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn debug_name(&self) -> &'static str {
        "Widget#ContextMenu"
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

    fn measure(&self, _ctx: &mut MeasureContext, _constraints: Constraints) -> MeasureResult {
        MeasureResult::new(0.0, 0.0)
    }

    fn layout(&mut self, rect: LayoutBox) {
        self.layout_box = rect;
    }

    fn layout_box(&self) -> &LayoutBox {
        &self.layout_box
    }

    fn paint(&self, _ctx: &mut PaintContext) {}

    fn paint_overlay(&self, ctx: &mut PaintContext) {
        let opacity = self.opacity_anim.get();
        if opacity <= 0.001 {
            return;
        }

        let (mx, my) = self.menu_pos.get();
        let (mw, mh) = self.menu_size.get();
        let theme = crate::current_theme();

        let bg = self.background.clone().unwrap_or(Background::Color(theme.surface));
        let border = self.border.as_ref();
        let border_color = border.map(|b| b.color).unwrap_or(theme.border);

        ctx.draw_rect(RectCommand {
            position: (mx, my),
            size: (mw, mh),
            background: Some(faded_background(bg, opacity)),
            border_radius: border.map(|b| b.radius),
            border_width: border.map(|b| b.width),
            border_color: Some(border_color.with_alpha_f32(border_color.a() * opacity)),
            clip_rect: None,
        });

        let hover_bg = self.item_hover_background.clone().unwrap_or(Background::Color(theme.hover));
        let divider_color = self.divider_color.unwrap_or(theme.border);

        for (i, entry) in self.entries.iter().enumerate() {
            let item = match entry {
                ContextMenuEntry::Divider => {
                    let (x, y, w, h) = self.entry_rect(i);
                    ctx.draw_rect(RectCommand {
                        position: (x, y),
                        size: (w, h),
                        background: Some(
                            Background::Color(
                                divider_color.with_alpha_f32(divider_color.a() * opacity)
                            )
                        ),
                        border_radius: None,
                        border_width: None,
                        border_color: None,
                        clip_rect: None,
                    });
                    continue;
                }
                ContextMenuEntry::Item(item) => item,
            };

            let (x, y, w, h) = self.entry_rect(i);

            if item.enabled && self.hovered_index.get() == Some(i) {
                ctx.draw_rect(RectCommand {
                    position: (x, y),
                    size: (w, h),
                    background: Some(faded_background(hover_bg.clone(), opacity)),
                    border_radius: Some(Length::px(4.0)),
                    border_width: None,
                    border_color: None,
                    clip_rect: None,
                });
            }

            let base_color = if item.enabled {
                self.item_text_color.unwrap_or(theme.foreground)
            } else {
                self.item_text_color.unwrap_or(theme.foreground_muted)
            };
            let alpha_scale = if item.enabled { 1.0 } else { 0.6 };

            let mut text_style = self.base.computed_style.clone();
            text_style.font_size.get_or_insert(DEFAULT_FONT_SIZE);
            text_style.color = Some(
                base_color.with_alpha_f32(base_color.a() * opacity * alpha_scale)
            );

            let text_h = DEFAULT_FONT_SIZE.value() * DEFAULT_LINE_HEIGHT_RATIO;

            ctx.draw_text(TextCommand {
                text: item.label.clone(),
                position: (x + ITEM_PADDING_X, y + (h - text_h) * 0.5),
                style: text_style,
                max_width: Some(w - ITEM_PADDING_X * 2.0),
                clip_rect: None,
            });
        }
    }

    fn hit_test(&self, point: (f32, f32)) -> bool {
        self.layout_box.contains_rounded(point, 0.0) ||
            (self.open.get() && self.index_at(point).is_some())
    }

    fn event(&mut self, event: &InputEvent, ctx: &mut EventCtx) -> EventStatus {
        match event {
            InputEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Right,
                position,
            } => {
                if self.layout_box.contains_rounded(*position, 0.0) {
                    self.open_at(*position, ctx);
                    self.base.dirty = true;
                    return EventStatus::Handled;
                }
                EventStatus::Ignored
            }

            InputEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                position,
            } if self.open.get() => {
                let Some(idx) = self.index_at(*position) else {
                    self.close(ctx);
                    return EventStatus::Handled;
                };
                if
                    let Some(ContextMenuEntry::Item(item)) = self.entries.get_mut(idx) &&
                    item.enabled &&
                    let Some(cb) = item.on_click.as_mut()
                {
                    cb(ctx);
                }
                self.close(ctx);
                EventStatus::Handled
            }

            InputEvent::MouseMoved { position } if self.open.get() => {
                let idx = self.index_at(*position);
                if idx != self.hovered_index.get() {
                    self.hovered_index.set(idx);
                    self.base.dirty = true;
                    ctx.request_redraw();
                }
                EventStatus::Handled
            }

            InputEvent::KeyInput { event: key_event, .. } if
                self.open.get() &&
                key_event.key == Key::Escape &&
                key_event.state == KeyState::Pressed
            => {
                self.close(ctx);
                EventStatus::Handled
            }

            _ => EventStatus::Ignored,
        }
    }

    fn content_eq(&self, other: &dyn Widget) -> bool {
        let Some(other) = other.as_any().downcast_ref::<ContextMenu>() else {
            return false;
        };
        self.base.style == other.base.style && self.entries.len() == other.entries.len()
    }

    fn cascade_style(&mut self, parent: &Style, anim: &mut AnimationManager) {
        self.base.inherited_style = parent.clone();
        self.recompute_style();

        self.animate_opacity(anim);

        for child in self.children.iter_mut() {
            child.cascade_style(&self.base.computed_style, anim);
        }
    }

    fn transfer_measured_state(&mut self, old: &dyn Widget) {
        if let Some(old) = old.as_any().downcast_ref::<ContextMenu>() {
            self.open.set(old.open.get());
            self.opacity_anim.set(old.opacity_anim.get());
            self.menu_pos.set(old.menu_pos.get());
            self.menu_size.set(old.menu_size.get());
            self.hovered_index.set(old.hovered_index.get());
            self.anim_id = old.anim_id;
        }
    }

    fn anim_id(&self) -> WidgetId {
        self.anim_id
    }
}
