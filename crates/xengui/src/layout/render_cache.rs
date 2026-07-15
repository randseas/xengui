// SPDX-License-Identifier: Apache-2.0
use crate::{ DrawCommand, LayoutBox, MeasureResult };
use std::collections::{ HashMap, HashSet };

struct CachedEntry {
    layout_box: LayoutBox,
    commands: Vec<DrawCommand>,
}

#[derive(Default)]
pub struct RenderCache {
    entries: HashMap<String, CachedEntry>,
    measured: HashMap<String, MeasureResult>,
}

impl RenderCache {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn cached_size(&self, key: &str) -> Option<(f32, f32)> {
        self.entries.get(key).map(|e| (e.layout_box.width, e.layout_box.height))
    }

    pub fn try_reuse(
        &self,
        key: &str,
        layout_box: LayoutBox,
        dirty: bool
    ) -> Option<&[DrawCommand]> {
        if dirty {
            return None;
        }
        self.entries
            .get(key)
            .and_then(|entry| (entry.layout_box == layout_box).then_some(entry.commands.as_slice()))
    }

    pub fn store(&mut self, key: &str, layout_box: LayoutBox, commands: Vec<DrawCommand>) {
        self.entries.insert(key.to_string(), CachedEntry {
            layout_box,
            commands,
        });
    }

    pub fn cached_measure(&self, key: &str) -> Option<MeasureResult> {
        self.measured.get(key).copied()
    }

    pub fn store_measure(&mut self, key: &str, size: MeasureResult) {
        self.measured.insert(key.to_string(), size);
    }

    pub fn retain_keys(&mut self, live_keys: &HashSet<String>) {
        self.entries.retain(|k, _| live_keys.contains(k));
        self.measured.retain(|k, _| live_keys.contains(k));
    }
}
