// SPDX-License-Identifier: Apache-2.0
use crate::{
    AnimationManager,
    Constraints,
    EventCtx,
    EventStatus,
    InputEvent,
    Interaction,
    LayoutBox,
    MeasureContext,
    MeasureResult,
    PaintContext,
    Style,
    StyleBuilder,
    TriangleCommand,
    Widget,
    WidgetBase,
    WidgetId,
    svg_compat::{ IntoSvgColor, from_svg_color },
};
use smol_str::SmolStr;
use std::sync::Arc;
use xen_svg::{
    PathCommand,
    SvgAttributes,
    SvgDocument,
    SvgElement,
    SvgTriangle,
    Transform2D,
    parse_svg,
    tessellate_document,
};

macro_rules! impl_svg_attrs_builder {
    ($ty:ident) => {
        impl $ty {
            pub fn fill(mut self, color: impl IntoSvgColor) -> Self {
                self.attrs.fill = color.into_svg_color();
                self
            }

            pub fn stroke(mut self, color: impl IntoSvgColor) -> Self {
                self.attrs.stroke = color.into_svg_color();
                self
            }

            pub fn stroke_width(mut self, width: f32) -> Self {
                self.attrs.stroke_width = width;
                self
            }

            pub fn opacity(mut self, opacity: f32) -> Self {
                self.attrs.opacity = opacity;
                self
            }

            pub fn transform(mut self, transform: Transform2D) -> Self {
                self.attrs.transform = transform;
                self
            }
        }
    };
}

pub struct SvgPathBuilder {
    commands: Vec<PathCommand>,
    attrs: SvgAttributes,
}

impl SvgPathBuilder {
    pub fn new() -> Self {
        Self { commands: Vec::new(), attrs: SvgAttributes::default() }
    }

    pub fn move_to(mut self, x: f32, y: f32) -> Self {
        self.commands.push(PathCommand::MoveTo(x, y));
        self
    }

    pub fn line_to(mut self, x: f32, y: f32) -> Self {
        self.commands.push(PathCommand::LineTo(x, y));
        self
    }

    pub fn quad_to(mut self, cx: f32, cy: f32, x: f32, y: f32) -> Self {
        self.commands.push(PathCommand::QuadTo(cx, cy, x, y));
        self
    }

    pub fn cubic_to(mut self, c1x: f32, c1y: f32, c2x: f32, c2y: f32, x: f32, y: f32) -> Self {
        self.commands.push(PathCommand::CubicTo(c1x, c1y, c2x, c2y, x, y));
        self
    }

    pub fn close(mut self) -> Self {
        self.commands.push(PathCommand::Close);
        self
    }

    fn build(self) -> SvgElement {
        SvgElement::Path { commands: self.commands, attrs: self.attrs }
    }
}

impl Default for SvgPathBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl_svg_attrs_builder!(SvgPathBuilder);

pub struct SvgRectBuilder {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    rx: f32,
    attrs: SvgAttributes,
}

impl SvgRectBuilder {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self { x, y, width, height, rx: 0.0, attrs: SvgAttributes::default() }
    }

    pub fn radius(mut self, rx: f32) -> Self {
        self.rx = rx;
        self
    }

    fn build(self) -> SvgElement {
        SvgElement::Rect {
            x: self.x,
            y: self.y,
            width: self.width,
            height: self.height,
            rx: self.rx,
            attrs: self.attrs,
        }
    }
}

impl_svg_attrs_builder!(SvgRectBuilder);

pub struct SvgCircleBuilder {
    cx: f32,
    cy: f32,
    r: f32,
    attrs: SvgAttributes,
}

impl SvgCircleBuilder {
    pub fn new(cx: f32, cy: f32, r: f32) -> Self {
        Self { cx, cy, r, attrs: SvgAttributes::default() }
    }

    fn build(self) -> SvgElement {
        SvgElement::Circle { cx: self.cx, cy: self.cy, r: self.r, attrs: self.attrs }
    }
}

impl_svg_attrs_builder!(SvgCircleBuilder);

pub struct SvgLineBuilder {
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    attrs: SvgAttributes,
}

impl SvgLineBuilder {
    pub fn new(x1: f32, y1: f32, x2: f32, y2: f32) -> Self {
        Self { x1, y1, x2, y2, attrs: SvgAttributes::default() }
    }

    fn build(self) -> SvgElement {
        SvgElement::Line { x1: self.x1, y1: self.y1, x2: self.x2, y2: self.y2, attrs: self.attrs }
    }
}

impl_svg_attrs_builder!(SvgLineBuilder);

pub struct SvgGroupBuilder {
    children: Vec<SvgElement>,
    attrs: SvgAttributes,
}

impl SvgGroupBuilder {
    pub fn new() -> Self {
        Self { children: Vec::new(), attrs: SvgAttributes::default() }
    }

    pub fn path(mut self, build: impl FnOnce(SvgPathBuilder) -> SvgPathBuilder) -> Self {
        self.children.push(build(SvgPathBuilder::new()).build());
        self
    }

    pub fn rect(
        mut self,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        build: impl FnOnce(SvgRectBuilder) -> SvgRectBuilder
    ) -> Self {
        self.children.push(build(SvgRectBuilder::new(x, y, w, h)).build());
        self
    }

    pub fn circle(
        mut self,
        cx: f32,
        cy: f32,
        r: f32,
        build: impl FnOnce(SvgCircleBuilder) -> SvgCircleBuilder
    ) -> Self {
        self.children.push(build(SvgCircleBuilder::new(cx, cy, r)).build());
        self
    }

    pub fn line(
        mut self,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        build: impl FnOnce(SvgLineBuilder) -> SvgLineBuilder
    ) -> Self {
        self.children.push(build(SvgLineBuilder::new(x1, y1, x2, y2)).build());
        self
    }

    pub fn group(mut self, build: impl FnOnce(SvgGroupBuilder) -> SvgGroupBuilder) -> Self {
        self.children.push(build(SvgGroupBuilder::new()).build());
        self
    }

    fn build(self) -> SvgElement {
        SvgElement::Group { children: self.children, attrs: self.attrs }
    }
}

impl Default for SvgGroupBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl_svg_attrs_builder!(SvgGroupBuilder);

/// A vector-graphics widget rendering a small subset of SVG (path, rect,
/// circle, line, group) through the existing triangle pipeline.
///
/// Colors may use [`SvgColor::CURRENT`] instead of a fixed [`crate::Color`]
/// to follow the widget's inherited `color` at render time, the same way
/// CSS's `currentColor` works - this is what lets `xengui-lucide` ship icons
/// that automatically match surrounding text color.
pub struct Svg {
    base: WidgetBase,
    anim_id: WidgetId,
    document: Arc<SvgDocument>,
    // Tessellated once per document change instead of on every paint, since
    // flattening curves and triangulating shapes isn't free.
    triangles: Arc<Vec<SvgTriangle>>,
    layout_box: LayoutBox,
}

impl Svg {
    pub fn new() -> Self {
        let document = SvgDocument::default();
        let triangles = Arc::new(tessellate_document(&document));
        let interaction = Interaction::new();

        Self {
            base: WidgetBase::new(interaction),
            anim_id: WidgetId::new_unique(),
            document: Arc::new(document),
            triangles,
            layout_box: LayoutBox::default(),
        }
    }

    /// Parses a full `<svg>...</svg>` document string.
    pub fn from_string(source: &str) -> Self {
        let mut svg = Self::new();
        match parse_svg(source) {
            Ok(document) => svg.set_document(document),
            Err(err) => log::error!("Svg::from_string parse error: {err}"),
        }
        svg
    }

    /// Parses raw UTF-8 SVG bytes; invalid UTF-8 or malformed markup logs
    /// an error and leaves the widget empty, matching `Image::bytes`.
    pub fn from_bytes(bytes: &[u8]) -> Self {
        match std::str::from_utf8(bytes) {
            Ok(source) => Self::from_string(source),
            Err(err) => {
                log::error!("Svg::from_bytes invalid utf-8: {err}");
                Self::new()
            }
        }
    }

    pub fn key(mut self, key: impl Into<SmolStr>) -> Self {
        self.base.key = Some(key.into());
        self
    }

    pub fn view_box(mut self, x: f32, y: f32, width: f32, height: f32) -> Self {
        let mut document = (*self.document).clone();
        document.view_box = (x, y, width, height);
        self.set_document(document);
        self
    }

    pub fn path(mut self, build: impl FnOnce(SvgPathBuilder) -> SvgPathBuilder) -> Self {
        self.push_element(build(SvgPathBuilder::new()).build());
        self
    }

    pub fn rect(
        mut self,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        build: impl FnOnce(SvgRectBuilder) -> SvgRectBuilder
    ) -> Self {
        self.push_element(build(SvgRectBuilder::new(x, y, w, h)).build());
        self
    }

    pub fn circle(
        mut self,
        cx: f32,
        cy: f32,
        r: f32,
        build: impl FnOnce(SvgCircleBuilder) -> SvgCircleBuilder
    ) -> Self {
        self.push_element(build(SvgCircleBuilder::new(cx, cy, r)).build());
        self
    }

    pub fn line(
        mut self,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        build: impl FnOnce(SvgLineBuilder) -> SvgLineBuilder
    ) -> Self {
        self.push_element(build(SvgLineBuilder::new(x1, y1, x2, y2)).build());
        self
    }

    pub fn group(mut self, build: impl FnOnce(SvgGroupBuilder) -> SvgGroupBuilder) -> Self {
        self.push_element(build(SvgGroupBuilder::new()).build());
        self
    }

    fn push_element(&mut self, element: SvgElement) {
        let mut document = (*self.document).clone();
        document.elements.push(element);
        self.set_document(document);
    }

    fn set_document(&mut self, document: SvgDocument) {
        self.triangles = Arc::new(tessellate_document(&document));
        self.document = Arc::new(document);
        self.mark_dirty();
    }

    fn recompute_style(&mut self) {
        self.base.computed_style = self.base.inherited_style.inherit_style(&self.base.style);
    }
}

impl Default for Svg {
    fn default() -> Self {
        Self::new()
    }
}

impl StyleBuilder for Svg {
    fn style_mut(&mut self) -> &mut Style {
        &mut self.base.style
    }

    fn mark_dirty(&mut self) {
        self.base.dirty = true;
        self.recompute_style();
    }
}

crate::impl_interaction_builders!(base Svg);

impl Widget for Svg {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn debug_name(&self) -> &'static str {
        "Widget#Svg"
    }

    fn get_key(&self) -> Option<&SmolStr> {
        self.base.key.as_ref()
    }

    fn is_dirty(&self) -> bool {
        self.base.dirty
    }

    fn set_dirty(&mut self, dirty: bool) {
        self.base.dirty = dirty;
    }

    fn style(&self) -> &Style {
        &self.base.style
    }

    fn style_mut(&mut self) -> &mut Style {
        &mut self.base.style
    }

    fn computed_style(&self) -> &Style {
        &self.base.computed_style
    }

    fn children(&self) -> &[Box<dyn Widget>] {
        &[]
    }

    fn interaction(&self) -> Option<&Interaction> {
        Some(&self.base.interaction)
    }

    fn interaction_mut(&mut self) -> Option<&mut Interaction> {
        Some(&mut self.base.interaction)
    }

    fn measure(&self, ctx: &mut MeasureContext, _constraints: Constraints) -> MeasureResult {
        let (_, _, w, h) = self.document.view_box;
        MeasureResult::new(w * ctx.scale_factor, h * ctx.scale_factor)
    }

    fn layout(&mut self, rect: LayoutBox) {
        self.layout_box = rect;
    }

    fn layout_box(&self) -> &LayoutBox {
        &self.layout_box
    }

    fn paint(&self, ctx: &mut PaintContext) {
        self.paint_box(ctx);
        self.paint_outline(ctx);

        let (vb_x, vb_y, vb_w, vb_h) = self.document.view_box;
        if self.triangles.is_empty() || vb_w <= 0.0 || vb_h <= 0.0 {
            return;
        }

        let b = self.layout_box;
        let scale_x = b.width / vb_w;
        let scale_y = b.height / vb_h;

        // `color` is the CSS-inherited text color; `currentColor` paints
        // resolve against it, so icons follow the parent's text color for
        // free instead of needing an explicit tint on every instance.
        let inherited_color = self.base.computed_style.color.unwrap_or(crate::Color::BLACK);
        let inherited_svg_color = xen_svg::Color::rgba_f32(
            inherited_color.r(),
            inherited_color.g(),
            inherited_color.b(),
            inherited_color.a()
        );

        for triangle in self.triangles.iter() {
            let Some(color) = triangle.paint.resolve(inherited_svg_color) else {
                continue;
            };
            let color = from_svg_color(color);
            let color = color.with_alpha_f32(color.a() * triangle.opacity);

            let map = |p: (f32, f32)| -> (f32, f32) {
                (b.x + (p.0 - vb_x) * scale_x, b.y + (p.1 - vb_y) * scale_y)
            };

            ctx.draw_triangle(TriangleCommand {
                p0: map(triangle.p0),
                p1: map(triangle.p1),
                p2: map(triangle.p2),
                color,
                clip_rect: None,
            });
        }
    }

    fn event(&mut self, event: &InputEvent, ctx: &mut EventCtx) -> EventStatus {
        if !self.base.interaction.is_active() {
            return EventStatus::Ignored;
        }
        self.base.interaction.handle(event, ctx)
    }

    fn content_eq(&self, other: &dyn Widget) -> bool {
        let Some(other) = other.as_any().downcast_ref::<Svg>() else {
            return false;
        };
        *self.document == *other.document && self.base.style == other.base.style
    }

    fn cascade_style(&mut self, parent: &Style, _anim: &mut AnimationManager) {
        self.base.inherited_style = parent.clone();
        self.recompute_style();
    }

    fn transfer_measured_state(&mut self, old: &dyn Widget) {
        if let Some(old) = old.as_any().downcast_ref::<Svg>() {
            self.anim_id = old.anim_id;
        }
    }

    fn anim_id(&self) -> WidgetId {
        self.anim_id
    }
}
