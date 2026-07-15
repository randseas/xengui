// SPDX-License-Identifier: Apache-2.0
use crate::{ LayoutBox, LayoutContext, RenderCache, Widget, style_to_taffy };
use taffy::prelude::*;

pub struct LayoutEngine;

impl LayoutEngine {
    pub fn layout(
        tree: &mut [Box<dyn Widget>],
        ctx: &mut LayoutContext,
        cache: &mut RenderCache,
        viewport_width: f32,
        viewport_height: f32
    ) {
        let mut taffy: TaffyTree<()> = TaffyTree::new();

        let child_ids: Vec<NodeId> = tree
            .iter()
            .enumerate()
            .map(|(i, w)| build_taffy_node(w.as_ref(), &mut taffy, ctx, cache, &i.to_string()))
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

        for (widget, node_id) in tree.iter_mut().zip(child_ids) {
            apply_layout(widget.as_mut(), &taffy, node_id, 0.0, 0.0);
        }
    }
}

fn build_taffy_node(
    widget: &dyn Widget,
    taffy: &mut TaffyTree<()>,
    ctx: &mut LayoutContext,
    cache: &mut RenderCache,
    path: &str
) -> NodeId {
    let mut style = style_to_taffy(widget.style(), ctx.scale_factor);
    let children = widget.children();

    if children.is_empty() {
        let auto_w = style.size.width == taffy::style::Dimension::auto();
        let auto_h = style.size.height == taffy::style::Dimension::auto();

        if auto_w && auto_h {
            let (w, h) = if widget.is_dirty() {
                let size = widget.measure(ctx);
                cache.store_measure(path, size);
                size
            } else {
                cache.cached_measure(path).unwrap_or_else(|| {
                    let size = widget.measure(ctx);
                    cache.store_measure(path, size);
                    size
                })
            };
            // Round intrinsic content size before it enters taffy's flex
            // solve. Otherwise sibling boxes accumulate independent
            // fractional heights from text metrics, and the shared edge
            // between two rows stops being the exact same float value -
            // which breaks the edge-snapping in apply_layout below.
            let (w, h) = (w.round(), h.round());
            style.size = Size {
                width: length(w),
                height: length(h),
            };
            if style.min_size.width == taffy::style::Dimension::auto() {
                style.min_size.width = length(w);
            }
            if style.min_size.height == taffy::style::Dimension::auto() {
                style.min_size.height = length(h);
            }
        }
        taffy.new_leaf(style).expect("cannot create taffy leaf")
    } else {
        let child_ids: Vec<NodeId> = children
            .iter()
            .enumerate()
            .map(|(i, c)| build_taffy_node(c.as_ref(), taffy, ctx, cache, &format!("{path}.{i}")))
            .collect();
        taffy.new_with_children(style, &child_ids).expect("cannot create taffy node")
    }
}

fn apply_layout(
    widget: &mut dyn Widget,
    taffy: &TaffyTree<()>,
    node_id: NodeId,
    parent_x: f32,
    parent_y: f32
) {
    let layout = taffy.layout(node_id).expect("cannot find taffy layout result");
    let abs_x = parent_x + layout.location.x;
    let abs_y = parent_y + layout.location.y;
    let abs_right = abs_x + layout.size.width;
    let abs_bottom = abs_y + layout.size.height;

    // Snap each absolute edge independently, not the width/height. Adjacent
    // widgets share the same float edge value (one's right == the other's
    // left), so rounding that shared value keeps them flush - no 1px gaps
    // or overlaps regardless of which side the value happens to round to.
    let snapped_x = abs_x.round();
    let snapped_y = abs_y.round();
    let snapped_right = abs_right.round();
    let snapped_bottom = abs_bottom.round();

    widget.layout(LayoutBox {
        x: snapped_x,
        y: snapped_y,
        width: snapped_right - snapped_x,
        height: snapped_bottom - snapped_y,
    });

    // Children accumulate from the already-snapped parent origin so
    // rounding error can't compound across nesting depth.
    if let Some(children) = widget.children_mut() && let Ok(child_ids) = taffy.children(node_id) {
        for (child, child_id) in children.iter_mut().zip(child_ids) {
            apply_layout(child.as_mut(), taffy, child_id, snapped_x, snapped_y);
        }
    }
}
