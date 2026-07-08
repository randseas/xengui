// SPDX-License-Identifier: Apache-2.0

/// Taffy'nin tam grid track API'sinin sadeleştirilmiş bir alt kümesi:
/// sabit piksel, esnek (fr) ve içerik bazlı (auto) track'ler.
#[derive(Clone, Copy, Debug)]
pub enum GridTrack {
    Px(f32),
    Fr(f32),
    Auto,
}

/// CSS `grid-column`/`grid-row` ile aynı 1-index'li konvansiyon.
#[derive(Clone, Copy, Debug, Default)]
pub struct GridPlacement {
    pub start: i16,
    pub end: i16,
}

impl GridPlacement {
    pub const fn span(start: i16, end: i16) -> Self {
        Self { start, end }
    }
}
