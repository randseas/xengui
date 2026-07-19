// SPDX-License-Identifier: Apache-2.0
use super::{ PathCommand, SvgAttributes, SvgColor, SvgDocument, SvgElement, Transform2D };

/// Number of line segments each cubic/quadratic bezier curve is flattened
/// into. A fixed subdivision count keeps tessellation simple; it's coarse
/// enough for icon-sized geometry but not meant for large/zoomed vector art.
const CURVE_SEGMENTS: u32 = 16;

/// A single filled triangle in the SVG's own `viewBox` coordinate space,
/// tagged with the paint it should be drawn with.
#[derive(Clone, Copy, Debug)]
pub struct SvgTriangle {
    pub p0: (f32, f32),
    pub p1: (f32, f32),
    pub p2: (f32, f32),
    pub paint: SvgColor,
    pub opacity: f32,
}

/// Flattens an entire document into a triangle list, ready to be scaled
/// into a widget's layout box and handed to the triangle pipeline.
pub fn tessellate_document(doc: &SvgDocument) -> Vec<SvgTriangle> {
    let mut out = Vec::new();
    for element in &doc.elements {
        tessellate_element(element, Transform2D::IDENTITY, 1.0, &mut out);
    }
    out
}

fn tessellate_element(
    element: &SvgElement,
    parent_transform: Transform2D,
    parent_opacity: f32,
    out: &mut Vec<SvgTriangle>
) {
    let transform = parent_transform.then(element.attrs().transform);
    let opacity = parent_opacity * element.attrs().opacity;

    match element {
        SvgElement::Group { children, .. } => {
            for child in children {
                tessellate_element(child, transform, opacity, out);
            }
        }
        SvgElement::Path { commands, attrs } => {
            emit_shape(&flatten_path(commands), attrs, transform, opacity, out);
        }
        SvgElement::Rect { x, y, width, height, rx, attrs } => {
            emit_shape(
                &[rect_polygon(*x, *y, *width, *height, *rx)],
                attrs,
                transform,
                opacity,
                out
            );
        }
        SvgElement::Circle { cx, cy, r, attrs } => {
            emit_shape(&[circle_polygon(*cx, *cy, *r)], attrs, transform, opacity, out);
        }
        SvgElement::Line { x1, y1, x2, y2, attrs } => {
            emit_shape(&[vec![(*x1, *y1), (*x2, *y2)]], attrs, transform, opacity, out);
        }
    }
}

fn emit_shape(
    polylines: &[Vec<(f32, f32)>],
    attrs: &SvgAttributes,
    transform: Transform2D,
    opacity: f32,
    out: &mut Vec<SvgTriangle>
) {
    for polyline in polylines {
        let transformed: Vec<(f32, f32)> = polyline
            .iter()
            .map(|&(x, y)| transform.apply(x, y))
            .collect();

        if !matches!(attrs.fill, SvgColor::None) {
            fan_triangulate(&transformed, attrs.fill, opacity, out);
        }

        if !matches!(attrs.stroke, SvgColor::None) && attrs.stroke_width > 0.0 {
            stroke_polyline(&transformed, attrs.stroke_width, attrs.stroke, opacity, out);
        }
    }
}

// Fan triangulation from the first vertex. Correct for convex and most
// simple star-shaped polygons; self-intersecting or concave-heavy paths
// need a real tessellator (e.g. the `lyon` crate) to render without
// artifacts - left as a future extension point.
fn fan_triangulate(
    points: &[(f32, f32)],
    paint: SvgColor,
    opacity: f32,
    out: &mut Vec<SvgTriangle>
) {
    if points.len() < 3 {
        return;
    }
    let p0 = points[0];
    for window in points[1..].windows(2) {
        out.push(SvgTriangle { p0, p1: window[0], p2: window[1], paint, opacity });
    }
}

// Turns a polyline into a strip of quads, one pair of triangles per
// segment, each segment extended by half the stroke width past both ends
// so consecutive segments still overlap enough to hide gaps at joints
// (a "square cap" approximation rather than true miter/round joins).
fn stroke_polyline(
    points: &[(f32, f32)],
    width: f32,
    paint: SvgColor,
    opacity: f32,
    out: &mut Vec<SvgTriangle>
) {
    let half = width * 0.5;
    for pair in points.windows(2) {
        let (x1, y1) = pair[0];
        let (x2, y2) = pair[1];
        let dx = x2 - x1;
        let dy = y2 - y1;
        let len = (dx * dx + dy * dy).sqrt();
        if len < f32::EPSILON {
            continue;
        }
        let (nx, ny) = ((-dy / len) * half, (dx / len) * half);
        let (ex, ey) = ((dx / len) * half, (dy / len) * half);

        let a = (x1 - ex + nx, y1 - ey + ny);
        let b = (x1 - ex - nx, y1 - ey - ny);
        let c = (x2 + ex + nx, y2 + ey + ny);
        let d = (x2 + ex - nx, y2 + ey - ny);

        out.push(SvgTriangle { p0: a, p1: b, p2: c, paint, opacity });
        out.push(SvgTriangle { p0: b, p1: d, p2: c, paint, opacity });
    }
}

fn rect_polygon(x: f32, y: f32, width: f32, height: f32, rx: f32) -> Vec<(f32, f32)> {
    if rx <= 0.0 {
        return vec![(x, y), (x + width, y), (x + width, y + height), (x, y + height)];
    }
    // Rounded corners are approximated with a quarter-circle fan per corner.
    let r = rx.min(width * 0.5).min(height * 0.5);
    let mut points = Vec::new();
    let corners = [
        (x + width - r, y + r, -90.0f32, 0.0f32),
        (x + width - r, y + height - r, 0.0, 90.0),
        (x + r, y + height - r, 90.0, 180.0),
        (x + r, y + r, 180.0, 270.0),
    ];
    for &(cx, cy, start_deg, end_deg) in &corners {
        let steps = 6;
        for i in 0..=steps {
            let t = start_deg + (end_deg - start_deg) * ((i as f32) / (steps as f32));
            let rad = t.to_radians();
            points.push((cx + rad.cos() * r, cy + rad.sin() * r));
        }
    }
    points
}

fn circle_polygon(cx: f32, cy: f32, r: f32) -> Vec<(f32, f32)> {
    const STEPS: u32 = 32;
    (0..STEPS)
        .map(|i| {
            let t = ((i as f32) / (STEPS as f32)) * std::f32::consts::TAU;
            (cx + t.cos() * r, cy + t.sin() * r)
        })
        .collect()
}

// Splits path commands on MoveTo/Close boundaries and flattens every curve
// into line segments, producing one polyline per subpath.
fn flatten_path(commands: &[PathCommand]) -> Vec<Vec<(f32, f32)>> {
    let mut subpaths = Vec::new();
    let mut current: Vec<(f32, f32)> = Vec::new();
    let mut cursor = (0.0, 0.0);
    let mut subpath_start = (0.0, 0.0);

    for command in commands {
        match *command {
            PathCommand::MoveTo(x, y) => {
                if current.len() > 1 {
                    subpaths.push(std::mem::take(&mut current));
                } else {
                    current.clear();
                }
                cursor = (x, y);
                subpath_start = cursor;
                current.push(cursor);
            }
            PathCommand::LineTo(x, y) => {
                cursor = (x, y);
                current.push(cursor);
            }
            PathCommand::QuadTo(cx, cy, x, y) => {
                for i in 1..=CURVE_SEGMENTS {
                    let t = (i as f32) / (CURVE_SEGMENTS as f32);
                    current.push(quad_point(cursor, (cx, cy), (x, y), t));
                }
                cursor = (x, y);
            }
            PathCommand::CubicTo(c1x, c1y, c2x, c2y, x, y) => {
                for i in 1..=CURVE_SEGMENTS {
                    let t = (i as f32) / (CURVE_SEGMENTS as f32);
                    current.push(cubic_point(cursor, (c1x, c1y), (c2x, c2y), (x, y), t));
                }
                cursor = (x, y);
            }
            PathCommand::Close => {
                current.push(subpath_start);
                cursor = subpath_start;
            }
        }
    }
    if current.len() > 1 {
        subpaths.push(current);
    }
    subpaths
}

fn quad_point(p0: (f32, f32), p1: (f32, f32), p2: (f32, f32), t: f32) -> (f32, f32) {
    let mt = 1.0 - t;
    (
        mt * mt * p0.0 + 2.0 * mt * t * p1.0 + t * t * p2.0,
        mt * mt * p0.1 + 2.0 * mt * t * p1.1 + t * t * p2.1,
    )
}

fn cubic_point(
    p0: (f32, f32),
    p1: (f32, f32),
    p2: (f32, f32),
    p3: (f32, f32),
    t: f32
) -> (f32, f32) {
    let mt = 1.0 - t;
    let a = mt * mt * mt;
    let b = 3.0 * mt * mt * t;
    let c = 3.0 * mt * t * t;
    let d = t * t * t;
    (a * p0.0 + b * p1.0 + c * p2.0 + d * p3.0, a * p0.1 + b * p1.1 + c * p2.1 + d * p3.1)
}
