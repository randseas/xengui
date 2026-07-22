// crates/xengui/src/widgets/context_menu.rs (full file)
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
    Edges,
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
    TriangleCommand,
    Widget,
    WidgetBase,
    WidgetId,
    properties::{ DEFAULT_FONT_SIZE, DEFAULT_LINE_HEIGHT_RATIO },
};
use smol_str::SmolStr;
use std::cell::{ Cell, RefCell };
use std::time::Duration;

pub struct ContextMenuItem {
    label: SmolStr,
    shortcut: Option<SmolStr>,
    #[allow(clippy::type_complexity)]
    on_click: Option<Box<dyn FnMut(&mut EventCtx)>>,
    enabled: bool,
    submenu: Vec<ContextMenuEntry>,
}

impl ContextMenuItem {
    pub fn new(label: impl Into<SmolStr>) -> Self {
        Self {
            label: label.into(),
            shortcut: None,
            on_click: None,
            enabled: true,
            submenu: Vec::new(),
        }
    }

    pub fn on_click(mut self, f: impl FnMut(&mut EventCtx) + 'static) -> Self {
        self.on_click = Some(Box::new(f));
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Purely visual shortcut label drawn at the item's right edge (e.g. "Ctrl+C").
    /// Does not register or handle the actual key combination.
    pub fn shortcut(mut self, text: impl Into<SmolStr>) -> Self {
        self.shortcut = Some(text.into());
        self
    }

    pub fn submenu_item(mut self, item: ContextMenuItem) -> Self {
        self.submenu.push(ContextMenuEntry::Item(item));
        self
    }

    pub fn submenu_divider(mut self) -> Self {
        self.submenu.push(ContextMenuEntry::Divider);
        self
    }

    fn has_submenu(&self) -> bool {
        !self.submenu.is_empty()
    }
}

enum ContextMenuEntry {
    Item(ContextMenuItem),
    Divider,
}

#[derive(Clone)]
struct OpenSubmenu {
    path: Vec<usize>,
    pos: Cell<(f32, f32)>,
    size: Cell<(f32, f32)>,
    hovered_index: Cell<Option<usize>>,
    pressed_index: Cell<Option<usize>>,
}

const ITEM_HEIGHT: f32 = 32.0;
const ITEM_PADDING_X: f32 = 28.0;
const MENU_PADDING: f32 = 4.0;
const MENU_WIDTH: f32 = 160.0;
const DIVIDER_HEIGHT: f32 = 9.0;
const DIVIDER_LINE_THICKNESS: f32 = 1.0;
const ARROW_SIZE: f32 = 4.0;

const OPACITY_TRANSITION: Transition = Transition::new(Duration::from_millis(150)).easing(
    Easing::EaseOut
);

fn faded_background(bg: Background, opacity: f32) -> Background {
    match bg {
        Background::Color(c) => Background::Color(c.with_alpha_f32(c.a() * opacity)),
    }
}

fn point_in_rect(point: (f32, f32), rect: (f32, f32, f32, f32)) -> bool {
    let (px, py) = point;
    let (rx, ry, rw, rh) = rect;
    px >= rx && px <= rx + rw && py >= ry && py <= ry + rh
}

fn menu_height(entries: &[ContextMenuEntry], padding: f32) -> f32 {
    let sum: f32 = entries
        .iter()
        .map(|e| {
            match e {
                ContextMenuEntry::Item(_) => ITEM_HEIGHT,
                ContextMenuEntry::Divider => DIVIDER_HEIGHT,
            }
        })
        .sum();
    sum + padding * 2.0
}

fn entry_rect_at(
    entries: &[ContextMenuEntry],
    pos: (f32, f32),
    size: (f32, f32),
    padding: f32,
    index: usize
) -> (f32, f32, f32, f32) {
    let (mx, my) = pos;
    let (mw, _) = size;

    let y_offset: f32 = entries[..index]
        .iter()
        .map(|e| {
            match e {
                ContextMenuEntry::Item(_) => ITEM_HEIGHT,
                ContextMenuEntry::Divider => DIVIDER_HEIGHT,
            }
        })
        .sum();

    match entries[index] {
        ContextMenuEntry::Item(_) =>
            (mx + padding, my + padding + y_offset, mw - padding * 2.0, ITEM_HEIGHT),
        ContextMenuEntry::Divider => {
            let line_y = my + padding + y_offset + DIVIDER_HEIGHT * 0.5;
            (mx + padding, line_y, mw - padding * 2.0, DIVIDER_LINE_THICKNESS)
        }
    }
}

fn index_at(
    entries: &[ContextMenuEntry],
    pos: (f32, f32),
    size: (f32, f32),
    padding: f32,
    point: (f32, f32)
) -> Option<usize> {
    if !point_in_rect(point, (pos.0, pos.1, size.0, size.1)) {
        return None;
    }
    (0..entries.len()).find(|&i| {
        if !matches!(entries[i], ContextMenuEntry::Item(_)) {
            return false;
        }
        let (x, y, w, h) = entry_rect_at(entries, pos, size, padding, i);
        point.0 >= x && point.0 <= x + w && point.1 >= y && point.1 <= y + h
    })
}

fn submenu_arrow(rect: (f32, f32, f32, f32)) -> ((f32, f32), (f32, f32), (f32, f32)) {
    let (x, y, w, h) = rect;
    let cy = y + h * 0.5;
    let right = x + w - 6.0;
    ((right - ARROW_SIZE, cy - ARROW_SIZE), (right - ARROW_SIZE, cy + ARROW_SIZE), (right, cy))
}

/// A page-wide right-click popup menu. Wraps arbitrary content (like
/// `View`) and should sit near the root of the tree so `paint_top`
/// runs after every other widget's own paint pass, guaranteeing the
/// popup always renders on top regardless of sibling paint order.
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
    pending_reopen: Cell<Option<(f32, f32)>>,
    pressed_index: Cell<Option<usize>>,
    submenu_stack: RefCell<Vec<OpenSubmenu>>,

    item_background: Option<Background>,
    item_hover_background: Option<Background>,
    item_pressed_background: Option<Background>,
    item_border: Option<Border>,
    item_hover_border: Option<Border>,
    item_pressed_border: Option<Border>,
    item_padding: Option<Edges>,
    item_text_color: Option<Color>,
    item_hover_text_color: Option<Color>,
    item_pressed_text_color: Option<Color>,
    divider_color: Option<Color>,

    background: Option<Background>,
    border: Option<Border>,
    menu_padding: Option<f32>,
    menu_min_width: Option<f32>,
    menu_max_width: Option<f32>,
    menu_min_height: Option<f32>,
    menu_max_height: Option<f32>,

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
            pending_reopen: Cell::new(None),
            pressed_index: Cell::new(None),
            submenu_stack: RefCell::new(Vec::new()),
            item_background: None,
            item_hover_background: None,
            item_pressed_background: None,
            item_border: None,
            item_hover_border: None,
            item_pressed_border: None,
            item_padding: None,
            item_text_color: None,
            item_hover_text_color: None,
            item_pressed_text_color: None,
            divider_color: None,
            background: None,
            border: None,
            menu_padding: None,
            menu_min_width: None,
            menu_max_width: None,
            menu_min_height: None,
            menu_max_height: None,
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

    pub fn font(mut self, font: impl Into<SmolStr>) -> Self {
        self.base.style.font = Some(font.into());
        self.recompute_style();
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
        self
    }

    pub fn border<M>(mut self, border: impl IntoThemed<Border, M>) -> Self {
        self.border = Some(border.resolve_themed());
        self
    }

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

    pub fn item_background<M>(mut self, background: impl IntoThemed<Background, M>) -> Self {
        self.item_background = Some(background.resolve_themed());
        self
    }

    pub fn item_pressed_background<M>(
        mut self,
        background: impl IntoThemed<Background, M>
    ) -> Self {
        self.item_pressed_background = Some(background.resolve_themed());
        self
    }

    pub fn item_border<M>(mut self, border: impl IntoThemed<Border, M>) -> Self {
        self.item_border = Some(border.resolve_themed());
        self
    }

    pub fn item_hover_border<M>(mut self, border: impl IntoThemed<Border, M>) -> Self {
        self.item_hover_border = Some(border.resolve_themed());
        self
    }

    pub fn item_pressed_border<M>(mut self, border: impl IntoThemed<Border, M>) -> Self {
        self.item_pressed_border = Some(border.resolve_themed());
        self
    }

    pub fn item_padding<M>(mut self, padding: impl IntoThemed<Edges, M>) -> Self {
        self.item_padding = Some(padding.resolve_themed());
        self
    }

    pub fn item_hover_text_color(mut self, color: Color) -> Self {
        self.item_hover_text_color = Some(color);
        self
    }

    pub fn item_pressed_text_color(mut self, color: Color) -> Self {
        self.item_pressed_text_color = Some(color);
        self
    }

    pub fn menu_min_width(mut self, width: f32) -> Self {
        self.menu_min_width = Some(width);
        self
    }

    pub fn menu_max_width(mut self, width: f32) -> Self {
        self.menu_max_width = Some(width);
        self
    }

    pub fn menu_min_height(mut self, height: f32) -> Self {
        self.menu_min_height = Some(height);
        self
    }

    pub fn menu_max_height(mut self, height: f32) -> Self {
        self.menu_max_height = Some(height);
        self
    }

    fn recompute_style(&mut self) {
        self.base.computed_style = self.base.inherited_style.inherit_style(&self.base.style);
    }

    fn effective_padding(&self) -> f32 {
        self.menu_padding.unwrap_or(MENU_PADDING)
    }

    fn effective_item_padding(&self) -> Edges {
        self.item_padding.unwrap_or_else(|| Edges::symmetric(ITEM_PADDING_X, 0.0))
    }

    fn point_in_menu(&self, point: (f32, f32)) -> bool {
        if !self.open.get() {
            return false;
        }
        point_in_rect(point, (
            self.menu_pos.get().0,
            self.menu_pos.get().1,
            self.menu_size.get().0,
            self.menu_size.get().1,
        ))
    }

    fn point_in_any_menu(&self, point: (f32, f32)) -> bool {
        if self.point_in_menu(point) {
            return true;
        }
        self.submenu_stack
            .borrow()
            .iter()
            .any(|l| {
                let (x, y) = l.pos.get();
                let (w, h) = l.size.get();
                point_in_rect(point, (x, y, w, h))
            })
    }

    fn entries_at<'a>(&'a self, path: &[usize]) -> &'a [ContextMenuEntry] {
        let mut current: &[ContextMenuEntry] = &self.entries;
        for &idx in path {
            match current.get(idx) {
                Some(ContextMenuEntry::Item(item)) => {
                    current = &item.submenu;
                }
                _ => {
                    return &[];
                }
            }
        }
        current
    }

    fn item_at_mut(&mut self, path: &[usize], idx: usize) -> Option<&mut ContextMenuItem> {
        // Returns the item directly instead of the parent Vec, since every
        // failure path here can just bail with None - no branch ever needs
        // to hand back the un-navigated container, which is what breaks
        // the borrow checker in the Vec-returning version.
        fn walk<'a>(
            entries: &'a mut Vec<ContextMenuEntry>,
            path: &[usize],
            idx: usize
        ) -> Option<&'a mut ContextMenuItem> {
            match path.split_first() {
                Some((&next, rest)) =>
                    match entries.get_mut(next)? {
                        ContextMenuEntry::Item(item) => walk(&mut item.submenu, rest, idx),
                        ContextMenuEntry::Divider => None,
                    }
                None =>
                    match entries.get_mut(idx)? {
                        ContextMenuEntry::Item(item) => Some(item),
                        ContextMenuEntry::Divider => None,
                    }
            }
        }
        walk(&mut self.entries, path, idx)
    }

    fn close(&self, ctx: &mut EventCtx) {
        if self.open.get() {
            self.open.set(false);
            self.hovered_index.set(None);
            self.pressed_index.set(None);
            self.submenu_stack.borrow_mut().clear();
            ctx.request_redraw();
        }
    }

    // Truncates the open-submenu chain, discarding every level at or
    // beyond `depth` (1-based: depth 0 keeps nothing open).
    fn close_from(&self, depth: usize) {
        self.submenu_stack.borrow_mut().truncate(depth);
    }

    fn open_at_impl(&self, position: (f32, f32)) {
        let height = menu_height(&self.entries, self.effective_padding());
        let width = MENU_WIDTH;

        let bounds_x = self.layout_box.x;
        let bounds_y = self.layout_box.y;
        let bounds_right = self.layout_box.x + self.layout_box.width;
        let bounds_bottom = self.layout_box.y + self.layout_box.height;

        // Anchors from whichever corner keeps the menu fully on screen
        // instead of only clamping, matching web context-menu behavior.
        let x = if position.0 + width > bounds_right {
            (position.0 - width).max(bounds_x)
        } else {
            position.0
        };
        let y = if position.1 + height > bounds_bottom {
            (position.1 - height).max(bounds_y)
        } else {
            position.1
        };

        self.menu_pos.set((
            x.min(bounds_right - width).max(bounds_x),
            y.min(bounds_bottom - height).max(bounds_y),
        ));
        self.menu_size.set((width, height));
        self.open.set(true);
        self.hovered_index.set(None);
        self.submenu_stack.borrow_mut().clear();
    }

    fn open_at(&self, position: (f32, f32), ctx: &mut EventCtx) {
        self.open_at_impl(position);
        ctx.request_redraw();
    }

    // Positions a submenu relative to the parent item's rect, opening to
    // the right by default and flipping to whichever side/edge keeps it
    // fully inside `self.layout_box`.
    fn position_submenu(
        &self,
        parent_rect: (f32, f32, f32, f32),
        width: f32,
        height: f32
    ) -> (f32, f32) {
        let (px, py, pw, _ph) = parent_rect;
        let bounds_x = self.layout_box.x;
        let bounds_y = self.layout_box.y;
        let bounds_right = self.layout_box.x + self.layout_box.width;
        let bounds_bottom = self.layout_box.y + self.layout_box.height;

        let x = if px + pw + width > bounds_right { (px - width).max(bounds_x) } else { px + pw };
        let y = if py + height > bounds_bottom {
            (bounds_bottom - height).max(bounds_y)
        } else {
            py
        };

        (x, y)
    }

    fn hit_level_index(&self, point: (f32, f32)) -> Option<(usize, usize)> {
        let depth = self.submenu_stack.borrow().len();
        let padding = self.effective_padding();

        for level in (0..=depth).rev() {
            let (path, pos, size) = if level == 0 {
                (Vec::new(), self.menu_pos.get(), self.menu_size.get())
            } else {
                let stack = self.submenu_stack.borrow();
                let l = &stack[level - 1];
                (l.path.clone(), l.pos.get(), l.size.get())
            };

            if !point_in_rect(point, (pos.0, pos.1, size.0, size.1)) {
                continue;
            }

            let entries = self.entries_at(&path);
            return index_at(entries, pos, size, padding, point).map(|idx| (level, idx));
        }
        None
    }

    fn update_hover(&self, point: (f32, f32), ctx: &mut EventCtx) {
        let depth = self.submenu_stack.borrow().len();
        let padding = self.effective_padding();

        for level in (0..=depth).rev() {
            let (path, pos, size) = if level == 0 {
                (Vec::new(), self.menu_pos.get(), self.menu_size.get())
            } else {
                let stack = self.submenu_stack.borrow();
                let l = &stack[level - 1];
                (l.path.clone(), l.pos.get(), l.size.get())
            };

            if !point_in_rect(point, (pos.0, pos.1, size.0, size.1)) {
                continue;
            }

            let entries = self.entries_at(&path);
            let idx = index_at(entries, pos, size, padding, point);

            let current = if level == 0 {
                self.hovered_index.get()
            } else {
                self.submenu_stack.borrow()[level - 1].hovered_index.get()
            };

            if idx != current {
                if level == 0 {
                    self.hovered_index.set(idx);
                } else {
                    self.submenu_stack.borrow()[level - 1].hovered_index.set(idx);
                }

                self.close_from(level);

                if
                    let Some(i) = idx &&
                    let Some(ContextMenuEntry::Item(item)) = entries.get(i) &&
                    item.enabled &&
                    item.has_submenu()
                {
                    let rect = entry_rect_at(entries, pos, size, padding, i);
                    let mut child_path = path.clone();
                    child_path.push(i);
                    let child_entries = self.entries_at(&child_path);
                    let child_h = menu_height(child_entries, padding);
                    let child_pos = self.position_submenu(rect, MENU_WIDTH, child_h);

                    self.submenu_stack.borrow_mut().push(OpenSubmenu {
                        path: child_path,
                        pos: Cell::new(child_pos),
                        size: Cell::new((MENU_WIDTH, child_h)),
                        hovered_index: Cell::new(None),
                        pressed_index: Cell::new(None),
                    });
                }

                ctx.request_redraw();
            }
            return;
        }
    }

    fn animate_opacity(&mut self, anim: &mut AnimationManager) {
        let target = if self.open.get() { 1.0 } else { 0.0 };
        let key = AnimKey {
            widget: self.anim_id,
            layer: AnimLayer::Root,
            property: AnimProperty::Opacity,
        };

        anim.set_target(key, AnimValue([target, 0.0, 0.0, 0.0]), Some(OPACITY_TRANSITION));

        match anim.value(key) {
            Some(v) => self.opacity_anim.set(v.0[0]),
            None => self.opacity_anim.set(target),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn paint_level(
        &self,
        ctx: &mut PaintContext,
        theme: &crate::Theme,
        opacity: f32,
        entries: &[ContextMenuEntry],
        pos: (f32, f32),
        size: (f32, f32),
        hovered_index: Option<usize>,
        pressed_index: Option<usize>,
        padding: f32
    ) {
        let (mx, my) = pos;
        let (mw, mh) = size;

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

        let divider_color = self.divider_color.unwrap_or(theme.border);
        let sf = ctx.scale_factor;
        let pad = self.effective_item_padding();
        let (pad_l, pad_r, pad_t, pad_b) = (
            pad.left.value(),
            pad.right.value(),
            pad.top.value(),
            pad.bottom.value(),
        );

        for (i, entry) in entries.iter().enumerate() {
            let item = match entry {
                ContextMenuEntry::Divider => {
                    let (x, y, w, h) = entry_rect_at(entries, pos, size, padding, i);
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

            let (x, y, w, h) = entry_rect_at(entries, pos, size, padding, i);
            let is_hovered = item.enabled && hovered_index == Some(i);
            let is_pressed = is_hovered && pressed_index == Some(i);

            let (bg, border, text_color) = if is_pressed {
                (
                    Some(
                        self.item_pressed_background
                            .clone()
                            .or_else(|| self.item_hover_background.clone())
                            .unwrap_or(Background::Color(theme.pressed))
                    ),
                    self.item_pressed_border.or(self.item_hover_border).or(self.item_border),
                    self.item_pressed_text_color
                        .or(self.item_hover_text_color)
                        .or(self.item_text_color),
                )
            } else if is_hovered {
                (
                    Some(
                        self.item_hover_background.clone().unwrap_or(Background::Color(theme.hover))
                    ),
                    self.item_hover_border.or(self.item_border),
                    self.item_hover_text_color.or(self.item_text_color),
                )
            } else {
                (self.item_background.clone(), self.item_border, self.item_text_color)
            };

            if let Some(bg) = bg {
                ctx.draw_rect(RectCommand {
                    position: (x, y),
                    size: (w, h),
                    background: Some(faded_background(bg, opacity)),
                    border_radius: border.map(|b| b.radius).or(Some(Length::px(4.0))),
                    border_width: border.map(|b| b.width),
                    border_color: border.map(|b| b.color.with_alpha_f32(b.color.a() * opacity)),
                    clip_rect: None,
                });
            }

            let base_color = if item.enabled {
                text_color.unwrap_or(theme.foreground)
            } else {
                text_color.unwrap_or(theme.foreground_muted)
            };
            let alpha_scale = if item.enabled { 1.0 } else { 0.6 };

            let mut text_style = self.base.computed_style.clone();
            text_style.font_size.get_or_insert(DEFAULT_FONT_SIZE);
            text_style.color = Some(
                base_color.with_alpha_f32(base_color.a() * opacity * alpha_scale)
            );

            let font_size = text_style.font_size
                .map(|f| f.to_physical(sf))
                .unwrap_or(DEFAULT_FONT_SIZE.to_physical(sf));
            let text_h = text_style.line_height
                .map(|lh| lh.value().to_physical(sf))
                .filter(|lh| *lh > 0.0)
                .unwrap_or(font_size * DEFAULT_LINE_HEIGHT_RATIO);

            let inner_h = (h - pad_t - pad_b).max(0.0);
            let text_y = y + pad_t + (inner_h - text_h).max(0.0) * 0.5;

            // Right side of the row shows either a submenu arrow or a
            // shortcut label, never both.
            let right_reserved = if item.has_submenu() {
                let (p0, p1, p2) = submenu_arrow((x, y, w, h));
                ctx.draw_triangle(TriangleCommand {
                    p0,
                    p1,
                    p2,
                    color: base_color.with_alpha_f32(base_color.a() * opacity * alpha_scale),
                    clip_rect: None,
                });
                16.0
            } else if let Some(shortcut) = &item.shortcut {
                let mut shortcut_style = text_style.clone();
                shortcut_style.color = Some(
                    base_color.with_alpha_f32(base_color.a() * opacity * alpha_scale * 0.6)
                );
                let shortcut_w = (w - pad_l - pad_r) * 0.5;
                ctx.draw_text(TextCommand {
                    text: shortcut.clone(),
                    position: (x + w - pad_r - shortcut_w, text_y),
                    style: shortcut_style,
                    max_width: Some(shortcut_w),
                    clip_rect: None,
                });
                shortcut_w + 6.0
            } else {
                0.0
            };

            ctx.draw_text(TextCommand {
                text: item.label.clone(),
                position: (x + pad_l, text_y),
                style: text_style,
                max_width: Some((w - pad_l - pad_r - right_reserved).max(0.0)),
                clip_rect: None,
            });
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

    fn paint_top(&self, ctx: &mut PaintContext) {
        let opacity = self.opacity_anim.get();
        if opacity <= 0.001 {
            return;
        }

        let theme = crate::current_theme();
        let padding = self.effective_padding();

        self.paint_level(
            ctx,
            &theme,
            opacity,
            &self.entries,
            self.menu_pos.get(),
            self.menu_size.get(),
            self.hovered_index.get(),
            self.pressed_index.get(),
            padding
        );

        #[allow(clippy::type_complexity)]
        let levels: Vec<
            (Vec<usize>, (f32, f32), (f32, f32), Option<usize>, Option<usize>)
        > = self.submenu_stack
            .borrow()
            .iter()
            .map(|l| (
                l.path.clone(),
                l.pos.get(),
                l.size.get(),
                l.hovered_index.get(),
                l.pressed_index.get(),
            ))
            .collect();

        for (path, pos, size, hovered, pressed) in levels {
            let entries = self.entries_at(&path);
            self.paint_level(ctx, &theme, opacity, entries, pos, size, hovered, pressed, padding);
        }
    }

    fn hit_test(&self, point: (f32, f32)) -> bool {
        self.layout_box.contains_rounded(point, 0.0) || self.point_in_any_menu(point)
    }

    fn blocks_children_hit_test(&self, point: (f32, f32)) -> bool {
        self.point_in_any_menu(point)
    }

    fn event(&mut self, event: &InputEvent, ctx: &mut EventCtx) -> EventStatus {
        match event {
            InputEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Right,
                position,
            } => {
                if self.layout_box.contains_rounded(*position, 0.0) {
                    if self.open.get() {
                        self.pending_reopen.set(Some(*position));
                        self.close(ctx);
                    } else {
                        self.open_at(*position, ctx);
                    }
                    return EventStatus::Handled;
                }
                EventStatus::Ignored
            }

            InputEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                position,
            } if self.open.get() => {
                if !self.point_in_any_menu(*position) {
                    self.close(ctx);
                    return EventStatus::Handled;
                }

                if let Some((depth, idx)) = self.hit_level_index(*position) {
                    if depth == 0 {
                        self.pressed_index.set(Some(idx));
                    } else {
                        self.submenu_stack.borrow()[depth - 1].pressed_index.set(Some(idx));
                    }
                    ctx.request_redraw();
                }

                EventStatus::Handled
            }

            InputEvent::MouseInput {
                state: ElementState::Released,
                button: MouseButton::Left,
                position,
            } if self.open.get() => {
                if !self.point_in_any_menu(*position) {
                    self.close(ctx);
                    return EventStatus::Handled;
                }

                if let Some((depth, idx)) = self.hit_level_index(*position) {
                    let was_pressed = if depth == 0 {
                        self.pressed_index.take() == Some(idx)
                    } else {
                        self.submenu_stack.borrow()[depth - 1].pressed_index.take() == Some(idx)
                    };

                    if was_pressed {
                        let path = if depth == 0 {
                            Vec::new()
                        } else {
                            self.submenu_stack.borrow()[depth - 1].path.clone()
                        };

                        if
                            let Some(item) = self.item_at_mut(&path, idx) &&
                            item.enabled &&
                            !item.has_submenu()
                        {
                            if let Some(cb) = item.on_click.as_mut() {
                                cb(ctx);
                            }
                            self.close(ctx);
                            return EventStatus::Handled;
                        }
                    }
                } else {
                    self.pressed_index.set(None);
                    for l in self.submenu_stack.borrow().iter() {
                        l.pressed_index.set(None);
                    }
                }

                ctx.request_redraw();
                EventStatus::Handled
            }

            InputEvent::MouseMoved { position } if self.open.get() => {
                self.update_hover(*position, ctx);
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

        if
            !self.open.get() &&
            self.opacity_anim.get() <= 0.001 &&
            let Some(pos) = self.pending_reopen.take()
        {
            self.open_at_impl(pos);
        }

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
            self.pending_reopen.set(old.pending_reopen.get());
            self.pressed_index.set(old.pressed_index.get());
            self.submenu_stack.replace(old.submenu_stack.borrow().clone());
            self.anim_id = old.anim_id;
        }
    }

    fn anim_id(&self) -> WidgetId {
        self.anim_id
    }
}
