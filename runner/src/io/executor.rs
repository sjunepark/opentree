//! Executor abstraction for agent invocation.
//!
//! The [`Executor`] trait decouples step orchestration from the actual agent
//! backend (currently `codex exec`). Tests use scripted executors that return
//! predetermined outputs without spawning processes.

use std::time::Duration;

use anyhow::{Context, Result, anyhow};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::{debug, info, instrument, warn};

use crate::core::types::AgentOutput;
use crate::io::process::{CommandOutput, run_command_with_stream};
use serde::de::DeserializeOwned;

/// Parameters for an executor invocation.
#[derive(Debug, Clone)]
pub struct ExecRequest {
    /// Working directory for the executor process.
    pub workdir: PathBuf,
    /// Prompt text to feed to the agent.
    pub prompt: String,
    /// Path to the JSON Schema that constrains agent output.
    pub output_schema_path: PathBuf,
    /// Path where the agent must write its output JSON.
    pub output_path: PathBuf,
    /// Path to write executor stdout/stderr log.
    pub executor_log_path: PathBuf,
    /// Maximum time to wait for the executor to complete.
    pub timeout: Duration,
    /// Truncate executor output logs beyond this many bytes.
    pub output_limit_bytes: usize,
    /// Path to write JSONL event stream. When `Some`, enables `--json` flag
    /// and writes stdout lines incrementally for real-time observability.
    pub stream_path: Option<PathBuf>,
}

/// Abstraction over agent execution backends.
pub trait Executor {
    /// Run the agent with the given request. Must write output to `request.output_path`.
    fn exec(&self, request: &ExecRequest) -> Result<()>;
}

/// Executor that spawns `codex exec`.
pub struct CodexExecutor;

impl Executor for CodexExecutor {
    #[instrument(skip_all, fields(timeout_secs = request.timeout.as_secs(), streaming = request.stream_path.is_some()))]
    fn exec(&self, request: &ExecRequest) -> Result<()> {
        info!(workdir = %request.workdir.display(), "starting codex exec");

        if !request.output_schema_path.exists() {
            return Err(anyhow!(
                "missing output schema {}",
                request.output_schema_path.display()
            ));
        }
        if let Some(parent) = request.output_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("create output dir {}", parent.display()))?;
        }
        let mut cmd = Command::new("codex");
        cmd.arg("exec")
            .arg("-c")
            .arg("model_reasoning_effort=medium")
            .arg("--sandbox")
            .arg("danger-full-access")
            // Allow running in directories without a git repository. Required for tests
            // that use temp directories, and for workspaces not yet under version control.
            .arg("--skip-git-repo-check");

        // Enable JSON streaming when stream_path is set
        if request.stream_path.is_some() {
            cmd.arg("--json");
        }

        cmd.arg("--output-schema")
            .arg(&request.output_schema_path)
            .arg("--output-last-message")
            .arg(&request.output_path)
            .arg("-")
            .current_dir(&request.workdir)
            .stdin(std::process::Stdio::piped());

        let output = run_command_with_stream(
            cmd,
            Some(request.prompt.as_bytes()),
            request.timeout,
            request.output_limit_bytes,
            request.stream_path.as_deref(),
        )
        .context("run codex exec")?;

        write_executor_log(
            &request.executor_log_path,
            &output,
            request.output_limit_bytes,
        )?;

        if output.timed_out {
            warn!(
                timeout_secs = request.timeout.as_secs(),
                "codex exec timed out"
            );
            return Err(anyhow!("codex exec timed out after {:?}", request.timeout));
        }
        if !output.status.success() {
            warn!(exit_code = ?output.status.code(), "codex exec failed");
            return Err(anyhow!(
                "codex exec failed with status {:?}",
                output.status.code()
            ));
        }

        debug!("codex exec completed successfully");
        Ok(())
    }
}

/// Execute the agent and load its output.
#[instrument(skip_all, fields(output_path = %request.output_path.display()))]
pub fn execute_and_load<E: Executor>(executor: &E, request: &ExecRequest) -> Result<AgentOutput> {
    let output: AgentOutput = execute_and_load_json(executor, request)?;
    debug!(status = ?output.status, "parsed agent output");
    Ok(output)
}

/// Execute the agent and load its output as JSON of type `T`.
#[instrument(skip_all, fields(output_path = %request.output_path.display()))]
pub fn execute_and_load_json<E: Executor, T: DeserializeOwned>(
    executor: &E,
    request: &ExecRequest,
) -> Result<T> {
    executor.exec(request)?;
    ensure_output_exists(&request.output_path)?;
    read_output_json(&request.output_path)
}

fn ensure_output_exists(path: &Path) -> Result<()> {
    if !path.exists() {
        return Err(anyhow!("missing executor output {}", path.display()));
    }
    Ok(())
}

fn read_output_json<T: DeserializeOwned>(path: &Path) -> Result<T> {
    let contents = fs::read_to_string(path)
        .with_context(|| format!("read agent output {}", path.display()))?;
    let value =
        serde_json::from_str(&contents).with_context(|| format!("parse {}", path.display()))?;
    Ok(value)
}

fn write_executor_log(path: &Path, output: &CommandOutput, output_limit: usize) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("create executor log dir {}", parent.display()))?;
    }
    let mut buf = String::new();
    buf.push_str("=== stdout ===\n");
    buf.push_str(&String::from_utf8_lossy(&output.stdout));
    buf.push_str(&output.stdout_truncated_notice("executor"));
    buf.push_str("\n=== stderr ===\n");
    buf.push_str(&String::from_utf8_lossy(&output.stderr));
    buf.push_str(&output.stderr_truncated_notice("executor"));
    if output.timed_out {
        buf.push_str("\n[executor timed out]\n");
    }

    if buf.len() > output_limit {
        let truncated = format!(
            "{}\n[truncated {} bytes]\n",
            &buf[..output_limit],
            buf.len() - output_limit
        );
        fs::write(path, truncated)
            .with_context(|| format!("write executor log {}", path.display()))?;
        return Ok(());
    }

    fs::write(path, buf).with_context(|| format!("write executor log {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::AgentStatus;
    use std::time::Duration;

    struct FakeExecutor {
        output: Option<AgentOutput>,
    }

    impl Executor for FakeExecutor {
        fn exec(&self, request: &ExecRequest) -> Result<()> {
            if let Some(output) = &self.output {
                let mut buf = serde_json::to_string_pretty(output)?;
                buf.push('\n');
                fs::write(&request.output_path, buf)?;
            }
            Ok(())
        }
    }

    /// Verifies execute_and_load successfully parses agent output.
    ///
    /// Uses a FakeExecutor that writes valid output, then checks the parsed result.
    #[test]
    fn execute_and_load_reads_output() {
        let temp = tempfile::tempdir().expect("tempdir");
        let request = ExecRequest {
            workdir: temp.path().to_path_buf(),
            prompt: "prompt".to_string(),
            output_schema_path: temp.path().join("schema.json"),
            output_path: temp.path().join("output.json"),
            executor_log_path: temp.path().join("executor.log"),
            timeout: Duration::from_secs(1),
            output_limit_bytes: 1000,
            stream_path: None,
        };
        let fake = FakeExecutor {
            output: Some(AgentOutput {
                status: AgentStatus::Done,
                summary: "ok".to_string(),
            }),
        };

        let output = execute_and_load(&fake, &request).expect("load");
        assert_eq!(output.summary, "ok");
    }

    /// Verifies execute_and_load fails when output file is missing.
    ///
    /// Uses a FakeExecutor that doesn't write output, expects an error.
    #[test]
    fn execute_and_load_errors_on_missing_output() {
        let temp = tempfile::tempdir().expect("tempdir");
        let request = ExecRequest {
            workdir: temp.path().to_path_buf(),
            prompt: "prompt".to_string(),
            output_schema_path: temp.path().join("schema.json"),
            output_path: temp.path().join("output.json"),
            executor_log_path: temp.path().join("executor.log"),
            timeout: Duration::from_secs(1),
            output_limit_bytes: 1000,
            stream_path: None,
        };
        let fake = FakeExecutor { output: None };

        let err = execute_and_load(&fake, &request).unwrap_err();
        assert!(err.to_string().contains("missing executor output"));
    }
}
