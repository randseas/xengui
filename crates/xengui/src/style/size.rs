// SPDX-License-Identifier: Apache-2.0
use super::Length;

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Size {
    pub width: Length,
    pub height: Length,
}

impl Size {
    pub const fn new(width: Length, height: Length) -> Self {
        Self { width, height }
    }
}
