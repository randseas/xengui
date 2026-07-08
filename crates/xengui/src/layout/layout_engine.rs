use crate::{LayoutBox, LayoutContext, RenderCache, VNode};

pub struct LayoutEngine;

impl LayoutEngine {
    pub fn layout(nodes: &mut [Box<dyn VNode>], ctx: &LayoutContext, cache: &RenderCache) {
        let mut current_y = 0.0;

        for node in nodes {
            let (w, h) = if node.is_dirty() {
                node.measure(ctx)
            } else {
                cache
                    .cached_size(node.key())
                    .unwrap_or_else(|| node.measure(ctx))
            };

            node.layout(LayoutBox {
                x: 0.0,
                y: current_y,
                width: w,
                height: h,
            });

            current_y += h;
        }
    }
}
