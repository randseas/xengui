// SPDX-License-Identifier: Apache-2.0
use super::{
    FillRule as SvgFillRule,
    LineCap as SvgLineCap,
    LineJoin as SvgLineJoin,
    PathCommand,
    SvgAttributes,
    SvgColor,
    SvgDocument,
    SvgElement,
    Transform2D,
};
use lyon::math::{ point, Point };
use lyon::path::Path;
use lyon::tessellation::{
    BuffersBuilder,
    FillOptions,
    FillRule,
    FillTessellator,
    FillVertex,
    LineCap,
    LineJoin,
    StrokeOptions,
    StrokeTessellator,
    StrokeVertex,
    VertexBuffers,
};

/// Maximum distance (in the SVG's own user-space units) between a curve
/// and its flattened approximation.
const TOLERANCE: f32 = 0.05;

/// Segment count used to approximate a full circle as a polygon before
/// handing it to lyon - lyon's tessellators only consume straight/bezier
/// path segments, not native arcs.
const CIRCLE_SEGMENTS: u32 = 48;

/// Segment count per rounded-rect corner, approximated the same way.
const CORNER_SEGMENTS: u32 = 12;

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
    // Element's own local transform must apply first, then the accumulated
    // ancestor chain - not the other way around.
    let transform = element.attrs().transform.then(parent_transform);
    let opacity = parent_opacity * element.attrs().opacity;

    match element {
        SvgElement::Group { children, .. } => {
            for child in children {
                tessellate_element(child, transform, opacity, out);
            }
        }
        SvgElement::Path { commands, attrs } => {
            let path = build_path_from_commands(commands, transform);
            emit_shape(&path, attrs, opacity, out);
        }
        SvgElement::Rect { x, y, width, height, rx, attrs } => {
            let polygon = rect_polygon(*x, *y, *width, *height, *rx);
            let path = build_polygon_path(&polygon, true, transform);
            emit_shape(&path, attrs, opacity, out);
        }
        SvgElement::Circle { cx, cy, r, attrs } => {
            let polygon = circle_polygon(*cx, *cy, *r);
            let path = build_polygon_path(&polygon, true, transform);
            emit_shape(&path, attrs, opacity, out);
        }
        SvgElement::Line { x1, y1, x2, y2, attrs } => {
            let path = build_polygon_path(
                &[
                    (*x1, *y1),
                    (*x2, *y2),
                ],
                false,
                transform
            );
            emit_stroke(&path, attrs, opacity, out);
        }
    }
}

// Maps a local-space point through the element's accumulated transform.
// Baking the transform in before tessellation (rather than after) keeps
// fills correct under rotation/skew, and applies stroke width in already
// transformed space, matching the previous renderer's behavior.
fn map_point(transform: Transform2D, x: f32, y: f32) -> Point {
    let (tx, ty) = transform.apply(x, y);
    point(tx, ty)
}

fn build_path_from_commands(commands: &[PathCommand], transform: Transform2D) -> Path {
    let mut builder = Path::builder();
    let mut in_subpath = false;

    for command in commands {
        match *command {
            PathCommand::MoveTo(x, y) => {
                if in_subpath {
                    builder.end(false);
                }
                builder.begin(map_point(transform, x, y));
                in_subpath = true;
            }
            PathCommand::LineTo(x, y) => {
                builder.line_to(map_point(transform, x, y));
            }
            PathCommand::QuadTo(cx, cy, x, y) => {
                builder.quadratic_bezier_to(
                    map_point(transform, cx, cy),
                    map_point(transform, x, y)
                );
            }
            PathCommand::CubicTo(c1x, c1y, c2x, c2y, x, y) => {
                builder.cubic_bezier_to(
                    map_point(transform, c1x, c1y),
                    map_point(transform, c2x, c2y),
                    map_point(transform, x, y)
                );
            }
            PathCommand::Close => {
                builder.close();
                in_subpath = false;
            }
        }
    }

    if in_subpath {
        builder.end(false);
    }

    builder.build()
}

fn build_polygon_path(points: &[(f32, f32)], closed: bool, transform: Transform2D) -> Path {
    let mut builder = Path::builder();
    let mut iter = points.iter();

    let Some(&(x0, y0)) = iter.next() else {
        return builder.build();
    };

    builder.begin(map_point(transform, x0, y0));
    for &(x, y) in iter {
        builder.line_to(map_point(transform, x, y));
    }
    builder.end(closed);

    builder.build()
}

fn emit_shape(path: &Path, attrs: &SvgAttributes, opacity: f32, out: &mut Vec<SvgTriangle>) {
    if !matches!(attrs.fill, SvgColor::None) {
        tessellate_fill(path, attrs, opacity, out);
    }
    emit_stroke(path, attrs, opacity, out);
}

fn emit_stroke(path: &Path, attrs: &SvgAttributes, opacity: f32, out: &mut Vec<SvgTriangle>) {
    if !matches!(attrs.stroke, SvgColor::None) && attrs.stroke_width > 0.0 {
        tessellate_stroke(path, attrs, opacity, out);
    }
}

fn tessellate_fill(path: &Path, attrs: &SvgAttributes, opacity: f32, out: &mut Vec<SvgTriangle>) {
    let mut geometry: VertexBuffers<[f32; 2], u16> = VertexBuffers::new();
    let options = FillOptions::default()
        .with_tolerance(TOLERANCE)
        .with_fill_rule(map_fill_rule(attrs.fill_rule));

    let result = FillTessellator::new().tessellate_path(
        path,
        &options,
        &mut BuffersBuilder::new(&mut geometry, |vertex: FillVertex| {
            vertex.position().to_array()
        })
    );

    if let Err(err) = result {
        log::error!("xen-svg: fill tessellation failed: {err:?}");
        return;
    }

    push_triangles(&geometry, attrs.fill, opacity, out);
}

fn tessellate_stroke(path: &Path, attrs: &SvgAttributes, opacity: f32, out: &mut Vec<SvgTriangle>) {
    let mut geometry: VertexBuffers<[f32; 2], u16> = VertexBuffers::new();
    let options = StrokeOptions::default()
        .with_tolerance(TOLERANCE)
        .with_line_width(attrs.stroke_width)
        .with_line_join(map_line_join(attrs.line_join))
        .with_line_cap(map_line_cap(attrs.line_cap))
        .with_miter_limit(attrs.miter_limit);

    let result = StrokeTessellator::new().tessellate_path(
        path,
        &options,
        &mut BuffersBuilder::new(&mut geometry, |vertex: StrokeVertex| {
            vertex.position().to_array()
        })
    );

    if let Err(err) = result {
        log::error!("xen-svg: stroke tessellation failed: {err:?}");
        return;
    }

    push_triangles(&geometry, attrs.stroke, opacity, out);
}

fn map_line_cap(cap: SvgLineCap) -> LineCap {
    match cap {
        SvgLineCap::Butt => LineCap::Butt,
        SvgLineCap::Round => LineCap::Round,
        SvgLineCap::Square => LineCap::Square,
    }
}

fn map_line_join(join: SvgLineJoin) -> LineJoin {
    match join {
        SvgLineJoin::Miter => LineJoin::Miter,
        SvgLineJoin::Round => LineJoin::Round,
        SvgLineJoin::Bevel => LineJoin::Bevel,
    }
}

fn map_fill_rule(rule: SvgFillRule) -> FillRule {
    match rule {
        SvgFillRule::NonZero => FillRule::NonZero,
        SvgFillRule::EvenOdd => FillRule::EvenOdd,
    }
}

fn push_triangles(
    geometry: &VertexBuffers<[f32; 2], u16>,
    paint: SvgColor,
    opacity: f32,
    out: &mut Vec<SvgTriangle>
) {
    for tri in geometry.indices.chunks_exact(3) {
        let p0 = geometry.vertices[tri[0] as usize];
        let p1 = geometry.vertices[tri[1] as usize];
        let p2 = geometry.vertices[tri[2] as usize];
        out.push(SvgTriangle {
            p0: (p0[0], p0[1]),
            p1: (p1[0], p1[1]),
            p2: (p2[0], p2[1]),
            paint,
            opacity,
        });
    }
}

fn rect_polygon(x: f32, y: f32, width: f32, height: f32, rx: f32) -> Vec<(f32, f32)> {
    if rx <= 0.0 {
        return vec![(x, y), (x + width, y), (x + width, y + height), (x, y + height)];
    }

    let r = rx.min(width * 0.5).min(height * 0.5);
    let mut points = Vec::new();
    let corners = [
        (x + width - r, y + r, -90.0f32, 0.0f32),
        (x + width - r, y + height - r, 0.0, 90.0),
        (x + r, y + height - r, 90.0, 180.0),
        (x + r, y + r, 180.0, 270.0),
    ];
    for &(cx, cy, start_deg, end_deg) in &corners {
        for i in 0..=CORNER_SEGMENTS {
            let t = start_deg + (end_deg - start_deg) * ((i as f32) / (CORNER_SEGMENTS as f32));
            let rad = t.to_radians();
            points.push((cx + rad.cos() * r, cy + rad.sin() * r));
        }
    }
    points
}

fn circle_polygon(cx: f32, cy: f32, r: f32) -> Vec<(f32, f32)> {
    (0..CIRCLE_SEGMENTS)
        .map(|i| {
            let t = ((i as f32) / (CIRCLE_SEGMENTS as f32)) * std::f32::consts::TAU;
            (cx + t.cos() * r, cy + t.sin() * r)
        })
        .collect()
}
