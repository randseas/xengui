// SPDX-License-Identifier: Apache-2.0

/// Which groups of style properties `Style::transition` should animate.
///
/// This is a bitset packed into a single `u8`, so combinations of groups
/// can be checked and combined cheaply via [`Self::contains`] and
/// [`Self::union`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct TransitionProperty(u8);

impl TransitionProperty {
    /// No property group animates.
    pub const NONE: Self = Self(0);
    /// Color-valued properties (text color, background, border color, etc).
    pub const COLORS: Self = Self(1 << 0);
    /// Opacity.
    pub const OPACITY: Self = Self(1 << 1);
    /// Box-shadow. Reserved for the future box-shadow system.
    pub const SHADOW: Self = Self(1 << 2);
    /// Transform-like properties (scale, etc).
    pub const TRANSFORM: Self = Self(1 << 3);
    /// Box-model properties: size, padding, margin, gap, border width/radius.
    /// Excluded from the default `transition` group, only included via
    /// `transition_all` - matches Tailwind's own default property list.
    pub const BOX: Self = Self(1 << 4);

    /// Every group except `BOX` - matches CSS's default `transition-property: all`
    /// behavior for the common groups most UIs animate.
    pub const DEFAULT: Self = Self(
        Self::COLORS.0 | Self::OPACITY.0 | Self::SHADOW.0 | Self::TRANSFORM.0
    );
    /// Every group, including `BOX`.
    pub const ALL: Self = Self(Self::DEFAULT.0 | Self::BOX.0);

    /// Returns true if every group set in `other` is also set in `self`.
    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    /// Returns the combination of both sets of groups.
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    /// Returns true if no group is set.
    pub const fn is_none(self) -> bool {
        self.0 == 0
    }
}
