// SPDX-License-Identifier: Apache-2.0
use super::{DrawCommand, RectCommand, TextCommand};

pub struct PaintContext<'a> {
    commands: &'a mut Vec<DrawCommand>,
    pub debug: bool,
}

impl<'a> PaintContext<'a> {
    pub(crate) fn new(commands: &'a mut Vec<DrawCommand>, debug: bool) -> Self {
        Self { commands, debug }
    }

    pub fn draw_text(&mut self, command: TextCommand) {
        self.commands.push(DrawCommand::Text(Box::new(command)));
    }

    pub fn draw_rect(&mut self, command: RectCommand) {
        self.commands.push(DrawCommand::Rect(command));
    }
}
