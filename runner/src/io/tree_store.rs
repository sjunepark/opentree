//! Tree load/save helpers with schema + invariant validation.

use std::fs;
use std::path::Path;

use anyhow::{Context, Result, anyhow};
use jsonschema::validator_for;
use serde_json::Value;

use crate::core::invariants::validate_invariants;
use crate::tree::Node;

/// Load and validate tree from disk (schema + invariants).
pub fn load_tree(schema_path: &Path, tree_path: &Path) -> Result<Node> {
    let tree_contents = fs::read_to_string(tree_path)
        .with_context(|| format!("read tree {}", tree_path.display()))?;
    let tree_value: Value = serde_json::from_str(&tree_contents)
        .with_context(|| format!("parse tree {}", tree_path.display()))?;
    validate_schema(schema_path, &tree_value)?;
    let tree: Node = serde_json::from_value(tree_value)
        .with_context(|| format!("deserialize tree {}", tree_path.display()))?;
    validate_tree_invariants(&tree)?;
    Ok(tree)
}

/// Write tree to disk with canonicalized formatting (sorted children).
pub fn write_tree(tree_path: &Path, tree: &Node) -> Result<()> {
    let mut cloned = tree.clone();
    cloned.sort_children();
    let mut buf = serde_json::to_string_pretty(&cloned)?;
    buf.push('\n');
    fs::write(tree_path, buf).with_context(|| format!("write tree {}", tree_path.display()))
}

fn validate_schema(schema_path: &Path, tree: &Value) -> Result<()> {
    let schema_contents = fs::read_to_string(schema_path)
        .with_context(|| format!("read schema {}", schema_path.display()))?;
    let schema_value: Value = serde_json::from_str(&schema_contents)
        .with_context(|| format!("parse schema {}", schema_path.display()))?;
    let compiled =
        validator_for(&schema_value).map_err(|err| anyhow!("invalid schema: {}", err))?;
    if !compiled.is_valid(tree) {
        let messages = compiled
            .iter_errors(tree)
            .map(|err| err.to_string())
            .collect::<Vec<_>>();
        return Err(anyhow!(
            "tree schema validation failed: {}",
            messages.join("; ")
        ));
    }
    Ok(())
}

fn validate_tree_invariants(tree: &Node) -> Result<()> {
    let errors = validate_invariants(tree);
    if errors.is_empty() {
        return Ok(());
    }
    Err(anyhow!("tree invariants failed: {}", errors.join("; ")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tree::default_tree;

    /// Verifies write → load round-trip preserves tree structure.
    ///
    /// Writes a tree, loads it back with schema validation, confirms root id matches.
    #[test]
    fn load_and_write_tree_round_trip() {
        let temp = tempfile::tempdir().expect("tempdir");
        let root = temp.path();
        let schema_path = root.join("schema.json");
        let tree_path = root.join("tree.json");

        fs::write(
            &schema_path,
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../schemas/task_tree/v1.schema.json"
            )),
        )
        .expect("write schema");
        write_tree(&tree_path, &default_tree()).expect("write tree");

        let tree = load_tree(&schema_path, &tree_path).expect("load tree");
        assert_eq!(tree.id, "root");
    }
}
