// SPDX-License-Identifier: Apache-2.0
use xengui::Cursor;

/// Converts xengui's platform-agnostic cursor kind into a winit cursor icon.
pub fn to_winit_cursor(cursor: Cursor) -> winit::window::CursorIcon {
    use winit::window::CursorIcon as W;
    match cursor {
        Cursor::Default => W::Default,
        Cursor::ContextMenu => W::ContextMenu,
        Cursor::Help => W::Help,
        Cursor::Pointer => W::Pointer,
        Cursor::Progress => W::Progress,
        Cursor::Wait => W::Wait,
        Cursor::Cell => W::Cell,
        Cursor::Crosshair => W::Crosshair,
        Cursor::Text => W::Text,
        Cursor::VerticalText => W::VerticalText,
        Cursor::Alias => W::Alias,
        Cursor::Copy => W::Copy,
        Cursor::Move => W::Move,
        Cursor::NoDrop => W::NoDrop,
        Cursor::NotAllowed => W::NotAllowed,
        Cursor::Grab => W::Grab,
        Cursor::Grabbing => W::Grabbing,
        Cursor::AllScroll => W::AllScroll,
        Cursor::ZoomIn => W::ZoomIn,
        Cursor::ZoomOut => W::ZoomOut,
        Cursor::EResize => W::EResize,
        Cursor::NResize => W::NResize,
        Cursor::NeResize => W::NeResize,
        Cursor::NwResize => W::NwResize,
        Cursor::SResize => W::SResize,
        Cursor::SeResize => W::SeResize,
        Cursor::SwResize => W::SwResize,
        Cursor::WResize => W::WResize,
        Cursor::EwResize => W::EwResize,
        Cursor::NsResize => W::NsResize,
        Cursor::NeswResize => W::NeswResize,
        Cursor::NwseResize => W::NwseResize,
        Cursor::ColResize => W::ColResize,
        Cursor::RowResize => W::RowResize,
    }
}
