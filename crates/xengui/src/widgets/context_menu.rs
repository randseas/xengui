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
    FontStyle,
    FontWeight,
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
    Point,
    Rect,
    RectCommand,
    Style,
    StyleBuilder,
    TextCommand,
    TextMeasurer,
    Transition,
    Triangle,
    TriangleCommand,
    Widget,
    WidgetBase,
    WidgetId,
    properties::{ DEFAULT_FONT_SIZE, DEFAULT_LINE_HEIGHT_RATIO },
};
use smol_str::SmolStr;
use std::cell::{ Cell, RefCell };
use std::rc::Rc;
use std::time::Duration;

type ClickCallback = Box<dyn FnMut(&mut EventCtx)>;

pub struct ContextMenuItem {
    label: SmolStr,
    shortcut: Option<SmolStr>,
    on_click: Option<ClickCallback>,
    enabled: bool,
    submenu: Vec<ContextMenuEntry>,
    anim_id: WidgetId,
    hover_scale: Option<f32>,
    hover_progress: Cell<f32>,
    scale_progress: Cell<f32>,
    // Natural pixel widths measured once per layout pass; used to
    // auto-size the menu and to right-align the shortcut exactly.
    label_width: Cell<f32>,
    shortcut_width: Cell<f32>,
    // Natural width of this item's own submenu, if any.
    submenu_width: Cell<f32>,
}

impl ContextMenuItem {
    pub fn new(label: impl Into<SmolStr>) -> Self {
        Self {
            label: label.into(),
            shortcut: None,
            on_click: None,
            enabled: true,
            submenu: Vec::new(),
            anim_id: WidgetId::new_unique(),
            hover_scale: None,
            hover_progress: Cell::new(0.0),
            scale_progress: Cell::new(1.0),
            label_width: Cell::new(0.0),
            shortcut_width: Cell::new(0.0),
            submenu_width: Cell::new(0.0),
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

    /// Scales the item up (or down) while hovered, animated with the
    /// menu's shared item hover transition (see `ContextMenu::item_transition`).
    pub fn hover_scale(mut self, scale: f32) -> Self {
        self.hover_scale = Some(scale);
        self
    }

    fn has_submenu(&self) -> bool {
        !self.submenu.is_empty()
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
    anim_id: WidgetId,
    opacity: Cell<f32>,
}

struct ClosingLevelSnapshot {
    path: Vec<usize>,
    pos: Point,
    size: (f32, f32),
    hovered: Option<usize>,
    pressed: Option<usize>,
    opacity: f32,
}

/// A lightweight, cloneable handle that lets a widget elsewhere in the
/// tree (e.g. a `View` bound via [`crate::View::context_menu`]) open a
/// [`ContextMenu`] without wrapping that widget as the menu's child.
///
/// Create it once with `ContextMenuHandle::new()` and keep it stable
/// across rebuilds (e.g. with `use_state`), then pass clones to both
/// `ContextMenu::bind` and every trigger widget.
#[derive(Clone)]
pub struct ContextMenuHandle(Rc<Cell<Option<(f32, f32)>>>);

impl ContextMenuHandle {
    pub fn new() -> Self {
        Self(Rc::new(Cell::new(None)))
    }

    /// Requests the bound `ContextMenu` to open at `position` on its next
    /// style cascade.
    pub fn open_at(&self, position: (f32, f32)) {
        self.0.set(Some(position));
    }

    fn take_request(&self) -> Option<(f32, f32)> {
        self.0.take()
    }
}

impl Default for ContextMenuHandle {
    fn default() -> Self {
        Self::new()
    }
}

const ITEM_HEIGHT: f32 = 32.0;
const ITEM_PADDING_X: f32 = 28.0;
const MENU_PADDING: f32 = 4.0;
const DEFAULT_MENU_MIN_WIDTH: f32 = 120.0;
const SUBMENU_ARROW_RESERVED: f32 = 16.0;
const SHORTCUT_GAP: f32 = 6.0;
const DIVIDER_HEIGHT: f32 = 12.0;
const DIVIDER_LINE_THICKNESS: f32 = 1.0;
const ARROW_SIZE: f32 = 4.0;
const SHORTCUT_RIGHT_PADDING: f32 = 8.0;
const ARROW_THICKNESS: f32 = 1.6;
const ARROW_CAP_SEGMENTS: usize = 8;

const OPACITY_TRANSITION: Transition = Transition::new(Duration::from_millis(120)).easing(
    Easing::EaseOut
);

const SUBMENU_OPACITY_TRANSITION: Transition = Transition::new(Duration::from_millis(200)).easing(
    Easing::EaseInOut
);

const ITEM_HOVER_TRANSITION: Transition = Transition::new(Duration::from_millis(150)).easing(
    Easing::EaseInOut
);

fn lerp_color(a: Color, b: Color, t: f32) -> Color {
    // Delegates to the same premultiplied-alpha blend xen-animation uses
    // for animated colors, so this paint-time blend doesn't flash dark
    // mid-fade either.
    let blended = AnimValue(a.to_f32_array()).lerp_premultiplied(AnimValue(b.to_f32_array()), t);
    Color::rgba_f32(blended.0[0], blended.0[1], blended.0[2], blended.0[3])
}

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

// Measures each item's label/shortcut natural width in `entries`, caches
// them on the items themselves, and returns the widest row - used to
// auto-size a menu level unless the user pins an explicit width.
#[allow(clippy::too_many_arguments)]
fn measure_entries_width(
    entries: &[ContextMenuEntry],
    text: &mut dyn TextMeasurer,
    font: Option<&str>,
    font_size: f32,
    weight: FontWeight,
    font_style: FontStyle,
    letter_spacing: f32,
    line_height: f32,
    pad_lr: f32
) -> f32 {
    let mut max_w: f32 = 0.0;

    for entry in entries {
        let ContextMenuEntry::Item(item) = entry else {
            continue;
        };

        let label_w = text.measure(
            &item.label,
            font,
            font_size,
            weight,
            font_style,
            letter_spacing,
            line_height,
            None
        ).width;
        item.label_width.set(label_w);

        let shortcut_w = item.shortcut
            .as_ref()
            .map_or(0.0, |s| {
                text.measure(
                    s,
                    font,
                    font_size,
                    weight,
                    font_style,
                    letter_spacing,
                    line_height,
                    None
                ).width
            });
        item.shortcut_width.set(shortcut_w);

        let arrow_w = if item.has_submenu() { SUBMENU_ARROW_RESERVED } else { 0.0 };
        let shortcut_gap = if shortcut_w > 0.0 { SHORTCUT_GAP } else { 0.0 };
        let shortcut_padding = if shortcut_w > 0.0 { SHORTCUT_RIGHT_PADDING } else { 0.0 };

        let row_w = label_w + shortcut_gap + shortcut_w + shortcut_padding + arrow_w + pad_lr;
        max_w = max_w.max(row_w);

        if item.has_submenu() {
            let sub_w = measure_entries_width(
                &item.submenu,
                text,
                font,
                font_size,
                weight,
                font_style,
                letter_spacing,
                line_height,
                pad_lr
            );
            item.submenu_width.set(sub_w);
        }
    }

    max_w
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

// Builds a rounded-corner chevron ("›") as a set of filled triangles:
// two thick line segments plus round caps at the joints, so the arrow
// reads as a smooth stroke instead of a sharp mitered triangle.

fn submenu_arrow_triangles(rect: Rect) -> Vec<Triangle> {
    let (x, y, w, h) = rect;
    let cy = y + h * 0.5;
    let right = x + w - SHORTCUT_RIGHT_PADDING;

    let top = (right - ARROW_SIZE, cy - ARROW_SIZE);
    let tip = (right, cy);
    let bottom = (right - ARROW_SIZE, cy + ARROW_SIZE);

    let mut tris = arrow_segment(top, tip);
    tris.extend(arrow_segment(tip, bottom));
    tris.extend(arrow_cap(top));
    tris.extend(arrow_cap(tip));
    tris.extend(arrow_cap(bottom));
    tris
}

fn arrow_segment(a: Point, b: Point) -> Vec<Triangle> {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let len = (dx * dx + dy * dy).sqrt().max(0.0001);
    let (nx, ny) = ((-dy / len) * ARROW_THICKNESS * 0.5, (dx / len) * ARROW_THICKNESS * 0.5);
    let p0 = (a.0 + nx, a.1 + ny);
    let p1 = (a.0 - nx, a.1 - ny);
    let p2 = (b.0 + nx, b.1 + ny);
    let p3 = (b.0 - nx, b.1 - ny);
    vec![(p0, p1, p2), (p1, p3, p2)]
}

// Small filled fan approximating a circle, rounding off a joint or line
// end that would otherwise show as a sharp corner.

fn arrow_cap(center: Point) -> Vec<Triangle> {
    let r = ARROW_THICKNESS * 0.5;
    (0..ARROW_CAP_SEGMENTS)
        .map(|i| {
            let a0 = ((i as f32) / (ARROW_CAP_SEGMENTS as f32)) * std::f32::consts::TAU;
            let a1 = (((i + 1) as f32) / (ARROW_CAP_SEGMENTS as f32)) * std::f32::consts::TAU;
            (
                center,
                (center.0 + a0.cos() * r, center.1 + a0.sin() * r),
                (center.0 + a1.cos() * r, center.1 + a1.sin() * r),
            )
        })
        .collect()
}

/// A page-wide right-click popup menu. Wraps arbitrary content (like
/// `View`) and should sit near the root of the tree so `paint_top`
/// runs after every other widget's own paint pass, guaranteeing the
/// popup always renders on top regardless of sibling paint order.
///
/// It can also be triggered from anywhere else in the tree without
/// wrapping: bind a [`ContextMenuHandle`] with [`ContextMenu::bind`] and
/// pass clones of it to trigger widgets (e.g. [`crate::View::context_menu`]).
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
    // Levels that just closed; kept here so they can fade out instead of
    // disappearing the instant hover moves away.
    closing_submenus: RefCell<Vec<OpenSubmenu>>,

    natural_width: Cell<f32>,
    external_open: ContextMenuHandle,

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
    menu_width: Option<f32>,
    menu_min_width: Option<f32>,
    menu_max_width: Option<f32>,
    menu_min_height: Option<f32>,
    menu_max_height: Option<f32>,
    menu_transition: Option<Transition>,
    submenu_transition: Option<Transition>,
    item_transition: Option<Transition>,

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
            closing_submenus: RefCell::new(Vec::new()),

            natural_width: Cell::new(0.0),
            external_open: ContextMenuHandle::new(),

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
            menu_width: None,
            menu_min_width: None,
            menu_max_width: None,
            menu_min_height: None,
            menu_max_height: None,
            menu_transition: None,
            submenu_transition: None,
            item_transition: None,

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

    /// Binds an externally-created [`ContextMenuHandle`] to this menu, so
    /// calling [`ContextMenuHandle::open_at`] from anywhere else (e.g. via
    /// [`crate::View::context_menu`]) opens this menu without wrapping the
    /// trigger widget as this menu's child.
    ///
    /// Create the handle once with `ContextMenuHandle::new()` and keep it
    /// stable across rebuilds (e.g. with `use_state`), then pass clones of
    /// it here and to every widget that should trigger this menu.
    pub fn bind(mut self, handle: ContextMenuHandle) -> Self {
        self.external_open = handle;
        self
    }

    /// Returns this menu's open-request handle. Only meaningful when kept
    /// stable across rebuilds (e.g. via `use_state`) - otherwise prefer
    /// creating the handle yourself with `ContextMenuHandle::new()` and
    /// passing it to [`Self::bind`].
    pub fn handle(&self) -> ContextMenuHandle {
        self.external_open.clone()
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

    /// Fixes the menu (and every submenu) to an exact width, overriding
    /// automatic content-based sizing and `menu_min_width`/`menu_max_width`.
    pub fn menu_width(mut self, width: f32) -> Self {
        self.menu_width = Some(width);
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

    /// Overrides the fade transition used when the menu opens/closes.
    /// Defaults to a quick ease-out fade when not set.
    pub fn menu_transition(mut self, transition: Transition) -> Self {
        self.menu_transition = Some(transition);
        self
    }

    /// Overrides the fade transition used when a submenu level opens or
    /// closes, independent of the top-level menu's own (usually snappier)
    /// open/close transition.
    pub fn submenu_transition(mut self, transition: Transition) -> Self {
        self.submenu_transition = Some(transition);
        self
    }

    /// Overrides the hover transition applied to every item (color, and
    /// scale where `ContextMenuItem::hover_scale` is set). One shared
    /// transition for all items instead of a per-item override. Defaults
    /// to a quick ease-out transition when not set.
    pub fn item_transition(mut self, transition: Transition) -> Self {
        self.item_transition = Some(transition);
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

    // Resolves the actual width a menu level should open at, given that
    // level's natural (content-measured) width.
    fn resolve_width(&self, natural: f32) -> f32 {
        if let Some(w) = self.menu_width {
            return w;
        }
        let min = self.menu_min_width.unwrap_or(DEFAULT_MENU_MIN_WIDTH);
        let mut w = natural.max(min);
        if let Some(max) = self.menu_max_width {
            w = w.min(max);
        }
        w
    }

    fn resolve_height(&self, natural: f32) -> f32 {
        let mut h = natural;
        if let Some(min) = self.menu_min_height {
            h = h.max(min);
        }
        if let Some(max) = self.menu_max_height {
            h = h.min(max);
        }
        h
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

    fn animate_entries(
        entries: &[ContextMenuEntry],
        hovered: Option<usize>,
        transition: Transition,
        anim: &mut AnimationManager
    ) {
        for (i, entry) in entries.iter().enumerate() {
            let ContextMenuEntry::Item(item) = entry else {
                continue;
            };

            let is_target = item.enabled && hovered == Some(i);

            let hover_key = AnimKey {
                widget: item.anim_id,
                layer: AnimLayer::Root,
                property: AnimProperty::Opacity,
            };
            let hover_target = if is_target { 1.0 } else { 0.0 };
            anim.set_target(hover_key, AnimValue([hover_target, 0.0, 0.0, 0.0]), Some(transition));
            item.hover_progress.set(anim.value(hover_key).map_or(hover_target, |v| v.0[0]));

            if let Some(hover_scale) = item.hover_scale {
                let scale_key = AnimKey {
                    widget: item.anim_id,
                    layer: AnimLayer::Root,
                    property: AnimProperty::Scale,
                };
                let scale_target = if is_target { hover_scale } else { 1.0 };
                anim.set_target(
                    scale_key,
                    AnimValue([scale_target, 0.0, 0.0, 0.0]),
                    Some(transition)
                );
                item.scale_progress.set(anim.value(scale_key).map_or(scale_target, |v| v.0[0]));
            } else {
                item.scale_progress.set(1.0);
            }
        }
    }

    fn item_at_mut(&mut self, path: &[usize], idx: usize) -> Option<&mut ContextMenuItem> {
        fn walk<'a>(
            entries: &'a mut [ContextMenuEntry],
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
            self.closing_submenus.borrow_mut().clear();
            ctx.request_redraw();
        }
    }

    // Moves every level at or beyond `depth` into the closing list
    // instead of dropping them immediately, so they can fade out.
    fn close_from(&self, depth: usize) {
        let removed: Vec<OpenSubmenu> = {
            let mut stack = self.submenu_stack.borrow_mut();
            if depth >= stack.len() {
                return;
            }
            stack.split_off(depth)
        };
        self.closing_submenus.borrow_mut().extend(removed);
    }

    fn open_at_impl(&self, position: (f32, f32)) {
        let width = self.resolve_width(self.natural_width.get());
        let height = self.resolve_height(menu_height(&self.entries, self.effective_padding()));

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
        self.closing_submenus.borrow_mut().clear();
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
                    let child_w = self.resolve_width(item.submenu_width.get());
                    let child_h = self.resolve_height(menu_height(child_entries, padding));
                    let child_pos = self.position_submenu(rect, child_w, child_h);

                    // Cancels any in-progress close fade for this submenu so
                    // reopening it doesn't fight the closing animation for the same key.
                    self.closing_submenus.borrow_mut().retain(|c| c.anim_id != item.anim_id);

                    self.submenu_stack.borrow_mut().push(OpenSubmenu {
                        path: child_path,
                        pos: Cell::new(child_pos),
                        size: Cell::new((child_w, child_h)),
                        hovered_index: Cell::new(None),
                        pressed_index: Cell::new(None),
                        anim_id: item.anim_id,
                        opacity: Cell::new(0.0),
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

        let transition = self.menu_transition.unwrap_or(OPACITY_TRANSITION);
        anim.set_target(key, AnimValue([target, 0.0, 0.0, 0.0]), Some(transition));

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
        let (pad_l, pad_t, pad_b) = (pad.left.value(), pad.top.value(), pad.bottom.value());

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

            // Only the highlight rect scales with hover; the label, shortcut
            // and arrow keep the row's real geometry so they don't visibly
            // detach from a still-settling scale animation.
            let item_scale = item.scale_progress.get();
            let (bg_x, bg_y, bg_w, bg_h) = if (item_scale - 1.0).abs() > f32::EPSILON {
                let scaled = crate::scaled_layout_box(
                    LayoutBox { x, y, width: w, height: h },
                    item_scale
                );
                (scaled.x, scaled.y, scaled.width, scaled.height)
            } else {
                (x, y, w, h)
            };

            let is_hovered = item.enabled && hovered_index == Some(i);
            let is_pressed = is_hovered && pressed_index == Some(i);

            // Keeps blending toward the hover/pressed color even after the
            // pointer leaves, so hover_progress drives a real fade back to
            // idle instead of an instant snap.
            let (target_bg, hover_text_color_opt) = if is_pressed {
                (
                    self.item_pressed_background
                        .clone()
                        .or_else(|| self.item_hover_background.clone())
                        .unwrap_or(Background::Color(theme.pressed)),
                    self.item_pressed_text_color
                        .or(self.item_hover_text_color)
                        .or(self.item_text_color),
                )
            } else {
                (
                    self.item_hover_background.clone().unwrap_or(Background::Color(theme.hover)),
                    self.item_hover_text_color.or(self.item_text_color),
                )
            };

            let border = if is_pressed {
                self.item_pressed_border.or(self.item_hover_border).or(self.item_border)
            } else if is_hovered {
                self.item_hover_border.or(self.item_border)
            } else {
                self.item_border
            };

            // A press snaps straight to its color; otherwise the highlight
            // fades using this item's own animated hover progress.
            let t = if is_pressed { 1.0 } else { item.hover_progress.get() };

            let idle_bg_color = match &self.item_background {
                Some(Background::Color(c)) => *c,
                None => Color::TRANSPARENT,
            };
            let target_bg_color = match &target_bg {
                Background::Color(c) => *c,
            };
            let blended_bg = lerp_color(idle_bg_color, target_bg_color, t);

            if blended_bg.a() > 0.0 {
                ctx.draw_rect(RectCommand {
                    position: (bg_x, bg_y),
                    size: (bg_w, bg_h),
                    background: Some(faded_background(Background::Color(blended_bg), opacity)),
                    border_radius: border.map(|b| b.radius).or(Some(Length::px(4.0))),
                    border_width: border.map(|b| b.width),
                    border_color: border.map(|b| b.color.with_alpha_f32(b.color.a() * opacity)),
                    clip_rect: None,
                });
            }

            let idle_text_color = if item.enabled {
                self.item_text_color.unwrap_or(theme.foreground)
            } else {
                theme.foreground_muted
            };
            let target_text_color = if item.enabled {
                hover_text_color_opt.unwrap_or(theme.foreground)
            } else {
                hover_text_color_opt.unwrap_or(theme.foreground_muted)
            };
            let base_color = lerp_color(idle_text_color, target_text_color, t);
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

            // Submenu arrow and shortcut are independent - an item with
            // both shows the shortcut just left of the arrow instead of
            // the arrow silently hiding it.
            let mut right_reserved = 0.0;

            if item.has_submenu() {
                let arrow_color = base_color.with_alpha_f32(base_color.a() * opacity * alpha_scale);
                for (p0, p1, p2) in submenu_arrow_triangles((x, y, w, h)) {
                    ctx.draw_triangle(TriangleCommand {
                        p0,
                        p1,
                        p2,
                        color: arrow_color,
                        clip_rect: None,
                    });
                }
                right_reserved += SUBMENU_ARROW_RESERVED;
            }

            if let Some(shortcut) = &item.shortcut {
                let mut shortcut_style = text_style.clone();
                shortcut_style.color = Some(
                    base_color.with_alpha_f32(base_color.a() * opacity * alpha_scale * 0.6)
                );

                let shortcut_w = item.shortcut_width.get();
                let shortcut_padding = SHORTCUT_RIGHT_PADDING;

                ctx.draw_text(TextCommand {
                    text: shortcut.clone(),
                    position: (x + w - right_reserved - shortcut_padding - shortcut_w, text_y),
                    style: shortcut_style,
                    max_width: Some(shortcut_w),
                    clip_rect: None,
                });
                right_reserved += shortcut_w + SHORTCUT_GAP + shortcut_padding;
            }

            ctx.draw_text(TextCommand {
                text: item.label.clone(),
                position: (x + pad_l, text_y),
                style: text_style,
                max_width: Some((w - pad_l - right_reserved).max(0.0)),
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

    fn on_layout_pass(&self, ctx: &mut MeasureContext) {
        let sf = ctx.scale_factor;
        let style = &self.base.computed_style;
        let padding = self.effective_item_padding();
        let font = style.font.as_deref();
        let font_size = style.font_size
            .map(|s| s.to_physical(sf))
            .unwrap_or(DEFAULT_FONT_SIZE.to_physical(sf));
        let weight = style.font_weight.unwrap_or_default();
        let font_style = style.font_style.unwrap_or_default();
        let letter_spacing = style.letter_spacing
            .map(|ls| ls.value().to_physical(sf))
            .unwrap_or(0.0);
        let line_height = style.line_height.map(|lh| lh.value().to_physical(sf)).unwrap_or(0.0);
        let pad_lr = padding.left.to_physical(sf);

        let width = measure_entries_width(
            &self.entries,
            ctx.text,
            font,
            font_size,
            weight,
            font_style,
            letter_spacing,
            line_height,
            pad_lr
        );
        self.natural_width.set(width);
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

        let closing_levels: Vec<ClosingLevelSnapshot> = self.submenu_stack
            .borrow()
            .iter()
            .map(|l| ClosingLevelSnapshot {
                path: l.path.clone(),
                pos: l.pos.get(),
                size: l.size.get(),
                hovered: l.hovered_index.get(),
                pressed: l.pressed_index.get(),
                opacity: l.opacity.get(),
            })
            .collect();

        for level in closing_levels {
            let entries = self.entries_at(&level.path);
            self.paint_level(
                ctx,
                &theme,
                opacity * level.opacity,
                entries,
                level.pos,
                level.size,
                level.hovered,
                level.pressed,
                padding
            );
        }
    }

    fn hit_test(&self, point: (f32, f32)) -> bool {
        self.layout_box.contains_rounded(point, 0.0) || self.point_in_any_menu(point)
    }

    fn blocks_children_hit_test(&self, _point: (f32, f32)) -> bool {
        // While open, every click must land here first so an outside
        // click can close the menu instead of being consumed by
        // whatever widget sits underneath it.
        self.open.get()
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

                        if let Some(item) = self.item_at_mut(&path, idx) && item.enabled {
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

        // Opens the menu when a bound `ContextMenuHandle` (e.g. from a
        // `View::context_menu`) requested it, without needing this widget
        // to wrap the trigger widget as its own child.
        if let Some(position) = self.external_open.take_request() {
            self.open_at_impl(position);
        }

        self.animate_opacity(anim);

        let submenu_transition = self.submenu_transition.unwrap_or(SUBMENU_OPACITY_TRANSITION);
        for level in self.submenu_stack.borrow().iter() {
            let key = AnimKey {
                widget: level.anim_id,
                layer: AnimLayer::Content,
                property: AnimProperty::Opacity,
            };
            anim.set_target(key, AnimValue([1.0, 0.0, 0.0, 0.0]), Some(submenu_transition));
            level.opacity.set(anim.value(key).map_or(1.0, |v| v.0[0]));
        }

        self.closing_submenus.borrow_mut().retain(|level| {
            let key = AnimKey {
                widget: level.anim_id,
                layer: AnimLayer::Content,
                property: AnimProperty::Opacity,
            };
            anim.set_target(key, AnimValue([0.0, 0.0, 0.0, 0.0]), Some(submenu_transition));
            let value = anim.value(key).map_or(0.0, |v| v.0[0]);
            level.opacity.set(value);
            value > 0.001
        });

        let item_transition = self.item_transition.unwrap_or(ITEM_HOVER_TRANSITION);

        Self::animate_entries(&self.entries, self.hovered_index.get(), item_transition, anim);

        let submenu_levels: Vec<(Vec<usize>, Option<usize>)> = self.submenu_stack
            .borrow()
            .iter()
            .map(|l| (l.path.clone(), l.hovered_index.get()))
            .collect();
        for (path, hovered) in submenu_levels {
            let entries = self.entries_at(&path);
            Self::animate_entries(entries, hovered, item_transition, anim);
        }

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
            self.closing_submenus.replace(old.closing_submenus.borrow().clone());
            self.anim_id = old.anim_id;
            transfer_entry_anim_state(&mut self.entries, &old.entries);
        }
    }

    fn anim_id(&self) -> WidgetId {
        self.anim_id
    }
}

// Preserves each item's animation identity and in-flight progress across
// rebuilds, since `entries` is rebuilt fresh by user code every render.
fn transfer_entry_anim_state(
    new_entries: &mut [ContextMenuEntry],
    old_entries: &[ContextMenuEntry]
) {
    for (new_entry, old_entry) in new_entries.iter_mut().zip(old_entries.iter()) {
        if
            let (ContextMenuEntry::Item(new_item), ContextMenuEntry::Item(old_item)) = (
                new_entry,
                old_entry,
            )
        {
            new_item.anim_id = old_item.anim_id;
            new_item.hover_progress.set(old_item.hover_progress.get());
            new_item.scale_progress.set(old_item.scale_progress.get());
            transfer_entry_anim_state(&mut new_item.submenu, &old_item.submenu);
        }
    }
}
