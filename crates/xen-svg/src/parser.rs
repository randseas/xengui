// SPDX-License-Identifier: Apache-2.0
use super::{
    FillRule,
    LineCap,
    LineJoin,
    PathCommand,
    SvgAttributes,
    SvgColor,
    SvgDocument,
    SvgElement,
    Transform2D,
    parse_transform,
};
use crate::Color;
use std::collections::HashMap;

/// Parses a UTF-8 SVG document (`<svg>...</svg>`) into an [`SvgDocument`].
///
/// This is a small hand-written parser covering the subset of SVG this
/// widget renders (`path`, `rect`, `circle`, `line`, `g`, and their
/// presentation attributes) - not a general-purpose XML parser.
pub fn parse_svg(input: &str) -> Result<SvgDocument, String> {
    let tags = tokenize(input);

    let Some(root) = tags.iter().position(|t| t.name == "svg") else {
        return Err("no <svg> root element found".to_string());
    };

    let view_box = tags[root].attrs
        .get("viewBox")
        .and_then(|v| parse_view_box(v))
        .unwrap_or((0.0, 0.0, 24.0, 24.0));

    let root_attrs = build_attrs(&tags[root].attrs, &SvgAttributes::default());

    let mut cursor = root + 1;
    let elements = parse_children(&tags, &mut cursor, "svg", &root_attrs);

    Ok(SvgDocument { view_box, elements })
}

fn parse_view_box(value: &str) -> Option<(f32, f32, f32, f32)> {
    let nums: Vec<f32> = value
        .split([',', ' '])
        .filter(|s| !s.is_empty())
        .filter_map(|s| s.parse().ok())
        .collect();
    match nums.as_slice() {
        [minx, miny, width, height] => Some((*minx, *miny, *width, *height)),
        _ => None,
    }
}

enum TagKind {
    Open,
    Close,
    SelfClose,
}

struct Tag {
    name: String,
    kind: TagKind,
    attrs: HashMap<String, String>,
}

// Scans the raw text into a flat list of open/close/self-close tags,
// ignoring text content, comments, and the XML prolog - everything this
// widget renders lives in element attributes, not text nodes.
fn tokenize(input: &str) -> Vec<Tag> {
    let mut tags = Vec::new();
    let mut i = 0;

    while i < input.len() {
        let Some(rel) = input[i..].find('<') else {
            break;
        };
        i += rel;

        if input[i..].starts_with("<!--") {
            match input[i..].find("-->") {
                Some(end) => {
                    i += end + 3;
                }
                None => {
                    break;
                }
            }
            continue;
        }
        if input[i..].starts_with("<?") {
            match input[i..].find("?>") {
                Some(end) => {
                    i += end + 2;
                }
                None => {
                    break;
                }
            }
            continue;
        }

        let Some(close) = input[i..].find('>') else {
            break;
        };
        let raw = &input[i + 1..i + close];
        i += close + 1;

        if let Some(name) = raw.strip_prefix('/') {
            tags.push(Tag {
                name: name.trim().to_string(),
                kind: TagKind::Close,
                attrs: HashMap::new(),
            });
            continue;
        }

        let (body, self_closing) = match raw.trim_end().strip_suffix('/') {
            Some(b) => (b, true),
            None => (raw, false),
        };

        let mut parts = body.splitn(2, char::is_whitespace);
        let name = parts.next().unwrap_or("").trim().to_string();
        let attrs = parts.next().map(parse_attrs).unwrap_or_default();

        tags.push(Tag {
            name,
            kind: if self_closing {
                TagKind::SelfClose
            } else {
                TagKind::Open
            },
            attrs,
        });
    }

    tags
}

fn parse_attrs(input: &str) -> HashMap<String, String> {
    let mut attrs = HashMap::new();
    let mut chars = input.char_indices().peekable();

    while let Some(&(start, c)) = chars.peek() {
        if c.is_whitespace() {
            chars.next();
            continue;
        }

        let key_start = start;
        let mut key_end = start;
        while let Some(&(idx, c)) = chars.peek() {
            if c == '=' || c.is_whitespace() {
                break;
            }
            key_end = idx + c.len_utf8();
            chars.next();
        }
        let key = input[key_start..key_end].to_string();

        while matches!(chars.peek(), Some((_, c)) if c.is_whitespace()) {
            chars.next();
        }

        if matches!(chars.peek(), Some((_, '='))) {
            chars.next();
            while matches!(chars.peek(), Some((_, c)) if c.is_whitespace()) {
                chars.next();
            }
            if let Some(&(_, quote)) = chars.peek() && (quote == '"' || quote == '\'') {
                chars.next();
                let value_start = chars
                    .peek()
                    .map(|&(idx, _)| idx)
                    .unwrap_or(input.len());
                let mut value_end = value_start;
                while let Some(&(idx, c)) = chars.peek() {
                    if c == quote {
                        value_end = idx;
                        chars.next();
                        break;
                    }
                    value_end = idx + c.len_utf8();
                    chars.next();
                }
                if !key.is_empty() {
                    attrs.insert(key, input[value_start..value_end].to_string());
                }
                continue;
            }
        }

        if !key.is_empty() {
            attrs.insert(key, String::new());
        }
    }

    attrs
}

// Consumes tags from `cursor` until the closing tag for `parent_name`,
// recursing into `<g>` groups and turning every recognized element into
// an `SvgElement`. Unknown/text elements are skipped but stay balanced.
fn parse_children(
    tags: &[Tag],
    cursor: &mut usize,
    parent_name: &str,
    parent_attrs: &SvgAttributes
) -> Vec<SvgElement> {
    let mut elements = Vec::new();

    while *cursor < tags.len() {
        let tag = &tags[*cursor];

        match tag.kind {
            TagKind::Close => {
                if tag.name == parent_name {
                    *cursor += 1;
                }
                return elements;
            }
            TagKind::SelfClose => {
                if let Some(element) = build_element(tag, parent_attrs) {
                    elements.push(element);
                }
                *cursor += 1;
            }
            TagKind::Open => {
                let name = tag.name.clone();
                let attrs_source = tag.attrs.clone();

                if name == "g" {
                    let group_attrs = build_attrs(&attrs_source, parent_attrs);
                    *cursor += 1;
                    let children = parse_children(tags, cursor, "g", &group_attrs);
                    elements.push(SvgElement::Group {
                        children,
                        attrs: group_attrs,
                    });
                } else {
                    if let Some(element) = build_element(tag, parent_attrs) {
                        elements.push(element);
                    }
                    *cursor += 1;
                    skip_until_close(tags, cursor, &name);
                }
            }
        }
    }

    elements
}

fn skip_until_close(tags: &[Tag], cursor: &mut usize, name: &str) {
    let mut depth = 1;
    while *cursor < tags.len() && depth > 0 {
        match tags[*cursor].kind {
            TagKind::Open if tags[*cursor].name == name => {
                depth += 1;
            }
            TagKind::Close if tags[*cursor].name == name => {
                depth -= 1;
            }
            _ => {}
        }
        *cursor += 1;
    }
}

fn build_element(tag: &Tag, parent_attrs: &SvgAttributes) -> Option<SvgElement> {
    let attrs = build_attrs(&tag.attrs, parent_attrs);
    let get = |key: &str|
        tag.attrs
            .get(key)
            .and_then(|v| v.parse::<f32>().ok())
            .unwrap_or(0.0);

    match tag.name.as_str() {
        "path" => {
            let d = tag.attrs.get("d")?;
            Some(SvgElement::Path { commands: parse_path_data(d), attrs })
        }
        "rect" =>
            Some(SvgElement::Rect {
                x: get("x"),
                y: get("y"),
                width: get("width"),
                height: get("height"),
                rx: get("rx"),
                attrs,
            }),
        "circle" => Some(SvgElement::Circle { cx: get("cx"), cy: get("cy"), r: get("r"), attrs }),
        "line" =>
            Some(SvgElement::Line {
                x1: get("x1"),
                y1: get("y1"),
                x2: get("x2"),
                y2: get("y2"),
                attrs,
            }),
        _ => None,
    }
}

// Fill/stroke and their related properties (width, cap, join, miter limit,
// fill-rule) inherit from the nearest ancestor that set them, matching
// SVG/CSS's own inheritance rules for these presentation attributes.
fn build_attrs(source: &HashMap<String, String>, parent: &SvgAttributes) -> SvgAttributes {
    let mut attrs = SvgAttributes {
        fill: parent.fill,
        fill_rule: parent.fill_rule,
        stroke: parent.stroke,
        stroke_width: parent.stroke_width,
        line_cap: parent.line_cap,
        line_join: parent.line_join,
        miter_limit: parent.miter_limit,
        opacity: 1.0,
        transform: Transform2D::IDENTITY,
    };

    if let Some(v) = source.get("fill") {
        attrs.fill = parse_paint(v);
    }
    if let Some(v) = source.get("fill-rule") {
        attrs.fill_rule = parse_fill_rule(v);
    }
    if let Some(v) = source.get("stroke") {
        attrs.stroke = parse_paint(v);
    }
    if let Some(v) = source.get("stroke-width").and_then(|v| v.parse().ok()) {
        attrs.stroke_width = v;
    }
    if let Some(v) = source.get("stroke-linecap") {
        attrs.line_cap = parse_line_cap(v);
    }
    if let Some(v) = source.get("stroke-linejoin") {
        attrs.line_join = parse_line_join(v);
    }
    if let Some(v) = source.get("stroke-miterlimit").and_then(|v| v.parse().ok()) {
        attrs.miter_limit = v;
    }
    if let Some(v) = source.get("opacity").and_then(|v| v.parse().ok()) {
        attrs.opacity = v;
    }
    if let Some(v) = source.get("transform") {
        attrs.transform = parse_transform(v);
    }

    attrs
}

fn parse_line_cap(value: &str) -> LineCap {
    match value.trim() {
        "round" => LineCap::Round,
        "square" => LineCap::Square,
        _ => LineCap::Butt,
    }
}

fn parse_line_join(value: &str) -> LineJoin {
    match value.trim() {
        "round" => LineJoin::Round,
        "bevel" => LineJoin::Bevel,
        _ => LineJoin::Miter,
    }
}

fn parse_fill_rule(value: &str) -> FillRule {
    match value.trim() {
        "evenodd" => FillRule::EvenOdd,
        _ => FillRule::NonZero,
    }
}

fn parse_paint(value: &str) -> SvgColor {
    let value = value.trim();
    match value {
        "none" => SvgColor::None,
        "currentColor" => SvgColor::Current,
        _ => {
            if let Some(inner) = value.strip_prefix("rgba(").and_then(|s| s.strip_suffix(')')) {
                parse_rgb_like(inner, 1.0).into()
            } else if
                let Some(inner) = value.strip_prefix("rgb(").and_then(|s| s.strip_suffix(')'))
            {
                parse_rgb_like(inner, 1.0).into()
            } else {
                Color::hex(value).into()
            }
        }
    }
}

// Parses the inner contents of a CSS `rgb(...)`/`rgba(...)` function.
// Each channel may be a plain 0-255 integer or a percentage (e.g. "50%");
// a trailing alpha component overrides `default_alpha` when present.
fn parse_rgb_like(inner: &str, default_alpha: f32) -> Color {
    let channel = |s: &str| -> f32 {
        let s = s.trim();
        if let Some(pct) = s.strip_suffix('%') {
            pct.parse::<f32>().unwrap_or(0.0).clamp(0.0, 100.0) / 100.0
        } else {
            (s.parse::<f32>().unwrap_or(0.0) / 255.0).clamp(0.0, 1.0)
        }
    };

    #[allow(unused_parens)]
    let parts: Vec<&str> = inner
        .split(|c: char| (c == ',' || c.is_whitespace()))
        .filter(|s| !s.is_empty())
        .collect();

    match parts.as_slice() {
        [r, g, b] => Color::rgba_f32(channel(r), channel(g), channel(b), default_alpha),
        [r, g, b, a] =>
            Color::rgba_f32(
                channel(r),
                channel(g),
                channel(b),
                a.trim().parse().unwrap_or(default_alpha)
            ),
        _ => {
            log::error!("xen-svg: malformed rgb()/rgba() value: 'rgb({inner})'");
            Color::BLACK
        }
    }
}

// Parses an SVG path `d` attribute into absolute-coordinate commands.
// Supports M/L/H/V/C/S/Q/T/A/Z (upper and lowercase) with implicit repeat
// of the last command for bare coordinate pairs, matching the SVG grammar.
// Arcs (A/a) follow the SVG endpoint-to-center parameterization (spec
// appendix F.6) and are emitted as cubic bezier segments.
fn parse_path_data(d: &str) -> Vec<PathCommand> {
    let mut commands = Vec::new();
    let mut tokens = PathTokenizer::new(d);

    let mut cursor = (0.0, 0.0);
    let mut subpath_start = (0.0, 0.0);
    let mut last_cubic_ctrl: Option<(f32, f32)> = None;
    let mut last_quad_ctrl: Option<(f32, f32)> = None;
    let mut current_op: Option<char> = None;

    while let Some(token) = tokens.next_command_or_number() {
        let op = match token {
            PathToken::Command(c) => {
                current_op = Some(c);
                c
            }
            PathToken::Number(_) => {
                tokens.push_back(token);
                let Some(op) = current_op else {
                    break;
                };
                op
            }
        };
        let relative = op.is_lowercase();

        let resolve = |relative: bool, x: f32, y: f32, from: (f32, f32)| {
            if relative { (from.0 + x, from.1 + y) } else { (x, y) }
        };

        match op.to_ascii_uppercase() {
            'M' => {
                let (x, y) = (tokens.number(), tokens.number());
                cursor = resolve(relative, x, y, cursor);
                subpath_start = cursor;
                commands.push(PathCommand::MoveTo(cursor.0, cursor.1));
                // Bare coordinate pairs after a MoveTo are implicit LineTos.
                current_op = Some(if relative { 'l' } else { 'L' });
                last_cubic_ctrl = None;
                last_quad_ctrl = None;
            }
            'L' => {
                let (x, y) = (tokens.number(), tokens.number());
                cursor = resolve(relative, x, y, cursor);
                commands.push(PathCommand::LineTo(cursor.0, cursor.1));
                last_cubic_ctrl = None;
                last_quad_ctrl = None;
            }
            'H' => {
                let x = tokens.number();
                cursor = (if relative { cursor.0 + x } else { x }, cursor.1);
                commands.push(PathCommand::LineTo(cursor.0, cursor.1));
                last_cubic_ctrl = None;
                last_quad_ctrl = None;
            }
            'V' => {
                let y = tokens.number();
                cursor = (cursor.0, if relative { cursor.1 + y } else { y });
                commands.push(PathCommand::LineTo(cursor.0, cursor.1));
                last_cubic_ctrl = None;
                last_quad_ctrl = None;
            }
            'C' => {
                let (x1, y1) = (tokens.number(), tokens.number());
                let (x2, y2) = (tokens.number(), tokens.number());
                let (x, y) = (tokens.number(), tokens.number());
                let c1 = resolve(relative, x1, y1, cursor);
                let c2 = resolve(relative, x2, y2, cursor);
                let end = resolve(relative, x, y, cursor);
                commands.push(PathCommand::CubicTo(c1.0, c1.1, c2.0, c2.1, end.0, end.1));
                last_cubic_ctrl = Some(c2);
                cursor = end;
            }
            'S' => {
                let (x2, y2) = (tokens.number(), tokens.number());
                let (x, y) = (tokens.number(), tokens.number());
                let c1 = last_cubic_ctrl
                    .map(|(cx, cy)| (2.0 * cursor.0 - cx, 2.0 * cursor.1 - cy))
                    .unwrap_or(cursor);
                let c2 = resolve(relative, x2, y2, cursor);
                let end = resolve(relative, x, y, cursor);
                commands.push(PathCommand::CubicTo(c1.0, c1.1, c2.0, c2.1, end.0, end.1));
                last_cubic_ctrl = Some(c2);
                cursor = end;
            }
            'Q' => {
                let (x1, y1) = (tokens.number(), tokens.number());
                let (x, y) = (tokens.number(), tokens.number());
                let c1 = resolve(relative, x1, y1, cursor);
                let end = resolve(relative, x, y, cursor);
                commands.push(PathCommand::QuadTo(c1.0, c1.1, end.0, end.1));
                last_quad_ctrl = Some(c1);
                cursor = end;
            }
            'T' => {
                let (x, y) = (tokens.number(), tokens.number());
                let c1 = last_quad_ctrl
                    .map(|(cx, cy)| (2.0 * cursor.0 - cx, 2.0 * cursor.1 - cy))
                    .unwrap_or(cursor);
                let end = resolve(relative, x, y, cursor);
                commands.push(PathCommand::QuadTo(c1.0, c1.1, end.0, end.1));
                last_quad_ctrl = Some(c1);
                cursor = end;
            }
            'A' => {
                let rx = tokens.number();
                let ry = tokens.number();
                let x_axis_rotation = tokens.number();
                let large_arc = tokens.flag();
                let sweep = tokens.flag();
                let (x, y) = (tokens.number(), tokens.number());
                let end = resolve(relative, x, y, cursor);
                push_arc(cursor, end, rx, ry, x_axis_rotation, large_arc, sweep, &mut commands);
                cursor = end;
                last_cubic_ctrl = None;
                last_quad_ctrl = None;
            }
            'Z' => {
                commands.push(PathCommand::Close);
                cursor = subpath_start;
            }
            _ => {
                break;
            }
        }
    }

    commands
}

// Converts an SVG elliptical arc segment into one or more cubic bezier
// curves, following the endpoint-to-center parameterization from the SVG
// 1.1 spec (appendix F.6), then approximating the resulting arc with a
// bezier segment per <=90 degrees of sweep - the same approach used by
// most browser and vector-graphics implementations.
#[allow(clippy::too_many_arguments)]
fn push_arc(
    from: (f32, f32),
    to: (f32, f32),
    mut rx: f32,
    mut ry: f32,
    x_axis_rotation_deg: f32,
    large_arc: bool,
    sweep: bool,
    out: &mut Vec<PathCommand>
) {
    // Coincident endpoints draw nothing at all (spec F.6.2).
    if (from.0 - to.0).abs() < f32::EPSILON && (from.1 - to.1).abs() < f32::EPSILON {
        return;
    }

    rx = rx.abs();
    ry = ry.abs();

    if rx < f32::EPSILON || ry < f32::EPSILON {
        out.push(PathCommand::LineTo(to.0, to.1));
        return;
    }

    let phi = x_axis_rotation_deg.to_radians();
    let (sin_phi, cos_phi) = phi.sin_cos();

    // Step 1: the endpoints in a frame centered on their midpoint and
    // un-rotated by the arc's own x-axis-rotation.
    let dx2 = (from.0 - to.0) * 0.5;
    let dy2 = (from.1 - to.1) * 0.5;
    let x1p = cos_phi * dx2 + sin_phi * dy2;
    let y1p = -sin_phi * dx2 + cos_phi * dy2;

    // Step 2: scale up radii that are too small to reach between the
    // endpoints at all (spec F.6.6).
    let lambda = (x1p * x1p) / (rx * rx) + (y1p * y1p) / (ry * ry);
    if lambda > 1.0 {
        let scale = lambda.sqrt();
        rx *= scale;
        ry *= scale;
    }

    // Step 3 & 4: solve for the ellipse center.
    let rx_sq = rx * rx;
    let ry_sq = ry * ry;
    let x1p_sq = x1p * x1p;
    let y1p_sq = y1p * y1p;

    let sign = if large_arc == sweep { -1.0 } else { 1.0 };
    let num = (rx_sq * ry_sq - rx_sq * y1p_sq - ry_sq * x1p_sq).max(0.0);
    let den = rx_sq * y1p_sq + ry_sq * x1p_sq;
    let co = if den > f32::EPSILON { sign * (num / den).sqrt() } else { 0.0 };

    let cxp = co * ((rx * y1p) / ry);
    let cyp = co * -((ry * x1p) / rx);

    let cx = cos_phi * cxp - sin_phi * cyp + (from.0 + to.0) * 0.5;
    let cy = sin_phi * cxp + cos_phi * cyp + (from.1 + to.1) * 0.5;

    // Step 5: the start angle and the total angle swept.
    let angle_between = |ux: f32, uy: f32, vx: f32, vy: f32| -> f32 {
        let dot = ux * vx + uy * vy;
        let len = ((ux * ux + uy * uy) * (vx * vx + vy * vy)).sqrt().max(f32::EPSILON);
        let mut a = (dot / len).clamp(-1.0, 1.0).acos();
        if ux * vy - uy * vx < 0.0 {
            a = -a;
        }
        a
    };

    let ux = (x1p - cxp) / rx;
    let uy = (y1p - cyp) / ry;
    let vx = (-x1p - cxp) / rx;
    let vy = (-y1p - cyp) / ry;

    let start_angle = angle_between(1.0, 0.0, ux, uy);
    let mut delta_angle = angle_between(ux, uy, vx, vy);

    if !sweep && delta_angle > 0.0 {
        delta_angle -= std::f32::consts::TAU;
    } else if sweep && delta_angle < 0.0 {
        delta_angle += std::f32::consts::TAU;
    }

    // Step 6: split into segments of at most 90 degrees and approximate
    // each with a cubic bezier using the standard circular-arc magic number.
    let segment_count = (delta_angle.abs() / std::f32::consts::FRAC_PI_2).ceil().max(1.0) as u32;
    let segment_angle = delta_angle / (segment_count as f32);
    let alpha = (segment_angle / 4.0).tan() * (4.0 / 3.0);

    let mut current_angle = start_angle;
    for i in 0..segment_count {
        let next_angle = current_angle + segment_angle;

        let (s0, c0) = current_angle.sin_cos();
        let (s1, c1) = next_angle.sin_cos();

        let p0 = (c0, s0);
        let p1 = (c1, s1);
        let t0 = (-s0, c0);
        let t1 = (-s1, c1);

        let ctrl1 = (p0.0 + alpha * t0.0, p0.1 + alpha * t0.1);
        let ctrl2 = (p1.0 - alpha * t1.0, p1.1 - alpha * t1.1);

        // Maps a point on the unrotated unit ellipse back into the final
        // rotated, translated coordinate space.
        let map = |p: (f32, f32)| -> (f32, f32) {
            let ex = p.0 * rx;
            let ey = p.1 * ry;
            (cx + cos_phi * ex - sin_phi * ey, cy + sin_phi * ex + cos_phi * ey)
        };

        let (c1x, c1y) = map(ctrl1);
        let (c2x, c2y) = map(ctrl2);
        // The very last segment snaps to the exact requested endpoint
        // instead of the ellipse-sampled point, avoiding float drift.
        let (ex, ey) = if i + 1 == segment_count { to } else { map(p1) };

        out.push(PathCommand::CubicTo(c1x, c1y, c2x, c2y, ex, ey));

        current_angle = next_angle;
    }
}

enum PathToken {
    Command(char),
    Number(f32),
}

// Splits path data into commands and numbers, understanding the SVG
// number grammar (signs, decimals, exponents).
struct PathTokenizer<'a> {
    chars: std::iter::Peekable<std::str::CharIndices<'a>>,
    source: &'a str,
    pushed_back: Option<PathToken>,
}

impl<'a> PathTokenizer<'a> {
    fn new(source: &'a str) -> Self {
        Self { chars: source.char_indices().peekable(), source, pushed_back: None }
    }

    fn push_back(&mut self, token: PathToken) {
        self.pushed_back = Some(token);
    }

    fn skip_separators(&mut self) {
        while matches!(self.chars.peek(), Some((_, c)) if c.is_whitespace() || *c == ',') {
            self.chars.next();
        }
    }

    fn next_command_or_number(&mut self) -> Option<PathToken> {
        if let Some(token) = self.pushed_back.take() {
            return Some(token);
        }
        self.skip_separators();
        let &(start, c) = self.chars.peek()?;

        if c.is_ascii_alphabetic() {
            self.chars.next();
            return Some(PathToken::Command(c));
        }

        self.read_number(start).map(PathToken::Number)
    }

    fn read_number(&mut self, start: usize) -> Option<f32> {
        let mut end = start;
        let mut seen_digit = false;
        let mut seen_dot = false;
        let mut seen_exp = false;

        if matches!(self.chars.peek(), Some((_, '+' | '-'))) {
            let (idx, c) = self.chars.next().unwrap();
            end = idx + c.len_utf8();
        }

        while let Some(&(idx, c)) = self.chars.peek() {
            match c {
                '0'..='9' => {
                    seen_digit = true;
                    end = idx + c.len_utf8();
                    self.chars.next();
                }
                '.' if !seen_dot && !seen_exp => {
                    seen_dot = true;
                    end = idx + c.len_utf8();
                    self.chars.next();
                }
                'e' | 'E' if !seen_exp && seen_digit => {
                    seen_exp = true;
                    end = idx + c.len_utf8();
                    self.chars.next();
                    if matches!(self.chars.peek(), Some((_, '+' | '-'))) {
                        let (idx, c) = self.chars.next().unwrap();
                        end = idx + c.len_utf8();
                    }
                }
                _ => {
                    break;
                }
            }
        }

        if !seen_digit {
            return None;
        }
        self.source[start..end].parse().ok()
    }

    // Reads the next number, defaulting to 0.0 on malformed/missing input
    // so a broken path fails soft instead of panicking.
    fn number(&mut self) -> f32 {
        loop {
            match self.next_command_or_number() {
                Some(PathToken::Number(n)) => {
                    return n;
                }
                Some(PathToken::Command(_)) => {
                    continue;
                }
                None => {
                    return 0.0;
                }
            }
        }
    }

    // Reads a single SVG arc flag (large-arc-flag / sweep-flag): exactly
    // one '0' or '1' character. Per spec these never need a separator, so
    // "11" after the x-axis-rotation legally means two flags of `1`, which
    // a generic number parser would misread as the single value 11.
    fn flag(&mut self) -> bool {
        if let Some(PathToken::Number(n)) = self.pushed_back.take() {
            return n != 0.0;
        }
        self.skip_separators();
        match self.chars.peek() {
            Some(&(_, '1')) => {
                self.chars.next();
                true
            }
            Some(&(_, '0')) => {
                self.chars.next();
                false
            }
            _ => false,
        }
    }
}
