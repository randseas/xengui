/*
 * Copyright (C) 2026 randseas
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program. If not, see <https://www.gnu.org/licenses/>.
 */
// xengui/src/core.rs
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
