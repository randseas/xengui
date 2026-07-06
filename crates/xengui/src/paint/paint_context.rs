// SPDX-License-Identifier: Apache-2.0
use super::{DrawCommand, TextCommand};

pub struct PaintContext<'a> {
    commands: &'a mut Vec<DrawCommand>,
}

impl<'a> PaintContext<'a> {
    pub(crate) fn new(commands: &'a mut Vec<DrawCommand>) -> Self {
        Self { commands }
    }

    pub fn draw_text(&mut self, command: TextCommand) {
        self.commands.push(DrawCommand::Text(command));
    }
}
