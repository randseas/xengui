// SPDX-License-Identifier: Apache-2.0
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct TextDecoration {
    underline: bool,
    strike: bool,
    overline: bool,
}

impl TextDecoration {
    pub const NONE: TextDecoration = TextDecoration {
        underline: false,
        strike: false,
        overline: false,
    };

    pub const UNDERLINE: TextDecoration = TextDecoration {
        underline: true,
        strike: false,
        overline: false,
    };

    pub fn underline(&self) -> bool {
        self.underline
    }

    pub fn strike(&self) -> bool {
        self.strike
    }

    pub fn overline(&self) -> bool {
        self.overline
    }

    pub fn with_underline(mut self, value: bool) -> Self {
        self.underline = value;
        self
    }

    pub fn with_strike(mut self, value: bool) -> Self {
        self.strike = value;
        self
    }

    pub fn with_overline(mut self, value: bool) -> Self {
        self.overline = value;
        self
    }
}
