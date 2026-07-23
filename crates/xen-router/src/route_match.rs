// SPDX-License-Identifier: Apache-2.0
use std::collections::HashMap;

/// Named parameters captured from a matched route pattern (e.g. the `id`
/// in `/users/:id`), plus any trailing wildcard capture (`*rest`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RouteParams(HashMap<String, String>);

impl RouteParams {
    pub fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).map(String::as_str)
    }
}

/// Matches `path` against `pattern`, returning the captured params on
/// success.
///
/// Pattern segments:
/// - a literal segment (`users`) must match exactly
/// - `:name` captures a single segment under `name`
/// - a trailing `*name` captures the remainder of the path (including
///   slashes) under `name`, and must be the pattern's last segment
pub fn match_route(pattern: &str, path: &str) -> Option<RouteParams> {
    let pattern_segments: Vec<&str> = pattern.trim_matches('/').split('/').collect();
    let path_segments: Vec<&str> = path.trim_matches('/').split('/').collect();

    let mut params = HashMap::new();

    for (i, seg) in pattern_segments.iter().enumerate() {
        if let Some(name) = seg.strip_prefix('*') {
            let rest = path_segments
                .get(i..)
                .unwrap_or(&[])
                .join("/");
            params.insert(name.to_string(), rest);
            return Some(RouteParams(params));
        }

        let actual = path_segments.get(i)?;

        if let Some(name) = seg.strip_prefix(':') {
            params.insert(name.to_string(), (*actual).to_string());
        } else if *seg != *actual {
            return None;
        }
    }

    if path_segments.len() != pattern_segments.len() {
        return None;
    }

    Some(RouteParams(params))
}
