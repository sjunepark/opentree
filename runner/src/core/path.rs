//! Helpers for rendering deterministic node paths.

use crate::tree::Node;

/// Return the `/`-separated id path to `target_id`, rooted at `root`.
pub fn node_path(root: &Node, target_id: &str) -> Option<String> {
    let mut path = Vec::new();
    if node_path_inner(root, target_id, &mut path) {
        return Some(path.join("/"));
    }
    None
}

fn node_path_inner(node: &Node, target_id: &str, path: &mut Vec<String>) -> bool {
    path.push(node.id.clone());
    if node.id == target_id {
        return true;
    }
    for child in &node.children {
        if node_path_inner(child, target_id, path) {
            return true;
        }
    }
    path.pop();
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tree::Node;

    #[test]
    fn node_path_returns_root_for_root_id() {
        let root = Node {
            id: "root".to_string(),
            order: 0,
            title: "Root".to_string(),
            goal: "Root".to_string(),
            acceptance: Vec::new(),
            next: crate::tree::NodeNext::Decompose,
            passes: false,
            attempts: 0,
            max_attempts: 3,
            children: Vec::new(),
        };

        assert_eq!(node_path(&root, "root"), Some("root".to_string()));
    }
}
