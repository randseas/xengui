// SPDX-License-Identifier: Apache-2.0
use crate::TextPipeline;

pub struct LayoutContext<'a> {
    pub text: &'a mut TextPipeline,
    pub scale_factor: f32,
}
