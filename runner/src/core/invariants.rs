//! Semantic invariants not expressible via JSON Schema.

use crate::tree::Node;
use std::collections::HashSet;

/// Check semantic invariants not expressible in JSON Schema:
/// - No duplicate ids
/// - `max_attempts > 0`
/// - `attempts <= max_attempts`
/// - Children sorted by `(order, id)`
pub fn validate_invariants(root: &Node) -> Vec<String> {
    let mut errors = Vec::new();
    let mut seen = HashSet::new();
    validate_node(root, &mut seen, &mut errors, root.id.as_str());
    errors
}

fn validate_node(node: &Node, seen: &mut HashSet<String>, errors: &mut Vec<String>, path: &str) {
    if !seen.insert(node.id.clone()) {
        errors.push(format!("duplicate id '{}' at {}", node.id, path));
    }

    if node.max_attempts == 0 {
        errors.push(format!("{}: max_attempts must be > 0", path));
    }

    if node.attempts > node.max_attempts {
        errors.push(format!(
            "{}: attempts {} exceeds max_attempts {}",
            path, node.attempts, node.max_attempts
        ));
    }

    if !children_sorted(&node.children) {
        errors.push(format!("{}: children must be sorted by (order,id)", path));
    }

    for child in &node.children {
        let child_path = format!("{}/{}", path, child.id);
        validate_node(child, seen, errors, &child_path);
    }
}

/// True if children are sorted by `(order, id)` ascending.
fn children_sorted(children: &[Node]) -> bool {
    children.windows(2).all(|pair| {
        let left = &pair[0];
        let right = &pair[1];
        (left.order, &left.id) <= (right.order, &right.id)
    })
}
