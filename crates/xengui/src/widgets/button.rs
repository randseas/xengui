// SPDX-License-Identifier: Apache-2.0
use crate::{
    Background, Interaction, LayoutBox, LayoutContext, PaintContext, RectCommand, Style,
    StyleBuilder, TextCommand, Widget, WidgetContent
};
use smol_str::SmolStr;
use std::cell::Cell;
use winit::window::CursorIcon;

/// Tıklanabilir buton widget'ı.
///
/// - Mouse click (press-inside + release-inside, sol tuş) VE klavye (widget
///   focus'lu iken Enter/Space) ile tetiklenir. Bu doğrulama mantığı artık
///   `Interaction::handle` içinde merkezi olarak yaşıyor (bkz.
///   `interaction.rs`) — Button sadece kendi görsel state'ini
///   `self.interaction`'dan okuyor, event işleme mantığını kendisi
///   içermiyor.
/// - Hover/pressed/disabled durumlarına göre arkaplanı otomatik değiştirir
///   (bkz. `effective_background`); her durum için renk `hover_background()`,
///   `pressed_background()`, `disabled_background()` ile açıkça verilir —
///   framework şu an bir renk aritmetiği (otomatik koyultma) API'si
///   sunmadığından varsayılan renk *icat edilmez*, sadece `.background()`'a
///   düşer.
/// - Basılıyken imleç butonun dışına sürüklenip orada bırakılırsa tıklama
///   İPTAL olur (standart buton davranışı) — bunun için `hovered`/`pressed`
///   ayrı state olarak tutuluyor (bkz. `Interaction`).
/// - Etiket, layout box içinde ortalanır. `measure()` sırasında ölçülen
///   içerik boyutu `Cell` içinde saklanır çünkü `paint()` `&self` alır ve
///   `PaintContext`'in metin ölçüm erişimi yoktur.
pub struct Button {
    dirty: bool,
    label: SmolStr,
    font: Option<SmolStr>,
    style: Style,

    hover_background: Option<Background>,
    pressed_background: Option<Background>,
    disabled_background: Option<Background>,

    interaction: Interaction,

    layout_box: LayoutBox,
    content_size: Cell<(f32, f32)>,
}

impl Button {
    pub fn new() -> Self {
        let mut interaction = Interaction::new();
        // Button doğası gereği her zaman interaktif ve focus alabilir
        // (on_click hiç set edilmemiş olsa bile hover/press görsel state'i
        // çalışmalı, tab ile focus alabilmeli).
        interaction.focusable = true;
        interaction.hover_cursor = Some(CursorIcon::Pointer);

        Self {
            dirty: true,
            label: SmolStr::new(""),
            font: None,
            style: Style::default(),
            hover_background: None,
            pressed_background: None,
            disabled_background: None,
            interaction,
            layout_box: LayoutBox::default(),
            content_size: Cell::new((0.0, 0.0)),
        }
    }

    /// Buton etiketi.
    pub fn label(mut self, text: impl Into<SmolStr>) -> Self {
        self.label = text.into();
        self.mark_dirty();
        self
    }

    pub fn font(mut self, font: impl Into<SmolStr>) -> Self {
        self.font = Some(font.into());
        self.mark_dirty();
        self
    }

    /// Hover durumunda kullanılacak arkaplan. Verilmezse `background()`
    /// aynen kullanılır (otomatik koyultma yapılmaz).
    pub fn hover_background<B: Into<Background>>(mut self, background: B) -> Self {
        self.hover_background = Some(background.into());
        self.mark_dirty();
        self
    }

    /// Basılı durumda kullanılacak arkaplan.
    pub fn pressed_background<B: Into<Background>>(mut self, background: B) -> Self {
        self.pressed_background = Some(background.into());
        self.mark_dirty();
        self
    }

    /// `enabled(false)` iken kullanılacak arkaplan.
    pub fn disabled_background<B: Into<Background>>(mut self, background: B) -> Self {
        self.disabled_background = Some(background.into());
        self.mark_dirty();
        self
    }

    /// `false` verilirse buton event almayı bırakır (hover/press/click yok)
    /// ve `disabled_background` (varsa) ile çizilir.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.interaction.set_enabled(enabled);
        self.mark_dirty();
        self
    }

    /// Mevcut state'e (disabled > pressed > hovered > normal, bu öncelik
    /// sırasıyla) göre kullanılacak arkaplanı seçer.
    fn effective_background(&self) -> Option<Background> {
        if !self.interaction.enabled {
            return self
                .disabled_background
                .clone()
                .or_else(|| self.style.background.clone());
        }
        if self.interaction.pressed {
            return self
                .pressed_background
                .clone()
                .or_else(|| self.hover_background.clone())
                .or_else(|| self.style.background.clone());
        }
        if self.interaction.hovered {
            return self
                .hover_background
                .clone()
                .or_else(|| self.style.background.clone());
        }
        self.style.background.clone()
    }
}

impl Default for Button {
    fn default() -> Self {
        Self::new()
    }
}

impl StyleBuilder for Button {
    fn style_mut(&mut self) -> &mut Style {
        &mut self.style
    }

    fn mark_dirty(&mut self) {
        self.dirty = true;
    }
}

/// `Button("Tıkla")` şeklinde `view!` makrosu içinde tek-argümanlı kullanım
/// desteği (bkz. `macros.rs`).
impl WidgetContent for Button {
    fn with_content(self, content: impl Into<SmolStr>) -> Self {
        self.label(content)
    }
}

// on_click / on_hover / on_mouse_enter / on_mouse_leave / on_mouse_input /
// on_key builder metodları burada üretiliyor (bkz. interaction.rs).
// NOT: on_click artık `FnMut()` değil `FnMut(&mut EventCtx)` alıyor —
// mevcut çağrı yerlerinin `.on_click(|_ctx| { ... })` şeklinde güncellenmesi
// gerekir.
crate::impl_interaction_builders!(Button);

impl Widget for Button {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn is_dirty(&self) -> bool {
        self.dirty
    }

    fn set_dirty(&mut self, dirty: bool) {
        self.dirty = dirty;
    }

    fn style(&self) -> &Style {
        &self.style
    }

    fn style_mut(&mut self) -> &mut Style {
        &mut self.style
    }

    fn children(&self) -> &[Box<dyn Widget>] {
        &[]
    }

    fn interaction(&self) -> Option<&Interaction> {
        Some(&self.interaction)
    }

    fn interaction_mut(&mut self) -> Option<&mut Interaction> {
        Some(&mut self.interaction)
    }

    fn measure(&self, ctx: &mut LayoutContext) -> (f32, f32) {
        let scale_factor = ctx.scale_factor;

        let font_size = self
            .style
            .font_size
            .map(|s| s.to_physical(scale_factor))
            .unwrap_or(20.0 * scale_factor);

        let letter_spacing = self
            .style
            .letter_spacing
            .map(|ls| ls.value().to_physical(scale_factor))
            .unwrap_or(0.0);

        let line_height = self
            .style
            .line_height
            .map(|lh| lh.value().to_physical(scale_factor))
            .unwrap_or(0.0);

        let (text_w, text_h) = ctx.text.measure(
            &self.label,
            self.font.as_deref(),
            font_size,
            self.style.font_weight.unwrap_or_default(),
            self.style.font_style.unwrap_or_default(),
            letter_spacing,
            line_height,
        );

        self.content_size.set((text_w, text_h));

        let padding = &self.style.padding.unwrap();

        (
            text_w + padding.left.value() + padding.right.value(),
            text_h + padding.top.value() + padding.bottom.value(),
        )
    }

    fn layout(&mut self, rect: LayoutBox) {
        self.layout_box = rect;
    }

    fn layout_box(&self) -> &LayoutBox {
        &self.layout_box
    }

    fn paint(&self, ctx: &mut PaintContext) {
        if let Some(background) = self.effective_background() {
            let border = self.style.border.as_ref();
            ctx.draw_rect(RectCommand {
                position: (self.layout_box.x, self.layout_box.y),
                size: (self.layout_box.width, self.layout_box.height),
                background: Some(background),
                border_radius: border.map(|b| b.radius),
                border_width: border.map(|b| b.width),
                border_color: border.map(|b| b.color),
            });
        }

        let (content_w, content_h) = self.content_size.get();
        let padding = self.style.padding.unwrap();
        let text_x = self.layout_box.x
            + padding.left.value()
            + (self.layout_box.width - padding.left.value() - padding.right.value() - content_w)
                .max(0.0)
                * 0.5;

        let text_y = self.layout_box.y
            + padding.top.value()
            + (self.layout_box.height - padding.top.value() - padding.bottom.value() - content_h)
                .max(0.0)
                * 0.5;

        ctx.draw_text(TextCommand {
            text: self.label.clone(),
            position: (text_x, text_y),
            style: self.style.clone(),
            font: self.font.clone(),
        });
    }

    // event() artık override edilmiyor: Widget trait'indeki varsayılan
    // implementasyon interaction()/interaction_mut() üzerinden otomatik
    // çalışıyor (bkz. widget.rs + interaction.rs).
}
