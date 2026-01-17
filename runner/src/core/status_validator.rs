//! Agent status invariants.
//!
//! Validates consistency between the agent's declared status and task-tree edits:
//! - `decomposed` requires selected node gained children.
//! - `done`/`retry` require selected node did not gain children.

#![allow(dead_code)]

use crate::core::types::AgentStatus;
use crate::tree::Node;

/// Validate agent status invariants between `prev` and `next` trees.
///
/// Returns a list of stable error messages (empty on success).
pub fn validate_status_invariants(
    prev: &Node,
    next: &Node,
    selected_id: &str,
    status: AgentStatus,
) -> Vec<String> {
    let mut errors = Vec::new();

    let prev_node = match find_node(prev, selected_id) {
        Some(node) => node,
        None => {
            errors.push(format!(
                "selected node '{}' missing in prev tree",
                selected_id
            ));
            return errors;
        }
    };

    let next_node = match find_node(next, selected_id) {
        Some(node) => node,
        None => {
            errors.push(format!(
                "selected node '{}' missing in next tree",
                selected_id
            ));
            return errors;
        }
    };

    let prev_children = prev_node.children.len();
    let next_children = next_node.children.len();
    let gained_children = next_children > prev_children;

    match status {
        AgentStatus::Decomposed if !gained_children => errors.push(format!(
            "status=decomposed but selected node '{}' did not gain children (prev={}, next={})",
            selected_id, prev_children, next_children
        )),
        AgentStatus::Done | AgentStatus::Retry if gained_children => errors.push(format!(
            "status={} but selected node '{}' gained children (prev={}, next={})",
            status_label(status),
            selected_id,
            prev_children,
            next_children
        )),
        _ => {}
    }

    errors
}

fn find_node<'a>(node: &'a Node, target_id: &str) -> Option<&'a Node> {
    if node.id == target_id {
        return Some(node);
    }

    for child in &node.children {
        if let Some(found) = find_node(child, target_id) {
            return Some(found);
        }
    }

    None
}

fn status_label(status: AgentStatus) -> &'static str {
    match status {
        AgentStatus::Done => "done",
        AgentStatus::Retry => "retry",
        AgentStatus::Decomposed => "decomposed",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{leaf, node_with_children};

    /// Decomposed status requires children to be added (that's the point of decomposition).
    #[test]
    fn decomposed_requires_children_added() {
        let prev = node_with_children("root", 0, vec![leaf("a", 0, false)]);
        let next = prev.clone();
        let errors = validate_status_invariants(&prev, &next, "a", AgentStatus::Decomposed);
        assert_eq!(
            errors,
            vec![
                "status=decomposed but selected node 'a' did not gain children (prev=0, next=0)"
                    .to_string()
            ]
        );
    }

    /// Done status forbids adding children (work is complete, not decomposed).
    #[test]
    fn done_rejects_children_added() {
        let prev = node_with_children("root", 0, vec![leaf("a", 0, false)]);
        let mut next = prev.clone();
        next.children[0].children.push(leaf("a1", 0, false));
        let errors = validate_status_invariants(&prev, &next, "a", AgentStatus::Done);
        assert_eq!(
            errors,
            vec!["status=done but selected node 'a' gained children (prev=0, next=1)".to_string()]
        );
    }

    /// Retry status forbids adding children (retry = try again, not decompose).
    #[test]
    fn retry_rejects_children_added() {
        let prev = node_with_children("root", 0, vec![leaf("a", 0, false)]);
        let mut next = prev.clone();
        next.children[0].children.push(leaf("a1", 0, false));
        let errors = validate_status_invariants(&prev, &next, "a", AgentStatus::Retry);
        assert_eq!(
            errors,
            vec!["status=retry but selected node 'a' gained children (prev=0, next=1)".to_string()]
        );
    }

    /// Decomposed status with children added is valid (happy path).
    #[test]
    fn decomposed_allows_children_added() {
        let prev = node_with_children("root", 0, vec![leaf("a", 0, false)]);
        let mut next = prev.clone();
        next.children[0].children.push(leaf("a1", 0, false));
        assert!(validate_status_invariants(&prev, &next, "a", AgentStatus::Decomposed).is_empty());
    }

    /// Missing selected node in prev tree is an error (indicates bug).
    #[test]
    fn errors_when_selected_missing() {
        let prev = node_with_children("root", 0, vec![]);
        let next = prev.clone();
        let errors = validate_status_invariants(&prev, &next, "missing", AgentStatus::Done);
        assert_eq!(
            errors,
            vec!["selected node 'missing' missing in prev tree".to_string()]
        );
    }
}
