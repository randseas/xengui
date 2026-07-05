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
// xengui/src/lib.rs
pub mod app;
pub mod components;
pub mod core;
pub mod renderer;

pub use app::App;
pub use app::AppConfig;
pub use app::WindowPosition;
pub use components::debug_text::DebugText;
pub use components::text::Text;
pub use components::text::TextProps;
pub use core::VNode;
pub use renderer::XenRenderer;
