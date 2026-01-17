//! Test-only helpers for constructing task tree nodes.

use crate::tree::Node;

/// Create a deterministic node with default fields and no children.
pub fn node(id: &str, order: i64) -> Node {
    Node {
        id: id.to_string(),
        order,
        title: format!("{} title", id),
        goal: format!("{} goal", id),
        acceptance: Vec::new(),
        passes: false,
        attempts: 0,
        max_attempts: 3,
        children: Vec::new(),
    }
}

/// Create a deterministic leaf node with explicit `passes`.
pub fn leaf(id: &str, order: i64, passes: bool) -> Node {
    let mut node = node(id, order);
    node.passes = passes;
    node
}

/// Create a node with children using deterministic defaults.
pub fn node_with_children(id: &str, order: i64, children: Vec<Node>) -> Node {
    Node {
        children,
        ..node(id, order)
    }
}

/// Create a node with explicit attempt state (useful for invariant tests).
pub fn node_with_attempts(id: &str, order: i64, attempts: u32, max_attempts: u32) -> Node {
    let mut node = node(id, order);
    node.attempts = attempts;
    node.max_attempts = max_attempts;
    node
}
