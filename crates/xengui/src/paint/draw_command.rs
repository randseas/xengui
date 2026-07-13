// SPDX-License-Identifier: Apache-2.0
use crate::{ Background, Color, Length, Style };
use smol_str::SmolStr;
use std::sync::Arc;

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
pub struct ImageData {
    pub id: u64,
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

#[derive(Clone, Debug)]
pub struct ImageCommand {
    pub position: (f32, f32),
    pub size: (f32, f32),
    pub image: Arc<ImageData>,
    pub border_radius: Option<Length>,
    pub tint: Option<Color>,
}

#[derive(Clone, Debug)]
pub enum DrawCommand {
    Rect(RectCommand),
    Text(Box<TextCommand>),
    Image(Box<ImageCommand>),
}
