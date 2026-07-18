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