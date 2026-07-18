// SPDX-License-Identifier: Apache-2.0

/// Which groups of style properties `Style::transition` should animate.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct TransitionProperty(u8);

impl TransitionProperty {
    pub const NONE: Self = Self(0);
    pub const COLORS: Self = Self(1 << 0);
    pub const OPACITY: Self = Self(1 << 1);
    // Reserved for the future box-shadow system.
    pub const SHADOW: Self = Self(1 << 2);
    pub const TRANSFORM: Self = Self(1 << 3);
    // Box-model properties: size, padding, margin, gap, border width/radius.
    // Excluded from the default `transition` group, only included via
    // `transition_all` - matches Tailwind's own default property list.
    pub const BOX: Self = Self(1 << 4);

    pub const DEFAULT: Self = Self(
        Self::COLORS.0 | Self::OPACITY.0 | Self::SHADOW.0 | Self::TRANSFORM.0
    );
    pub const ALL: Self = Self(Self::DEFAULT.0 | Self::BOX.0);

    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    pub const fn is_none(self) -> bool {
        self.0 == 0
    }
}
