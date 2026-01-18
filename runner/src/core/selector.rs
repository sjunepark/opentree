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
    use crate::test_support::{leaf, node_with_children};

    /// Verifies `leftmost_open_leaf` depth-first traversal in a single test.
    ///
    /// Cases covered:
    /// - Single unpassed leaf → returns it
    /// - All passed → returns None
    /// - Leftmost-first ordering (respects `order` field)
    /// - Deep nesting → returns deepest leftmost
    /// - Wide tree → returns first by order
    #[test]
    fn leftmost_open_leaf_traversal_cases() {
        // Case 1: Single unpassed leaf → returns it
        let single = leaf("a", 0, false);
        assert_eq!(leftmost_open_leaf(&single).map(|n| &n.id), Some(&"a".into()));

        // Case 2: Single passed leaf → returns None
        let passed = leaf("a", 0, true);
        assert!(leftmost_open_leaf(&passed).is_none());

        // Case 3: All passed → returns None
        let all_passed = node_with_children("root", 0, vec![leaf("a", 0, true), leaf("b", 1, true)]);
        assert!(leftmost_open_leaf(&all_passed).is_none());

        // Case 4: Respects order field (lower order first)
        let ordered = node_with_children(
            "root",
            0,
            vec![
                leaf("second", 1, false),
                leaf("first", 0, false), // lower order, should be selected
            ],
        );
        let mut sorted = ordered.clone();
        sorted.sort_children();
        assert_eq!(
            leftmost_open_leaf(&sorted).map(|n| &n.id),
            Some(&"first".into())
        );

        // Case 5: Deep nesting → returns deepest leftmost
        let deep = node_with_children(
            "root",
            0,
            vec![node_with_children(
                "branch",
                0,
                vec![node_with_children("subbranch", 0, vec![leaf("deep", 0, false)])],
            )],
        );
        assert_eq!(leftmost_open_leaf(&deep).map(|n| &n.id), Some(&"deep".into()));

        // Case 6: Wide tree with mixed passes → returns first open by order
        let wide = node_with_children(
            "root",
            0,
            vec![
                leaf("passed", 0, true),
                leaf("open", 1, false), // first open
                leaf("also-open", 2, false),
            ],
        );
        assert_eq!(leftmost_open_leaf(&wide).map(|n| &n.id), Some(&"open".into()));
    }

    /// Verifies `is_stuck` edge cases beyond the basic "attempts maxed + not passed".
    ///
    /// Cases covered:
    /// - attempts maxed + not passed → stuck
    /// - passes=true → not stuck (even if attempts maxed)
    /// - attempts < max_attempts → not stuck
    #[test]
    fn is_stuck_edge_cases() {
        use crate::test_support::node_with_attempts;

        // Case 1: attempts maxed + not passed → stuck
        let maxed = node_with_attempts("n", 0, 3, 3);
        assert!(is_stuck(&maxed));

        // Case 2: passes=true → not stuck (even if attempts maxed)
        let mut passed = node_with_attempts("n", 0, 3, 3);
        passed.passes = true;
        assert!(!is_stuck(&passed));

        // Case 3: attempts < max_attempts → not stuck
        let has_attempts = node_with_attempts("n", 0, 1, 3);
        assert!(!is_stuck(&has_attempts));

        // Case 4: attempts == 0 → not stuck
        let no_attempts = node_with_attempts("n", 0, 0, 3);
        assert!(!is_stuck(&no_attempts));
    }
}
