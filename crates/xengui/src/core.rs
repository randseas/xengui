// SPDX-License-Identifier: Apache-2.0
// crates/xengui/src/core.rs
pub trait VNode {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
    fn key(&self) -> &str;
    fn is_dirty(&self) -> bool;
    fn set_dirty(&mut self, dirty: bool);
    fn render(
        &mut self,
        _render_pass: &mut wgpu::RenderPass,
        glyph_brush: &mut wgpu_glyph::GlyphBrush<()>,
        theme: &Option<winit::window::Theme>,
        _debug_mode: &bool,
    );
}
