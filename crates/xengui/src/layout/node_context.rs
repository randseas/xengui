// SPDX-License-Identifier: Apache-2.0

use crate::{ MeasureResult, WidgetId };

/// Context associated with a layout node.
///
/// The layout engine owns this structure and passes it to the intrinsic
/// measurement system. Widgets never access it directly.
#[derive(Clone, Debug)]
pub struct NodeContext {
    /// Stable identifier of the associated widget.
    pub widget_id: WidgetId,

    /// Cached intrinsic measurement.
    ///
    /// The cache is invalidated whenever the widget becomes dirty.
    pub measure: Option<MeasureResult>,
}

impl NodeContext {
    /// Creates a new node context.
    pub const fn new(widget_id: WidgetId) -> Self {
        Self {
            widget_id,
            measure: None,
        }
    }

    /// Returns the cached measurement, if available.
    #[inline]
    pub const fn measure(&self) -> Option<MeasureResult> {
        self.measure
    }

    /// Stores the latest measurement.
    #[inline]
    pub fn set_measure(&mut self, measure: MeasureResult) {
        self.measure = Some(measure);
    }

    /// Invalidates the cached measurement.
    #[inline]
    pub fn invalidate_measure(&mut self) {
        self.measure = None;
    }
}
