// SPDX-License-Identifier: Apache-2.0
use crate::{DrawCommand, LayoutBox};
use std::collections::{HashMap, HashSet};

struct CachedEntry {
    layout_box: LayoutBox,
    commands: Vec<DrawCommand>,
}

/// Frame'ler arası kalıcı bir cache: her VNode'un `key()`'ine göre son
/// layout box'ını ve o box için üretilmiş DrawCommand'ları tutar.
/// React'in "reconciliation" fikrinin sadeleştirilmiş hâli:
/// - node dirty değilse VE layout box'ı hiç değişmemişse -> paint() atlanır,
///   önceki komutlar aynen yeniden kullanılır.
/// - node'un boyutu (width/height) cache'te varsa ve node dirty değilse ->
///   measure() atlanır, önceki boyut kullanılır (font shaping tekrar çalışmaz).
#[derive(Default)]
pub struct RenderCache {
    entries: HashMap<String, CachedEntry>,
}

impl RenderCache {
    pub fn new() -> Self {
        Self::default()
    }

    /// Dirty olmayan bir node için önbelleklenmiş (width, height) döner.
    /// İlk frame'de veya cache'te hiç yoksa `None` döner; çağıran taraf
    /// bu durumda `measure()`'ı normal şekilde çağırmalı.
    pub fn cached_size(&self, key: &str) -> Option<(f32, f32)> {
        self.entries
            .get(key)
            .map(|e| (e.layout_box.width, e.layout_box.height))
    }

    /// Dirty değilse ve layout box tam olarak aynıysa önceki DrawCommand'ları
    /// döner (paint() tekrar çağrılmadan). Aksi durumda `None`.
    pub fn try_reuse(
        &self,
        key: &str,
        layout_box: LayoutBox,
        dirty: bool,
    ) -> Option<&[DrawCommand]> {
        if dirty {
            return None;
        }
        self.entries
            .get(key)
            .and_then(|entry| (entry.layout_box == layout_box).then_some(entry.commands.as_slice()))
    }

    /// Bir node yeniden paint edildiğinde sonucu cache'e yazar.
    pub fn store(&mut self, key: &str, layout_box: LayoutBox, commands: Vec<DrawCommand>) {
        self.entries.insert(
            key.to_string(),
            CachedEntry {
                layout_box,
                commands,
            },
        );
    }

    /// Bu frame'de artık ağaçta olmayan (kaldırılmış) node'ların cache
    /// girdilerini temizler; aksi halde uzun ömürlü uygulamalarda bellek sızar.
    pub fn retain_keys(&mut self, live_keys: &HashSet<&str>) {
        self.entries.retain(|k, _| live_keys.contains(k.as_str()));
    }
}
