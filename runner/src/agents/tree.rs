//! Tree agent for decomposition decisions.

use std::path::{Path, PathBuf};
use std::time::Instant;

use anyhow::Result;

use crate::core::budget::remaining_budget;
use crate::core::types::TreeDecision;
use crate::io::executor::{ExecRequest, Executor, execute_and_load_json};
use crate::io::prompt::{PromptBuilder, PromptInputs};

use super::write_output_schema;

const TREE_DECISION_SCHEMA: &str = include_str!("../../schemas/tree_decision.schema.json");

/// Configuration for a tree-agent invocation.
#[derive(Debug, Clone)]
pub struct TreeAgentConfig {
    pub prompt_budget_bytes: usize,
    pub output_limit_bytes: usize,
}

/// Tree agent wrapper that owns schema and prompt settings.
#[derive(Debug, Clone)]
pub struct TreeAgent {
    schema_path: PathBuf,
    config: TreeAgentConfig,
}

impl TreeAgent {
    pub fn new(state_dir: &Path, prompt_budget_bytes: usize, output_limit_bytes: usize) -> Self {
        Self {
            schema_path: state_dir.join("tree_decision.schema.json"),
            config: TreeAgentConfig {
                prompt_budget_bytes,
                output_limit_bytes,
            },
        }
    }

    pub fn allows_side_effects(&self) -> bool {
        false
    }

    pub fn run<E: Executor>(
        &self,
        executor: &E,
        root: &Path,
        iter_dir: &Path,
        inputs: &PromptInputs,
        deadline: Instant,
    ) -> Result<TreeDecision> {
        write_output_schema(&self.schema_path, TREE_DECISION_SCHEMA)?;

        let prompt = PromptBuilder::new(self.config.prompt_budget_bytes)
            .build_tree_agent(inputs)
            .render();

        let request = ExecRequest {
            workdir: root.to_path_buf(),
            prompt,
            output_schema_path: self.schema_path.clone(),
            output_path: iter_dir.join("planner_output.json"),
            executor_log_path: iter_dir.join("planner_executor.log"),
            timeout: remaining_budget(deadline)?,
            output_limit_bytes: self.config.output_limit_bytes,
            stream_path: Some(iter_dir.join("planner_stream.jsonl")),
        };

        execute_and_load_json(executor, &request)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::TreeDecisionKind;
    use crate::io::executor::ExecRequest;
    use crate::test_support::node;
    use std::cell::RefCell;
    use std::fs;
    use std::time::Duration;

    struct CapturingExecutor {
        decision: TreeDecision,
        last_request: RefCell<Option<ExecRequest>>,
    }

    impl CapturingExecutor {
        fn new(decision: TreeDecision) -> Self {
            Self {
                decision,
                last_request: RefCell::new(None),
            }
        }
    }

    impl Executor for CapturingExecutor {
        fn exec(&self, request: &ExecRequest) -> Result<()> {
            *self.last_request.borrow_mut() = Some(request.clone());
            if let Some(parent) = request.output_path.parent() {
                fs::create_dir_all(parent)?;
            }
            let mut buf = serde_json::to_string_pretty(&self.decision)?;
            buf.push('\n');
            fs::write(&request.output_path, buf)?;
            Ok(())
        }
    }

    fn sample_inputs() -> PromptInputs {
        PromptInputs {
            selected_path: "root".to_string(),
            selected_node: node("root", 0),
            tree_summary: "- root".to_string(),
            context_goal: "goal".to_string(),
            context_history: None,
            context_failure: None,
            assumptions: String::new(),
            questions: String::new(),
        }
    }

    #[test]
    fn tree_agent_runs_with_schema_and_prompt() {
        let temp = tempfile::tempdir().expect("tempdir");
        let state_dir = temp.path().join(".runner/state");
        let iter_dir = temp.path().join(".runner/iterations/run-1/1");
        fs::create_dir_all(&state_dir).expect("state dir");
        fs::create_dir_all(&iter_dir).expect("iter dir");

        let decision = TreeDecision {
            decision: TreeDecisionKind::Execute,
            summary: "go".to_string(),
            children: Vec::new(),
        };
        let executor = CapturingExecutor::new(decision.clone());
        let agent = TreeAgent::new(&state_dir, 1024, 2048);

        let got = agent
            .run(
                &executor,
                temp.path(),
                &iter_dir,
                &sample_inputs(),
                Instant::now() + Duration::from_secs(5),
            )
            .expect("run");

        assert_eq!(got, decision);
        assert!(agent.schema_path.exists());
        let request = executor.last_request.borrow().clone().expect("request");
        assert!(request.prompt.contains("Tree Agent Contract"));
        assert!(request.output_path.ends_with("planner_output.json"));
        assert!(!agent.allows_side_effects());
    }
}
