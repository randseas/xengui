// SPDX-License-Identifier: Apache-2.0
use crate::{LayoutBox, PaintContext, LayoutContext};

pub trait VNode {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
    fn key(&self) -> &str;
    fn is_dirty(&self) -> bool;
    fn set_dirty(&mut self, dirty: bool);
    fn paint(&self, ctx: &mut PaintContext);
    fn measure(&self, ctx: &LayoutContext) -> (f32, f32);
    fn layout(&mut self, rect: LayoutBox);
    fn layout_box(&self) -> &LayoutBox;
}
