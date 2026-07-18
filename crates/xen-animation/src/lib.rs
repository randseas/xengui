// SPDX-License-Identifier: Apache-2.0

//! A high-performance, framework-agnostic animation library for Rust.
//!
//! `xen-animation` centralizes the lifecycle of every animated value behind
//! a single [`AnimationManager`]: callers report the current target value
//! each frame, and the manager takes care of starting, easing, retargeting,
//! and finishing the transition on its own. It has no rendering or
//! windowing dependencies, so it can be reused across GUI frameworks, game
//! engines, or any other application that needs frame-based interpolation.

mod easing;
mod manager;
mod transition;
mod transition_property;
mod value;

pub use easing::{ CubicBezier, Easing };
pub use manager::AnimationManager;
pub use transition::{ Transition, TransitionOverrides };
pub use transition_property::TransitionProperty;
pub use value::AnimValue;
