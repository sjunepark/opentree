//! Validation for restrictions on adding new children to nodes.
//!
//! We allow agents to edit open nodes, but restrict *where* new children can be
//! introduced. This prevents decomposing future goals ahead of time and keeps
//! decomposition localized to the selected node.

#![allow(dead_code)]

use crate::tree::Node;
use std::collections::{BTreeMap, BTreeSet};

/// Validate that no node gained new children, except optionally `allowed_parent_id`.
///
/// "New child" is defined by child id set difference between `prev` and `next`
/// for each parent that exists in both trees.
///
/// Returns a list of stable error messages (sorted by parent id).
pub fn validate_child_additions_restricted(
    prev: &Node,
    next: &Node,
    allowed_parent_id: Option<&str>,
) -> Vec<String> {
    let prev_index = index_child_ids(prev);
    let next_index = index_child_ids(next);

    let mut errors = Vec::new();
    for (parent_id, next_children) in next_index {
        let Some(prev_children) = prev_index.get(&parent_id) else {
            continue;
        };

        let mut added: Vec<String> = next_children.difference(prev_children).cloned().collect();
        if added.is_empty() {
            continue;
        }
        added.sort();

        if allowed_parent_id == Some(parent_id.as_str()) {
            continue;
        }
        errors.push(format!(
            "node '{}' gained new children: {}",
            parent_id,
            added.join(", ")
        ));
    }

    errors
}

fn index_child_ids(root: &Node) -> BTreeMap<String, BTreeSet<String>> {
    let mut index = BTreeMap::new();
    index_child_ids_inner(root, &mut index);
    index
}

fn index_child_ids_inner(node: &Node, index: &mut BTreeMap<String, BTreeSet<String>>) {
    let children = node.children.iter().map(|c| c.id.clone()).collect();
    index.insert(node.id.clone(), children);
    for child in &node.children {
        index_child_ids_inner(child, index);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{node, node_with_children};

    #[test]
    fn rejects_new_children_on_other_nodes() {
        let prev = node_with_children(
            "root",
            0,
            vec![node_with_children("a", 0, Vec::new()), node("b", 1)],
        );
        let mut next = prev.clone();
        next.children[0].children.push(node("a1", 0));

        let errors = validate_child_additions_restricted(&prev, &next, Some("b"));
        assert_eq!(errors, vec!["node 'a' gained new children: a1".to_string()]);
    }

    #[test]
    fn allows_new_children_only_on_allowed_parent() {
        let prev = node_with_children("root", 0, vec![node_with_children("a", 0, Vec::new())]);
        let mut next = prev.clone();
        next.children[0].children.push(node("a1", 0));
        next.children[0].children.push(node("a2", 1));

        let errors = validate_child_additions_restricted(&prev, &next, Some("a"));
        assert!(errors.is_empty());
    }

    #[test]
    fn detects_replacements_even_when_child_count_is_unchanged() {
        let prev = node_with_children(
            "root",
            0,
            vec![node_with_children("a", 0, vec![node("x", 0)])],
        );
        let mut next = prev.clone();
        next.children[0].children.clear();
        next.children[0].children.push(node("y", 0));

        let errors = validate_child_additions_restricted(&prev, &next, None);
        assert_eq!(errors, vec!["node 'a' gained new children: y".to_string()]);
    }
}
