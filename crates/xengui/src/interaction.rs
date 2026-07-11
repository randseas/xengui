// SPDX-License-Identifier: Apache-2.0
use crate::{EventCtx, EventStatus, InputEvent, Key, KeyState, KeyboardEvent};
use winit::{
    event::{ElementState, MouseButton},
    window::CursorIcon,
};

type Callback = Box<dyn FnMut(&mut EventCtx)>;
type HoverCallback = Box<dyn FnMut(bool, &mut EventCtx)>;
type MouseInputCallback = Box<dyn FnMut(ElementState, MouseButton, &mut EventCtx)>;
type KeyCallback = Box<dyn FnMut(&KeyboardEvent, &mut EventCtx)>;

/// Fare/klavye ile temel etkileşimi (hover, press, focus) ve buna bağlı
/// callback'leri tutan, widget'lar arası paylaşılan state.
///
/// Button, View gibi interaktif widget'lar bunu bir alan olarak embed edip
/// `Widget::interaction()` / `interaction_mut()` üzerinden expose eder;
/// böylece `Widget::event()`'in varsayılan implementasyonu tüm mouse/
/// keyboard event'lerini otomatik işler — her widget kendi `event()`
/// eşlemesini elle yazmak zorunda kalmaz. Leaf/dekoratif widget'lar (Text
/// gibi) bu struct'ı hiç embed etmez; `Widget::interaction()`'ın varsayılanı
/// `None`'dur ve bu durumda `event()` hiçbir şey yapmaz. Yani mouse/keyboard
/// event altyapısı TÜM widget'larda temelde mevcuttur, sadece isteyen
/// widget bunu "açar".
pub struct Interaction {
    pub enabled: bool,
    /// `true` ise (veya herhangi bir callback set edilmişse, bkz.
    /// `is_active`) bu widget sol mouse press'te focus talep eder ve
    /// klavye aktivasyon tuşlarına (Enter/Space) tepki verir.
    pub focusable: bool,
    /// Hover sırasında kullanılacak imleç ikonu. `None` ise imleç hiç
    /// değiştirilmez (Button `Some(CursorIcon::Pointer)` verir; sade bir
    /// View varsayılan olarak vermez).
    pub hover_cursor: Option<CursorIcon>,

    pub hovered: bool,
    pub pressed: bool,
    pub focused: bool,

    pub on_mouse_enter: Option<Callback>,
    pub on_mouse_leave: Option<Callback>,
    /// Hover durumu her değiştiğinde (enter -> true, exit -> false) çağrılır.
    pub on_hover: Option<HoverCallback>,
    /// TÜM mouse tuş event'lerini (herhangi bir buton, press VEYA release)
    /// ham haliyle iletir; hangi tuşa/duruma göre davranmak istediğinize
    /// callback içinde siz karar verirsiniz (tip kontrolü burada yapılır).
    pub on_mouse_input: Option<MouseInputCallback>,
    /// TÜM klavye event'lerini (herhangi bir tuş, press/release/repeat) ham
    /// haliyle iletir; hangi tuşa basıldığı `KeyEvent.logical_key` üzerinden
    /// okunabilir.
    pub on_key: Option<KeyCallback>,
    /// Sadece GEÇERLİ bir "mantıksal tıklama" olduğunda çağrılır:
    ///
    /// 1. Bu widget focus'luyken Enter/Space'e basılması, VEYA
    /// 2. Sol mouse tuşuyla widget üzerinde press + widget üzerinde
    ///    (hover korunarak) release.
    ///
    /// Başka hiçbir durumda (dışarıda release, sağ/orta tık, disabled iken,
    /// vb.) tetiklenmez — ham event'leri görmek isterseniz `on_mouse_input`
    /// / `on_key` kullanın.
    pub on_click: Option<Callback>,
}

impl Interaction {
    pub fn new() -> Self {
        Self {
            enabled: true,
            focusable: false,
            hover_cursor: None,
            hovered: false,
            pressed: false,
            focused: false,
            on_mouse_enter: None,
            on_mouse_leave: None,
            on_hover: None,
            on_mouse_input: None,
            on_key: None,
            on_click: None,
        }
    }

    /// `enabled(false)` olduğunda ephemeral state'i (hover/press) temizler.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.hovered = false;
            self.pressed = false;
        }
    }

    /// Bu interaction'ın event almasının anlamlı olup olmadığı: en az bir
    /// callback set edilmiş veya `focusable = true` ise `true`. Callback'siz,
    /// `focusable(true)` da çağrılmamış sade bir View
    /// (`View::new().child(...)`) bu yüzden tamamen inert kalır — `event()`
    /// hiçbir şey yapmaz, focus çalmaz, hover izlemez.
    pub fn is_active(&self) -> bool {
        self.focusable
            || self.on_mouse_enter.is_some()
            || self.on_mouse_leave.is_some()
            || self.on_hover.is_some()
            || self.on_mouse_input.is_some()
            || self.on_key.is_some()
            || self.on_click.is_some()
    }

    /// Reconciliation sırasında eski widget'ın interaction'ından transient
    /// state'i (hovered/pressed/focused) kopyalar. Callback'ler kopyalanmaz
    /// — yeni ağaç zaten kendi closure'larını taşıyor. Declarative/rebuild
    /// tarzı bir UI'da (state değiştikçe widget ağacı yeniden inşa
    /// ediliyorsa) reconciler, key eşleşen eski/yeni widget çiftinde
    /// `new_widget.transfer_interaction_state(old_widget)` çağırmalı; aksi
    /// halde hover/focus her rebuild'de sıfırlanır (muhtemel
    /// `hover_background` bug'ının kök nedeni budur — bkz. widget.rs).
    pub fn transfer_from(&mut self, old: &Interaction) {
        self.hovered = old.hovered;
        self.pressed = old.pressed;
        self.focused = old.focused;
    }

    fn is_activation_key(key: Key) -> bool {
        matches!(key, Key::Enter | Key::Space)
    }

    /// Tüm mouse/keyboard event'lerini işler (bkz. struct-level dokümantasyon
    /// ve her `on_*` alanının açıklaması).
    pub fn handle(&mut self, event: &InputEvent, ctx: &mut EventCtx) -> EventStatus {
        if !self.enabled {
            return EventStatus::Ignored;
        }

        match event {
            InputEvent::MouseEntered => {
                self.hovered = true;
                if let Some(icon) = self.hover_cursor {
                    ctx.set_cursor_icon(icon);
                }
                if let Some(cb) = self.on_mouse_enter.as_mut() {
                    cb(ctx);
                }
                if let Some(cb) = self.on_hover.as_mut() {
                    cb(true, ctx);
                }
                ctx.request_redraw();
                EventStatus::Handled
            }

            InputEvent::MouseExited => {
                self.hovered = false;
                // Basılıyken dışarı çıkılırsa tıklama iptal olsun (release
                // burada hâlâ `pressed = true` görürse yanlışlıkla
                // tetiklenmesin — bkz. `on_click` dokümantasyonu).
                self.pressed = false;
                if self.hover_cursor.is_some() {
                    ctx.set_cursor_icon(CursorIcon::Default);
                }
                if let Some(cb) = self.on_mouse_leave.as_mut() {
                    cb(ctx);
                }
                if let Some(cb) = self.on_hover.as_mut() {
                    cb(false, ctx);
                }
                ctx.request_redraw();
                EventStatus::Handled
            }

            InputEvent::MouseInput { state, button, .. } => {
                // Ham event: tip kontrolü çağıran tarafın sorumluluğunda.
                if let Some(cb) = self.on_mouse_input.as_mut() {
                    cb(*state, *button, ctx);
                }

                if *button == MouseButton::Left {
                    match state {
                        ElementState::Pressed => {
                            self.pressed = true;
                            if self.focusable {
                                ctx.request_focus();
                            }
                        }
                        ElementState::Released => {
                            let was_click = self.pressed && self.hovered;
                            self.pressed = false;
                            if was_click && let Some(cb) = self.on_click.as_mut() {
                                cb(ctx);
                            }
                        }
                    }
                }
                ctx.request_redraw();
                EventStatus::Handled
            }

            InputEvent::KeyInput {
                event: key_event, ..
            } => {
                // Ham event: hangi tuşa basıldığı callback içinde
                // `key_event.logical_key` ile okunabilir.
                if let Some(cb) = self.on_key.as_mut() {
                    cb(key_event, ctx);
                }

                if self.focused
                    && key_event.state == KeyState::Pressed
                    && !key_event.repeat
                    && Self::is_activation_key(key_event.key)
                {
                    if let Some(cb) = self.on_click.as_mut() {
                        cb(ctx);
                    }
                    ctx.request_redraw();
                }
                EventStatus::Handled
            }

            InputEvent::FocusGained => {
                self.focused = true;
                ctx.request_redraw();
                EventStatus::Handled
            }

            InputEvent::FocusLost => {
                // Focus kaybedilirken yarım kalmış bir basılı state varsa
                // temizle (ör. Tab ile başka widget'a geçildi).
                self.focused = false;
                self.pressed = false;
                ctx.request_redraw();
                EventStatus::Handled
            }

            _ => EventStatus::Ignored,
        }
    }
}

impl Default for Interaction {
    fn default() -> Self {
        Self::new()
    }
}

/// Bir widget struct'ı (kendi içinde `interaction: Interaction` alanı olan)
/// için standart mouse/keyboard builder metodlarını (`on_click`, `on_hover`,
/// `on_mouse_enter`, `on_mouse_leave`, `on_mouse_input`, `on_key`) üretir.
/// Yeni interaktif bir widget yazarken bu 6 metodu elle tekrarlamamak için
/// kullanılır — bkz. `button.rs`, `view.rs`.
#[macro_export]
macro_rules! impl_interaction_builders {
    ($ty:ty) => {
        impl $ty {
            /// Sadece GEÇERLİ mantıksal tıklamada çağrılır (bkz.
            /// `Interaction::on_click` dokümantasyonu).
            pub fn on_click(mut self, f: impl FnMut(&mut $crate::EventCtx) + 'static) -> Self {
                self.interaction.on_click = Some(Box::new(f));
                self
            }

            /// Hover durumu her değiştiğinde (true/false) çağrılır.
            pub fn on_hover(
                mut self,
                f: impl FnMut(bool, &mut $crate::EventCtx) + 'static,
            ) -> Self {
                self.interaction.on_hover = Some(Box::new(f));
                self
            }

            pub fn on_mouse_enter(
                mut self,
                f: impl FnMut(&mut $crate::EventCtx) + 'static,
            ) -> Self {
                self.interaction.on_mouse_enter = Some(Box::new(f));
                self
            }

            pub fn on_mouse_leave(
                mut self,
                f: impl FnMut(&mut $crate::EventCtx) + 'static,
            ) -> Self {
                self.interaction.on_mouse_leave = Some(Box::new(f));
                self
            }

            /// TÜM mouse tuş event'lerini (herhangi bir buton, press/release)
            /// ham haliyle alır; tip kontrolü callback içinde yapılır.
            pub fn on_mouse_input(
                mut self,
                f: impl FnMut(
                    ::winit::event::ElementState,
                    ::winit::event::MouseButton,
                    &mut $crate::EventCtx,
                ) + 'static,
            ) -> Self {
                self.interaction.on_mouse_input = Some(Box::new(f));
                self
            }

            /// TÜM klavye event'lerini (herhangi bir tuş) ham haliyle alır.
            pub fn on_key(
                mut self,
                f: impl FnMut(&$crate::KeyboardEvent, &mut $crate::EventCtx) + 'static,
            ) -> Self {
                self.interaction.on_key = Some(Box::new(f));
                self
            }
        }
    };
}
