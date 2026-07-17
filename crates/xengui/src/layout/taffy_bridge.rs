// SPDX-License-Identifier: Apache-2.0
use crate::{
    AlignItems as XAlign,
    Display as XDisplay,
    FlexDirection as XFlexDir,
    FlexWrap as XFlexWrap,
    JustifyContent as XJustify,
    Overflow as XOverflow,
    Position as XPosition,
    Style,
};
use taffy::prelude::*;
use taffy::style::Style as TaffyStyle;

fn dim<T>(l: crate::Length, scale_factor: f32) -> T
    where T: taffy::style_helpers::FromLength + taffy::style_helpers::FromPercent
{
    match l {
        crate::Length::Px(v) => length(v * scale_factor),
        crate::Length::Percent(v) => percent(v / 100.0),
    }
}

pub fn style_to_taffy(style: &Style, scale_factor: f32) -> TaffyStyle {
    let mut t = TaffyStyle {
        display: match style.display.unwrap_or_default() {
            XDisplay::Flex => taffy::style::Display::Flex,
            XDisplay::Grid => taffy::style::Display::Grid,
            XDisplay::Block => taffy::style::Display::Block,
            XDisplay::None => taffy::style::Display::None,
        },
        position: match style.position.unwrap_or_default() {
            XPosition::Relative => taffy::style::Position::Relative,
            XPosition::Absolute => taffy::style::Position::Absolute,
        },
        ..Default::default()
    };

    if let Some(ox) = style.overflow_x {
        t.overflow.x = map_overflow(ox);
    }
    if let Some(oy) = style.overflow_y {
        t.overflow.y = map_overflow(oy);
    }

    if let Some(dir) = style.flex_direction {
        t.flex_direction = match dir {
            XFlexDir::Row => taffy::style::FlexDirection::Row,
            XFlexDir::RowReverse => taffy::style::FlexDirection::RowReverse,
            XFlexDir::Column => taffy::style::FlexDirection::Column,
            XFlexDir::ColumnReverse => taffy::style::FlexDirection::ColumnReverse,
        };
    }

    if let Some(wrap) = style.flex_wrap {
        t.flex_wrap = match wrap {
            XFlexWrap::NoWrap => taffy::style::FlexWrap::NoWrap,
            XFlexWrap::Wrap => taffy::style::FlexWrap::Wrap,
            XFlexWrap::WrapReverse => taffy::style::FlexWrap::WrapReverse,
        };
    }

    if let Some(v) = style.flex_grow {
        t.flex_grow = v;
    }
    if let Some(v) = style.flex_shrink {
        t.flex_shrink = v;
    }
    if let Some(v) = style.flex_basis {
        t.flex_basis = dim(v, scale_factor);
    }

    if let Some(align) = style.align_items {
        t.align_items = Some(map_align(align));
    }
    if let Some(align) = style.align_self {
        t.align_self = Some(map_align(align));
    }
    if let Some(j) = style.justify_content {
        t.justify_content = Some(map_justify(j));
    }
    if let Some(j) = style.align_content {
        t.align_content = Some(map_justify(j));
    }

    if let Some((gx, gy)) = style.gap {
        t.gap = Size {
            width: dim(gx, scale_factor),
            height: dim(gy, scale_factor),
        };
    }

    if let Some(size) = &style.size {
        if let Some(w) = size.width {
            t.size.width = dim(w, scale_factor);
        }
        if let Some(h) = size.height {
            t.size.height = dim(h, scale_factor);
        }
    }
    if let Some(size) = &style.min_size {
        if let Some(w) = size.width {
            t.min_size.width = dim(w, scale_factor);
        }
        if let Some(h) = size.height {
            t.min_size.height = dim(h, scale_factor);
        }
    }
    if let Some(size) = &style.max_size {
        if let Some(w) = size.width {
            t.max_size.width = dim(w, scale_factor);
        }
        if let Some(h) = size.height {
            t.max_size.height = dim(h, scale_factor);
        }
    }

    if let Some(p) = &style.padding {
        t.padding = Rect {
            left: dim(p.left, scale_factor),
            right: dim(p.right, scale_factor),
            top: dim(p.top, scale_factor),
            bottom: dim(p.bottom, scale_factor),
        };
    }

    if let Some(m) = &style.margin {
        t.margin = Rect {
            left: dim(m.left, scale_factor),
            right: dim(m.right, scale_factor),
            top: dim(m.top, scale_factor),
            bottom: dim(m.bottom, scale_factor),
        };
    }

    if let Some(b) = &style.border {
        let w = length(b.width.value() * scale_factor);
        t.border = Rect {
            left: w,
            right: w,
            top: w,
            bottom: w,
        };
    }

    if let Some(cols) = &style.grid_template_columns {
        t.grid_template_columns = cols
            .iter()
            .map(|track| map_grid_track(track, scale_factor))
            .collect();
    }
    if let Some(rows) = &style.grid_template_rows {
        t.grid_template_rows = rows
            .iter()
            .map(|track| map_grid_track(track, scale_factor))
            .collect();
    }
    if let Some(p) = style.grid_column {
        t.grid_column = Line {
            start: line(p.start),
            end: line(p.end),
        };
    }
    if let Some(p) = style.grid_row {
        t.grid_row = Line {
            start: line(p.start),
            end: line(p.end),
        };
    }

    t
}

fn map_align(align: XAlign) -> AlignItems {
    match align {
        XAlign::Stretch => AlignItems::STRETCH,
        XAlign::Start => AlignItems::START,
        XAlign::End => AlignItems::END,
        XAlign::Center => AlignItems::CENTER,
        XAlign::Baseline => AlignItems::BASELINE,
    }
}

fn map_justify(j: XJustify) -> JustifyContent {
    match j {
        XJustify::Start => JustifyContent::START,
        XJustify::End => JustifyContent::END,
        XJustify::Center => JustifyContent::CENTER,
        XJustify::SpaceBetween => JustifyContent::SPACE_BETWEEN,
        XJustify::SpaceAround => JustifyContent::SPACE_AROUND,
        XJustify::SpaceEvenly => JustifyContent::SPACE_EVENLY,
    }
}

fn map_grid_track(
    track: &crate::GridTrack,
    scale_factor: f32
) -> taffy::style::GridTemplateComponent<String> {
    let sizing_function = match track {
        crate::GridTrack::Px(px) => length(*px * scale_factor),
        crate::GridTrack::Fr(f) => fr(*f),
        crate::GridTrack::Auto => auto(),
    };
    taffy::style::GridTemplateComponent::Single(sizing_function)
}

fn map_overflow(overflow: XOverflow) -> taffy::style::Overflow {
    match overflow {
        XOverflow::Visible => taffy::style::Overflow::Visible,
        XOverflow::Hidden => taffy::style::Overflow::Hidden,
        XOverflow::Scroll => taffy::style::Overflow::Scroll,
        XOverflow::Auto => taffy::style::Overflow::Scroll,
    }
}
