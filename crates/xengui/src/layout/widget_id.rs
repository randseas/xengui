// SPDX-License-Identifier: Apache-2.0

/// A stable identifier assigned to a widget by the layout system.
///
/// This identifier is used internally by the framework for layout,
/// measurement, caching, focus management, and future reconciliation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct WidgetId(pub(crate) u64);

impl WidgetId {
    /// Creates a new widget identifier.
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    /// Returns the underlying numeric identifier.
    #[inline]
    pub const fn get(self) -> u64 {
        self.0
    }
}

use std::sync::atomic::{ AtomicU64, Ordering };

impl WidgetId {
    /// Fresh identifier for a newly-constructed widget instance; carry it
    /// forward via `transfer_measured_state` so animation keys stay stable
    /// across reconciliation.
    pub fn new_unique() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(1);
        Self(NEXT.fetch_add(1, Ordering::Relaxed))
    }
}
