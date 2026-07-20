// SPDX-License-Identifier: Apache-2.0
use crate::{ AnimationManager, TextMeasurer };

pub struct LayoutContext<'a> {
    pub text: &'a mut dyn TextMeasurer,
    pub anim: &'a mut AnimationManager,
    pub scale_factor: f32,
}
