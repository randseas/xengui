// SPDX-License-Identifier: Apache-2.0
#[derive(Clone, Copy, Debug)]
pub enum GridTrack {
    Px(f32),
    Fr(f32),
    Auto,
}

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
