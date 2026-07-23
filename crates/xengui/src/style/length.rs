// SPDX-License-Identifier: Apache-2.0
use std::cell::Cell;

thread_local! {
    static VIEWPORT_SIZE: Cell<(f32, f32)> = const { Cell::new((0.0, 0.0)) };
}

/// Updates the viewport size used to resolve `Length::ViewportWidth` and
/// `Length::ViewportHeight` values. Called once per layout pass.
pub fn set_viewport_size(width: f32, height: f32) {
    VIEWPORT_SIZE.with(|cell| cell.set((width, height)));
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Length {
    Px(f32),
    Percent(f32),
    ViewportWidth(f32),
    ViewportHeight(f32),
}

impl Default for Length {
    fn default() -> Self {
        Self::Px(0.0)
    }
}

impl Length {
    pub const fn px(value: f32) -> Self {
        Self::Px(value)
    }

    pub fn percent(value: f32) -> Self {
        Self::Percent(value)
    }

    pub const fn vw(value: f32) -> Self {
        Self::ViewportWidth(value)
    }

    pub const fn vh(value: f32) -> Self {
        Self::ViewportHeight(value)
    }

    pub const fn value(&self) -> f32 {
        match self {
            Self::Px(v) => *v,
            Self::Percent(v) => *v,
            Self::ViewportWidth(v) => *v,
            Self::ViewportHeight(v) => *v,
        }
    }

    pub fn to_physical(self, scale_factor: f32) -> f32 {
        match self {
            Self::Px(v) => v * scale_factor,
            Self::Percent(v) => v,
            Self::ViewportWidth(v) => {
                let (vw, _) = VIEWPORT_SIZE.with(Cell::get);
                vw * (v / 100.0)
            }
            Self::ViewportHeight(v) => {
                let (_, vh) = VIEWPORT_SIZE.with(Cell::get);
                vh * (v / 100.0)
            }
        }
    }

    pub fn add_px(self, value: f32) -> Self {
        Self::px(self.value() + value)
    }

    pub fn sub_px(self, value: f32) -> Self {
        Self::px((self.value() - value).max(0.0))
    }
}

macro_rules! impl_length_from {
    ($($t:ty),*) => {
        $(
            impl From<$t> for Length {
                fn from(value: $t) -> Self {
                    Self::Px(value as f32)
                }
            }
        )*
    };
}

impl_length_from!(u8, u16, u32, u64, usize, i8, i16, i32, i64, isize, f32, f64);
