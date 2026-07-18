// SPDX-License-Identifier: Apache-2.0
use std::time::Duration;
use super::Easing;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Transition {
    pub duration: Duration,
    pub delay: Duration,
    pub easing: Easing,
}

impl Transition {
    pub const fn new(duration: Duration) -> Self {
        Self { duration, delay: Duration::ZERO, easing: Easing::Linear }
    }

    pub const fn delay(mut self, delay: Duration) -> Self {
        self.delay = delay;
        self
    }

    pub const fn easing(mut self, easing: Easing) -> Self {
        self.easing = easing;
        self
    }
}

impl Default for Transition {
    fn default() -> Self {
        Self::new(Duration::from_millis(200))
    }
}

/// Per-group transition overrides layered on top of `Style::transition`.
/// Each field takes priority over the general transition for its own
/// property group, so `.transition_colors(...)` and `.transition_transform(...)`
/// can be chained together with different durations/easings.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TransitionOverrides {
    pub colors: Option<Transition>,
    pub opacity: Option<Transition>,
    pub shadow: Option<Transition>,
    pub transform: Option<Transition>,
    pub box_model: Option<Transition>,
}

impl TransitionOverrides {
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
