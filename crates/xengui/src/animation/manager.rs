// SPDX-License-Identifier: Apache-2.0
use super::{ AnimValue, Transition };
use crate::WidgetId;
use std::collections::HashMap;
use std::time::Duration;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum AnimLayer {
    #[default]
    Root,
    Background,
    Content,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AnimProperty {
    BackgroundColor,
    TextColor,
    BorderColor,
    Opacity,
    Scale,
    ShadowColor,
}

/// Identifies one animatable value on one widget's layer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct AnimKey {
    pub widget: WidgetId,
    pub layer: AnimLayer,
    pub property: AnimProperty,
}

struct Anim {
    from: AnimValue,
    to: AnimValue,
    transition: Transition,
    elapsed: Duration,
}

impl Anim {
    fn value_at(&self) -> AnimValue {
        let past_delay = self.elapsed.saturating_sub(self.transition.delay);
        let t = if self.transition.duration.is_zero() {
            1.0
        } else {
            past_delay.as_secs_f32() / self.transition.duration.as_secs_f32()
        };
        self.from.lerp(self.to, self.transition.easing.apply(t))
    }

    fn finished(&self) -> bool {
        self.elapsed >= self.transition.delay + self.transition.duration
    }
}

/// Central, Qt-style animation driver. Widgets never own a timer; they
/// only report their current target value and the manager owns the
/// entire lifecycle (starting, easing, retargeting, finishing).
#[derive(Default)]
pub struct AnimationManager {
    active: HashMap<AnimKey, Anim>,
}

impl AnimationManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Called once per widget per frame during `cascade_style`, before
    /// layout/paint. Starts or retargets a transition towards `target`;
    /// passing `None` for `transition` snaps instantly and clears state.
    pub fn set_target(&mut self, key: AnimKey, target: AnimValue, transition: Option<Transition>) {
        let Some(transition) = transition else {
            self.active.remove(&key);
            return;
        };

        match self.active.get_mut(&key) {
            Some(anim) if anim.to == target => {}
            Some(anim) => {
                anim.from = anim.value_at();
                anim.to = target;
                anim.transition = transition;
                anim.elapsed = Duration::ZERO;
            }
            None => {
                // Already resting on `target`; nothing to animate from yet.
                self.active.insert(key, Anim {
                    from: target,
                    to: target,
                    transition,
                    elapsed: transition.delay + transition.duration,
                });
            }
        }
    }

    /// Advances every active transition by one frame's delta time. Call
    /// exactly once per frame, before layout/paint.
    pub fn tick(&mut self, dt: Duration) {
        for anim in self.active.values_mut() {
            anim.elapsed += dt;
        }
        self.active.retain(|_, anim| !anim.finished());
    }

    /// Current (possibly mid-transition) value for `key`, or `None` once
    /// the transition has settled - callers should fall back to their
    /// own resolved target value in that case.
    pub fn value(&self, key: AnimKey) -> Option<AnimValue> {
        self.active.get(&key).map(Anim::value_at)
    }

    pub fn is_animating(&self) -> bool {
        !self.active.is_empty()
    }
}
