//! Passed-node immutability checks.

#![allow(dead_code)]

use crate::tree::Node;
use std::collections::HashMap;

/// Validate that nodes with `passes=true` in the previous tree remain immutable.
///
/// For each passed node in `prev`, this check enforces:
/// - The node still exists in `next`.
/// - The node's parent id is unchanged.
/// - The node is identical by value (all fields, including children).
///
/// Returns a list of stable error messages (sorted by node id).
pub fn check_passed_node_immutability(prev: &Node, next: &Node) -> Vec<String> {
    let mut passed_nodes = Vec::new();
    collect_passed_nodes(prev, None, &mut passed_nodes);
    passed_nodes.sort_by(|a, b| a.id.cmp(b.id));

    let mut next_index = HashMap::new();
    index_nodes(next, None, &mut next_index);

    let mut errors = Vec::new();
    for passed in passed_nodes {
        match next_index.get(passed.id) {
            None => errors.push(format!("passed node '{}' missing in next tree", passed.id)),
            Some(info) => {
                if info.parent_id != passed.parent_id {
                    errors.push(format!(
                        "passed node '{}' moved from parent '{}' to '{}'",
                        passed.id,
                        parent_label(passed.parent_id),
                        parent_label(info.parent_id)
                    ));
                }
                if info.node != passed.node {
                    errors.push(format!("passed node '{}' changed in next tree", passed.id));
                }
            }
        }
    }

    errors
}

struct PassedNode<'a> {
    id: &'a str,
    parent_id: Option<&'a str>,
    node: &'a Node,
}

struct NodeInfo<'a> {
    parent_id: Option<&'a str>,
    node: &'a Node,
}

fn collect_passed_nodes<'a>(
    node: &'a Node,
    parent_id: Option<&'a str>,
    output: &mut Vec<PassedNode<'a>>,
) {
    if node.passes {
        output.push(PassedNode {
            id: node.id.as_str(),
            parent_id,
            node,
        });
    }

    for child in &node.children {
        collect_passed_nodes(child, Some(node.id.as_str()), output);
    }
}

fn index_nodes<'a>(
    node: &'a Node,
    parent_id: Option<&'a str>,
    index: &mut HashMap<&'a str, NodeInfo<'a>>,
) {
    index.insert(node.id.as_str(), NodeInfo { parent_id, node });

    for child in &node.children {
        index_nodes(child, Some(node.id.as_str()), index);
    }
}

fn parent_label(parent_id: Option<&str>) -> String {
    parent_id.unwrap_or("<root>").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{leaf, node, node_with_children};

    /// Passed nodes that remain identical should not trigger errors.
    #[test]
    fn immutability_allows_identical_passed_nodes() {
        let prev = node_with_children("root", 0, vec![leaf("a", 0, true)]);
        let next = node_with_children("root", 0, vec![leaf("a", 0, true)]);
        assert!(check_passed_node_immutability(&prev, &next).is_empty());
    }

    /// Deleting a passed node must be reported as an error.
    #[test]
    fn immutability_reports_missing_passed_node() {
        let prev = node_with_children("root", 0, vec![leaf("a", 0, true)]);
        let next = node_with_children("root", 0, vec![leaf("b", 0, false)]);
        let errors = check_passed_node_immutability(&prev, &next);
        assert!(errors.iter().any(|err| err.contains("'a'")));
        assert!(errors.iter().any(|err| err.contains("missing")));
    }

    /// Moving a passed node to a different parent must be reported.
    #[test]
    fn immutability_reports_parent_move() {
        let prev = node_with_children(
            "root",
            0,
            vec![node_with_children("p1", 0, vec![leaf("a", 0, true)])],
        );
        let next = node_with_children(
            "root",
            0,
            vec![node_with_children("p2", 0, vec![leaf("a", 0, true)])],
        );
        let errors = check_passed_node_immutability(&prev, &next);
        assert!(errors.iter().any(|err| err.contains("'a'")));
        assert!(errors.iter().any(|err| err.contains("moved")));
    }

    /// Modifying any field of a passed node must be reported.
    #[test]
    fn immutability_reports_changed_node() {
        let prev = node_with_children("root", 0, vec![leaf("a", 0, true)]);
        let mut changed = node("a", 0);
        changed.passes = true;
        changed.title = "mutated".to_string();
        let next = node_with_children("root", 0, vec![changed]);
        let errors = check_passed_node_immutability(&prev, &next);
        assert!(errors.iter().any(|err| err.contains("'a'")));
        assert!(errors.iter().any(|err| err.contains("changed")));
    }

    /// Open nodes (passes=false) can be freely edited or removed.
    #[test]
    fn immutability_allows_open_node_edits_and_removals() {
        let prev = node_with_children("root", 0, vec![leaf("a", 0, false)]);
        let mut edited = node("a", 0);
        edited.goal = "updated goal".to_string();
        let next = node_with_children("root", 0, vec![edited, leaf("b", 1, false)]);
        assert!(check_passed_node_immutability(&prev, &next).is_empty());

        let next_removed = node_with_children("root", 0, vec![leaf("b", 0, false)]);
        assert!(check_passed_node_immutability(&prev, &next_removed).is_empty());
    }
}
