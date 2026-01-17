//! Initialization helpers for `.runner/` scaffolding.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow};
use serde::Serialize;

use super::run_state::{RunState, write_run_state};
use crate::tree::default_tree;

const TREE_SCHEMA: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../schemas/task_tree/v1.schema.json"
));

/// All canonical paths within `.runner/` for a project root.
#[derive(Debug, Clone)]
pub struct RunnerPaths {
    pub root: PathBuf,
    pub runner_dir: PathBuf,
    pub state_dir: PathBuf,
    pub context_dir: PathBuf,
    pub iterations_dir: PathBuf,
    pub gitignore_path: PathBuf,
    pub goal_path: PathBuf,
    pub tree_path: PathBuf,
    pub schema_path: PathBuf,
    pub config_path: PathBuf,
    pub assumptions_path: PathBuf,
    pub questions_path: PathBuf,
    pub run_state_path: PathBuf,
    pub context_goal_path: PathBuf,
    pub context_history_path: PathBuf,
    pub context_failure_path: PathBuf,
}

impl RunnerPaths {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        let root = root.into();
        let runner_dir = root.join(".runner");
        let state_dir = runner_dir.join("state");
        let context_dir = runner_dir.join("context");
        let iterations_dir = runner_dir.join("iterations");
        Self {
            root: root.clone(),
            runner_dir: runner_dir.clone(),
            state_dir: state_dir.clone(),
            context_dir: context_dir.clone(),
            iterations_dir: iterations_dir.clone(),
            gitignore_path: runner_dir.join(".gitignore"),
            goal_path: runner_dir.join("GOAL.md"),
            tree_path: state_dir.join("tree.json"),
            schema_path: state_dir.join("schema.json"),
            config_path: state_dir.join("config.json"),
            assumptions_path: state_dir.join("assumptions.md"),
            questions_path: state_dir.join("questions.md"),
            run_state_path: state_dir.join("run_state.json"),
            context_goal_path: context_dir.join("goal.md"),
            context_history_path: context_dir.join("history.md"),
            context_failure_path: context_dir.join("failure.md"),
        }
    }
}

/// Options for `init_runner`.
#[derive(Debug, Clone)]
pub struct InitOptions {
    /// If true, overwrite existing runner-owned files.
    pub force: bool,
}

#[derive(Debug, Serialize)]
struct RunnerConfig {
    max_attempts_default: u32,
    guard: GuardConfig,
}

#[derive(Debug, Serialize)]
struct GuardConfig {
    command: Vec<String>,
}

/// Create `.runner/` scaffolding in `root`.
///
/// Fails if `.runner/` already exists unless `options.force` is set.
pub fn init_runner(root: &Path, options: &InitOptions) -> Result<RunnerPaths> {
    let paths = RunnerPaths::new(root);
    if paths.runner_dir.exists() && !options.force {
        return Err(anyhow!(
            "runner init: .runner already exists (use --force to overwrite)"
        ));
    }
    if paths.runner_dir.exists() && !paths.runner_dir.is_dir() {
        return Err(anyhow!(
            "runner init: .runner exists but is not a directory"
        ));
    }

    create_dir(&paths.runner_dir)?;
    create_dir(&paths.state_dir)?;
    create_dir(&paths.context_dir)?;
    create_dir(&paths.iterations_dir)?;

    write_file(&paths.gitignore_path, RUNNER_GITIGNORE)?;
    write_file(&paths.goal_path, GOAL_PLACEHOLDER)?;
    write_tree(&paths.tree_path)?;
    write_file(&paths.schema_path, TREE_SCHEMA)?;
    write_json(&paths.config_path, &default_config())?;
    write_run_state(&paths.run_state_path, &RunState::default())?;
    write_file(&paths.assumptions_path, ASSUMPTIONS_PLACEHOLDER)?;
    write_file(&paths.questions_path, QUESTIONS_PLACEHOLDER)?;
    write_file(&paths.context_goal_path, CONTEXT_GOAL_PLACEHOLDER)?;
    write_file(&paths.context_history_path, CONTEXT_HISTORY_PLACEHOLDER)?;
    write_file(&paths.context_failure_path, CONTEXT_FAILURE_PLACEHOLDER)?;

    Ok(paths)
}

fn create_dir(path: &Path) -> Result<()> {
    fs::create_dir_all(path).with_context(|| format!("create directory {}", path.display()))
}

fn write_file(path: &Path, contents: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        create_dir(parent)?;
    }
    fs::write(path, contents).with_context(|| format!("write file {}", path.display()))
}

fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    let mut buf = serde_json::to_string_pretty(value)?;
    buf.push('\n');
    write_file(path, &buf)
}

fn write_tree(path: &Path) -> Result<()> {
    let mut tree = default_tree();
    tree.sort_children();
    write_json(path, &tree)
}

fn default_config() -> RunnerConfig {
    RunnerConfig {
        max_attempts_default: 3,
        guard: GuardConfig {
            command: vec!["just".to_string(), "ci".to_string()],
        },
    }
}

const GOAL_PLACEHOLDER: &str = "# Goal\n\nDescribe the overall project goal here.\n";
const ASSUMPTIONS_PLACEHOLDER: &str = "# Assumptions\n\n";
const QUESTIONS_PLACEHOLDER: &str = "# Open Questions\n\n";
const CONTEXT_GOAL_PLACEHOLDER: &str = "# Goal (current node)\n\nGenerated by `runner step`.\n";
const CONTEXT_HISTORY_PLACEHOLDER: &str =
    "# History (previous attempt)\n\nGenerated by `runner step` on retry.\n";
const CONTEXT_FAILURE_PLACEHOLDER: &str =
    "# Failure (guard output)\n\nGenerated by `runner step` when guards fail.\n";
const RUNNER_GITIGNORE: &str = "context/\niterations/\n";

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn read_to_string(path: &Path) -> String {
        fs::read_to_string(path).expect("read file")
    }

    /// Verifies init_runner creates the complete directory structure and files.
    ///
    /// Checks all expected directories (state, context, iterations) and files
    /// (tree.json, schema.json, config.json, etc.) exist with correct content.
    #[test]
    fn init_creates_expected_layout() {
        let temp = tempfile::tempdir().expect("tempdir");
        let root = temp.path();

        let paths = init_runner(root, &InitOptions { force: false }).expect("init");

        assert!(paths.runner_dir.is_dir());
        assert!(paths.state_dir.is_dir());
        assert!(paths.context_dir.is_dir());
        assert!(paths.iterations_dir.is_dir());
        assert!(paths.gitignore_path.is_file());
        assert!(paths.goal_path.is_file());
        assert!(paths.tree_path.is_file());
        assert!(paths.schema_path.is_file());
        assert!(paths.config_path.is_file());
        assert!(paths.run_state_path.is_file());
        assert!(paths.assumptions_path.is_file());
        assert!(paths.questions_path.is_file());
        assert!(paths.context_goal_path.is_file());
        assert!(paths.context_history_path.is_file());
        assert!(paths.context_failure_path.is_file());

        let tree_contents = read_to_string(&paths.tree_path);
        let mut expected_tree = default_tree();
        expected_tree.sort_children();
        let mut expected_tree_json =
            serde_json::to_string_pretty(&expected_tree).expect("serialize tree");
        expected_tree_json.push('\n');
        assert_eq!(tree_contents, expected_tree_json);

        let gitignore = read_to_string(&paths.gitignore_path);
        assert_eq!(gitignore, RUNNER_GITIGNORE);
    }

    /// Verifies init_runner refuses to overwrite without --force.
    ///
    /// First init succeeds, second init (without force) fails with "already exists".
    #[test]
    fn init_without_force_refuses_existing_runner_dir() {
        let temp = tempfile::tempdir().expect("tempdir");
        let root = temp.path();

        init_runner(root, &InitOptions { force: false }).expect("init");
        let err = init_runner(root, &InitOptions { force: false }).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("already exists"));
    }

    /// Verifies init_runner with --force overwrites customized files.
    ///
    /// Writes custom content to files, re-inits with force, confirms placeholders restored.
    #[test]
    fn init_with_force_rewrites_placeholders() {
        let temp = tempfile::tempdir().expect("tempdir");
        let root = temp.path();
        let paths = init_runner(root, &InitOptions { force: false }).expect("init");

        fs::write(&paths.assumptions_path, "custom").expect("write custom");
        fs::write(&paths.context_goal_path, "custom").expect("write custom");

        init_runner(root, &InitOptions { force: true }).expect("re-init");

        let assumptions = read_to_string(&paths.assumptions_path);
        let context_goal = read_to_string(&paths.context_goal_path);
        assert_eq!(assumptions, ASSUMPTIONS_PLACEHOLDER);
        assert_eq!(context_goal, CONTEXT_GOAL_PLACEHOLDER);
    }
}
