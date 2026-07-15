// SPDX-License-Identifier: Apache-2.0
use crate::Widget;
use smol_str::SmolStr;
use std::collections::HashMap;

/// Diffs a freshly rendered widget tree against the previous frame's tree and
/// transfers persistent state (interaction, measured layout, dirty flag)
/// from matching old widgets into the new ones in place.
///
/// Matching rules:
/// - A widget with a key is matched only against an old widget with the same
///   key, regardless of position, so it survives reordering.
/// - A widget without a key is matched against the next unclaimed unkeyed
///   old widget in sibling order.
/// - A match is only honored when both widgets share the same concrete
///   type; a type change at a given key/position is treated as unmount +
///   mount rather than an update, so state never leaks across widget kinds.
pub(crate) fn reconcile(new_nodes: &mut [Box<dyn Widget>], old_nodes: &[Box<dyn Widget>]) {
    let mut keyed_old: HashMap<&SmolStr, usize> = HashMap::new();
    let mut consumed = vec![false; old_nodes.len()];
    for (i, old) in old_nodes.iter().enumerate() {
        if let Some(key) = old.get_key() {
            keyed_old.entry(key).or_insert(i);
        }
    }

    let mut positional_cursor = 0usize;

    for new_node in new_nodes.iter_mut() {
        let matched_idx = match new_node.get_key() {
            Some(key) => keyed_old.get(key).copied(),
            None => {
                let mut found = None;
                while positional_cursor < old_nodes.len() {
                    let candidate = positional_cursor;
                    positional_cursor += 1;
                    if consumed[candidate] || old_nodes[candidate].get_key().is_some() {
                        continue;
                    }
                    found = Some(candidate);
                    break;
                }
                found
            }
        };

        let Some(old_idx) = matched_idx else {
            continue;
        };
        if consumed[old_idx] {
            // Duplicate key in the old tree: first match wins, rest mount fresh.
            continue;
        }

        let old_node = &old_nodes[old_idx];

        if new_node.as_any().type_id() != old_node.as_any().type_id() {
            continue;
        }

        consumed[old_idx] = true;

        let interaction_changed = new_node.transfer_interaction_state(old_node.as_ref());
        new_node.after_interaction_transfer();

        if !interaction_changed && new_node.content_eq(old_node.as_ref()) {
            new_node.transfer_measured_state(old_node.as_ref());
            new_node.set_dirty(false);
        }

        if let Some(new_children) = new_node.children_mut() {
            reconcile(new_children, old_node.children());
        }
    }
}
