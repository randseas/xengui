// SPDX-License-Identifier: Apache-2.0
use crate::Widget;

pub(crate) fn reconcile(new_nodes: &mut [Box<dyn Widget>], old_nodes: &[Box<dyn Widget>]) {
    for (i, new_node) in new_nodes.iter_mut().enumerate() {
        let matched_old = match new_node.get_key() {
            Some(key) => old_nodes.iter().find(|old| old.get_key() == Some(key)),
            None => old_nodes.get(i),
        };

        let Some(old_node) = matched_old else {
            continue;
        };

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
