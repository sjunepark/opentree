//! Selection helpers for `runner select` and `runner step`.

use std::path::Path;

use anyhow::{Context, Result, anyhow};

use crate::core::path::node_path;
use crate::core::selector::{is_stuck, leftmost_open_leaf};
use crate::io::init::RunnerPaths;
use crate::io::tree_store::load_tree;
use crate::tree::Node;

/// Structured selection outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelectOutcome {
    /// Tree is complete (no open leaves).
    Complete,
    /// Open leaf selected.
    Open(SelectedLeaf),
    /// Selected leaf is stuck (attempts exhausted).
    Stuck(SelectedLeaf),
}

/// Minimal selected leaf metadata for reporting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectedLeaf {
    pub id: String,
    pub path: String,
    pub attempts: u32,
    pub max_attempts: u32,
}

/// Select the next leaf from an in-memory tree.
pub fn select_leaf(tree: &Node) -> Result<SelectOutcome> {
    let selected = match leftmost_open_leaf(tree) {
        Some(node) => node,
        None => return Ok(SelectOutcome::Complete),
    };
    let path =
        node_path(tree, &selected.id).ok_or_else(|| anyhow!("selected node path not found"))?;
    let leaf = SelectedLeaf {
        id: selected.id.clone(),
        path,
        attempts: selected.attempts,
        max_attempts: selected.max_attempts,
    };
    if is_stuck(selected) {
        return Ok(SelectOutcome::Stuck(leaf));
    }
    Ok(SelectOutcome::Open(leaf))
}

/// Load tree from disk and select the next leaf.
pub fn select_from_root(root: &Path) -> Result<SelectOutcome> {
    let paths = RunnerPaths::new(root);
    let tree = load_tree(&paths.schema_path, &paths.tree_path)
        .with_context(|| "load tree for selection")?;
    select_leaf(&tree)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::load_tree_fixture;
    use crate::tree::default_tree;

    #[test]
    fn select_returns_open_leaf() {
        let tree = load_tree_fixture("tree_with_passed_node").expect("fixture");
        let outcome = select_leaf(&tree).expect("select");
        assert_eq!(
            outcome,
            SelectOutcome::Open(SelectedLeaf {
                id: "open".to_string(),
                path: "root/open".to_string(),
                attempts: 0,
                max_attempts: 3,
            })
        );
    }

    #[test]
    fn select_returns_stuck_leaf() {
        let tree = load_tree_fixture("tree_with_stuck_leaf").expect("fixture");
        let outcome = select_leaf(&tree).expect("select");
        assert_eq!(
            outcome,
            SelectOutcome::Stuck(SelectedLeaf {
                id: "stuck".to_string(),
                path: "root/stuck".to_string(),
                attempts: 2,
                max_attempts: 2,
            })
        );
    }

    #[test]
    fn select_returns_complete_when_no_open_leaf() {
        let mut tree = default_tree();
        tree.passes = true;
        let outcome = select_leaf(&tree).expect("select");
        assert_eq!(outcome, SelectOutcome::Complete);
    }
}
