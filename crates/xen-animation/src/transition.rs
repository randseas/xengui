// SPDX-License-Identifier: Apache-2.0
use std::time::Duration;
use super::Easing;

/// Describes how a single animated value should move from its current
/// value to a new target: how long it takes, how long to wait before
/// starting, and the easing curve applied over that duration.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Transition {
    /// How long the transition takes once it starts, excluding `delay`.
    pub duration: Duration,
    /// How long to wait after retargeting before the transition begins.
    pub delay: Duration,
    /// The timing curve applied to elapsed progress within `duration`.
    pub easing: Easing,
}

impl Transition {
    /// Creates a transition with the given duration, no delay, and linear easing.
    pub const fn new(duration: Duration) -> Self {
        Self { duration, delay: Duration::ZERO, easing: Easing::Linear }
    }

    /// Returns a copy of this transition with `delay` set.
    pub const fn delay(mut self, delay: Duration) -> Self {
        self.delay = delay;
        self
    }

    /// Returns a copy of this transition with `easing` set.
    pub const fn easing(mut self, easing: Easing) -> Self {
        self.easing = easing;
        self
    }
}

impl Default for Transition {
    /// 200ms, no delay, linear easing.
    fn default() -> Self {
        Self::new(Duration::from_millis(200))
    }
}

/// Per-group transition overrides layered on top of a base `Transition`.
/// Each field takes priority over the general transition for its own
/// property group, so e.g. colors and transforms can animate with
/// different durations/easings while sharing one base transition.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TransitionOverrides {
    /// Override applied to color-valued properties.
    pub colors: Option<Transition>,
    /// Override applied to opacity.
    pub opacity: Option<Transition>,
    /// Override applied to box-shadow (reserved for future use).
    pub shadow: Option<Transition>,
    /// Override applied to transform-like properties (scale, etc).
    pub transform: Option<Transition>,
    /// Override applied to box-model properties (size, padding, margin, etc).
    pub box_model: Option<Transition>,
}

impl TransitionOverrides {
    /// Merges `patch` on top of `self`, field by field - each field in
    /// `patch` takes precedence when set, otherwise `self`'s value is kept.
    pub fn overlay(&self, patch: &Self) -> Self {
        Self {
            colors: patch.colors.or(self.colors),
            opacity: patch.opacity.or(self.opacity),
            shadow: patch.shadow.or(self.shadow),
            transform: patch.transform.or(self.transform),
            box_model: patch.box_model.or(self.box_model),
        }
    }
}
