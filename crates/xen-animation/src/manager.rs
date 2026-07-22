// SPDX-License-Identifier: Apache-2.0
use super::{ AnimValue, Transition };
use std::collections::HashMap;
use std::hash::Hash;
use std::time::Duration;

// A single in-flight transition: where it started from, where it's headed,
// the timing function driving it, and how much time has elapsed since it began.
struct Anim {
    from: AnimValue,
    to: AnimValue,
    transition: Transition,
    elapsed: Duration,
    premultiplied: bool,
}

impl Anim {
    // Current interpolated value given elapsed time, transition delay,
    // duration, and easing curve.
    fn value_at(&self) -> AnimValue {
        let past_delay = self.elapsed.saturating_sub(self.transition.delay);
        let t = if self.transition.duration.is_zero() {
            1.0
        } else {
            past_delay.as_secs_f32() / self.transition.duration.as_secs_f32()
        };
        let eased = self.transition.easing.apply(t);
        if self.premultiplied {
            self.from.lerp_premultiplied(self.to, eased)
        } else {
            self.from.lerp(self.to, eased)
        }
    }

    // Whether delay + duration has fully elapsed.
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
    /// Creates an empty manager with no active or resting animations.
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_target(&mut self, key: K, target: AnimValue, transition: Option<Transition>) {
        self.set_target_impl(key, target, transition, false);
    }

    /// Like `set_target`, but blends the transition using premultiplied
    /// alpha - use this when `target` packs an RGBA color, so a
    /// transparent endpoint's meaningless RGB doesn't leak into the blend
    /// as a visible flash partway through the fade.
    pub fn set_color_target(&mut self, key: K, target: AnimValue, transition: Option<Transition>) {
        self.set_target_impl(key, target, transition, true);
    }

    fn set_target_impl(
        &mut self,
        key: K,
        target: AnimValue,
        transition: Option<Transition>,
        premultiplied: bool
    ) {
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
                anim.premultiplied = premultiplied;
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
                    premultiplied,
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

    /// Iterates the keys of every animation currently mid-transition,
    /// letting callers check *what* is animating instead of only *whether*
    /// anything is - e.g. to skip layout work for animations that only
    /// affect paint (colors, opacity) and not the box model.
    pub fn active_keys(&self) -> impl Iterator<Item = &K> {
        self.active.keys()
    }

    /// Whether any animation is currently in flight across all keys. Useful
    /// to decide whether the render loop needs to keep polling for frames.
    pub fn is_animating(&self) -> bool {
        !self.active.is_empty()
    }
}
