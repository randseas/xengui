// SPDX-License-Identifier: Apache-2.0
use super::{ DrawCommand, ImageCommand, RectCommand, TextCommand, TriangleCommand };

pub struct PaintContext<'a> {
    commands: &'a mut Vec<DrawCommand>,
    pub scale_factor: f32,
}

impl<'a> PaintContext<'a> {
    pub(crate) fn new(commands: &'a mut Vec<DrawCommand>, scale_factor: f32) -> Self {
        Self { commands, scale_factor }
    }

    pub fn draw_text(&mut self, command: TextCommand) {
        self.commands.push(DrawCommand::Text(Box::new(command)));
    }

    pub fn draw_rect(&mut self, command: RectCommand) {
        self.commands.push(DrawCommand::Rect(command));
    }

    pub fn draw_triangle(&mut self, command: TriangleCommand) {
        self.commands.push(DrawCommand::Triangle(command));
    }

    pub fn draw_image(&mut self, command: ImageCommand) {
        self.commands.push(DrawCommand::Image(Box::new(command)));
    }
}
