// SPDX-License-Identifier: Apache-2.0
use crate::{Background, Color, Length, Style};
use smol_str::SmolStr;

#[derive(Clone, Debug)]
pub struct RectCommand {
    pub position: (f32, f32),
    pub size: (f32, f32),
    pub background: Option<Background>,
    pub border_radius: Option<Length>,
    pub border_width: Option<Length>,
    pub border_color: Option<Color>,
}

#[derive(Clone, Debug)]
pub struct TextCommand {
    pub text: SmolStr,
    pub position: (f32, f32),
    pub style: Style,
    pub font: Option<SmolStr>,
}

#[derive(Clone, Debug)]
pub enum DrawCommand {
    Rect(RectCommand),
    Text(Box<TextCommand>),
}
