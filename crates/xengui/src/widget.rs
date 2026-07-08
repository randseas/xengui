// SPDX-License-Identifier: Apache-2.0

use smol_str::SmolStr;

use crate::{LayoutBox, LayoutContext, PaintContext, Style};

use std::any::Any;

pub trait Widget: Any {
    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn key(&self) -> Option<&SmolStr> {
        None
    }

    fn is_dirty(&self) -> bool;

    fn set_dirty(&mut self, dirty: bool);

    fn style(&self) -> &Style;

    fn style_mut(&mut self) -> &mut Style;

    /// Varsayılan: children'sız leaf widget. Container widget'lar (View gibi)
    /// bunu override eder.
    fn children(&self) -> &[Box<dyn Widget>] {
        &[]
    }

    /// Varsayılan: `None` — bu widget children tutamıyor. Container
    /// widget'lar `Some(&mut self.children)` döner. Recursive fonksiyonlar
    /// (paint/dirty-reset) `children_mut()` yerine bunu kullanmalı; panic
    /// yerine `None` ile "bu dalda inecek bir şey yok" bilgisini taşır.
    fn children_mut(&mut self) -> Option<&mut Vec<Box<dyn Widget>>> {
        None
    }

    fn measure(&self, ctx: &LayoutContext) -> (f32, f32);

    fn layout(&mut self, rect: LayoutBox);

    fn layout_box(&self) -> &LayoutBox;

    fn paint(&self, ctx: &mut PaintContext);
}
