//! Task tree data structures for the goal-driven agent loop.
//!
//! The tree represents a hierarchy of tasks where each node tracks execution
//! state (attempts, passes) and ordering for deterministic traversal.

use serde::{Deserialize, Serialize};

/// A node in the task tree, representing a goal or sub-goal.
///
/// Nodes form a tree where leaves are executable tasks. The runner traverses
/// depth-first in `(order, id)` order to find the next open leaf (passes=false).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Node {
    /// Unique identifier within the tree; used for duplicate detection.
    pub id: String,
    /// Sort key for sibling ordering; lower values processed first.
    pub order: i64,
    pub title: String,
    /// What this node aims to accomplish.
    pub goal: String,
    /// Criteria that must hold for `passes` to become true.
    pub acceptance: Vec<String>,
    /// Whether this node's goal has been achieved.
    pub passes: bool,
    /// How many times execution has been attempted.
    pub attempts: u32,
    /// Upper bound on attempts before the node is considered failed.
    pub max_attempts: u32,
    pub children: Vec<Node>,
}

impl Node {
    /// Recursively sorts children by `(order, id)` for deterministic traversal.
    pub fn sort_children(&mut self) {
        self.children
            .sort_by(|a, b| a.order.cmp(&b.order).then_with(|| a.id.cmp(&b.id)));
        for child in &mut self.children {
            child.sort_children();
        }
    }
}

/// Returns a minimal root node for bootstrapping a new task tree.
pub fn default_tree() -> Node {
    default_tree_with_max_attempts(3)
}

/// Returns a minimal root node for bootstrapping a new task tree.
pub fn default_tree_with_max_attempts(max_attempts: u32) -> Node {
    Node {
        id: "root".to_string(),
        order: 0,
        title: "Root".to_string(),
        goal: "Top-level goal (see .runner/GOAL.md)".to_string(),
        acceptance: Vec::new(),
        passes: false,
        attempts: 0,
        max_attempts,
        children: Vec::new(),
    }
}
