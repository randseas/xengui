// SPDX-License-Identifier: Apache-2.0
mod key;
mod style_animator;

pub use key::{ AnimKey, AnimLayer, AnimProperty };
pub use style_animator::animate_computed_style;

pub use xen_animation::{
    AnimValue,
    CubicBezier,
    Easing,
    Transition,
    TransitionOverrides,
    TransitionProperty,
};

/// Concrete animation driver used throughout xengui, keyed by [`AnimKey`].
pub type AnimationManager = xen_animation::AnimationManager<AnimKey>;
