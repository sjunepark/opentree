//! Guard runner adapter for `just ci`.

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::Duration;

use anyhow::{Context, Result};
use wait_timeout::ChildExt;

use crate::core::types::{AgentStatus, GuardOutcome};

pub const DEFAULT_GUARD_TIMEOUT: Duration = Duration::from_secs(30 * 60);
pub const DEFAULT_OUTPUT_LIMIT_BYTES: usize = 1_000_000;

#[derive(Debug, Clone)]
pub struct GuardRequest {
    pub workdir: PathBuf,
    pub log_path: PathBuf,
    pub timeout: Duration,
    pub output_limit_bytes: usize,
}

pub trait GuardRunner {
    fn run(&self, request: &GuardRequest) -> Result<GuardOutcome>;
}

pub struct JustGuardRunner;

impl GuardRunner for JustGuardRunner {
    fn run(&self, request: &GuardRequest) -> Result<GuardOutcome> {
        let mut child = Command::new("just")
            .arg("ci")
            .current_dir(&request.workdir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("spawn just ci")?;

        let status = match child
            .wait_timeout(request.timeout)
            .context("wait for guard timeout")?
        {
            Some(status) => status,
            None => {
                child.kill().context("kill guard process")?;
                child.wait().context("wait guard process")?;
                write_guard_log(
                    &request.log_path,
                    b"",
                    b"guard timed out",
                    request.output_limit_bytes,
                )?;
                return Ok(GuardOutcome::Fail);
            }
        };

        let output = child.wait_with_output().context("collect guard output")?;
        write_guard_log(
            &request.log_path,
            &output.stdout,
            &output.stderr,
            request.output_limit_bytes,
        )?;

        if status.success() {
            Ok(GuardOutcome::Pass)
        } else {
            Ok(GuardOutcome::Fail)
        }
    }
}

pub fn run_guards_if_needed<R: GuardRunner>(
    status: AgentStatus,
    runner: &R,
    request: &GuardRequest,
) -> Result<GuardOutcome> {
    if status != AgentStatus::Done {
        return Ok(GuardOutcome::Skipped);
    }
    runner.run(request)
}

fn write_guard_log(
    path: &PathBuf,
    stdout: &[u8],
    stderr: &[u8],
    output_limit: usize,
) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("create guard log dir {}", parent.display()))?;
    }
    let mut buf = String::new();
    buf.push_str("=== stdout ===\n");
    buf.push_str(&String::from_utf8_lossy(stdout));
    buf.push_str("\n=== stderr ===\n");
    buf.push_str(&String::from_utf8_lossy(stderr));

    if buf.len() > output_limit {
        let truncated = format!(
            "{}\n[truncated {} bytes]\n",
            &buf[..output_limit],
            buf.len() - output_limit
        );
        fs::write(path, truncated)
            .with_context(|| format!("write guard log {}", path.display()))?;
        return Ok(());
    }

    fs::write(path, buf).with_context(|| format!("write guard log {}", path.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FakeGuardRunner {
        outcome: GuardOutcome,
    }

    impl GuardRunner for FakeGuardRunner {
        fn run(&self, _request: &GuardRequest) -> Result<GuardOutcome> {
            Ok(self.outcome)
        }
    }

    #[test]
    fn guards_skip_when_not_done() {
        let temp = tempfile::tempdir().expect("tempdir");
        let request = GuardRequest {
            workdir: temp.path().to_path_buf(),
            log_path: temp.path().join("guard.log"),
            timeout: Duration::from_secs(1),
            output_limit_bytes: 100,
        };
        let runner = FakeGuardRunner {
            outcome: GuardOutcome::Pass,
        };

        let outcome =
            run_guards_if_needed(AgentStatus::Retry, &runner, &request).expect("guard outcome");
        assert_eq!(outcome, GuardOutcome::Skipped);
    }

    #[test]
    fn guards_run_when_done() {
        let temp = tempfile::tempdir().expect("tempdir");
        let request = GuardRequest {
            workdir: temp.path().to_path_buf(),
            log_path: temp.path().join("guard.log"),
            timeout: Duration::from_secs(1),
            output_limit_bytes: 100,
        };
        let runner = FakeGuardRunner {
            outcome: GuardOutcome::Fail,
        };

        let outcome =
            run_guards_if_needed(AgentStatus::Done, &runner, &request).expect("guard outcome");
        assert_eq!(outcome, GuardOutcome::Fail);
    }
}
