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
