//! Executor agent for performing work on a selected node.

use std::path::{Path, PathBuf};
use std::time::Instant;

use anyhow::Result;

use crate::core::budget::remaining_budget;
use crate::core::types::AgentOutput;
use crate::io::executor::{ExecRequest, Executor, execute_and_load};
use crate::io::prompt::{PromptBuilder, PromptInputs};

use super::write_output_schema;

const EXECUTOR_OUTPUT_SCHEMA: &str = include_str!("../../schemas/executor_output.schema.json");

/// Configuration for an executor-agent invocation.
#[derive(Debug, Clone)]
pub struct ExecutorAgentConfig {
    pub prompt_budget_bytes: usize,
    pub output_limit_bytes: usize,
}

/// Executor agent wrapper that owns schema and prompt settings.
#[derive(Debug, Clone)]
pub struct ExecutorAgent {
    schema_path: PathBuf,
    config: ExecutorAgentConfig,
}

impl ExecutorAgent {
    pub fn new(state_dir: &Path, prompt_budget_bytes: usize, output_limit_bytes: usize) -> Self {
        Self {
            schema_path: state_dir.join("executor_output.schema.json"),
            config: ExecutorAgentConfig {
                prompt_budget_bytes,
                output_limit_bytes,
            },
        }
    }

    pub fn allows_side_effects(&self) -> bool {
        true
    }

    pub fn run<E: Executor>(
        &self,
        executor: &E,
        root: &Path,
        iter_dir: &Path,
        inputs: &PromptInputs,
        planner_notes: Option<&str>,
        deadline: Instant,
    ) -> Result<AgentOutput> {
        write_output_schema(&self.schema_path, EXECUTOR_OUTPUT_SCHEMA)?;

        let prompt = PromptBuilder::new(self.config.prompt_budget_bytes)
            .build_executor(inputs, planner_notes)
            .render();

        let request = ExecRequest {
            workdir: root.to_path_buf(),
            prompt,
            output_schema_path: self.schema_path.clone(),
            output_path: iter_dir.join("output.json"),
            executor_log_path: iter_dir.join("executor.log"),
            timeout: remaining_budget(deadline)?,
            output_limit_bytes: self.config.output_limit_bytes,
            stream_path: Some(iter_dir.join("stream.jsonl")),
        };

        execute_and_load(executor, &request)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::AgentStatus;
    use crate::io::executor::ExecRequest;
    use crate::test_support::node;
    use std::cell::RefCell;
    use std::fs;
    use std::time::Duration;

    struct CapturingExecutor {
        output: AgentOutput,
        last_request: RefCell<Option<ExecRequest>>,
    }

    impl CapturingExecutor {
        fn new(output: AgentOutput) -> Self {
            Self {
                output,
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
            let mut buf = serde_json::to_string_pretty(&self.output)?;
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
    fn executor_agent_runs_with_schema_and_prompt() {
        let temp = tempfile::tempdir().expect("tempdir");
        let state_dir = temp.path().join(".runner/state");
        let iter_dir = temp.path().join(".runner/iterations/run-1/1");
        fs::create_dir_all(&state_dir).expect("state dir");
        fs::create_dir_all(&iter_dir).expect("iter dir");

        let output = AgentOutput {
            status: AgentStatus::Done,
            summary: "ok".to_string(),
        };
        let executor = CapturingExecutor::new(output.clone());
        let agent = ExecutorAgent::new(&state_dir, 1024, 2048);

        let got = agent
            .run(
                &executor,
                temp.path(),
                &iter_dir,
                &sample_inputs(),
                Some("notes"),
                Instant::now() + Duration::from_secs(5),
            )
            .expect("run");

        assert_eq!(got, output);
        assert!(agent.schema_path.exists());
        let request = executor.last_request.borrow().clone().expect("request");
        assert!(request.prompt.contains("Executor Contract"));
        assert!(request.output_path.ends_with("output.json"));
        assert!(agent.allows_side_effects());
    }
}
