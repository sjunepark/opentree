mod tree;

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};
use jsonschema::Draft;
use serde::Serialize;
use serde_json::Value;
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use tree::{Node, default_tree};

const V1_SCHEMA: &str = r##"{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://runner.local/schemas/task_tree/v1.schema.json",
  "title": "Task Tree v1",
  "$ref": "#/$defs/node",
  "$defs": {
    "node": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "id",
        "order",
        "title",
        "goal",
        "acceptance",
        "passes",
        "attempts",
        "max_attempts",
        "children"
      ],
      "properties": {
        "id": {
          "type": "string",
          "minLength": 1
        },
        "order": {
          "type": "integer"
        },
        "title": {
          "type": "string"
        },
        "goal": {
          "type": "string"
        },
        "acceptance": {
          "type": "array",
          "items": {
            "type": "string"
          }
        },
        "passes": {
          "type": "boolean"
        },
        "attempts": {
          "type": "integer",
          "minimum": 0
        },
        "max_attempts": {
          "type": "integer",
          "minimum": 0
        },
        "children": {
          "type": "array",
          "items": {
            "$ref": "#/$defs/node"
          }
        }
      }
    }
  }
}
"##;

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
    Init {
        #[arg(short, long)]
        force: bool,
    },
    Validate,
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

fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    let mut payload = serde_json::to_string_pretty(value).context("serialize json")?;
    payload.push('\n');
    fs::write(path, payload).with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

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

fn validate_invariants(root: &Node) -> Vec<String> {
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

fn children_sorted(children: &[Node]) -> bool {
    children.windows(2).all(|pair| {
        let left = &pair[0];
        let right = &pair[1];
        (left.order, &left.id) <= (right.order, &right.id)
    })
}

fn leftmost_open_leaf<'a>(node: &'a Node) -> Option<&'a Node> {
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
