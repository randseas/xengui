// SPDX-License-Identifier: Apache-2.0
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Default)]
pub struct Length(f32);

impl Length {
    /// Creates the length in pixels.
    pub const fn pixels(px: f32) -> Self {
        Self(px)
    }

    /// Returns the value in pixels.
    pub const fn px(&self) -> f32 {
        self.0
    }
}

impl From<f32> for Length {
    fn from(value: f32) -> Self {
        Self(value)
    }
}

impl From<f64> for Length {
    fn from(value: f64) -> Self {
        Self(value as f32)
    }
}

impl From<u8> for Length {
    fn from(value: u8) -> Self {
        Self(value as f32)
    }
}

impl From<u16> for Length {
    fn from(value: u16) -> Self {
        Self(value as f32)
    }
}

impl From<u32> for Length {
    fn from(value: u32) -> Self {
        Self(value as f32)
    }
}

impl From<usize> for Length {
    fn from(value: usize) -> Self {
        Self(value as f32)
    }
}

impl From<i8> for Length {
    fn from(value: i8) -> Self {
        Self(value as f32)
    }
}

impl From<i16> for Length {
    fn from(value: i16) -> Self {
        Self(value as f32)
    }
}

impl From<i32> for Length {
    fn from(value: i32) -> Self {
        Self(value as f32)
    }
}

impl From<isize> for Length {
    fn from(value: isize) -> Self {
        Self(value as f32)
    }
}
