// SPDX-License-Identifier: Apache-2.0
use super::{
    AnimKey,
    AnimLayer,
    AnimProperty,
    AnimValue,
    AnimationManager,
    Transition,
    TransitionProperty,
};
use crate::{ Background, Color, Edges, Length, Style, WidgetId };

fn animate_length(
    anim: &mut AnimationManager,
    key: AnimKey,
    transition: Option<Transition>,
    value: Length,
    animating: &mut bool
) -> Length {
    // Percent lengths need a resolved parent size to blend meaningfully,
    // so only Px values are interpolated here.
    let Length::Px(v) = value else {
        return value;
    };
    anim.set_target(key, AnimValue([v, 0.0, 0.0, 0.0]), transition);
    match anim.value(key) {
        Some(r) => {
            *animating = true;
            Length::Px(r.0[0])
        }
        None => value,
    }
}

fn animate_color(
    anim: &mut AnimationManager,
    key: AnimKey,
    transition: Option<Transition>,
    value: Color,
    animating: &mut bool
) -> Color {
    anim.set_target(key, AnimValue(value.to_f32_array()), transition);
    match anim.value(key) {
        Some(v) => {
            *animating = true;
            Color::rgba_f32(v.0[0], v.0[1], v.0[2], v.0[3])
        }
        None => value,
    }
}

/// Interpolates every field of `style` allowed by its own
/// `transition_properties`, mutating it in place with the current
/// (possibly mid-transition) value. Returns whether anything is still
/// animating, so the caller can force a repaint for this frame.
pub fn animate_computed_style(
    widget_id: WidgetId,
    style: &mut Style,
    anim: &mut AnimationManager
) -> bool {
    let Some(properties) = style.transition_properties else {
        return false;
    };
    if properties.is_none() {
        return false;
    }

    let default_transition = style.transition;
    let overrides = style.transition_overrides;
    let mut animating = false;

    let key = |property: AnimProperty| AnimKey {
        widget: widget_id,
        layer: AnimLayer::Root,
        property,
    };

    if properties.contains(TransitionProperty::COLORS) {
        let transition = overrides.colors.or(default_transition);

        if let Some(color) = style.color {
            style.color = Some(
                animate_color(anim, key(AnimProperty::TextColor), transition, color, &mut animating)
            );
        }

        if let Some(Background::Color(color)) = &style.background {
            let color = *color;
            let animated = animate_color(
                anim,
                key(AnimProperty::BackgroundColor),
                transition,
                color,
                &mut animating
            );
            style.background = Some(Background::Color(animated));
        }

        if let Some(mut border) = style.border {
            border.color = animate_color(
                anim,
                key(AnimProperty::BorderColor),
                transition,
                border.color,
                &mut animating
            );
            style.border = Some(border);
        }
    }

    if properties.contains(TransitionProperty::TRANSFORM) {
        let transition = overrides.transform.or(default_transition);

        // Both targets are resolved from the static style values up front,
        // not from each other's mid-animation output - otherwise content_scale
        // would chase a constantly shifting target and never settle smoothly.
        let scale_target = style.scale.unwrap_or(1.0);
        let content_scale_target = style.content_scale.unwrap_or(scale_target);

        // Always assign a resolved value, even when resting (anim.value
        // returns None) - leaving the field unset here would make paint()
        // fall back to the *other* channel's live value instead of this
        // channel's own settled target.
        let k = key(AnimProperty::Scale);
        anim.set_target(k, AnimValue([scale_target, 0.0, 0.0, 0.0]), transition);
        let resolved_scale = anim.value(k).map_or(scale_target, |v| {
            animating = true;
            v.0[0]
        });
        style.scale = Some(resolved_scale);

        let k = key(AnimProperty::ContentScale);
        anim.set_target(k, AnimValue([content_scale_target, 0.0, 0.0, 0.0]), transition);
        let resolved_content_scale = anim.value(k).map_or(content_scale_target, |v| {
            animating = true;
            v.0[0]
        });
        style.content_scale = Some(resolved_content_scale);
    }

    if properties.contains(TransitionProperty::BOX) {
        let transition = overrides.box_model.or(default_transition);

        if let Some(mut size) = style.size {
            if let Some(w) = size.width {
                size.width = Some(
                    animate_length(anim, key(AnimProperty::Width), transition, w, &mut animating)
                );
            }
            if let Some(h) = size.height {
                size.height = Some(
                    animate_length(anim, key(AnimProperty::Height), transition, h, &mut animating)
                );
            }
            style.size = Some(size);
        }

        if let Some(padding) = style.padding {
            style.padding = Some(Edges {
                left: animate_length(
                    anim,
                    key(AnimProperty::PaddingLeft),
                    transition,
                    padding.left,
                    &mut animating
                ),
                top: animate_length(
                    anim,
                    key(AnimProperty::PaddingTop),
                    transition,
                    padding.top,
                    &mut animating
                ),
                right: animate_length(
                    anim,
                    key(AnimProperty::PaddingRight),
                    transition,
                    padding.right,
                    &mut animating
                ),
                bottom: animate_length(
                    anim,
                    key(AnimProperty::PaddingBottom),
                    transition,
                    padding.bottom,
                    &mut animating
                ),
            });
        }

        if let Some(margin) = style.margin {
            style.margin = Some(Edges {
                left: animate_length(
                    anim,
                    key(AnimProperty::MarginLeft),
                    transition,
                    margin.left,
                    &mut animating
                ),
                top: animate_length(
                    anim,
                    key(AnimProperty::MarginTop),
                    transition,
                    margin.top,
                    &mut animating
                ),
                right: animate_length(
                    anim,
                    key(AnimProperty::MarginRight),
                    transition,
                    margin.right,
                    &mut animating
                ),
                bottom: animate_length(
                    anim,
                    key(AnimProperty::MarginBottom),
                    transition,
                    margin.bottom,
                    &mut animating
                ),
            });
        }

        if let Some(mut border) = style.border {
            border.width = animate_length(
                anim,
                key(AnimProperty::BorderWidth),
                transition,
                border.width,
                &mut animating
            );
            border.radius = animate_length(
                anim,
                key(AnimProperty::BorderRadius),
                transition,
                border.radius,
                &mut animating
            );
            style.border = Some(border);
        }

        if let Some((gx, gy)) = style.gap {
            style.gap = Some((
                animate_length(anim, key(AnimProperty::GapX), transition, gx, &mut animating),
                animate_length(anim, key(AnimProperty::GapY), transition, gy, &mut animating),
            ));
        }
    }

    animating
}
