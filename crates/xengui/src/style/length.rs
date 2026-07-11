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
    pub fn px<T: Into<f32>>(value: T) -> Self {
        Self::Px(value.into())
    }

    pub fn percent<T: Into<f32>>(value: T) -> Self {
        Self::Percent(value.into())
    }

    /// Ham sayısal değeri döner (Px için piksel, Percent için 0..100 arası sayı).
    pub const fn value(&self) -> f32 {
        match self {
            Self::Px(v) => *v,
            Self::Percent(v) => *v,
        }
    }

    /// `Px` için piksel * scale_factor döner. `Percent` bir üst elemana göre
    /// çözülmesi gerektiğinden burada scale_factor uygulanmadan ham yüzde
    /// değeri döner; asıl çözümleme taffy/layout aşamasında yapılır.
    pub fn to_physical(self, scale_factor: f32) -> f32 {
        match self {
            Self::Px(v) => v * scale_factor,
            Self::Percent(v) => v,
        }
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
