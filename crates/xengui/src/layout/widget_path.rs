// SPDX-License-Identifier: Apache-2.0
use crate::Widget;
use std::fmt::Write;

const ROOT_PATH: &str = "root";

pub(crate) type PathCheckpoint = usize;

pub(crate) struct WidgetPath {
    buf: String,
}

impl WidgetPath {
    pub(crate) fn new() -> Self {
        Self {
            buf: String::from(ROOT_PATH),
        }
    }

    #[inline]
    pub(crate) fn as_str(&self) -> &str {
        &self.buf
    }

    #[inline]
    pub(crate) fn checkpoint(&self) -> PathCheckpoint {
        self.buf.len()
    }

    #[inline]
    pub(crate) fn restore(&mut self, checkpoint: PathCheckpoint) {
        self.buf.truncate(checkpoint);
    }

    pub(crate) fn push(&mut self, widget: &dyn Widget, index: usize) {
        self.buf.push('.');

        if let Some(key) = widget.get_key() {
            self.buf.push_str(key);
        } else {
            self.buf.push('#');
            write!(&mut self.buf, "{index}").unwrap();
        }
    }
}
