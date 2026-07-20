// SPDX-License-Identifier: Apache-2.0
use crate::{
    Color,
    ImageCommand,
    RectCommand,
    SystemTheme,
    TextCommand,
    TextMeasurer,
    TriangleCommand,
};

/// Abstracts the GPU backend so xengui's core (layout, widgets,
/// reconciler, `FrameRenderer`) never depends on a concrete graphics API.
/// Implemented by `xengui-wgpu`; any other host (e.g. a Bevy render node)
/// can implement it too.
pub trait RenderBackend {
    fn text_measurer(&mut self) -> &mut dyn TextMeasurer;

    /// Prepares a new frame. Returning `false` skips the frame entirely
    /// (e.g. a native swapchain temporarily unavailable).
    fn begin_frame(&mut self, background: Color, width: u32, height: u32) -> bool;

    fn draw_rects(&mut self, cmds: &[RectCommand]);
    fn draw_triangles(&mut self, cmds: &[TriangleCommand]);
    fn draw_images(&mut self, cmds: &[ImageCommand]);
    fn draw_text(&mut self, theme: SystemTheme, scale_factor: f32, cmd: &TextCommand);

    /// Drains underline/strike/overline rects queued by `draw_text` calls
    /// since the last call to this method.
    fn take_text_decorations(&mut self) -> Vec<RectCommand>;

    /// Flushes queued text to the GPU. Must be called after every
    /// `draw_text` and before anything meant to render above text
    /// (e.g. a focus ring).
    fn flush_text(&mut self);

    /// Submits/presents the frame prepared by `begin_frame`.
    fn end_frame(&mut self);

    fn resize(&mut self, width: u32, height: u32);
}
