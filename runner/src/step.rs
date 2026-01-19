//! Orchestration for a single deterministic `runner step`.

use std::fs;
use std::path::Path;
use std::time::{Duration, Instant};

use anyhow::{Context, Result, anyhow};

use crate::agents::executor::ExecutorAgent;
use crate::agents::tree::TreeAgent;
use crate::core::budget::remaining_budget;
use crate::core::child_additions::validate_child_additions_restricted;
use crate::core::immutability::check_passed_node_immutability;
use crate::core::path::node_path;
use crate::core::selector::{is_stuck, leftmost_open_leaf};
use crate::core::state_update::apply_state_updates;
use crate::core::status_validator::validate_status_invariants;
use crate::core::types::{AgentOutput, AgentStatus, GuardOutcome, TreeChildSpec, TreeDecisionKind};
use crate::io::config::load_config;
use crate::io::context::{ContextPayload, write_context};
use crate::io::executor::Executor;
use crate::io::git::Git;
use crate::io::goal::read_goal_id;
use crate::io::guards::{GuardRequest, GuardRunner, run_guards_if_needed};
use crate::io::iteration_log::{IterationMeta, IterationWriteRequest, write_iteration};
use crate::io::prompt::PromptInputs;
use crate::io::run_state::{RunState, load_run_state, write_run_state};
use crate::io::tree_store::{load_tree, write_tree};
use crate::tree::Node;

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

/// Error when a stuck leaf is selected (hard-stop).
#[derive(Debug, Clone)]
pub struct StuckLeafError {
    pub id: String,
    pub path: String,
    pub attempts: u32,
    pub max_attempts: u32,
}

impl std::fmt::Display for StuckLeafError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "stuck leaf selected (hard-stop): id={} path={} attempts={}/{}",
            self.id, self.path, self.attempts, self.max_attempts
        )
    }
}

impl std::error::Error for StuckLeafError {}

/// Error when the run has exceeded the configured maximum number of iterations.
///
/// Note: `next_iter` is the *next* iteration number to run (1-indexed).
#[derive(Debug, Clone)]
pub struct MaxIterationsExceededError {
    pub next_iter: u32,
    pub max_iterations: u32,
}

impl std::fmt::Display for MaxIterationsExceededError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "max iterations exceeded: next_iter={} max_iterations={}",
            self.next_iter, self.max_iterations
        )
    }
}

impl std::error::Error for MaxIterationsExceededError {}

/// Execute one deterministic iteration of the agent loop.
///
/// Selects the leftmost open leaf, writes context, executes the agent,
/// validates output, runs guards if needed, and updates state.
#[tracing::instrument(skip_all, fields(run_id, iter, node_id))]
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
    if iter > cfg.max_iterations {
        return Err(MaxIterationsExceededError {
            next_iter: iter,
            max_iterations: cfg.max_iterations,
        }
        .into());
    }

    let prev_tree = load_tree(&schema_path, &tree_path)?;
    let selected = leftmost_open_leaf(&prev_tree)
        .ok_or_else(|| anyhow!("no open leaf found (tree already complete)"))?;
    let selected_id = selected.id.clone();
    tracing::Span::current().record("run_id", &run_id);
    tracing::Span::current().record("iter", iter);
    tracing::Span::current().record("node_id", &selected_id);

    let selected_path = node_path(&prev_tree, &selected_id)
        .ok_or_else(|| anyhow!("selected node path not found"))?;

    if is_stuck(selected) {
        return Err(StuckLeafError {
            id: selected.id.clone(),
            path: selected_path.clone(),
            attempts: selected.attempts,
            max_attempts: selected.max_attempts,
        }
        .into());
    }

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
    let tree_agent = TreeAgent::new(
        &state_dir,
        config.prompt_budget_bytes,
        cfg.executor_output_limit_bytes,
    );
    let executor_agent = ExecutorAgent::new(
        &state_dir,
        config.prompt_budget_bytes,
        cfg.executor_output_limit_bytes,
    );

    let iter_dir = root
        .join(".runner")
        .join("iterations")
        .join(&run_id)
        .join(iter.to_string());
    fs::create_dir_all(&iter_dir)
        .with_context(|| format!("create iteration dir {}", iter_dir.display()))?;

    let guard_log_path = iter_dir.join("guard.log");
    let runner_error_log_path = iter_dir.join("runner_error.log");

    let attempt = (|| -> Result<(AgentOutput, GuardOutcome, Node, Option<String>)> {
        // Phase 1: tree agent decides whether to decompose or execute.
        let tree_decision = tree_agent.run(executor, root, &iter_dir, &prompt_inputs, deadline)?;

        match tree_decision.decision {
            TreeDecisionKind::Decompose => {
                if tree_decision.children.is_empty() {
                    let msg =
                        "tree agent returned decision=decompose but children is empty".to_string();
                    let (output, tree_after) =
                        apply_agent_retry(&prev_tree, &selected_id, msg.clone())?;
                    write_tree(&tree_path, &tree_after)?;
                    return Ok((output, GuardOutcome::Skipped, tree_after, Some(msg)));
                }

                let next_tree = decompose_tree(
                    &prev_tree,
                    &selected_id,
                    &tree_decision.children,
                    cfg.max_attempts_default,
                )?;

                // Sanity-check: decomposition must not introduce children elsewhere.
                let errors =
                    validate_child_additions_restricted(&prev_tree, &next_tree, Some(&selected_id));
                if !errors.is_empty() {
                    return Err(anyhow!(
                        "child-additions restriction failed after decomposition: {}",
                        errors.join("; ")
                    ));
                }

                validate_post_exec_tree(&prev_tree, &next_tree)?;
                validate_status(
                    &prev_tree,
                    &next_tree,
                    &selected_id,
                    AgentStatus::Decomposed,
                )?;

                let output = AgentOutput {
                    status: AgentStatus::Decomposed,
                    summary: tree_decision.summary,
                };
                let guard_outcome = GuardOutcome::Skipped;

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

                Ok((output, guard_outcome, updated_tree, None))
            }
            TreeDecisionKind::Execute => {
                // Phase 2: executor agent performs work for the selected node.
                let output = executor_agent.run(
                    executor,
                    root,
                    &iter_dir,
                    &prompt_inputs,
                    Some(tree_decision.summary.as_str()),
                    deadline,
                )?;

                let next_tree = match load_tree(&schema_path, &tree_path) {
                    Ok(tree) => tree,
                    Err(err) => {
                        let msg = format!("tree invalid after executor: {err}");
                        let (output, tree_after) =
                            apply_agent_retry(&prev_tree, &selected_id, msg.clone())?;
                        write_tree(&tree_path, &tree_after)?;
                        return Ok((output, GuardOutcome::Skipped, tree_after, Some(msg)));
                    }
                };

                // Treat post-exec tree violations as agent contract errors (retry with context),
                // rather than hard-stopping the loop.
                let mut contract_errors = Vec::new();
                contract_errors.extend(check_passed_node_immutability(&prev_tree, &next_tree));
                contract_errors.extend(validate_child_additions_restricted(
                    &prev_tree, &next_tree, None,
                ));
                contract_errors.extend(validate_status_invariants(
                    &prev_tree,
                    &next_tree,
                    &selected_id,
                    output.status,
                ));
                if !contract_errors.is_empty() {
                    contract_errors.sort();
                    let msg = format!("agent contract violation: {}", contract_errors.join("; "));
                    let (output, tree_after) =
                        apply_agent_retry(&prev_tree, &selected_id, msg.clone())?;
                    write_tree(&tree_path, &tree_after)?;
                    return Ok((output, GuardOutcome::Skipped, tree_after, Some(msg)));
                }

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

                Ok((output, guard_outcome, updated_tree, None))
            }
        }
    })();

    let output: AgentOutput;
    let guard_outcome: GuardOutcome;
    let tree_after: Node;
    let runner_error_log: Option<String>;
    let mut step_error: Option<anyhow::Error> = None;

    match attempt {
        Ok((out, guard, tree, err_log)) => {
            output = out;
            guard_outcome = guard;
            tree_after = tree;
            runner_error_log = err_log;
        }
        Err(err) => {
            step_error = Some(err);

            runner_error_log = Some(format!("runner error: {}", step_error.as_ref().unwrap()));

            output = AgentOutput {
                status: AgentStatus::Retry,
                summary: "runner error (see runner_error.log)".to_string(),
            };
            guard_outcome = GuardOutcome::Skipped;
            // Runner-internal failures do not consume node attempts. Attempts increment only from
            // successful agent outputs via `apply_state_updates()`.
            tree_after = prev_tree.clone();

            write_tree(&tree_path, &tree_after)?;
        }
    }

    if let Some(contents) = &runner_error_log {
        if let Some(parent) = runner_error_log_path.parent() {
            fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
        }
        fs::write(&runner_error_log_path, format!("{contents}\n"))
            .with_context(|| format!("write {}", runner_error_log_path.display()))?;
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

fn apply_agent_retry(
    prev_tree: &Node,
    selected_id: &str,
    summary: String,
) -> Result<(AgentOutput, Node)> {
    let output = AgentOutput {
        status: AgentStatus::Retry,
        summary,
    };
    let guard_outcome = GuardOutcome::Skipped;

    let mut updated_tree = prev_tree.clone();
    apply_state_updates(
        prev_tree,
        &mut updated_tree,
        selected_id,
        output.status,
        guard_outcome,
    )
    .map_err(|err| anyhow!("state update failed: {err}"))?;
    Ok((output, updated_tree))
}

fn decompose_tree(
    prev_tree: &Node,
    selected_id: &str,
    children: &[TreeChildSpec],
    max_attempts_default: u32,
) -> Result<Node> {
    let mut used_ids = std::collections::HashSet::new();
    collect_ids(prev_tree, &mut used_ids);

    let mut next_tree = prev_tree.clone();
    let selected = find_node_mut(&mut next_tree, selected_id)
        .ok_or_else(|| anyhow!("selected node '{}' not found in tree", selected_id))?;

    for (idx, child) in children.iter().enumerate() {
        let id = allocate_child_id(selected_id, &used_ids);
        used_ids.insert(id.clone());

        selected.children.push(Node {
            id,
            order: idx as i64,
            title: child.title.clone(),
            goal: child.goal.clone(),
            acceptance: child.acceptance.clone(),
            passes: false,
            attempts: 0,
            max_attempts: max_attempts_default,
            children: Vec::new(),
        });
    }

    next_tree.sort_children();
    Ok(next_tree)
}

fn allocate_child_id(parent_id: &str, used_ids: &std::collections::HashSet<String>) -> String {
    // Deterministic: lowest available numeric suffix wins.
    for n in 1u32.. {
        let id = format!("{parent_id}.{n}");
        if !used_ids.contains(&id) {
            return id;
        }
    }
    unreachable!("u32 iterator is infinite")
}

fn collect_ids(node: &Node, out: &mut std::collections::HashSet<String>) {
    out.insert(node.id.clone());
    for child in &node.children {
        collect_ids(child, out);
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
    use crate::core::types::{AgentOutput, TreeChildSpec, TreeDecision, TreeDecisionKind};
    use crate::io::git::Git;
    use crate::io::guards::GuardRunner;
    use crate::test_support::{
        ScriptedExec, ScriptedExecutor, ScriptedGuard, ScriptedGuardRunner, ScriptedOutput,
        TestRepo, load_tree_fixture,
    };

    /// Verifies a retry iteration updates run_state and writes iteration logs.
    ///
    /// Uses scripted executor returning Retry status. Asserts:
    /// - run_state.next_iter increments
    /// - run_state.last_status is Retry
    /// - Iteration logs (meta, output, tree snapshots) exist
    /// - No guard.log (guards skipped for retry)
    #[test]
    fn step_updates_run_state_and_tree_on_retry() {
        let repo = TestRepo::new().expect("repo");
        let root = repo.root();
        repo.start_run().expect("start");

        let executor = ScriptedExecutor::new(vec![
            ScriptedExec {
                output: ScriptedOutput::TreeDecision(TreeDecision {
                    decision: TreeDecisionKind::Execute,
                    summary: "execute".to_string(),
                    children: Vec::new(),
                }),
                tree_update: None,
            },
            ScriptedExec {
                output: ScriptedOutput::AgentOutput(AgentOutput {
                    status: AgentStatus::Retry,
                    summary: "needs more".to_string(),
                }),
                tree_update: None,
            },
        ]);
        let guard_runner = ScriptedGuardRunner::new(vec![ScriptedGuard {
            outcome: GuardOutcome::Pass,
            log: "guard output".to_string(),
        }]);

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
        assert_eq!(guard_runner.remaining(), 1);
        executor.assert_drained().expect("executor drained");
    }

    /// Verifies Done + Pass marks the node as passed and writes guard log.
    ///
    /// Uses scripted executor returning Done status with passing guards. Asserts:
    /// - outcome.guard is Pass
    /// - guard.log exists (guards ran)
    /// - tree.passes is true (node completed successfully)
    /// - Iteration logs exist
    #[test]
    fn step_marks_done_and_writes_guard_log() {
        let repo = TestRepo::new().expect("repo");
        let root = repo.root();
        repo.start_run().expect("start");

        let executor = ScriptedExecutor::new(vec![
            ScriptedExec {
                output: ScriptedOutput::TreeDecision(TreeDecision {
                    decision: TreeDecisionKind::Execute,
                    summary: "execute".to_string(),
                    children: Vec::new(),
                }),
                tree_update: None,
            },
            ScriptedExec {
                output: ScriptedOutput::AgentOutput(AgentOutput {
                    status: AgentStatus::Done,
                    summary: "done".to_string(),
                }),
                tree_update: None,
            },
        ]);
        let guard_runner = ScriptedGuardRunner::new(vec![ScriptedGuard {
            outcome: GuardOutcome::Pass,
            log: "guard output".to_string(),
        }]);

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
        guard_runner.assert_drained().expect("guard drained");
        executor.assert_drained().expect("executor drained");
    }

    /// Verifies guard failure produces a failure log and that the next iteration includes it in
    /// context.
    #[test]
    fn step_replays_failure_context_after_guard_fail() {
        let repo = TestRepo::new().expect("repo");
        let root = repo.root();
        repo.start_run().expect("start");

        let executor = ScriptedExecutor::new(vec![
            ScriptedExec {
                output: ScriptedOutput::TreeDecision(TreeDecision {
                    decision: TreeDecisionKind::Execute,
                    summary: "execute".to_string(),
                    children: Vec::new(),
                }),
                tree_update: None,
            },
            ScriptedExec {
                output: ScriptedOutput::AgentOutput(AgentOutput {
                    status: AgentStatus::Done,
                    summary: "done".to_string(),
                }),
                tree_update: None,
            },
            ScriptedExec {
                output: ScriptedOutput::TreeDecision(TreeDecision {
                    decision: TreeDecisionKind::Execute,
                    summary: "execute".to_string(),
                    children: Vec::new(),
                }),
                tree_update: None,
            },
            ScriptedExec {
                output: ScriptedOutput::AgentOutput(AgentOutput {
                    status: AgentStatus::Retry,
                    summary: "retry".to_string(),
                }),
                tree_update: None,
            },
        ]);
        let guard_runner = ScriptedGuardRunner::new(vec![ScriptedGuard {
            outcome: GuardOutcome::Fail,
            log: "guard failure".to_string(),
        }]);

        let outcome1 =
            run_step(root, &executor, &guard_runner, &StepConfig::default()).expect("step1");
        assert_eq!(outcome1.guard, GuardOutcome::Fail);

        let iter1_dir = root
            .join(".runner/iterations")
            .join(&outcome1.run_id)
            .join(outcome1.iter.to_string());
        assert!(iter1_dir.join("guard.log").exists());

        let tree_after = repo.read_tree().expect("read tree");
        assert_eq!(tree_after.attempts, 1);
        assert!(!tree_after.passes);

        run_step(root, &executor, &guard_runner, &StepConfig::default()).expect("step2");

        let failure_md = fs::read_to_string(root.join(".runner/context/failure.md"))
            .expect("read .runner/context/failure.md");
        assert!(failure_md.contains("guard failure"));
        guard_runner.assert_drained().expect("guard drained");
        executor.assert_drained().expect("executor drained");
    }

    /// Verifies runner-internal errors are not propagated into the agent context files.
    #[test]
    fn runner_errors_do_not_propagate_to_agent_context() {
        let repo = TestRepo::new().expect("repo");
        let root = repo.root();
        repo.start_run().expect("start");

        let executor = ScriptedExecutor::new(vec![
            ScriptedExec {
                output: ScriptedOutput::TreeDecision(TreeDecision {
                    decision: TreeDecisionKind::Execute,
                    summary: "execute".to_string(),
                    children: Vec::new(),
                }),
                tree_update: None,
            },
            ScriptedExec {
                output: ScriptedOutput::AgentOutput(AgentOutput {
                    status: AgentStatus::Done,
                    summary: "ignored".to_string(),
                }),
                tree_update: None,
            },
            ScriptedExec {
                output: ScriptedOutput::TreeDecision(TreeDecision {
                    decision: TreeDecisionKind::Execute,
                    summary: "execute".to_string(),
                    children: Vec::new(),
                }),
                tree_update: None,
            },
            ScriptedExec {
                output: ScriptedOutput::AgentOutput(AgentOutput {
                    status: AgentStatus::Retry,
                    summary: "needs work".to_string(),
                }),
                tree_update: None,
            },
        ]);

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

        let tree = load_tree(
            &root.join(".runner/state/schema.json"),
            &root.join(".runner/state/tree.json"),
        )
        .expect("load tree");
        assert_eq!(tree.attempts, 0);

        // Next step should not include the runner error in history/failure context.
        let guard_runner2 = ScriptedGuardRunner::new(Vec::new());
        run_step(root, &executor, &guard_runner2, &StepConfig::default()).expect("step2");

        let history_md = fs::read_to_string(root.join(".runner/context/history.md"))
            .expect("read .runner/context/history.md");
        assert!(!history_md.contains("boom"));

        let failure_md = fs::read_to_string(root.join(".runner/context/failure.md"))
            .expect("read .runner/context/failure.md");
        assert!(!failure_md.contains("boom"));

        executor.assert_drained().expect("executor drained");
    }

    /// Verifies decomposition adds children to the tree and skips guards.
    ///
    /// Uses scripted executor returning Decomposed status with tree update. Asserts:
    /// - outcome.status is Decomposed
    /// - Tree has new children added
    /// - No guard.log (guards skipped for decomposition)
    #[test]
    fn step_accepts_decomposition() {
        let repo = TestRepo::new().expect("repo");
        let root = repo.root();
        repo.start_run().expect("start");

        let executor = ScriptedExecutor::new(vec![ScriptedExec {
            output: ScriptedOutput::TreeDecision(TreeDecision {
                decision: TreeDecisionKind::Decompose,
                summary: "split".to_string(),
                children: vec![TreeChildSpec {
                    title: "Child".to_string(),
                    goal: "Do child work".to_string(),
                    acceptance: Vec::new(),
                }],
            }),
            tree_update: None,
        }]);
        let guard_runner = ScriptedGuardRunner::new(Vec::new());

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
        guard_runner.assert_drained().expect("guard drained");
        executor.assert_drained().expect("executor drained");
    }

    /// Verifies retry then done writes history context and guard log on the second iter.
    #[test]
    fn step_retries_then_done_writes_history_and_guards() {
        let repo = TestRepo::new().expect("repo");
        let root = repo.root();
        let start = repo.start_run().expect("start");

        let executor = ScriptedExecutor::new(vec![
            ScriptedExec {
                output: ScriptedOutput::TreeDecision(TreeDecision {
                    decision: TreeDecisionKind::Execute,
                    summary: "execute".to_string(),
                    children: Vec::new(),
                }),
                tree_update: None,
            },
            ScriptedExec {
                output: ScriptedOutput::AgentOutput(AgentOutput {
                    status: AgentStatus::Retry,
                    summary: "needs more".to_string(),
                }),
                tree_update: None,
            },
            ScriptedExec {
                output: ScriptedOutput::TreeDecision(TreeDecision {
                    decision: TreeDecisionKind::Execute,
                    summary: "execute".to_string(),
                    children: Vec::new(),
                }),
                tree_update: None,
            },
            ScriptedExec {
                output: ScriptedOutput::AgentOutput(AgentOutput {
                    status: AgentStatus::Done,
                    summary: "done".to_string(),
                }),
                tree_update: None,
            },
        ]);
        let guard_runner = ScriptedGuardRunner::new(vec![ScriptedGuard {
            outcome: GuardOutcome::Pass,
            log: "guard ok".to_string(),
        }]);

        let outcome1 =
            run_step(root, &executor, &guard_runner, &StepConfig::default()).expect("step1");
        assert_eq!(outcome1.guard, GuardOutcome::Skipped);

        let iter1_dir = root
            .join(".runner/iterations")
            .join(&start.run_id)
            .join(outcome1.iter.to_string());
        assert!(iter1_dir.join("meta.json").exists());
        assert!(iter1_dir.join("output.json").exists());
        assert!(iter1_dir.join("tree.before.json").exists());
        assert!(iter1_dir.join("tree.after.json").exists());
        assert!(!iter1_dir.join("guard.log").exists());

        let outcome2 =
            run_step(root, &executor, &guard_runner, &StepConfig::default()).expect("step2");
        assert_eq!(outcome2.guard, GuardOutcome::Pass);

        let iter2_dir = root
            .join(".runner/iterations")
            .join(&start.run_id)
            .join(outcome2.iter.to_string());
        assert!(iter2_dir.join("meta.json").exists());
        assert!(iter2_dir.join("output.json").exists());
        assert!(iter2_dir.join("tree.before.json").exists());
        assert!(iter2_dir.join("tree.after.json").exists());
        assert!(iter2_dir.join("guard.log").exists());

        let run_state = repo.read_run_state().expect("run state");
        assert_eq!(run_state.next_iter, 3);
        assert_eq!(run_state.last_status, Some(AgentStatus::Done));
        assert_eq!(run_state.last_guard, Some(GuardOutcome::Pass));

        let history_md = fs::read_to_string(root.join(".runner/context/history.md"))
            .expect("read .runner/context/history.md");
        assert!(history_md.contains("needs more"));

        guard_runner.assert_drained().expect("guard drained");
        executor.assert_drained().expect("executor drained");
    }

    /// Verifies a stuck leaf triggers a hard-stop without invoking the executor.
    #[test]
    fn step_hard_stops_on_stuck_leaf() {
        let repo = TestRepo::new().expect("repo");
        let root = repo.root();
        let start = repo.start_run().expect("start");

        let fixture = load_tree_fixture("tree_with_stuck_leaf").expect("fixture");
        repo.write_tree(&fixture).expect("write tree");
        let git = Git::new(root);
        git.add_all().expect("git add");
        assert!(git.commit_staged("chore: fixture").expect("git commit"));

        let executor = ScriptedExecutor::new(Vec::new());
        let guard_runner = ScriptedGuardRunner::new(Vec::new());

        let err =
            run_step(root, &executor, &guard_runner, &StepConfig::default()).expect_err("step");
        let msg = err.to_string();
        assert!(msg.contains("stuck leaf selected"));
        assert!(msg.contains("id=stuck"));
        assert!(msg.contains("path=root/stuck"));
        assert!(msg.contains("attempts=2/2"));

        let iter_dir = root
            .join(".runner/iterations")
            .join(&start.run_id)
            .join("1");
        assert!(!iter_dir.exists());

        assert!(executor.last_request().is_none());
        assert!(guard_runner.last_request().is_none());
    }

    /// Verifies the runner retries when the tree agent requests decomposition with no children.
    #[test]
    fn step_retries_when_tree_agent_decomposes_without_children() {
        let repo = TestRepo::new().expect("repo");
        let root = repo.root();
        let start = repo.start_run().expect("start");

        let fixture = load_tree_fixture("simple_tree").expect("fixture");
        repo.write_tree(&fixture).expect("write tree");
        let git = Git::new(root);
        git.add_all().expect("git add");
        assert!(git.commit_staged("chore: fixture").expect("git commit"));

        let executor = ScriptedExecutor::new(vec![ScriptedExec {
            output: ScriptedOutput::TreeDecision(TreeDecision {
                decision: TreeDecisionKind::Decompose,
                summary: "missing children".to_string(),
                children: Vec::new(),
            }),
            tree_update: None,
        }]);
        let guard_runner = ScriptedGuardRunner::new(Vec::new());

        let outcome =
            run_step(root, &executor, &guard_runner, &StepConfig::default()).expect("step");
        assert_eq!(outcome.status, AgentStatus::Retry);

        let iter_dir = root
            .join(".runner/iterations")
            .join(&start.run_id)
            .join("1");
        let runner_error =
            fs::read_to_string(iter_dir.join("runner_error.log")).expect("read runner_error.log");
        assert!(runner_error.contains("decision=decompose"));

        guard_runner.assert_drained().expect("guard drained");
        executor.assert_drained().expect("executor drained");
    }

    /// Verifies passed-node immutability violations are detected and logged.
    #[test]
    fn step_detects_passed_node_immutability_violation() {
        let repo = TestRepo::new().expect("repo");
        let root = repo.root();
        let start = repo.start_run().expect("start");

        let fixture = load_tree_fixture("tree_with_passed_node").expect("fixture");
        repo.write_tree(&fixture).expect("write tree");
        let git = Git::new(root);
        git.add_all().expect("git add");
        assert!(git.commit_staged("chore: fixture").expect("git commit"));

        let mut mutated = fixture.clone();
        if let Some(passed_node) = mutated.children.first_mut() {
            passed_node.title = "mutated".to_string();
        }

        let executor = ScriptedExecutor::new(vec![
            ScriptedExec {
                output: ScriptedOutput::TreeDecision(TreeDecision {
                    decision: TreeDecisionKind::Execute,
                    summary: "execute".to_string(),
                    children: Vec::new(),
                }),
                tree_update: None,
            },
            ScriptedExec {
                output: ScriptedOutput::AgentOutput(AgentOutput {
                    status: AgentStatus::Retry,
                    summary: "retry".to_string(),
                }),
                tree_update: Some(mutated),
            },
        ]);
        let guard_runner = ScriptedGuardRunner::new(Vec::new());

        let outcome =
            run_step(root, &executor, &guard_runner, &StepConfig::default()).expect("step");
        assert_eq!(outcome.status, AgentStatus::Retry);

        let stored = repo.read_tree().expect("read tree");
        assert_eq!(stored.children[0].id, "done");
        assert_eq!(stored.children[0].title, "Done node");
        assert_eq!(stored.children[1].id, "open");
        assert_eq!(stored.children[1].attempts, 1);

        let iter_dir = root
            .join(".runner/iterations")
            .join(&start.run_id)
            .join("1");
        let runner_error =
            fs::read_to_string(iter_dir.join("runner_error.log")).expect("read runner_error.log");
        assert!(runner_error.contains("passed node 'done' changed"));

        guard_runner.assert_drained().expect("guard drained");
        executor.assert_drained().expect("executor drained");
    }
}
