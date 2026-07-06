// SPDX-License-Identifier: Apache-2.0

use smol_str::SmolStr;

use crate::Style;

#[derive(Clone, Debug)]
pub struct TextCommand {
    pub text: SmolStr,
    pub position: (f32, f32),
    pub style: Style,
    pub font: Option<SmolStr>,
}

#[derive(Clone, Debug)]
pub enum DrawCommand {
    Text(TextCommand),
}
