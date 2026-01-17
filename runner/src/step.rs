//! Orchestration for a single deterministic `runner step`.

use std::fs;
use std::path::Path;
use std::time::Instant;

use anyhow::{Context, Result, anyhow};

use crate::core::immutability::check_passed_node_immutability;
use crate::core::selector::leftmost_open_leaf;
use crate::core::state_update::apply_state_updates;
use crate::core::status_validator::validate_status_invariants;
use crate::core::types::{AgentStatus, GuardOutcome};
use crate::io::context::{ContextPayload, write_context};
use crate::io::executor::{ExecRequest, Executor, execute_and_load};
use crate::io::guards::{
    DEFAULT_GUARD_TIMEOUT, DEFAULT_OUTPUT_LIMIT_BYTES, GuardRequest, GuardRunner,
    run_guards_if_needed,
};
use crate::io::iteration_log::{IterationMeta, IterationWriteRequest, write_iteration};
use crate::io::prompt::{PromptBuilder, PromptInputs};
use crate::io::run_state::{RunState, load_run_state, write_run_state};
use crate::io::tree_store::{load_tree, write_tree};
use crate::tree::Node;

const AGENT_OUTPUT_SCHEMA: &str = r#"{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "AgentOutput",
  "type": "object",
  "additionalProperties": false,
  "required": ["status", "summary"],
  "properties": {
    "status": {
      "type": "string",
      "enum": ["done", "retry", "decomposed"]
    },
    "summary": {
      "type": "string"
    }
  }
}
"#;

#[derive(Debug, Clone)]
pub struct StepConfig {
    pub prompt_budget_bytes: usize,
}

impl Default for StepConfig {
    fn default() -> Self {
        Self {
            prompt_budget_bytes: 40_000,
        }
    }
}

#[derive(Debug, Clone)]
pub struct StepOutcome {
    pub run_id: String,
    pub iter: u32,
    pub selected_id: String,
    pub status: AgentStatus,
    pub guard: GuardOutcome,
}

pub fn run_step<E: Executor, G: GuardRunner>(
    root: &Path,
    executor: &E,
    guard_runner: &G,
    config: &StepConfig,
) -> Result<StepOutcome> {
    let start = Instant::now();
    let state_dir = root.join(".runner").join("state");
    let tree_path = state_dir.join("tree.json");
    let schema_path = state_dir.join("schema.json");
    let run_state_path = state_dir.join("run_state.json");
    let output_schema_path = state_dir.join("agent_output.schema.json");

    let mut run_state = load_or_default_run_state(&run_state_path)?;
    let run_id = run_state
        .run_id
        .clone()
        .unwrap_or_else(|| "run-1".to_string());
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

    let exec_request = ExecRequest {
        workdir: root.to_path_buf(),
        prompt: prompt_pack.render(),
        output_schema_path: output_schema_path.clone(),
        output_path: iter_dir.join("output.json"),
        executor_log_path: iter_dir.join("executor.log"),
    };

    let output = execute_and_load(executor, &exec_request)?;

    let next_tree = load_tree(&schema_path, &tree_path)?;
    validate_post_exec_tree(&prev_tree, &next_tree)?;
    validate_status(&prev_tree, &next_tree, &selected_id, output.status)?;

    let guard_log_path = iter_dir.join("guard.log");
    let guard_outcome = run_guards_if_needed(
        output.status,
        guard_runner,
        &GuardRequest {
            workdir: root.to_path_buf(),
            log_path: guard_log_path.clone(),
            timeout: DEFAULT_GUARD_TIMEOUT,
            output_limit_bytes: DEFAULT_OUTPUT_LIMIT_BYTES,
        },
    )?;

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

    let guard_log = if guard_outcome == GuardOutcome::Pass || guard_outcome == GuardOutcome::Fail {
        fs::read_to_string(&guard_log_path).ok()
    } else {
        None
    };
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
        tree_after: &updated_tree,
    })?;

    run_state.run_id = Some(run_id.clone());
    run_state.next_iter = iter + 1;
    run_state.last_status = Some(output.status);
    run_state.last_summary = Some(output.summary.clone());
    run_state.last_guard = Some(guard_outcome);
    write_run_state(&run_state_path, &run_state)?;

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
    use crate::io::init::{InitOptions, init_runner};
    use crate::tree::default_tree;

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

    #[test]
    fn step_updates_run_state_and_tree_on_retry() {
        let temp = tempfile::tempdir().expect("tempdir");
        let root = temp.path();
        init_runner(root, &InitOptions { force: false }).expect("init");
        write_tree(&root.join(".runner/state/tree.json"), &default_tree()).expect("write tree");

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

    #[test]
    fn step_marks_done_and_writes_guard_log() {
        let temp = tempfile::tempdir().expect("tempdir");
        let root = temp.path();
        init_runner(root, &InitOptions { force: false }).expect("init");
        write_tree(&root.join(".runner/state/tree.json"), &default_tree()).expect("write tree");

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

    #[test]
    fn step_accepts_decomposition() {
        let temp = tempfile::tempdir().expect("tempdir");
        let root = temp.path();
        init_runner(root, &InitOptions { force: false }).expect("init");
        write_tree(&root.join(".runner/state/tree.json"), &default_tree()).expect("write tree");

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
}
