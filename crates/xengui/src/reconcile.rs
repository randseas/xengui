// SPDX-License-Identifier: Apache-2.0
//! Fiber-style interruptible reconciler.
//!
//! Instead of a single recursive walk, reconciliation runs as an explicit
//! work stack: each stack frame owns one level of new-tree siblings still
//! waiting to be matched against the corresponding old-tree siblings.
//! `WorkLoop::perform_work` pops and processes one node at a time and can
//! be stopped after any node and resumed later with a fresh deadline -
//! the same shape as React's `workLoopConcurrent` / `performUnitOfWork`.
//!
//! Matching rules are unchanged from the previous synchronous version:
//! - A keyed new widget only matches an old widget with the same key.
//! - An unkeyed new widget matches the next unclaimed unkeyed old widget
//!   in sibling order.
//! - A match is only honored when both widgets share the same concrete
//!   type; a type change is treated as unmount + mount.
//!
//! # Safety
//! The old ("current") tree is not owned by `WorkLoop` - it keeps being
//! painted on screen for the whole duration of a possibly multi-frame
//! reconciliation pass, so `WorkLoop` only ever reads it through a raw
//! pointer. This is sound as long as the caller upholds one invariant:
//! the old tree's node addresses and structure (children, keys, types)
//! must not change while a `WorkLoop` referencing it is alive. Flipping a
//! `dirty` flag on it (as `XenRenderer::render_frame` does after
//! painting) is fine; adding, removing, reordering, or replacing nodes is
//! not.

use crate::Widget;
use smol_str::SmolStr;
use std::collections::HashMap;
use web_time::Instant;

/// One level of siblings still waiting to be matched, plus everything
/// needed to resume exactly where processing left off.
struct Frame {
    /// Owned new-tree siblings at this level. Once every sibling here has
    /// been processed, this `Vec` is handed back to whichever new-tree
    /// node's `children_mut()` slot it was taken from.
    new_siblings: Vec<Box<dyn Widget>>,
    old_siblings: *const [Box<dyn Widget>],
    keyed_old: HashMap<SmolStr, usize>,
    consumed: Vec<bool>,
    positional_cursor: usize,
    next_index: usize,
}

impl Frame {
    fn new(new_siblings: Vec<Box<dyn Widget>>, old_siblings: &[Box<dyn Widget>]) -> Self {
        let mut keyed_old = HashMap::new();
        for (i, old) in old_siblings.iter().enumerate() {
            if let Some(key) = old.get_key() {
                keyed_old.entry(key.clone()).or_insert(i);
            }
        }

        Self {
            new_siblings,
            old_siblings: old_siblings as *const [Box<dyn Widget>],
            keyed_old,
            consumed: vec![false; old_siblings.len()],
            positional_cursor: 0,
            next_index: 0,
        }
    }
}

/// Result of a single [`WorkLoop::perform_work`] call.
pub(crate) enum WorkLoopStatus {
    /// The time budget ran out before the tree was fully reconciled; call
    /// `perform_work` again with a fresh deadline to keep going.
    Yielded,
    /// Reconciliation finished; this is the fully reconciled tree, ready
    /// to be committed as the new current tree.
    Complete(Vec<Box<dyn Widget>>),
}

/// Number of nodes processed between deadline checks, so `Instant::now()`
/// isn't paid for on every single node.
const YIELD_CHECK_INTERVAL: u32 = 8;

/// An in-progress, interruptible reconciliation pass.
///
/// Create with [`WorkLoop::new`], then call [`WorkLoop::perform_work`]
/// repeatedly (once per time slice) until it reports
/// [`WorkLoopStatus::Complete`].
pub(crate) struct WorkLoop {
    stack: Vec<Frame>,
    units_since_check: u32,
}

impl WorkLoop {
    /// Begins reconciling `new_root` against `old_root`. `old_root` must
    /// stay alive and structurally unchanged (see module docs) for as
    /// long as the returned `WorkLoop` is alive.
    pub(crate) fn new(new_root: Vec<Box<dyn Widget>>, old_root: &[Box<dyn Widget>]) -> Self {
        Self {
            stack: vec![Frame::new(new_root, old_root)],
            units_since_check: 0,
        }
    }

    /// Runs until either the whole tree has been reconciled or `deadline`
    /// is reached, whichever comes first.
    pub(crate) fn perform_work(&mut self, deadline: Instant) -> WorkLoopStatus {
        loop {
            let frame_done = {
                let frame = self.stack.last().expect("root frame always present");
                frame.next_index >= frame.new_siblings.len()
            };

            if frame_done {
                let finished = self.stack.pop().expect("frame exists");
                match self.stack.last_mut() {
                    Some(parent) => {
                        let parent_idx = parent.next_index - 1;
                        if let Some(slot) = parent.new_siblings[parent_idx].children_mut() {
                            *slot = finished.new_siblings;
                        }
                    }
                    None => {
                        return WorkLoopStatus::Complete(finished.new_siblings);
                    }
                }
                continue;
            }

            self.process_one_node();

            self.units_since_check += 1;
            if self.units_since_check >= YIELD_CHECK_INTERVAL {
                self.units_since_check = 0;
                if Instant::now() >= deadline {
                    return WorkLoopStatus::Yielded;
                }
            }
        }
    }

    fn process_one_node(&mut self) {
        let frame = self.stack.last_mut().expect("root frame always present");
        let idx = frame.next_index;
        frame.next_index += 1;

        let Some(old_idx) = Self::find_match(frame, idx) else {
            return;
        };
        if frame.consumed[old_idx] {
            // Duplicate key in the old tree: first match wins, rest mount fresh.
            return;
        }

        // SAFETY: see module-level safety invariant.
        let old_siblings: &[Box<dyn Widget>] = unsafe { &*frame.old_siblings };
        let old_node = &old_siblings[old_idx];

        if frame.new_siblings[idx].as_any().type_id() != old_node.as_any().type_id() {
            return;
        }

        frame.consumed[old_idx] = true;

        let new_node = &mut frame.new_siblings[idx];
        let interaction_changed = new_node.transfer_interaction_state(old_node.as_ref());
        new_node.after_interaction_transfer();

        if !interaction_changed && new_node.content_eq(old_node.as_ref()) {
            new_node.transfer_measured_state(old_node.as_ref());
            new_node.set_dirty(false);
        }

        let old_children: *const [Box<dyn Widget>] = old_node.children() as *const _;

        if let Some(child_slot) = frame.new_siblings[idx].children_mut()
            && !child_slot.is_empty() {
                let taken_children = std::mem::take(child_slot);
                // SAFETY: `old_children` is derived from the same frozen
                // old tree this WorkLoop was constructed with.
                let old_children: &[Box<dyn Widget>] = unsafe { &*old_children };
                self.stack.push(Frame::new(taken_children, old_children));
            }
    }

    fn find_match(frame: &mut Frame, idx: usize) -> Option<usize> {
        let key = frame.new_siblings[idx].get_key().cloned();
        if let Some(key) = key {
            return frame.keyed_old.get(&key).copied();
        }

        // SAFETY: see module-level safety invariant.
        let old_siblings: &[Box<dyn Widget>] = unsafe { &*frame.old_siblings };
        while frame.positional_cursor < old_siblings.len() {
            let candidate = frame.positional_cursor;
            frame.positional_cursor += 1;
            if frame.consumed[candidate] || old_siblings[candidate].get_key().is_some() {
                continue;
            }
            return Some(candidate);
        }
        None
    }
}
