// SPDX-License-Identifier: Apache-2.0
use crate::{ AnimationManager, TextPipeline };

pub struct LayoutContext<'a> {
    pub text: &'a mut TextPipeline,
    pub anim: &'a mut AnimationManager,
    pub scale_factor: f32,
}
