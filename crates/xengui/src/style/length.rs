// SPDX-License-Identifier: Apache-2.0

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Length {
    /// Sabit uzunluk (mantıksal piksel).
    Px(f32),
    /// Üst elemanın ilgili boyutuna göre yüzde (0.0..=100.0).
    /// Sadece width/height, padding, margin, flex_basis, gap gibi
    /// layout alanlarında anlamlıdır; border width/radius'ta göz ardı
    /// edilip ham sayı piksel olarak kullanılır.
    Percent(f32),
}

impl Default for Length {
    fn default() -> Self {
        Self::Px(0.0)
    }
}

impl Length {
    /// Piksel cinsinden uzunluk oluşturur.
    pub const fn pixels(px: f32) -> Self {
        Self::Px(px)
    }

    /// Yüzde cinsinden uzunluk oluşturur (0.0..=100.0 aralığında).
    pub const fn percent(pct: f32) -> Self {
        Self::Percent(pct)
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

impl From<f32> for Length {
    fn from(value: f32) -> Self {
        Self::Px(value)
    }
}

impl From<f64> for Length {
    fn from(value: f64) -> Self {
        Self::Px(value as f32)
    }
}

impl From<u8> for Length {
    fn from(value: u8) -> Self {
        Self::Px(value as f32)
    }
}

impl From<u16> for Length {
    fn from(value: u16) -> Self {
        Self::Px(value as f32)
    }
}

impl From<u32> for Length {
    fn from(value: u32) -> Self {
        Self::Px(value as f32)
    }
}

impl From<usize> for Length {
    fn from(value: usize) -> Self {
        Self::Px(value as f32)
    }
}

impl From<i8> for Length {
    fn from(value: i8) -> Self {
        Self::Px(value as f32)
    }
}

impl From<i16> for Length {
    fn from(value: i16) -> Self {
        Self::Px(value as f32)
    }
}

impl From<i32> for Length {
    fn from(value: i32) -> Self {
        Self::Px(value as f32)
    }
}

impl From<isize> for Length {
    fn from(value: isize) -> Self {
        Self::Px(value as f32)
    }
}
