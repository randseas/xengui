// SPDX-License-Identifier: Apache-2.0
// crates/xengui/src/core.rs
use crate::PaintContext;

pub trait VNode {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
    fn key(&self) -> &str;
    fn is_dirty(&self) -> bool;
    fn set_dirty(&mut self, dirty: bool);
    fn paint(&self, ctx: &mut PaintContext);
}
