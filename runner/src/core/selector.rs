//! Deterministic selection logic for the task tree.

use crate::tree::Node;

/// Find the first leaf with `passes=false` via depth-first traversal.
///
/// Returns `None` if all leaves pass (tree is complete).
pub fn leftmost_open_leaf(node: &Node) -> Option<&Node> {
    if node.children.is_empty() {
        return if node.passes { None } else { Some(node) };
    }

    for child in &node.children {
        if let Some(found) = leftmost_open_leaf(child) {
            return Some(found);
        }
    }

    None
}

/// Returns true if a leaf is considered stuck (attempts exhausted, not passed).
pub fn is_stuck(node: &Node) -> bool {
    !node.passes && node.attempts >= node.max_attempts
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tree::Node;

    #[test]
    fn is_stuck_requires_attempts_maxed_and_not_passed() {
        let node = Node {
            id: "n".to_string(),
            order: 0,
            title: "N".to_string(),
            goal: "G".to_string(),
            acceptance: Vec::new(),
            passes: false,
            attempts: 2,
            max_attempts: 2,
            children: Vec::new(),
        };
        assert!(is_stuck(&node));
    }
}
