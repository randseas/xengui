// SPDX-License-Identifier: Apache-2.0
use crate::{
    AlignItems as XAlign, Display as XDisplay, FlexDirection as XFlexDir, FlexWrap as XFlexWrap,
    JustifyContent as XJustify, Position as XPosition, Style,
};
use taffy::prelude::*;
use taffy::style::Style as TaffyStyle;

/// `crate::Length` (Px/Percent) değerini, atandığı taffy alanının tipine
/// (Dimension / LengthPercentage / LengthPercentageAuto) çevirir.
fn dim<T>(l: crate::Length) -> T
where
    T: taffy::style_helpers::FromLength + taffy::style_helpers::FromPercent,
{
    match l {
        crate::Length::Px(v) => length(v),
        crate::Length::Percent(v) => percent(v / 100.0), // taffy 0.0..=1.0 bekler
    }
}

pub fn style_to_taffy(style: &Style) -> TaffyStyle {
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
        t.flex_basis = dim(v);
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
            width: dim(gx),
            height: dim(gy),
        };
    }

    if let Some(size) = &style.size {
        t.size = Size {
            width: dim(size.width),
            height: dim(size.height),
        };
    }
    if let Some(size) = &style.min_size {
        t.min_size = Size {
            width: dim(size.width),
            height: dim(size.height),
        };
    }
    if let Some(size) = &style.max_size {
        t.max_size = Size {
            width: dim(size.width),
            height: dim(size.height),
        };
    }

    if let Some(p) = &style.padding {
        t.padding = Rect {
            left: dim(p.left),
            right: dim(p.right),
            top: dim(p.top),
            bottom: dim(p.bottom),
        };
    }

    if let Some(m) = &style.margin {
        t.margin = Rect {
            left: dim(m.left),
            right: dim(m.right),
            top: dim(m.top),
            bottom: dim(m.bottom),
        };
    }

    // Görsel border zaten rect_pipeline'da SDF ile çiziliyor; burada border
    // genişliğini taffy'nin box-model hesaplamasına da veriyoruz ki içerik
    // border'ın altında ezilmesin (CSS border-box davranışı).
    if let Some(b) = &style.border {
        let w = length(b.width.value());
        t.border = Rect {
            left: w,
            right: w,
            top: w,
            bottom: w,
        };
    }

    if let Some(cols) = &style.grid_template_columns {
        t.grid_template_columns = cols.iter().map(map_grid_track).collect();
    }
    if let Some(rows) = &style.grid_template_rows {
        t.grid_template_rows = rows.iter().map(map_grid_track).collect();
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

fn map_grid_track(track: &crate::GridTrack) -> taffy::style::GridTemplateComponent<String> {
    let sizing_function = match track {
        crate::GridTrack::Px(px) => length(*px),
        crate::GridTrack::Fr(f) => fr(*f),
        crate::GridTrack::Auto => auto(),
    };
    taffy::style::GridTemplateComponent::Single(sizing_function)
}
