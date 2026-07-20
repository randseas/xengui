// SPDX-License-Identifier: Apache-2.0
use crate::{
    AnimationManager,
    DrawCommand,
    ImageCommand,
    LayoutContext,
    LayoutEngine,
    PaintContext,
    RectCommand,
    RenderBackend,
    RenderCache,
    SystemTheme,
    TextCommand,
    TriangleCommand,
    Widget,
};
use std::collections::HashSet;
use web_time::Instant;

/// Backend-agnostic frame orchestration: layout, paint-tree walk, command
/// batching and z-ordering. Every actual draw call is delegated to a
/// [`RenderBackend`] implementation.
pub struct FrameRenderer {
    render_cache: RenderCache,
    anim: AnimationManager,
    last_tick: Instant,
    force_layout: bool,
}

impl FrameRenderer {
    pub fn new() -> Self {
        Self {
            render_cache: RenderCache::new(),
            anim: AnimationManager::new(),
            last_tick: Instant::now(),
            force_layout: false,
        }
    }

    pub fn anim(&mut self) -> &mut AnimationManager {
        &mut self.anim
    }

    pub fn is_animating(&self) -> bool {
        self.anim.is_animating()
    }

    pub fn resize(&mut self) {
        self.force_layout = true;
    }

    pub fn render_frame(
        &mut self,
        tree: &mut [Box<dyn Widget>],
        backend: &mut dyn RenderBackend,
        theme: SystemTheme,
        scale_factor: f32,
        width: u32,
        height: u32
    ) {
        let now = Instant::now();
        let dt = now.duration_since(self.last_tick);
        self.last_tick = now;
        self.anim.tick(dt);

        let app_background = crate::current_theme().background;
        if !backend.begin_frame(app_background, width, height) {
            return;
        }

        let needs_full_layout =
            std::mem::take(&mut self.force_layout) ||
            tree_is_dirty(tree) ||
            self.anim.active_keys().any(|k| k.property.affects_layout());

        let mut layout_ctx = LayoutContext {
            text: backend.text_measurer(),
            anim: &mut self.anim,
            scale_factor,
        };

        if needs_full_layout {
            LayoutEngine::layout(
                tree,
                &mut layout_ctx,
                &mut self.render_cache,
                width as f32,
                height as f32
            );
        } else {
            LayoutEngine::cascade(tree, &mut layout_ctx);
        }

        let mut commands: Vec<(i32, DrawCommand)> = Vec::new();
        let mut focus_commands: Vec<RectCommand> = Vec::new();
        let mut live_keys: HashSet<String> = HashSet::new();

        for (i, node) in tree.iter().enumerate() {
            let segment = crate::path_segment(node.as_ref(), i);
            paint_recursive(
                node.as_ref(),
                &segment,
                &mut self.render_cache,
                &mut commands,
                &mut focus_commands,
                &mut live_keys,
                None,
                scale_factor
            );
        }
        self.render_cache.retain_keys(&live_keys);

        for node in tree.iter_mut() {
            reset_dirty_recursive(node.as_mut());
        }

        // Stable sort keeps original paint order for widgets sharing the
        // same z-index; only different values get reordered.
        commands.sort_by_key(|(z, _)| *z);

        #[derive(PartialEq, Clone, Copy)]
        enum RunKind {
            Rect,
            Triangle,
            Image,
        }

        let mut current_kind: Option<RunKind> = None;
        let mut rect_buf: Vec<RectCommand> = Vec::new();
        let mut tri_buf: Vec<TriangleCommand> = Vec::new();
        let mut img_buf: Vec<ImageCommand> = Vec::new();
        let mut text_cmds: Vec<TextCommand> = Vec::new();

        macro_rules! flush_run {
            () => {
                match current_kind {
                    Some(RunKind::Rect) => backend.draw_rects(&rect_buf),
                    Some(RunKind::Triangle) => backend.draw_triangles(&tri_buf),
                    Some(RunKind::Image) => backend.draw_images(&img_buf),
                    None => {}
                }
                rect_buf.clear();
                tri_buf.clear();
                img_buf.clear();
            };
        }

        // Draws each contiguous run of same-type commands in the order
        // z-index (then paint order) puts them in, instead of always
        // drawing every rect, then every triangle, then every image.
        for (_, command) in commands {
            match command {
                DrawCommand::Text(cmd) => text_cmds.push(*cmd),
                DrawCommand::Rect(cmd) => {
                    if current_kind != Some(RunKind::Rect) {
                        flush_run!();
                        current_kind = Some(RunKind::Rect);
                    }
                    rect_buf.push(cmd);
                }
                DrawCommand::Triangle(cmd) => {
                    if current_kind != Some(RunKind::Triangle) {
                        flush_run!();
                        current_kind = Some(RunKind::Triangle);
                    }
                    tri_buf.push(cmd);
                }
                DrawCommand::Image(cmd) => {
                    if current_kind != Some(RunKind::Image) {
                        flush_run!();
                        current_kind = Some(RunKind::Image);
                    }
                    img_buf.push(*cmd);
                }
            }
        }
        flush_run!();

        for cmd in &text_cmds {
            backend.draw_text(theme, scale_factor, cmd);
        }

        // Underline/strike/overline quads produced while queueing text
        // above; drawn once, after every layer.
        let decorations = backend.take_text_decorations();
        if !decorations.is_empty() {
            backend.draw_rects(&decorations);
        }

        backend.flush_text();

        // Drawn after text is flushed so the focus ring is always visible
        // above absolutely everything else in the frame.
        if !focus_commands.is_empty() {
            backend.draw_rects(&focus_commands);
        }

        backend.end_frame();
    }
}

impl Default for FrameRenderer {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(clippy::too_many_arguments)]
fn paint_recursive(
    widget: &dyn Widget,
    path: &str,
    cache: &mut RenderCache,
    commands: &mut Vec<(i32, DrawCommand)>,
    focus_commands: &mut Vec<RectCommand>,
    live_keys: &mut HashSet<String>,
    clip_rect: Option<(f32, f32, f32, f32)>,
    scale_factor: f32
) {
    let layout_box = *widget.layout_box();

    if let Some((cx, cy, cw, ch)) = clip_rect {
        let visible =
            layout_box.x < cx + cw &&
            layout_box.x + layout_box.width > cx &&
            layout_box.y < cy + ch &&
            layout_box.y + layout_box.height > cy;
        if !visible {
            return;
        }
    }

    live_keys.insert(path.to_string());

    let z_index = widget.computed_style().z_index.unwrap_or(0);

    let own_commands: Vec<DrawCommand> = match cache.try_reuse(path, layout_box, widget.is_dirty()) {
        Some(cached) => cached.to_vec(),
        None => {
            let mut local = Vec::new();
            {
                let mut paint_ctx = PaintContext::new(&mut local, scale_factor);
                widget.paint(&mut paint_ctx);
            }
            cache.store(path, layout_box, local.clone());
            local
        }
    };

    for mut command in own_commands {
        apply_clip(&mut command, clip_rect);
        commands.push((z_index, command));
    }

    let child_clip = match widget.clip_children() {
        Some(rect) => Some(clip_intersect(clip_rect, rect)),
        None => clip_rect,
    };

    for (i, child) in widget.children().iter().enumerate() {
        let segment = crate::path_segment(child.as_ref(), i);
        paint_recursive(
            child.as_ref(),
            &format!("{path}.{segment}"),
            cache,
            commands,
            focus_commands,
            live_keys,
            child_clip,
            scale_factor
        );
    }

    let mut overlay = Vec::new();
    {
        let mut paint_ctx = PaintContext::new(&mut overlay, scale_factor);
        widget.paint_overlay(&mut paint_ctx);
    }
    for mut command in overlay {
        apply_clip(&mut command, clip_rect);
        commands.push((z_index, command));
    }

    let mut focus_local = Vec::new();
    {
        let mut paint_ctx = PaintContext::new(&mut focus_local, scale_factor);
        widget.paint_focus(&mut paint_ctx);
    }
    for mut command in focus_local {
        apply_clip(&mut command, clip_rect);
        if let DrawCommand::Rect(rect_cmd) = command {
            focus_commands.push(rect_cmd);
        }
    }
}

fn clip_intersect(
    existing: Option<(f32, f32, f32, f32)>,
    ancestor: (f32, f32, f32, f32)
) -> (f32, f32, f32, f32) {
    let Some((ex, ey, ew, eh)) = existing else {
        return ancestor;
    };
    let (ax, ay, aw, ah) = ancestor;
    let x0 = ex.max(ax);
    let y0 = ey.max(ay);
    let x1 = (ex + ew).min(ax + aw);
    let y1 = (ey + eh).min(ay + ah);
    (x0, y0, (x1 - x0).max(0.0), (y1 - y0).max(0.0))
}

fn apply_clip(command: &mut DrawCommand, clip_rect: Option<(f32, f32, f32, f32)>) {
    let Some(ancestor_clip) = clip_rect else {
        return;
    };
    let target = match command {
        DrawCommand::Rect(cmd) => &mut cmd.clip_rect,
        DrawCommand::Image(cmd) => &mut cmd.clip_rect,
        DrawCommand::Text(cmd) => &mut cmd.clip_rect,
        DrawCommand::Triangle(cmd) => &mut cmd.clip_rect,
    };
    *target = Some(clip_intersect(*target, ancestor_clip));
}

fn reset_dirty_recursive(widget: &mut dyn Widget) {
    widget.set_dirty(false);
    if let Some(children) = widget.children_mut() {
        for child in children.iter_mut() {
            reset_dirty_recursive(child.as_mut());
        }
    }
}

fn tree_is_dirty(tree: &[Box<dyn Widget>]) -> bool {
    tree.iter().any(|w| widget_dirty_recursive(w.as_ref()))
}

fn widget_dirty_recursive(widget: &dyn Widget) -> bool {
    widget.is_dirty() ||
        widget
            .children()
            .iter()
            .any(|c| widget_dirty_recursive(c.as_ref()))
}
