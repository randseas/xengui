// SPDX-License-Identifier: Apache-2.0
//! A lightweight client-side router for xengui.
//!
//! Route state lives in a single thread-local, the same pattern xengui
//! itself uses for the active theme and for `use_state`: navigating just
//! updates that state and asks xengui to redraw, and the app's `render`
//! closure re-evaluates `Router::build` against the new path on the next
//! frame.
//!
//! On `wasm32`, the current path is kept in sync with the browser's
//! address bar via the History API (`pushState`/`popstate`), so links
//! are shareable and the back/forward buttons work. On native targets
//! there is no real URL - navigation only changes in-memory state.

mod state;
mod route_match;
mod router;
mod link;

pub use state::{ current_path, navigate, replace };
pub use route_match::{ match_route, RouteParams };
pub use router::Router;
pub use link::router_link;