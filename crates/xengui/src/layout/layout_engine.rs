// SPDX-License-Identifier: Apache-2.0
use crate::{LayoutBox, LayoutContext, RenderCache, Widget, style_to_taffy};
use taffy::prelude::*;

pub struct LayoutEngine;

impl LayoutEngine {
    /// `tree`'yi taffy ile layout eder, sonuçları (mutlak x/y dahil) tüm
    /// ağaca (children dahil) `layout()` çağrısıyla geri yazar.
    pub fn layout(
        tree: &mut [Box<dyn Widget>],
        ctx: &LayoutContext,
        cache: &RenderCache,
        viewport_width: f32,
        viewport_height: f32,
    ) {
        let mut taffy: TaffyTree<()> = TaffyTree::new();

        let child_ids: Vec<NodeId> = tree
            .iter()
            .enumerate()
            .map(|(i, w)| build_taffy_node(w.as_ref(), &mut taffy, ctx, cache, &i.to_string()))
            .collect();

        // Sanal kök: eski davranışla (dikey stack) geriye dönük uyum için
        // varsayılan flex-column; kök seviyesindeki View'lere flex_direction
        // vererek bu değiştirilebilir (kök View'i tek child olarak verirseniz).
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
            .expect("taffy kök düğümü oluşturulamadı");

        taffy
            .compute_layout(
                root_id,
                Size {
                    width: AvailableSpace::Definite(viewport_width),
                    height: AvailableSpace::Definite(viewport_height),
                },
            )
            .expect("taffy layout hesaplanamadı");

        for (widget, node_id) in tree.iter_mut().zip(child_ids) {
            apply_layout(widget.as_mut(), &taffy, node_id, 0.0, 0.0);
        }
    }
}

fn build_taffy_node(
    widget: &dyn Widget,
    taffy: &mut TaffyTree<()>,
    ctx: &LayoutContext,
    cache: &RenderCache,
    path: &str,
) -> NodeId {
    let mut style = style_to_taffy(widget.style());
    let children = widget.children();

    if children.is_empty() {
        let auto_w = style.size.width == taffy::style::Dimension::auto();
        let auto_h = style.size.height == taffy::style::Dimension::auto();

        // Kullanıcı açıkça width/height vermemişse, widget'ın kendi içerik
        // boyutunu kullan (ör. Text için font shaping sonucu). Dirty
        // değilse ve önceki frame'den cache'lenmiş bir boyut varsa, pahalı
        // measure() tekrar çağrılmaz.
        if auto_w && auto_h {
            let (w, h) = if widget.is_dirty() {
                widget.measure(ctx)
            } else {
                cache
                    .cached_size(path)
                    .unwrap_or_else(|| widget.measure(ctx))
            };
            style.size = Size {
                width: length(w),
                height: length(h),
            };
        }
        taffy.new_leaf(style).expect("taffy leaf oluşturulamadı")
    } else {
        let child_ids: Vec<NodeId> = children
            .iter()
            .enumerate()
            .map(|(i, c)| build_taffy_node(c.as_ref(), taffy, ctx, cache, &format!("{path}.{i}")))
            .collect();
        taffy
            .new_with_children(style, &child_ids)
            .expect("taffy düğümü oluşturulamadı")
    }
}

fn apply_layout(
    widget: &mut dyn Widget,
    taffy: &TaffyTree<()>,
    node_id: NodeId,
    parent_x: f32,
    parent_y: f32,
) {
    let layout = taffy
        .layout(node_id)
        .expect("taffy layout sonucu bulunamadı");
    let abs_x = parent_x + layout.location.x;
    let abs_y = parent_y + layout.location.y;

    widget.layout(LayoutBox {
        x: abs_x,
        y: abs_y,
        width: layout.size.width,
        height: layout.size.height,
    });

    if let Some(children) = widget.children_mut()
        && let Ok(child_ids) = taffy.children(node_id) {
            for (child, child_id) in children.iter_mut().zip(child_ids) {
                apply_layout(child.as_mut(), taffy, child_id, abs_x, abs_y);
            }
        }
}
