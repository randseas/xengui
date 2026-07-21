// SPDX-License-Identifier: Apache-2.0
use crate::{ Interaction, Style };
use smol_str::SmolStr;

pub struct WidgetBase {
    pub key: Option<SmolStr>,
    pub dirty: bool,
    pub style: Style,
    pub inherited_style: Style,
    pub computed_style: Style,

    pub hover_style: Option<Style>,
    pub pressed_style: Option<Style>,
    pub disabled_style: Option<Style>,
    pub focus_style: Option<Style>,

    pub interaction: Interaction,
}

impl WidgetBase {
    pub fn new(interaction: Interaction) -> Self {
        Self {
            key: None,
            dirty: true,
            style: Style::default(),
            inherited_style: Style::default(),
            computed_style: Style::default(),
            hover_style: None,
            pressed_style: None,
            disabled_style: None,
            focus_style: None,
            interaction,
        }
    }

    pub fn recompute_style(&mut self) {
        let patch = if !self.interaction.enabled {
            self.disabled_style.as_ref()
        } else if self.interaction.pressed {
            self.pressed_style.as_ref().or(self.hover_style.as_ref())
        } else if self.interaction.focused {
            self.focus_style.as_ref()
        } else if self.interaction.hovered {
            self.hover_style.as_ref()
        } else {
            None
        };

        let base = self.inherited_style.inherit_style(&self.style);
        self.computed_style = match patch {
            Some(patch) => base.overlay(patch),
            None => base,
        };
    }

    pub fn mark_dirty(&mut self) {
        self.dirty = true;
        self.recompute_style();
    }
}
