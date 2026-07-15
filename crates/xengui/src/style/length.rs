// SPDX-License-Identifier: Apache-2.0

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Length {
    Px(f32),
    Percent(f32),
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

    pub const fn value(&self) -> f32 {
        match self {
            Self::Px(v) => *v,
            Self::Percent(v) => *v,
        }
    }

    pub fn to_physical(self, scale_factor: f32) -> f32 {
        match self {
            Self::Px(v) => v * scale_factor,
            Self::Percent(v) => v,
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
