//! Test-only helpers for deterministic runner tests.
//!
//! Includes:
//! - Node builders for invariants/unit tests.
//! - `TestRepo` harness for temp git repos + runner state (requires `test-support` feature).
//! - Scripted executor/guard runners for multi-iteration scenarios.
//! - Fixture loaders for JSON/TOML seeds.

use std::cell::RefCell;
use std::collections::VecDeque;
use std::fs;
use std::path::{Path, PathBuf};
#[cfg(feature = "test-support")]
use std::process::Command;

use anyhow::{Context, Result, anyhow};
#[cfg(feature = "test-support")]
use tempfile::TempDir;

use crate::core::types::{AgentOutput, GuardOutcome};
use crate::io::config::RunnerConfig;
use crate::io::executor::{ExecRequest, Executor};
use crate::io::guards::{GuardRequest, GuardRunner};
#[cfg(feature = "test-support")]
use crate::io::run_state::{RunState, load_run_state};
use crate::io::tree_store::{load_tree, write_tree};
#[cfg(feature = "test-support")]
use crate::start::{StartOutcome, start_run};
use crate::tree::Node;

/// Ephemeral git repository harness for runner tests.
///
/// Invariants:
/// - Hermetic temp directory (cleaned on drop).
/// - Deterministic git config + bootstrap commit.
/// - No external codex execution or network activity.
///
/// Future considerations:
/// - Promote to a reusable `runner-test-support` crate when shared across crates.
#[cfg(feature = "test-support")]
#[derive(Debug)]
pub struct TestRepo {
    #[allow(dead_code)]
    temp: TempDir,
    root: PathBuf,
}

#[cfg(feature = "test-support")]
impl TestRepo {
    /// Create a new temp repo with an initial commit.
    pub fn new() -> Result<Self> {
        let temp = tempfile::tempdir().context("create tempdir")?;
        let root = temp.path().to_path_buf();

        run_git(&root, &["init"])?;
        run_git(&root, &["config", "user.email", "test@example.com"])?;
        run_git(&root, &["config", "user.name", "test"])?;

        fs::write(root.join("README.md"), "hi\n").context("write README")?;
        run_git(&root, &["add", "README.md"])?;
        run_git(&root, &["commit", "-m", "chore: init"])?;

        Ok(Self { temp, root })
    }

    /// Root directory of the temp repo.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Alias for `root()` for ergonomic use in tests.
    pub fn path(&self) -> &Path {
        &self.root
    }

    /// Run `runner start` and return the outcome.
    pub fn start_run(&self) -> Result<StartOutcome> {
        start_run(self.root())
    }

    /// Read the current task tree from `.runner/state/tree.json`.
    pub fn read_tree(&self) -> Result<Node> {
        load_tree(&self.schema_path(), &self.tree_path())
    }

    /// Write a task tree to `.runner/state/tree.json` with canonical formatting.
    pub fn write_tree(&self, tree: &Node) -> Result<()> {
        write_tree(&self.tree_path(), tree)
    }

    /// Read the current run state from `.runner/state/run_state.json`.
    pub fn read_run_state(&self) -> Result<RunState> {
        load_run_state(&self.run_state_path())
    }

    fn tree_path(&self) -> PathBuf {
        self.root.join(".runner/state/tree.json")
    }

    fn schema_path(&self) -> PathBuf {
        self.root.join(".runner/state/schema.json")
    }

    fn run_state_path(&self) -> PathBuf {
        self.root.join(".runner/state/run_state.json")
    }
}

/// Fixture root for test seeds (small, deterministic files).
///
/// Fixture philosophy:
/// - Prefer Rust builders for happy-path logic tests.
/// - Use JSON/TOML fixtures for format/invariant edge cases and canonicalization checks.
/// - Keep fixtures tiny and deterministic.
/// - Future: add schema-evolution notes alongside fixture changes.
pub fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

/// Resolve a fixture path under `runner/tests/fixtures`.
pub fn fixture_path(relative: impl AsRef<Path>) -> PathBuf {
    fixture_root().join(relative)
}

/// Load a JSON tree fixture by name (without extension).
pub fn load_tree_fixture(name: &str) -> Result<Node> {
    let path = fixture_path(format!("trees/{name}.json"));
    let contents = fs::read_to_string(&path)
        .with_context(|| format!("read tree fixture {}", path.display()))?;
    let tree: Node = serde_json::from_str(&contents)
        .with_context(|| format!("parse tree fixture {}", path.display()))?;
    Ok(tree)
}

/// Load a config fixture by name (without extension).
pub fn load_config_fixture(name: &str) -> Result<RunnerConfig> {
    let path = fixture_path(format!("configs/{name}.toml"));
    let contents = fs::read_to_string(&path)
        .with_context(|| format!("read config fixture {}", path.display()))?;
    let cfg: RunnerConfig =
        toml::from_str(&contents).with_context(|| format!("parse config {}", path.display()))?;
    cfg.validate()?;
    Ok(cfg)
}

/// Scripted executor response for a single invocation.
#[derive(Debug, Clone)]
pub struct ScriptedExec {
    pub output: AgentOutput,
    pub tree_update: Option<Node>,
}

/// Scripted executor for deterministic multi-iteration tests.
///
/// Invariants:
/// - FIFO queue consumption, single-threaded tests (interior mutability).
/// - Writes `output.json` exactly once per invocation.
/// - Optional tree updates written to `.runner/state/tree.json`.
///
/// Future considerations:
/// - Add request matchers (prompt contents, selected node id, etc.).
#[derive(Debug, Default)]
pub struct ScriptedExecutor {
    queue: RefCell<VecDeque<ScriptedExec>>,
    requests: RefCell<Vec<ExecRequest>>,
}

impl ScriptedExecutor {
    pub fn new(queue: Vec<ScriptedExec>) -> Self {
        Self {
            queue: RefCell::new(queue.into()),
            requests: RefCell::new(Vec::new()),
        }
    }

    pub fn remaining(&self) -> usize {
        self.queue.borrow().len()
    }

    pub fn assert_drained(&self) -> Result<()> {
        let remaining = self.remaining();
        if remaining == 0 {
            Ok(())
        } else {
            Err(anyhow!(
                "scripted executor has {remaining} unused responses"
            ))
        }
    }

    pub fn last_request(&self) -> Option<ExecRequest> {
        self.requests.borrow().last().cloned()
    }
}

impl Executor for ScriptedExecutor {
    fn exec(&self, request: &ExecRequest) -> Result<()> {
        self.requests.borrow_mut().push(request.clone());
        let next = self
            .queue
            .borrow_mut()
            .pop_front()
            .ok_or_else(|| anyhow!("scripted executor queue exhausted"))?;

        if let Some(parent) = request.output_path.parent() {
            fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
        }
        let mut buf = serde_json::to_string_pretty(&next.output)?;
        buf.push('\n');
        fs::write(&request.output_path, buf)
            .with_context(|| format!("write {}", request.output_path.display()))?;

        if let Some(tree) = &next.tree_update {
            let tree_path = request.workdir.join(".runner/state/tree.json");
            write_tree(&tree_path, tree)?;
        }

        Ok(())
    }
}

/// Scripted guard response for a single invocation.
#[derive(Debug, Clone)]
pub struct ScriptedGuard {
    pub outcome: GuardOutcome,
    pub log: String,
}

/// Scripted guard runner with queued outcomes and logs.
///
/// Invariants:
/// - FIFO queue consumption, single-threaded tests (interior mutability).
/// - Writes `guard.log` on every invocation for parity with real runner.
#[derive(Debug, Default)]
pub struct ScriptedGuardRunner {
    queue: RefCell<VecDeque<ScriptedGuard>>,
    requests: RefCell<Vec<GuardRequest>>,
}

impl ScriptedGuardRunner {
    pub fn new(queue: Vec<ScriptedGuard>) -> Self {
        Self {
            queue: RefCell::new(queue.into()),
            requests: RefCell::new(Vec::new()),
        }
    }

    pub fn remaining(&self) -> usize {
        self.queue.borrow().len()
    }

    pub fn assert_drained(&self) -> Result<()> {
        let remaining = self.remaining();
        if remaining == 0 {
            Ok(())
        } else {
            Err(anyhow!(
                "scripted guard runner has {remaining} unused responses"
            ))
        }
    }

    pub fn last_request(&self) -> Option<GuardRequest> {
        self.requests.borrow().last().cloned()
    }
}

impl GuardRunner for ScriptedGuardRunner {
    fn run(&self, request: &GuardRequest) -> Result<GuardOutcome> {
        self.requests.borrow_mut().push(request.clone());
        let next = self
            .queue
            .borrow_mut()
            .pop_front()
            .ok_or_else(|| anyhow!("scripted guard runner queue exhausted"))?;

        if let Some(parent) = request.log_path.parent() {
            fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
        }
        let mut buf = next.log;
        if !buf.ends_with('\n') {
            buf.push('\n');
        }
        fs::write(&request.log_path, buf)
            .with_context(|| format!("write {}", request.log_path.display()))?;
        Ok(next.outcome)
    }
}

/// Create a deterministic node with default fields and no children.
pub fn node(id: &str, order: i64) -> Node {
    Node {
        id: id.to_string(),
        order,
        title: format!("{} title", id),
        goal: format!("{} goal", id),
        acceptance: Vec::new(),
        passes: false,
        attempts: 0,
        max_attempts: 3,
        children: Vec::new(),
    }
}

/// Create a deterministic leaf node with explicit `passes`.
pub fn leaf(id: &str, order: i64, passes: bool) -> Node {
    let mut node = node(id, order);
    node.passes = passes;
    node
}

/// Create a node with children using deterministic defaults.
pub fn node_with_children(id: &str, order: i64, children: Vec<Node>) -> Node {
    Node {
        children,
        ..node(id, order)
    }
}

/// Create a node with explicit attempt state (useful for invariant tests).
pub fn node_with_attempts(id: &str, order: i64, attempts: u32, max_attempts: u32) -> Node {
    let mut node = node(id, order);
    node.attempts = attempts;
    node.max_attempts = max_attempts;
    node
}

#[cfg(feature = "test-support")]
fn run_git(root: &Path, args: &[&str]) -> Result<()> {
    let status = Command::new("git")
        .args(args)
        .current_dir(root)
        .status()
        .with_context(|| format!("spawn git {}", args.join(" ")))?;
    if status.success() {
        Ok(())
    } else {
        Err(anyhow!("git {} failed", args.join(" ")))
    }
}
