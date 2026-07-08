// SPDX-License-Identifier: Apache-2.0
use crate::TextPipeline;

pub struct LayoutContext<'a> {
    pub text: &'a TextPipeline,
    pub scale_factor: f32,
    pub debug: bool,
}
