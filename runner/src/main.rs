//! Goal-driven agent loop runner.
//!
//! Manages a task tree (`.runner/tree.json`) that tracks hierarchical goals.
//! The runner selects the leftmost open leaf for execution, enabling
//! deterministic, resumable agent loops.

mod core;
mod tree;

use crate::core::invariants::validate_invariants;
use crate::core::selector::leftmost_open_leaf;
use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};
use jsonschema::Draft;
use serde::Serialize;
use serde_json::Value;
use std::fs;
use std::path::Path;
use tree::{Node, default_tree};

const V1_SCHEMA: &str = include_str!("../../schemas/task_tree/v1.schema.json");
const EMPTY_DOC: &str = "";

#[derive(Parser)]
#[command(
    name = "runner",
    version,
    about = "Deterministic goal-driven agent loop runner"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Create `.runner/tree.json` and schema files if missing.
    Init {
        /// Overwrite existing files.
        #[arg(short, long)]
        force: bool,
    },
    /// Check tree against schema and invariants (unique ids, sorted children, etc.).
    Validate,
    /// Print the id of the leftmost open leaf (first incomplete task).
    Select,
}

fn main() {
    if let Err(err) = run() {
        eprintln!("{:#}", err);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Init { force } => cmd_init(force),
        Command::Validate => cmd_validate(),
        Command::Select => cmd_select(),
    }
}

fn cmd_init(force: bool) -> Result<()> {
    let tree_path = Path::new(".runner/tree.json");
    let schema_path = Path::new("schemas/task_tree/v1.schema.json");

    fs::create_dir_all(".runner").context("create .runner directory")?;
    fs::create_dir_all("schemas/task_tree").context("create schema directory")?;

    write_if_missing_or_force(Path::new(".runner/ASSUMPTIONS.md"), EMPTY_DOC, force)?;
    write_if_missing_or_force(Path::new(".runner/FEEDBACK_LOG.md"), EMPTY_DOC, force)?;
    write_if_missing_or_force(Path::new(".runner/GLOSSARY.md"), EMPTY_DOC, force)?;
    write_if_missing_or_force(Path::new(".runner/GOAL.md"), EMPTY_DOC, force)?;
    write_if_missing_or_force(Path::new(".runner/HUMAN_QUESTIONS.md"), EMPTY_DOC, force)?;
    write_if_missing_or_force(Path::new(".runner/IMPROVEMENTS.md"), EMPTY_DOC, force)?;

    if force || !schema_path.exists() {
        fs::write(schema_path, V1_SCHEMA).context("write v1 schema")?;
    }

    if force || !tree_path.exists() {
        let mut tree = default_tree();
        tree.sort_children();
        write_json(tree_path, &tree).context("write .runner/tree.json")?;
    }

    Ok(())
}

fn cmd_validate() -> Result<()> {
    let tree_raw = fs::read_to_string(".runner/tree.json").context("read .runner/tree.json")?;
    let schema_raw =
        fs::read_to_string("schemas/task_tree/v1.schema.json").context("read v1 schema")?;
    validate_tree(&tree_raw, &schema_raw)?;
    Ok(())
}

fn cmd_select() -> Result<()> {
    let tree_raw = fs::read_to_string(".runner/tree.json").context("read .runner/tree.json")?;
    let schema_raw =
        fs::read_to_string("schemas/task_tree/v1.schema.json").context("read v1 schema")?;
    let tree = validate_tree(&tree_raw, &schema_raw)?;
    let selected =
        leftmost_open_leaf(&tree).context("no open leaf: all leaves have passes=true")?;
    println!("{}", selected.id);
    Ok(())
}

/// Serialize `value` to pretty-printed JSON with trailing newline.
fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    let mut payload = serde_json::to_string_pretty(value).context("serialize json")?;
    payload.push('\n');
    fs::write(path, payload).with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

fn write_if_missing_or_force(path: &Path, contents: &str, force: bool) -> Result<()> {
    if !force && path.exists() {
        return Ok(());
    }
    fs::write(path, contents).with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

/// Parse and validate tree: schema conformance + semantic invariants.
///
/// Returns the parsed `Node` on success, or an error describing violations.
fn validate_tree(tree_raw: &str, schema_raw: &str) -> Result<Node> {
    let tree_json: Value = serde_json::from_str(tree_raw).context("parse tree json")?;
    let schema_json: Value = serde_json::from_str(schema_raw).context("parse schema json")?;
    validate_schema(&tree_json, &schema_json)?;
    let tree: Node = serde_json::from_str(tree_raw).context("parse tree as v1 struct")?;
    let errors = validate_invariants(&tree);
    if !errors.is_empty() {
        bail!("invariant violations:\n- {}", errors.join("\n- "));
    }
    Ok(tree)
}

/// Validate JSON instance against a JSON Schema (Draft 2020-12).
fn validate_schema(instance: &Value, schema: &Value) -> Result<()> {
    let compiled = jsonschema::options()
        .with_draft(Draft::Draft202012)
        .build(schema)
        .context("compile json schema")?;
    let messages: Vec<String> = compiled
        .iter_errors(instance)
        .map(|err| err.to_string())
        .collect();
    if !messages.is_empty() {
        bail!("schema validation failed:\n- {}", messages.join("\n- "));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_init() {
        let cli = Cli::parse_from(["runner", "init"]);
        assert!(matches!(cli.command, Command::Init { force: false }));
    }

    #[test]
    fn parse_init_force() {
        let cli = Cli::parse_from(["runner", "init", "--force"]);
        assert!(matches!(cli.command, Command::Init { force: true }));
    }

    #[test]
    fn sort_children_orders_by_order_then_id() {
        let mut node = Node {
            id: "root".to_string(),
            order: 0,
            title: "Root".to_string(),
            goal: "Goal".to_string(),
            acceptance: Vec::new(),
            passes: false,
            attempts: 0,
            max_attempts: 3,
            children: vec![
                Node {
                    id: "b".to_string(),
                    order: 2,
                    title: "B".to_string(),
                    goal: "B goal".to_string(),
                    acceptance: Vec::new(),
                    passes: false,
                    attempts: 0,
                    max_attempts: 3,
                    children: Vec::new(),
                },
                Node {
                    id: "c".to_string(),
                    order: 1,
                    title: "C".to_string(),
                    goal: "C goal".to_string(),
                    acceptance: Vec::new(),
                    passes: false,
                    attempts: 0,
                    max_attempts: 3,
                    children: Vec::new(),
                },
                Node {
                    id: "a".to_string(),
                    order: 1,
                    title: "A".to_string(),
                    goal: "A goal".to_string(),
                    acceptance: Vec::new(),
                    passes: false,
                    attempts: 0,
                    max_attempts: 3,
                    children: Vec::new(),
                },
            ],
        };

        node.sort_children();
        let ids: Vec<&str> = node
            .children
            .iter()
            .map(|child| child.id.as_str())
            .collect();
        assert_eq!(ids, vec!["a", "c", "b"]);
    }

    #[test]
    fn select_leftmost_open_leaf_depth_first() {
        let tree = Node {
            id: "root".to_string(),
            order: 0,
            title: "Root".to_string(),
            goal: "Goal".to_string(),
            acceptance: Vec::new(),
            passes: false,
            attempts: 0,
            max_attempts: 3,
            children: vec![
                Node {
                    id: "a".to_string(),
                    order: 0,
                    title: "A".to_string(),
                    goal: "A goal".to_string(),
                    acceptance: Vec::new(),
                    passes: true,
                    attempts: 0,
                    max_attempts: 3,
                    children: Vec::new(),
                },
                Node {
                    id: "b".to_string(),
                    order: 1,
                    title: "B".to_string(),
                    goal: "B goal".to_string(),
                    acceptance: Vec::new(),
                    passes: false,
                    attempts: 0,
                    max_attempts: 3,
                    children: vec![Node {
                        id: "b1".to_string(),
                        order: 0,
                        title: "B1".to_string(),
                        goal: "B1 goal".to_string(),
                        acceptance: Vec::new(),
                        passes: false,
                        attempts: 0,
                        max_attempts: 3,
                        children: Vec::new(),
                    }],
                },
                Node {
                    id: "c".to_string(),
                    order: 2,
                    title: "C".to_string(),
                    goal: "C goal".to_string(),
                    acceptance: Vec::new(),
                    passes: false,
                    attempts: 0,
                    max_attempts: 3,
                    children: Vec::new(),
                },
            ],
        };

        let selected = leftmost_open_leaf(&tree).expect("expected open leaf");
        assert_eq!(selected.id, "b1");
    }

    #[test]
    fn validate_invariants_reports_errors() {
        let tree = Node {
            id: "root".to_string(),
            order: 0,
            title: "Root".to_string(),
            goal: "Goal".to_string(),
            acceptance: Vec::new(),
            passes: false,
            attempts: 2,
            max_attempts: 1,
            children: vec![
                Node {
                    id: "dup".to_string(),
                    order: 1,
                    title: "Dup1".to_string(),
                    goal: "Dup1 goal".to_string(),
                    acceptance: Vec::new(),
                    passes: false,
                    attempts: 0,
                    max_attempts: 1,
                    children: Vec::new(),
                },
                Node {
                    id: "dup".to_string(),
                    order: 0,
                    title: "Dup2".to_string(),
                    goal: "Dup2 goal".to_string(),
                    acceptance: Vec::new(),
                    passes: false,
                    attempts: 0,
                    max_attempts: 1,
                    children: Vec::new(),
                },
            ],
        };

        let errors = validate_invariants(&tree);
        assert!(errors.iter().any(|err| err.contains("duplicate id")));
        assert!(errors.iter().any(|err| err.contains("max_attempts")));
        assert!(
            errors
                .iter()
                .any(|err| err.contains("children must be sorted"))
        );
    }
}
