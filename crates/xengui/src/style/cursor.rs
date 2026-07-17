// SPDX-License-Identifier: Apache-2.0

// Mirrors winit::window::CursorIcon 1:1, kept as our own type so Style
// doesn't need to depend on winit's enum being exhaustive/stable.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Cursor {
    #[default]
    Default,
    ContextMenu,
    Help,
    Pointer,
    Progress,
    Wait,
    Cell,
    Crosshair,
    Text,
    VerticalText,
    Alias,
    Copy,
    Move,
    NoDrop,
    NotAllowed,
    Grab,
    Grabbing,
    AllScroll,
    ZoomIn,
    ZoomOut,
    EResize,
    NResize,
    NeResize,
    NwResize,
    SResize,
    SeResize,
    SwResize,
    WResize,
    EwResize,
    NsResize,
    NeswResize,
    NwseResize,
    ColResize,
    RowResize,
}

impl Cursor {
    pub const fn to_winit(self) -> winit::window::CursorIcon {
        use winit::window::CursorIcon as W;
        match self {
            Self::Default => W::Default,
            Self::ContextMenu => W::ContextMenu,
            Self::Help => W::Help,
            Self::Pointer => W::Pointer,
            Self::Progress => W::Progress,
            Self::Wait => W::Wait,
            Self::Cell => W::Cell,
            Self::Crosshair => W::Crosshair,
            Self::Text => W::Text,
            Self::VerticalText => W::VerticalText,
            Self::Alias => W::Alias,
            Self::Copy => W::Copy,
            Self::Move => W::Move,
            Self::NoDrop => W::NoDrop,
            Self::NotAllowed => W::NotAllowed,
            Self::Grab => W::Grab,
            Self::Grabbing => W::Grabbing,
            Self::AllScroll => W::AllScroll,
            Self::ZoomIn => W::ZoomIn,
            Self::ZoomOut => W::ZoomOut,
            Self::EResize => W::EResize,
            Self::NResize => W::NResize,
            Self::NeResize => W::NeResize,
            Self::NwResize => W::NwResize,
            Self::SResize => W::SResize,
            Self::SeResize => W::SeResize,
            Self::SwResize => W::SwResize,
            Self::WResize => W::WResize,
            Self::EwResize => W::EwResize,
            Self::NsResize => W::NsResize,
            Self::NeswResize => W::NeswResize,
            Self::NwseResize => W::NwseResize,
            Self::ColResize => W::ColResize,
            Self::RowResize => W::RowResize,
        }
    }
}
