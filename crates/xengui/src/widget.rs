// SPDX-License-Identifier: Apache-2.0
use smol_str::SmolStr;

use crate::{
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
    Outline,
    PaintContext,
    RectCommand,
    Style,
    properties::StyleValue,
};

use std::any::Any;

pub trait Widget: Any {
    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn debug_name(&self) -> &'static str {
        "Widget"
    }

    fn get_key(&self) -> Option<&SmolStr> {
        None
    }

    fn is_dirty(&self) -> bool;

    fn set_dirty(&mut self, dirty: bool);

    fn style(&self) -> &Style;

    fn style_mut(&mut self) -> &mut Style;

    fn computed_style(&self) -> &Style {
        self.style()
    }

    fn children(&self) -> &[Box<dyn Widget>] {
        &[]
    }

    fn children_mut(&mut self) -> Option<&mut Vec<Box<dyn Widget>>> {
        None
    }

    /// Additional scroll translation applied to this widget's children
    /// during layout, in logical pixels.
    fn scroll_offset(&self) -> (f32, f32) {
        (0.0, 0.0)
    }

    /// Reports this widget's total content size after layout, which may
    /// exceed `layout_box()` when children overflow it.
    fn set_content_size(&mut self, _size: (f32, f32)) {}

    /// The rectangle this widget clips its children's painted output to,
    /// in absolute screen coordinates. `None` means no clipping.
    fn clip_children(&self) -> Option<(f32, f32, f32, f32)> {
        None
    }

    fn measure(&self, ctx: &mut MeasureContext, constraints: Constraints) -> MeasureResult;

    fn layout(&mut self, rect: LayoutBox);

    fn layout_box(&self) -> &LayoutBox;

    fn paint(&self, ctx: &mut PaintContext);

    /// Painted after every descendant, on top of them, and never cached -
    /// used for overlays like a scrollbar thumb that depend on live state.
    fn paint_overlay(&self, _ctx: &mut PaintContext) {}

    fn paint_box(&self, ctx: &mut PaintContext) {
        let style = self.computed_style();

        if style.background.is_none() && style.border.is_none() {
            return;
        }

        let border = style.border.as_ref();

        ctx.draw_rect(RectCommand {
            position: (self.layout_box().x, self.layout_box().y),
            size: (self.layout_box().width, self.layout_box().height),
            background: style.background.clone(),
            border_radius: border.map(|b| b.radius),
            border_color: border.map(|b| b.color),
            border_width: border.map(|b| b.width),
            clip_rect: None,
        });
    }

    fn paint_outline(&self, ctx: &mut PaintContext) {
        let style = self.computed_style();

        let outline = match &style.outline {
            StyleValue::None => {
                return;
            }
            StyleValue::Value(outline) => outline,
            StyleValue::Default => {
                return;
            }
        };

        let layout = self.layout_box();
        let offset = outline.offset.value();
        let radius = outline.radius.or_else(|| { style.border.as_ref().map(|b| b.radius) });

        ctx.draw_rect(RectCommand {
            position: (layout.x - offset, layout.y - offset),
            size: (layout.width + offset * 2.0, layout.height + offset * 2.0),
            background: None,
            border_radius: radius,
            border_color: Some(outline.color),
            border_width: Some(outline.width),
            clip_rect: None,
        });
    }

    fn paint_focus(&self, ctx: &mut PaintContext) {
        let Some(interaction) = self.interaction() else {
            return;
        };

        if !interaction.focused || !interaction.focus_visible {
            return;
        }

        let style = self.computed_style();
        let layout = self.layout_box();

        let focus_outline = match &style.focus_outline {
            StyleValue::None => {
                return;
            }
            StyleValue::Value(outline) => *outline,
            StyleValue::Default =>
                Outline {
                    width: Length::px(2.5),
                    color: Color::BLUE_500,
                    radius: style.border.as_ref().map(|b| b.radius.add_px(4.0)),
                    offset: Length::px(4.0),
                },
        };

        let offset = focus_outline.offset.value();
        let radius = focus_outline.radius.or_else(|| { style.border.as_ref().map(|b| b.radius) });

        ctx.draw_rect(RectCommand {
            position: (layout.x - offset, layout.y - offset),
            size: (layout.width + offset * 2.0, layout.height + offset * 2.0),
            background: None,
            border_radius: radius,
            border_width: Some(focus_outline.width),
            border_color: Some(focus_outline.color),
            clip_rect: None,
        });
    }

    fn hit_test(&self, point: (f32, f32)) -> bool {
        let b = self.layout_box();

        if point.0 < b.x || point.0 > b.x + b.width || point.1 < b.y || point.1 > b.y + b.height {
            return false;
        }

        let Some(border) = &self.style().border else {
            return true;
        };

        let radius = border.radius.value();

        if radius <= 0.0 {
            return true;
        }

        let r = radius.min(b.width * 0.5).min(b.height * 0.5);

        let local_x = point.0 - b.x;
        let local_y = point.1 - b.y;

        if local_x >= r && local_x <= b.width - r {
            return true;
        }

        if local_y >= r && local_y <= b.height - r {
            return true;
        }

        let cx = if local_x < r { r } else { b.width - r };

        let cy = if local_y < r { r } else { b.height - r };

        let dx = local_x - cx;
        let dy = local_y - cy;

        dx * dx + dy * dy <= r * r
    }

    fn interaction(&self) -> Option<&Interaction> {
        None
    }

    fn interaction_mut(&mut self) -> Option<&mut Interaction> {
        None
    }

    fn transfer_interaction_state(&mut self, old: &dyn Widget) {
        if let (Some(new), Some(old)) = (self.interaction_mut(), old.interaction()) {
            new.transfer_from(old);
        }
    }

    fn event(&mut self, event: &InputEvent, ctx: &mut EventCtx) -> EventStatus {
        let status = match self.interaction_mut() {
            Some(interaction) if interaction.is_active() => interaction.handle(event, ctx),
            _ => EventStatus::Ignored,
        };

        if matches!(status, EventStatus::Handled) {
            self.set_dirty(true);
        }

        status
    }

    fn content_eq(&self, _other: &dyn Widget) -> bool {
        false
    }

    fn cascade_style(&mut self, parent: &Style) {
        if let Some(children) = self.children_mut() {
            for child in children.iter_mut() {
                child.cascade_style(parent);
            }
        }
    }

    fn after_interaction_transfer(&mut self) {}

    fn transfer_measured_state(&mut self, _old: &dyn Widget) {}

    fn blink_interval(&self) -> Option<std::time::Duration> {
        None
    }

    /// Whether this widget needs a continuous per-frame animation callback
    /// (`InputEvent::AnimationTick`) right now, independent of focus.
    fn wants_animation_frame(&self) -> bool {
        false
    }

    /// HTML-style selectable text content for mouse selection / Ctrl+C.
    /// `None` opts the widget out entirely (e.g. Button, TextBox).
    fn selectable_text(&self) -> Option<&str> {
        None
    }

    fn text_selection(&self) -> Option<(usize, usize)> {
        None
    }

    fn set_text_selection(&mut self, _range: Option<(usize, usize)>) {}

    // Called by the global Escape handler; clears the selection and also
    // stops any drag-in-progress so a still-held mouse button can't
    // immediately re-create the selection on the next move.
    fn cancel_text_selection(&mut self) {
        self.set_text_selection(None);
    }

    /// Nearest character index to an absolute screen point, used by
    /// cross-widget drag selection to know where a widget's own
    /// selection should start or end.
    fn text_index_at(&self, _point: (f32, f32)) -> usize {
        0
    }

    fn select_all_text(&mut self) {}
}
