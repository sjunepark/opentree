//! Executor interface and Codex backend.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use anyhow::{Context, Result, anyhow};

use crate::core::types::AgentOutput;

#[derive(Debug, Clone)]
pub struct ExecRequest {
    pub workdir: PathBuf,
    pub prompt: String,
    pub output_schema_path: PathBuf,
    pub output_path: PathBuf,
    pub executor_log_path: PathBuf,
}

pub trait Executor {
    fn exec(&self, request: &ExecRequest) -> Result<()>;
}

pub struct CodexExecutor;

impl Executor for CodexExecutor {
    fn exec(&self, request: &ExecRequest) -> Result<()> {
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
            .arg("--output-schema")
            .arg(&request.output_schema_path)
            .arg("--output-last-message")
            .arg(&request.output_path)
            .arg("-")
            .current_dir(&request.workdir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = cmd.spawn().context("spawn codex exec")?;
        if let Some(stdin) = child.stdin.as_mut() {
            stdin
                .write_all(request.prompt.as_bytes())
                .context("write prompt to codex stdin")?;
        }

        let output = child.wait_with_output().context("wait for codex exec")?;
        write_executor_log(&request.executor_log_path, &output.stdout, &output.stderr)?;

        if !output.status.success() {
            return Err(anyhow!(
                "codex exec failed with status {:?}",
                output.status.code()
            ));
        }

        Ok(())
    }
}

pub fn execute_and_load<E: Executor>(executor: &E, request: &ExecRequest) -> Result<AgentOutput> {
    executor.exec(request)?;
    ensure_output_exists(&request.output_path)?;
    read_agent_output(&request.output_path)
}

fn ensure_output_exists(path: &Path) -> Result<()> {
    if !path.exists() {
        return Err(anyhow!("missing executor output {}", path.display()));
    }
    Ok(())
}

fn read_agent_output(path: &Path) -> Result<AgentOutput> {
    let contents = fs::read_to_string(path)
        .with_context(|| format!("read agent output {}", path.display()))?;
    let output = serde_json::from_str(&contents)
        .with_context(|| format!("parse agent output {}", path.display()))?;
    Ok(output)
}

fn write_executor_log(path: &Path, stdout: &[u8], stderr: &[u8]) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("create executor log dir {}", parent.display()))?;
    }
    let mut buf = String::new();
    buf.push_str("=== stdout ===\n");
    buf.push_str(&String::from_utf8_lossy(stdout));
    buf.push_str("\n=== stderr ===\n");
    buf.push_str(&String::from_utf8_lossy(stderr));
    fs::write(path, buf).with_context(|| format!("write executor log {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::AgentStatus;

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

    #[test]
    fn execute_and_load_reads_output() {
        let temp = tempfile::tempdir().expect("tempdir");
        let request = ExecRequest {
            workdir: temp.path().to_path_buf(),
            prompt: "prompt".to_string(),
            output_schema_path: temp.path().join("schema.json"),
            output_path: temp.path().join("output.json"),
            executor_log_path: temp.path().join("executor.log"),
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

    #[test]
    fn execute_and_load_errors_on_missing_output() {
        let temp = tempfile::tempdir().expect("tempdir");
        let request = ExecRequest {
            workdir: temp.path().to_path_buf(),
            prompt: "prompt".to_string(),
            output_schema_path: temp.path().join("schema.json"),
            output_path: temp.path().join("output.json"),
            executor_log_path: temp.path().join("executor.log"),
        };
        let fake = FakeExecutor { output: None };

        let err = execute_and_load(&fake, &request).unwrap_err();
        assert!(err.to_string().contains("missing executor output"));
    }
}
