// SPDX-License-Identifier: Apache-2.0
use super::{ DrawCommand, ImageCommand, RectCommand, TextCommand };

pub struct PaintContext<'a> {
    commands: &'a mut Vec<DrawCommand>,
}

impl<'a> PaintContext<'a> {
    pub(crate) fn new(commands: &'a mut Vec<DrawCommand>) -> Self {
        Self { commands }
    }

    pub fn draw_text(&mut self, command: TextCommand) {
        self.commands.push(DrawCommand::Text(Box::new(command)));
    }

    pub fn draw_rect(&mut self, command: RectCommand) {
        self.commands.push(DrawCommand::Rect(command));
    }

    pub fn draw_image(&mut self, command: ImageCommand) {
        self.commands.push(DrawCommand::Image(Box::new(command)));
    }
}
