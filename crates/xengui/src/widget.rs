// SPDX-License-Identifier: Apache-2.0

use smol_str::SmolStr;

use crate::{
    EventCtx, EventStatus, InputEvent, Interaction, LayoutBox, LayoutContext, PaintContext, Style,
};

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

    /// Varsayılan hit-test: widget'ın layout box'ı `point`'i (ekran/piksel
    /// uzayında, absolute) içeriyor mu? Özel şekilli hit-alanı gereken
    /// widget'lar (ör. dairesel buton) bunu override edebilir.
    fn hit_test(&self, point: (f32, f32)) -> bool {
        let b = self.layout_box();

        // Önce bounding box kontrolü
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

        // Radius, kutunun yarısından büyük olamaz
        let r = radius.min(b.width * 0.5).min(b.height * 0.5);

        let local_x = point.0 - b.x;
        let local_y = point.1 - b.y;

        // Köşe olmayan alanlar direkt içeride
        if local_x >= r && local_x <= b.width - r {
            return true;
        }

        if local_y >= r && local_y <= b.height - r {
            return true;
        }

        // Hangi köşedeyiz?
        let cx = if local_x < r { r } else { b.width - r };

        let cy = if local_y < r { r } else { b.height - r };

        let dx = local_x - cx;
        let dy = local_y - cy;

        // Çember denklemi
        dx * dx + dy * dy <= r * r
    }

    /// Bu widget'ın mouse/keyboard etkileşim state'ini (hover/press/focus +
    /// callback'ler) tutuyorsa referansını döner. İnteraktif olmak isteyen
    /// her widget (Button, View, ...) bunu `Some(&self.interaction)` ile
    /// override eder. Leaf/dekoratif widget'lar (Text gibi) override etmeye
    /// gerek duymaz; `None` varsayılanı `event()`'in hiçbir şey yapmamasını
    /// sağlar. Yani mouse/keyboard event altyapısı TÜM widget'larda
    /// temelde vardır, sadece kullanmak isteyenler `interaction()` /
    /// `interaction_mut()`'ı override eder.
    fn interaction(&self) -> Option<&Interaction> {
        None
    }

    fn interaction_mut(&mut self) -> Option<&mut Interaction> {
        None
    }

    /// Reconciler, eski ağaçtaki bir widget'ı key eşleşmesiyle yeni
    /// ağaçtaki karşılığıyla değiştirirken bunu çağırmalı: `hovered`/
    /// `pressed`/`focused` gibi transient etkileşim state'ini eskiden
    /// yeniye taşır. BUNU YAPMAZSANIZ: declarative rebuild üzerine kurulu
    /// bir UI'da (widget ağacı her state güncellemesinde yeniden inşa
    /// ediliyorsa) hover/focus state'i her redraw'da sıfırlanır —
    /// `hover_background` gibi durumların "çalışmıyormuş gibi" görünmesinin
    /// en olası nedeni budur.
    fn transfer_interaction_state(&mut self, old: &dyn Widget) {
        if let (Some(new), Some(old)) = (self.interaction_mut(), old.interaction()) {
            new.transfer_from(old);
        }
    }

    /// Bir input event'i işler. Varsayılan davranış: `interaction()`
    /// `Some` dönüyorsa VE aktifse (bkz. `Interaction::is_active`) event'i
    /// ona devreder; aksi halde event'i yok sayar (`Ignored`) ve bubble üst
    /// widget'a devam eder. Kendi özel event mantığı gereken widget'lar
    /// bunu yine de override edebilir.
    fn event(&mut self, event: &InputEvent, ctx: &mut EventCtx) -> EventStatus {
        match self.interaction_mut() {
            Some(interaction) if interaction.is_active() => interaction.handle(event, ctx),
            _ => EventStatus::Ignored,
        }
    }
}
