// SPDX-License-Identifier: Apache-2.0
use crate::{
    LayoutBox,
    LayoutContext,
    MeasureContext,
    Position,
    RenderCache,
    Style,
    Widget,
    WidgetPath,
    style_to_taffy,
};
use taffy::prelude::*;

pub struct LayoutEngine;

impl LayoutEngine {
    /// Cascades computed style (including active property animations)
    /// down the tree without touching the layout tree itself - cheap
    /// enough to run every frame so paint-only transitions stay live
    /// even on frames where the box model doesn't need recomputing.
    pub fn cascade(tree: &mut [Box<dyn Widget>], ctx: &mut LayoutContext) {
        for widget in tree.iter_mut() {
            widget.cascade_style(&Style::default(), ctx.anim);
        }
    }

    pub fn layout(
        tree: &mut [Box<dyn Widget>],
        ctx: &mut LayoutContext,
        cache: &mut RenderCache,
        viewport_width: f32,
        viewport_height: f32
    ) {
        // Lets Length::ViewportWidth/ViewportHeight resolve against the
        // current frame's viewport size during measurement and layout.
        crate::set_viewport_size(viewport_width, viewport_height);

        Self::cascade(tree, ctx);
        let mut taffy: TaffyTree<()> = TaffyTree::new();
        let mut path = WidgetPath::new();

        let child_ids: Vec<NodeId> = tree
            .iter()
            .enumerate()
            .map(|(i, c)| {
                let checkpoint = path.checkpoint();
                path.push(c.as_ref(), i);
                let id = build_taffy_node(c.as_ref(), &mut taffy, ctx, cache, &mut path);
                path.restore(checkpoint);
                id
            })
            .collect();

        let root_style = taffy::style::Style {
            display: taffy::style::Display::Flex,
            flex_direction: taffy::style::FlexDirection::Column,
            size: Size {
                width: length(viewport_width),
                height: length(viewport_height),
            },
            ..Default::default()
        };
        let root_id = taffy
            .new_with_children(root_style, &child_ids)
            .expect("cannot create taffy root node");

        taffy
            .compute_layout(root_id, Size {
                width: AvailableSpace::Definite(viewport_width),
                height: AvailableSpace::Definite(viewport_height),
            })
            .expect("cannot calculate taffy layout");

        let viewport = (viewport_width, viewport_height);

        for (widget, node_id) in tree.iter_mut().zip(child_ids) {
            apply_layout(
                widget.as_mut(),
                &taffy,
                node_id,
                0.0,
                0.0,
                ctx.scale_factor,
                viewport,
                None
            );
        }
    }
}

fn build_taffy_node(
    widget: &dyn Widget,
    taffy: &mut TaffyTree<()>,
    ctx: &mut LayoutContext,
    cache: &mut RenderCache,
    path: &mut WidgetPath
) -> NodeId {
    let mut measure_ctx = MeasureContext::new(ctx.text, ctx.scale_factor);

    // Runs for every widget, independent of whether it ends up in the
    // leaf/auto-size branch below - lets non-leaf widgets (e.g. a
    // ContextMenu wrapping content) cache their own intrinsic
    // measurements too.
    widget.on_layout_pass(&mut measure_ctx);

    let mut style = style_to_taffy(widget.computed_style(), ctx.scale_factor);
    let children = widget.children();

    if children.is_empty() {
        let auto_w = style.size.width == taffy::style::Dimension::auto();
        let auto_h = style.size.height == taffy::style::Dimension::auto();

        if auto_w || auto_h {
            let mut constraints = super::Constraints::default();

            if let Some(max_size) = widget.computed_style().max_size {
                if let Some(crate::Length::Px(w)) = max_size.width {
                    constraints = constraints.with_max_width(w * ctx.scale_factor);
                }
                if let Some(crate::Length::Px(h)) = max_size.height {
                    constraints = constraints.with_max_height(h * ctx.scale_factor);
                }
            }

            // If one axis is already fixed (e.g. width set, height auto), read
            // it from xengui's own Style (not taffy's Dimension, which is an
            // opaque struct in this taffy version) and pass it as a known
            // constraint so the auto axis measures against the real size.
            if let Some(size) = widget.computed_style().size {
                if !auto_w && let Some(crate::Length::Px(w)) = size.width {
                    constraints = constraints.with_known_width(w * ctx.scale_factor);
                }
                if !auto_h && let Some(crate::Length::Px(h)) = size.height {
                    constraints = constraints.with_known_height(h * ctx.scale_factor);
                }
            }

            let measure = if widget.is_dirty() {
                let size = widget.measure(&mut measure_ctx, constraints);
                cache.store_measure(path.as_str(), size);
                size
            } else {
                cache.cached_measure(path.as_str()).unwrap_or_else(|| {
                    let size = widget.measure(&mut measure_ctx, constraints);
                    cache.store_measure(path.as_str(), size);
                    size
                })
            };

            let (w, h) = (measure.width.round(), measure.height.round());

            if auto_w {
                style.size.width = length(w);
                if style.min_size.width == taffy::style::Dimension::auto() {
                    style.min_size.width = length(w);
                }
            }
            if auto_h {
                style.size.height = length(h);
                if style.min_size.height == taffy::style::Dimension::auto() {
                    style.min_size.height = length(h);
                }
            }
        }
        taffy.new_leaf(style).expect("cannot create taffy leaf")
    } else {
        let child_ids: Vec<NodeId> = children
            .iter()
            .enumerate()
            .map(|(i, c)| {
                let checkpoint = path.checkpoint();
                path.push(c.as_ref(), i);
                let id = build_taffy_node(c.as_ref(), taffy, ctx, cache, path);
                path.restore(checkpoint);
                id
            })
            .collect();
        taffy.new_with_children(style, &child_ids).expect("cannot create taffy node")
    }
}

#[allow(clippy::too_many_arguments)]
fn apply_layout(
    widget: &mut dyn Widget,
    taffy: &TaffyTree<()>,
    node_id: NodeId,
    parent_x: f32,
    parent_y: f32,
    scale_factor: f32,
    viewport: (f32, f32),
    scroll_viewport: Option<LayoutBox>
) {
    let layout = taffy.layout(node_id).expect("cannot find taffy layout result");
    let abs_x = parent_x + layout.location.x;
    let abs_y = parent_y + layout.location.y;
    let abs_right = abs_x + layout.size.width;
    let abs_bottom = abs_y + layout.size.height;

    let mut snapped_x = abs_x.round();
    let mut snapped_y = abs_y.round();
    let snapped_right = abs_right.round();
    let snapped_bottom = abs_bottom.round();
    let width = snapped_right - snapped_x;
    let height = snapped_bottom - snapped_y;

    let position = widget.computed_style().position.unwrap_or_default();

    // Fixed is anchored to the viewport itself rather than the flow
    // position taffy computed, matching CSS's `position: fixed`.
    if position == Position::Fixed {
        let (vw, vh) = viewport;
        let style = widget.computed_style();
        let (top, right, bottom, left) = (style.top, style.right, style.bottom, style.left);

        if let Some(top) = top {
            snapped_y = top.to_physical(scale_factor).round();
        } else if let Some(bottom) = bottom {
            snapped_y = (vh - bottom.to_physical(scale_factor) - height).round();
        }
        if let Some(left) = left {
            snapped_x = left.to_physical(scale_factor).round();
        } else if let Some(right) = right {
            snapped_x = (vw - right.to_physical(scale_factor) - width).round();
        }
    }

    // Sticky clamps the in-flow position so it can't scroll past its
    // nearest scrollable ancestor's edge, matching CSS's `position: sticky`.
    if position == Position::Sticky && let Some(container) = scroll_viewport {
        let style = widget.computed_style();
        if let Some(top) = style.top {
            snapped_y = snapped_y.max(container.y + top.to_physical(scale_factor));
        }
        if let Some(bottom) = style.bottom {
            snapped_y = snapped_y.min(
                container.y + container.height - height - bottom.to_physical(scale_factor)
            );
        }
        if let Some(left) = style.left {
            snapped_x = snapped_x.max(container.x + left.to_physical(scale_factor));
        }
        if let Some(right) = style.right {
            snapped_x = snapped_x.min(
                container.x + container.width - width - right.to_physical(scale_factor)
            );
        }
    }

    widget.layout(LayoutBox {
        x: snapped_x,
        y: snapped_y,
        width,
        height,
    });

    let child_ids = taffy.children(node_id).ok();

    // Union of every in-flow child's own box gives the total scrollable
    // content size, which can exceed this node's own box when content
    // overflows it. Out-of-flow children (absolute/fixed) are skipped,
    // since they don't participate in the normal flow's overflow box.
    if let Some(ids) = &child_ids {
        let mut content_w: f32 = layout.size.width;
        let mut content_h: f32 = layout.size.height;
        let children_ref = widget.children();

        for (i, &child_id) in ids.iter().enumerate() {
            let out_of_flow = children_ref
                .get(i)
                .is_some_and(|c| {
                    matches!(
                        c.computed_style().position.unwrap_or_default(),
                        Position::Absolute | Position::Fixed
                    )
                });
            if out_of_flow {
                continue;
            }
            if let Ok(child_layout) = taffy.layout(child_id) {
                content_w = content_w.max(child_layout.location.x + child_layout.size.width);
                content_h = content_h.max(child_layout.location.y + child_layout.size.height);
            }
        }
        widget.set_content_size((content_w, content_h));
    }

    let (offset_x, offset_y) = widget.scroll_offset();

    let next_scroll_viewport = widget
        .clip_children()
        .map(|(x, y, w, h)| LayoutBox { x, y, width: w, height: h })
        .or(scroll_viewport);

    if let (Some(children), Some(ids)) = (widget.children_mut(), child_ids) {
        for (child, child_id) in children.iter_mut().zip(ids) {
            apply_layout(
                child.as_mut(),
                taffy,
                child_id,
                snapped_x - offset_x,
                snapped_y - offset_y,
                scale_factor,
                viewport,
                next_scroll_viewport
            );
        }
    }
}
