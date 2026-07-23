// SPDX-License-Identifier: Apache-2.0
use crate::{ current_path, match_route, RouteParams };
use smol_str::SmolStr;
use xengui::{ component, Widget };

type RouteBuilder = Box<dyn Fn(&RouteParams) -> Box<dyn Widget>>;

/// Picks one child widget tree to render based on the current route path
/// (see [`crate::navigate`]), re-evaluated every time the app's render
/// closure runs.
///
/// Routes are checked in the order they were added; the first matching
/// pattern wins. Each matched branch is wrapped in its own
/// `xengui::component`, keyed by the pattern, so `use_state` inside a
/// route builder keeps its identity across re-renders of that same route.
pub struct Router {
    routes: Vec<(SmolStr, RouteBuilder)>,
    not_found: Option<RouteBuilder>,
}

impl Router {
    pub fn new() -> Self {
        Self { routes: Vec::new(), not_found: None }
    }

    /// Registers a route. `pattern` supports `:param` segments and a
    /// trailing `*rest` wildcard segment - see [`crate::match_route`].
    pub fn route(
        mut self,
        pattern: impl Into<SmolStr>,
        builder: impl (Fn(&RouteParams) -> Box<dyn Widget>) + 'static
    ) -> Self {
        self.routes.push((pattern.into(), Box::new(builder)));
        self
    }

    /// Widget rendered when no route matches. Falls back to an empty
    /// `View` when not set.
    pub fn not_found(mut self, builder: impl (Fn() -> Box<dyn Widget>) + 'static) -> Self {
        self.not_found = Some(Box::new(move |_params| builder()));
        self
    }

    /// Resolves the current path against the registered routes and
    /// builds the matched widget tree. Call this inside `App::render`'s
    /// closure.
    pub fn build(self) -> Box<dyn Widget> {
        let path = current_path();

        for (pattern, builder) in &self.routes {
            if let Some(params) = match_route(pattern, &path) {
                return component(pattern.clone(), || builder(&params));
            }
        }

        match &self.not_found {
            Some(builder) =>
                component("xen_router::not_found", || builder(&RouteParams::default())),
            None =>
                component("xen_router::empty", || Box::new(xengui::View::new()) as Box<dyn Widget>),
        }
    }
}

impl Default for Router {
    fn default() -> Self {
        Self::new()
    }
}
