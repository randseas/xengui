use crate::{LayoutBox, LayoutContext, VNode};

pub struct LayoutEngine;

impl LayoutEngine {
    pub fn layout(nodes: &mut [Box<dyn VNode>], ctx: &LayoutContext) {
        let mut current_y = 0.0;

        for node in nodes {
            let (w, h) = node.measure(ctx);

            node.layout(LayoutBox {
                x: 0.0,
                y: current_y,
                width: w,
                height: h,
            });

            if ctx.debug {
                log::trace!("layout: x={} y={} w={} h={}", 0.0, current_y, w, h);
            }

            current_y += h;
        }
    }
}
