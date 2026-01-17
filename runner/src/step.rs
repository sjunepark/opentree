//! Orchestration for a single deterministic `runner step`.

use std::fs;
use std::path::Path;
use std::time::{Duration, Instant};

use anyhow::{Context, Result, anyhow};

use crate::core::immutability::check_passed_node_immutability;
use crate::core::selector::leftmost_open_leaf;
use crate::core::state_update::apply_state_updates;
use crate::core::status_validator::validate_status_invariants;
use crate::core::types::{AgentOutput, AgentStatus, GuardOutcome};
use crate::io::config::load_config;
use crate::io::context::{ContextPayload, write_context};
use crate::io::executor::{ExecRequest, Executor, execute_and_load};
use crate::io::git::Git;
use crate::io::goal::read_goal_id;
use crate::io::guards::{GuardRequest, GuardRunner, run_guards_if_needed};
use crate::io::iteration_log::{IterationMeta, IterationWriteRequest, write_iteration};
use crate::io::prompt::{PromptBuilder, PromptInputs};
use crate::io::run_state::{RunState, load_run_state, write_run_state};
use crate::io::tree_store::{load_tree, write_tree};
use crate::tree::Node;

const AGENT_OUTPUT_SCHEMA: &str = include_str!("../schemas/agent_output.schema.json");

/// Configuration for a single step iteration.
#[derive(Debug, Clone)]
pub struct StepConfig {
    /// Maximum bytes for the prompt pack before dropping sections.
    pub prompt_budget_bytes: usize,
}

impl Default for StepConfig {
    fn default() -> Self {
        Self {
            prompt_budget_bytes: 40_000,
        }
    }
}

/// Result of a single step iteration.
#[derive(Debug, Clone)]
pub struct StepOutcome {
    /// Identifier for the current execution run.
    pub run_id: String,
    /// Iteration number (1-indexed).
    pub iter: u32,
    /// ID of the node that was worked on.
    pub selected_id: String,
    /// Status declared by the agent.
    pub status: AgentStatus,
    /// Guard outcome (pass/fail/skipped).
    pub guard: GuardOutcome,
}

/// Execute one deterministic iteration of the agent loop.
///
/// Selects the leftmost open leaf, writes context, executes the agent,
/// validates output, runs guards if needed, and updates state.
pub fn run_step<E: Executor, G: GuardRunner>(
    root: &Path,
    executor: &E,
    guard_runner: &G,
    config: &StepConfig,
) -> Result<StepOutcome> {
    let start = Instant::now();
    enforce_git_policy_pre_step(root)?;
    let state_dir = root.join(".runner").join("state");
    let tree_path = state_dir.join("tree.json");
    let schema_path = state_dir.join("schema.json");
    let run_state_path = state_dir.join("run_state.json");
    let output_schema_path = state_dir.join("agent_output.schema.json");
    let config_path = state_dir.join("config.toml");
    let cfg = load_config(&config_path)?;
    let deadline = start + Duration::from_secs(cfg.iteration_timeout_secs);

    let mut run_state = load_or_default_run_state(&run_state_path)?;
    let run_id = run_state
        .run_id
        .clone()
        .ok_or_else(|| anyhow!("missing run id (run `runner start` first)"))?;
    enforce_run_id_matches_goal(root, &run_id)?;
    enforce_on_run_branch(root, &run_id)?;
    let iter = run_state.next_iter;

    write_output_schema(&output_schema_path)?;

    let prev_tree = load_tree(&schema_path, &tree_path)?;
    let selected = leftmost_open_leaf(&prev_tree)
        .ok_or_else(|| anyhow!("no open leaf found (tree already complete)"))?;
    let selected_id = selected.id.clone();
    let selected_path = find_node_path(&prev_tree, &selected_id)
        .ok_or_else(|| anyhow!("selected node path not found"))?;

    let goal_body = render_goal(selected);
    let history = history_from_run_state(&run_state);
    let failure = failure_from_run_state(root, &run_id, iter, &run_state);
    write_context(
        root,
        &ContextPayload {
            goal: goal_body,
            history,
            failure,
        },
    )?;

    let tree_summary = summarize_tree(&prev_tree, 200);
    let prompt_inputs =
        PromptInputs::from_root(root, selected_path, selected.to_owned(), tree_summary)?;
    let prompt_pack = PromptBuilder::new(config.prompt_budget_bytes).build(&prompt_inputs);

    let iter_dir = root
        .join(".runner")
        .join("iterations")
        .join(&run_id)
        .join(iter.to_string());
    fs::create_dir_all(&iter_dir)
        .with_context(|| format!("create iteration dir {}", iter_dir.display()))?;

    let guard_log_path = iter_dir.join("guard.log");
    let runner_error_log_path = iter_dir.join("runner_error.log");
    // Attempt counters must only move forward once we actually attempt an agent execution.
    let mut attempted_agent = false;

    let attempt = (|| -> Result<(AgentOutput, GuardOutcome, Node)> {
        let exec_timeout = remaining_budget(deadline)?;
        attempted_agent = true;

        let exec_request = ExecRequest {
            workdir: root.to_path_buf(),
            prompt: prompt_pack.render(),
            output_schema_path: output_schema_path.clone(),
            output_path: iter_dir.join("output.json"),
            executor_log_path: iter_dir.join("executor.log"),
            timeout: exec_timeout,
            output_limit_bytes: cfg.executor_output_limit_bytes,
        };

        let output = execute_and_load(executor, &exec_request)?;

        let next_tree = load_tree(&schema_path, &tree_path)?;
        validate_post_exec_tree(&prev_tree, &next_tree)?;
        validate_status(&prev_tree, &next_tree, &selected_id, output.status)?;

        let guard_outcome = if output.status == AgentStatus::Done {
            // Guards only run when the agent claims completion, and they receive the remaining
            // budget from the per-iteration timeout.
            let guard_timeout = remaining_budget(deadline)?;
            run_guards_if_needed(
                output.status,
                guard_runner,
                &GuardRequest {
                    workdir: root.to_path_buf(),
                    log_path: guard_log_path.clone(),
                    timeout: guard_timeout,
                    output_limit_bytes: cfg.guard_output_limit_bytes,
                },
            )?
        } else {
            GuardOutcome::Skipped
        };

        let mut updated_tree = next_tree.clone();
        apply_state_updates(
            &prev_tree,
            &mut updated_tree,
            &selected_id,
            output.status,
            guard_outcome,
        )
        .map_err(|err| anyhow!("state update failed: {err}"))?;
        write_tree(&tree_path, &updated_tree)?;

        Ok((output, guard_outcome, updated_tree))
    })();

    let output: AgentOutput;
    let guard_outcome: GuardOutcome;
    let mut tree_after: Node;
    let mut step_error: Option<anyhow::Error> = None;

    match attempt {
        Ok((out, guard, tree)) => {
            output = out;
            guard_outcome = guard;
            tree_after = tree;
        }
        Err(err) => {
            step_error = Some(err);

            let err_msg = format!("runner error: {}", step_error.as_ref().unwrap());
            if let Some(parent) = runner_error_log_path.parent() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("create {}", parent.display()))?;
            }
            fs::write(&runner_error_log_path, format!("{err_msg}\n"))
                .with_context(|| format!("write {}", runner_error_log_path.display()))?;

            output = AgentOutput {
                status: AgentStatus::Retry,
                summary: "runner error (see runner_error.log)".to_string(),
            };
            guard_outcome = GuardOutcome::Fail;
            tree_after = prev_tree.clone();
            if attempted_agent {
                increment_attempts(&mut tree_after, &selected_id);
            }

            write_tree(&tree_path, &tree_after)?;
        }
    }

    let guard_log = fs::read_to_string(&guard_log_path).ok();
    let meta = IterationMeta {
        run_id: run_id.clone(),
        iter,
        node_id: selected_id.clone(),
        status: output.status,
        guard: guard_outcome,
        started_at: None,
        ended_at: None,
        duration_ms: Some(start.elapsed().as_millis() as u64),
    };
    write_iteration(&IterationWriteRequest {
        root,
        run_id: &run_id,
        iter,
        meta: &meta,
        output: &output,
        guard_log: guard_log.as_deref(),
        tree_before: &prev_tree,
        tree_after: &tree_after,
    })?;

    run_state.run_id = Some(run_id.clone());
    run_state.next_iter = iter + 1;
    run_state.last_status = Some(output.status);
    // Do not feed runner-internal failures to the node agent via history.md. Those errors are
    // returned to the caller and logged under the iteration directory.
    run_state.last_summary = if step_error.is_some() {
        None
    } else {
        Some(output.summary.clone())
    };
    run_state.last_guard = Some(guard_outcome);
    write_run_state(&run_state_path, &run_state)?;

    commit_iteration(
        root,
        &run_id,
        iter,
        &selected_id,
        output.status,
        guard_outcome,
    )?;

    if let Some(err) = step_error {
        return Err(err);
    }

    Ok(StepOutcome {
        run_id,
        iter,
        selected_id,
        status: output.status,
        guard: guard_outcome,
    })
}

fn load_or_default_run_state(path: &Path) -> Result<RunState> {
    if path.exists() {
        return load_run_state(path);
    }
    Ok(RunState::default())
}

fn remaining_budget(deadline: Instant) -> Result<Duration> {
    let remaining = deadline
        .checked_duration_since(Instant::now())
        .unwrap_or(Duration::from_secs(0));
    if remaining.is_zero() {
        return Err(anyhow!("iteration timed out"));
    }
    Ok(remaining)
}

fn enforce_git_policy_pre_step(root: &Path) -> Result<()> {
    let git = Git::new(root);
    let branch = git.current_branch()?;
    if branch == "main" || branch == "master" {
        return Err(anyhow!(
            "refuse to run on '{branch}' (run `runner start` to create runner/<run-id> branch)"
        ));
    }
    git.ensure_clean()?;
    ensure_runner_gitignore(root)?;
    Ok(())
}

fn ensure_runner_gitignore(root: &Path) -> Result<()> {
    let path = root.join(".runner").join(".gitignore");
    if !path.exists() {
        return Err(anyhow!("missing {} (run `runner start`)", path.display()));
    }
    let contents = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    for required in ["context/", "iterations/"] {
        if !contents.lines().any(|l| l.trim() == required) {
            return Err(anyhow!(
                "missing '{}' in {} (run `runner start`)",
                required,
                path.display()
            ));
        }
    }
    Ok(())
}

fn enforce_run_id_matches_goal(root: &Path, run_id: &str) -> Result<()> {
    let goal_path = root.join(".runner").join("GOAL.md");
    let goal_id = read_goal_id(&goal_path)?.ok_or_else(|| {
        anyhow!(
            "missing goal id in {} (run `runner start`)",
            goal_path.display()
        )
    })?;
    if goal_id != run_id {
        return Err(anyhow!(
            "run id mismatch: run_state has '{run_id}' but GOAL.md has '{goal_id}' (run `runner start`)"
        ));
    }
    Ok(())
}

fn enforce_on_run_branch(root: &Path, run_id: &str) -> Result<()> {
    let git = Git::new(root);
    let expected = format!("runner/{run_id}");
    let branch = git.current_branch()?;
    if branch != expected {
        return Err(anyhow!(
            "expected to be on '{expected}' but on '{branch}' (run `runner start`)"
        ));
    }
    Ok(())
}

fn commit_iteration(
    root: &Path,
    run_id: &str,
    iter: u32,
    node_id: &str,
    status: AgentStatus,
    guard: GuardOutcome,
) -> Result<()> {
    let git = Git::new(root);
    git.add_all()?;

    let status_str = match status {
        AgentStatus::Done => "done",
        AgentStatus::Retry => "retry",
        AgentStatus::Decomposed => "decomposed",
    };
    let guard_str = match guard {
        GuardOutcome::Pass => "pass",
        GuardOutcome::Fail => "fail",
        GuardOutcome::Skipped => "skipped",
    };
    let msg = format!(
        "chore(loop): run {run_id} iter {iter} node {node_id} status={status_str} guard={guard_str}"
    );
    let committed = git.commit_staged(&msg)?;
    if !committed {
        return Err(anyhow!("expected changes to commit for iteration {iter}"));
    }
    Ok(())
}

fn write_output_schema(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("create output schema dir {}", parent.display()))?;
    }
    fs::write(path, AGENT_OUTPUT_SCHEMA)
        .with_context(|| format!("write output schema {}", path.display()))
}

fn validate_post_exec_tree(prev: &Node, next: &Node) -> Result<()> {
    let errors = check_passed_node_immutability(prev, next);
    if !errors.is_empty() {
        return Err(anyhow!("immutability failed: {}", errors.join("; ")));
    }
    Ok(())
}

fn validate_status(prev: &Node, next: &Node, selected_id: &str, status: AgentStatus) -> Result<()> {
    let errors = validate_status_invariants(prev, next, selected_id, status);
    if errors.is_empty() {
        return Ok(());
    }
    Err(anyhow!("status invariants failed: {}", errors.join("; ")))
}

fn increment_attempts(tree: &mut Node, selected_id: &str) {
    let Some(node) = find_node_mut(tree, selected_id) else {
        return;
    };
    if node.attempts < node.max_attempts {
        node.attempts += 1;
    }
}

fn find_node_mut<'a>(node: &'a mut Node, target_id: &str) -> Option<&'a mut Node> {
    if node.id == target_id {
        return Some(node);
    }
    for child in &mut node.children {
        if let Some(found) = find_node_mut(child, target_id) {
            return Some(found);
        }
    }
    None
}

fn render_goal(node: &Node) -> String {
    let mut buf = String::new();
    buf.push_str(&format!("title: {}\n", node.title));
    buf.push_str(&format!("goal: {}\n", node.goal));
    if !node.acceptance.is_empty() {
        buf.push_str("\nacceptance:\n");
        for item in &node.acceptance {
            buf.push_str(&format!("- {}\n", item));
        }
    }
    buf
}

fn history_from_run_state(run_state: &RunState) -> Option<String> {
    if run_state.last_status == Some(AgentStatus::Retry) {
        return run_state.last_summary.clone();
    }
    None
}

fn failure_from_run_state(
    root: &Path,
    run_id: &str,
    iter: u32,
    run_state: &RunState,
) -> Option<String> {
    // Only show guard output, never runner-internal errors.
    if run_state.last_status != Some(AgentStatus::Done) {
        return None;
    }
    if run_state.last_guard != Some(GuardOutcome::Fail) {
        return None;
    }
    let prev_iter = iter.saturating_sub(1);
    if prev_iter == 0 {
        return None;
    }
    let guard_log = root
        .join(".runner")
        .join("iterations")
        .join(run_id)
        .join(prev_iter.to_string())
        .join("guard.log");
    fs::read_to_string(guard_log).ok()
}

fn find_node_path(root: &Node, target_id: &str) -> Option<String> {
    let mut path = Vec::new();
    if find_node_path_inner(root, target_id, &mut path) {
        return Some(path.join("/"));
    }
    None
}

fn find_node_path_inner(node: &Node, target_id: &str, path: &mut Vec<String>) -> bool {
    path.push(node.id.clone());
    if node.id == target_id {
        return true;
    }
    for child in &node.children {
        if find_node_path_inner(child, target_id, path) {
            return true;
        }
    }
    path.pop();
    false
}

fn summarize_tree(root: &Node, max_nodes: usize) -> String {
    let mut lines = Vec::new();
    summarize_tree_inner(root, 0, max_nodes, &mut lines);
    lines.join("\n")
}

fn summarize_tree_inner(node: &Node, depth: usize, max_nodes: usize, lines: &mut Vec<String>) {
    if lines.len() >= max_nodes {
        return;
    }
    let indent = "  ".repeat(depth);
    lines.push(format!(
        "{}- {} (passes={}, attempts={}/{})",
        indent, node.id, node.passes, node.attempts, node.max_attempts
    ));
    for child in &node.children {
        summarize_tree_inner(child, depth + 1, max_nodes, lines);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::AgentOutput;
    use crate::io::executor::Executor;
    use crate::io::guards::GuardRunner;
    use crate::start::start_run;
    use crate::tree::default_tree;
    use std::process::Command;

    struct FakeExecutor {
        output: AgentOutput,
        tree_update: Option<Node>,
    }

    impl Executor for FakeExecutor {
        fn exec(&self, request: &ExecRequest) -> Result<()> {
            let mut buf = serde_json::to_string_pretty(&self.output)?;
            buf.push('\n');
            fs::write(&request.output_path, buf)?;
            if let Some(tree) = &self.tree_update {
                write_tree(&request.workdir.join(".runner/state/tree.json"), tree)?;
            }
            Ok(())
        }
    }

    struct FakeGuardRunner {
        outcome: GuardOutcome,
    }

    impl GuardRunner for FakeGuardRunner {
        fn run(&self, request: &GuardRequest) -> Result<GuardOutcome> {
            fs::write(&request.log_path, "guard output")?;
            Ok(self.outcome)
        }
    }

    /// Verifies a retry iteration updates run_state and writes iteration logs.
    ///
    /// Uses FakeExecutor returning Retry status. Asserts:
    /// - run_state.next_iter increments
    /// - run_state.last_status is Retry
    /// - Iteration logs (meta, output, tree snapshots) exist
    /// - No guard.log (guards skipped for retry)
    #[test]
    fn step_updates_run_state_and_tree_on_retry() {
        let temp = tempfile::tempdir().expect("tempdir");
        let root = temp.path();
        init_git_repo(root);
        start_run(root).expect("start");

        let executor = FakeExecutor {
            output: AgentOutput {
                status: AgentStatus::Retry,
                summary: "needs more".to_string(),
            },
            tree_update: None,
        };
        let guard_runner = FakeGuardRunner {
            outcome: GuardOutcome::Pass,
        };

        let outcome =
            run_step(root, &executor, &guard_runner, &StepConfig::default()).expect("step");

        assert_eq!(outcome.iter, 1);
        let run_state =
            load_run_state(&root.join(".runner/state/run_state.json")).expect("run state");
        assert_eq!(run_state.next_iter, 2);
        assert_eq!(run_state.last_status, Some(AgentStatus::Retry));
        assert_eq!(run_state.last_summary, Some("needs more".to_string()));

        let iter_dir = root
            .join(".runner/iterations")
            .join(&outcome.run_id)
            .join(outcome.iter.to_string());
        assert!(iter_dir.join("meta.json").exists());
        assert!(iter_dir.join("output.json").exists());
        assert!(iter_dir.join("tree.before.json").exists());
        assert!(iter_dir.join("tree.after.json").exists());
        assert!(!iter_dir.join("guard.log").exists());
    }

    /// Verifies Done + Pass marks the node as passed and writes guard log.
    ///
    /// Uses FakeExecutor returning Done status with passing guards. Asserts:
    /// - outcome.guard is Pass
    /// - guard.log exists (guards ran)
    /// - tree.passes is true (node completed successfully)
    /// - Iteration logs exist
    #[test]
    fn step_marks_done_and_writes_guard_log() {
        let temp = tempfile::tempdir().expect("tempdir");
        let root = temp.path();
        init_git_repo(root);
        start_run(root).expect("start");

        let executor = FakeExecutor {
            output: AgentOutput {
                status: AgentStatus::Done,
                summary: "done".to_string(),
            },
            tree_update: None,
        };
        let guard_runner = FakeGuardRunner {
            outcome: GuardOutcome::Pass,
        };

        let outcome =
            run_step(root, &executor, &guard_runner, &StepConfig::default()).expect("step");
        assert_eq!(outcome.guard, GuardOutcome::Pass);

        let guard_log = root
            .join(".runner/iterations")
            .join(&outcome.run_id)
            .join(outcome.iter.to_string())
            .join("guard.log");
        assert!(guard_log.exists());

        let tree = load_tree(
            &root.join(".runner/state/schema.json"),
            &root.join(".runner/state/tree.json"),
        )
        .expect("load tree");
        assert!(tree.passes);

        let iter_dir = root
            .join(".runner/iterations")
            .join(&outcome.run_id)
            .join(outcome.iter.to_string());
        assert!(iter_dir.join("meta.json").exists());
        assert!(iter_dir.join("output.json").exists());
    }

    /// Verifies guard failure produces a failure log and that the next iteration includes it in
    /// context.
    #[test]
    fn step_replays_failure_context_after_guard_fail() {
        let temp = tempfile::tempdir().expect("tempdir");
        let root = temp.path();
        init_git_repo(root);
        start_run(root).expect("start");

        let executor1 = FakeExecutor {
            output: AgentOutput {
                status: AgentStatus::Done,
                summary: "done".to_string(),
            },
            tree_update: None,
        };
        let guard_runner_fail = FakeGuardRunner {
            outcome: GuardOutcome::Fail,
        };

        let outcome1 =
            run_step(root, &executor1, &guard_runner_fail, &StepConfig::default()).expect("step1");
        assert_eq!(outcome1.guard, GuardOutcome::Fail);

        let iter1_dir = root
            .join(".runner/iterations")
            .join(&outcome1.run_id)
            .join(outcome1.iter.to_string());
        assert!(iter1_dir.join("guard.log").exists());

        let executor2 = FakeExecutor {
            output: AgentOutput {
                status: AgentStatus::Retry,
                summary: "retry".to_string(),
            },
            tree_update: None,
        };
        let guard_runner_unused = FakeGuardRunner {
            outcome: GuardOutcome::Pass,
        };
        run_step(
            root,
            &executor2,
            &guard_runner_unused,
            &StepConfig::default(),
        )
        .expect("step2");

        let failure_md = fs::read_to_string(root.join(".runner/context/failure.md"))
            .expect("read .runner/context/failure.md");
        assert!(failure_md.contains("guard output"));
    }

    /// Verifies runner-internal errors are not propagated into the agent context files.
    #[test]
    fn runner_errors_do_not_propagate_to_agent_context() {
        let temp = tempfile::tempdir().expect("tempdir");
        let root = temp.path();
        init_git_repo(root);
        start_run(root).expect("start");

        let executor = FakeExecutor {
            output: AgentOutput {
                status: AgentStatus::Done,
                summary: "ignored".to_string(),
            },
            tree_update: None,
        };

        struct AlwaysFailGuardRunner;
        impl GuardRunner for AlwaysFailGuardRunner {
            fn run(&self, _request: &crate::io::guards::GuardRequest) -> Result<GuardOutcome> {
                Err(anyhow!("boom"))
            }
        }

        let err = run_step(
            root,
            &executor,
            &AlwaysFailGuardRunner,
            &StepConfig::default(),
        )
        .expect_err("step should error");
        assert!(err.to_string().contains("boom"));

        let run_state =
            load_run_state(&root.join(".runner/state/run_state.json")).expect("run state");
        assert_eq!(run_state.last_status, Some(AgentStatus::Retry));
        assert_eq!(run_state.last_summary, None);

        // Next step should not include the runner error in history/failure context.
        let executor2 = FakeExecutor {
            output: AgentOutput {
                status: AgentStatus::Retry,
                summary: "needs work".to_string(),
            },
            tree_update: None,
        };
        let guard_runner2 = FakeGuardRunner {
            outcome: GuardOutcome::Pass,
        };
        run_step(root, &executor2, &guard_runner2, &StepConfig::default()).expect("step2");

        let history_md = fs::read_to_string(root.join(".runner/context/history.md"))
            .expect("read .runner/context/history.md");
        assert!(!history_md.contains("boom"));

        let failure_md = fs::read_to_string(root.join(".runner/context/failure.md"))
            .expect("read .runner/context/failure.md");
        assert!(!failure_md.contains("boom"));
    }

    /// Verifies decomposition adds children to the tree and skips guards.
    ///
    /// Uses FakeExecutor returning Decomposed status with tree update. Asserts:
    /// - outcome.status is Decomposed
    /// - Tree has new children added
    /// - No guard.log (guards skipped for decomposition)
    #[test]
    fn step_accepts_decomposition() {
        let temp = tempfile::tempdir().expect("tempdir");
        let root = temp.path();
        init_git_repo(root);
        start_run(root).expect("start");

        let mut decomposed = default_tree();
        decomposed
            .children
            .push(crate::test_support::node("child", 0));

        let executor = FakeExecutor {
            output: AgentOutput {
                status: AgentStatus::Decomposed,
                summary: "split".to_string(),
            },
            tree_update: Some(decomposed),
        };
        let guard_runner = FakeGuardRunner {
            outcome: GuardOutcome::Pass,
        };

        let outcome =
            run_step(root, &executor, &guard_runner, &StepConfig::default()).expect("step");
        assert_eq!(outcome.status, AgentStatus::Decomposed);

        let tree = load_tree(
            &root.join(".runner/state/schema.json"),
            &root.join(".runner/state/tree.json"),
        )
        .expect("load tree");
        assert_eq!(tree.children.len(), 1);

        let iter_dir = root
            .join(".runner/iterations")
            .join(&outcome.run_id)
            .join(outcome.iter.to_string());
        assert!(!iter_dir.join("guard.log").exists());
    }

    fn init_git_repo(root: &Path) {
        let status = Command::new("git")
            .arg("init")
            .current_dir(root)
            .status()
            .expect("git init");
        assert!(status.success());

        let status = Command::new("git")
            .args(["config", "user.email", "test@example.com"])
            .current_dir(root)
            .status()
            .expect("git config email");
        assert!(status.success());

        let status = Command::new("git")
            .args(["config", "user.name", "test"])
            .current_dir(root)
            .status()
            .expect("git config name");
        assert!(status.success());

        fs::write(root.join("README.md"), "hi\n").expect("write");
        let status = Command::new("git")
            .args(["add", "README.md"])
            .current_dir(root)
            .status()
            .expect("git add");
        assert!(status.success());

        let status = Command::new("git")
            .args(["commit", "-m", "chore: init"])
            .current_dir(root)
            .status()
            .expect("git commit");
        assert!(status.success());
    }
}
