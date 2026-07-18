// SPDX-License-Identifier: Apache-2.0
use super::{ AnimValue, Transition };
use std::collections::HashMap;
use std::hash::Hash;
use std::time::Duration;

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

/// Central, Qt-style animation driver, generic over any hashable key type
/// so it can be reused outside of any specific GUI framework. Callers never
/// own a timer themselves; they only report their current target value and
/// the manager owns the entire lifecycle (starting, easing, retargeting,
/// finishing).
pub struct AnimationManager<K: Eq + Hash + Copy> {
    active: HashMap<K, Anim>,
    // Last settled value per key, kept around after an animation finishes
    // and is dropped from `active` - without this, the next retarget would
    // have no baseline to interpolate from and would snap instantly.
    resting: HashMap<K, AnimValue>,
}

impl<K: Eq + Hash + Copy> Default for AnimationManager<K> {
    fn default() -> Self {
        Self { active: HashMap::new(), resting: HashMap::new() }
    }
}

impl<K: Eq + Hash + Copy> AnimationManager<K> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Starts or retargets a transition towards `target`; passing `None`
    /// for `transition` snaps instantly and clears state.
    pub fn set_target(&mut self, key: K, target: AnimValue, transition: Option<Transition>) {
        let Some(transition) = transition else {
            self.active.remove(&key);
            self.resting.insert(key, target);
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
                let from = self.resting.get(&key).copied().unwrap_or(target);
                self.resting.insert(key, target);

                if from == target {
                    return;
                }

                self.active.insert(key, Anim {
                    from,
                    to: target,
                    transition,
                    elapsed: Duration::ZERO,
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

        for (key, anim) in self.active.iter() {
            if anim.finished() {
                self.resting.insert(*key, anim.to);
            }
        }

        self.active.retain(|_, anim| !anim.finished());
    }

    /// Current (possibly mid-transition) value for `key`, or `None` once
    /// the transition has settled - callers should fall back to their
    /// own resolved target value in that case.
    pub fn value(&self, key: K) -> Option<AnimValue> {
        self.active.get(&key).map(Anim::value_at)
    }

    pub fn is_animating(&self) -> bool {
        !self.active.is_empty()
    }
}
