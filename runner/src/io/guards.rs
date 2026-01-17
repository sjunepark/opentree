//! Guard runner adapter for `just ci`.

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::Duration;

use anyhow::{Context, Result};
use wait_timeout::ChildExt;

use crate::core::types::{AgentStatus, GuardOutcome};

/// Default timeout for guard execution (30 minutes).
pub const DEFAULT_GUARD_TIMEOUT: Duration = Duration::from_secs(30 * 60);
/// Default output limit to prevent huge logs (1 MB).
pub const DEFAULT_OUTPUT_LIMIT_BYTES: usize = 1_000_000;

/// Parameters for guard execution.
#[derive(Debug, Clone)]
pub struct GuardRequest {
    /// Working directory for the guard process.
    pub workdir: PathBuf,
    /// Path to write guard stdout/stderr log.
    pub log_path: PathBuf,
    /// Maximum time to wait for guards to complete.
    pub timeout: Duration,
    /// Truncate guard output if it exceeds this size.
    pub output_limit_bytes: usize,
}

/// Abstraction over guard execution backends.
pub trait GuardRunner {
    /// Run guards and return the outcome.
    fn run(&self, request: &GuardRequest) -> Result<GuardOutcome>;
}

/// Guard runner that executes `just ci`.
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

/// Run guards only if status is `Done`; otherwise return `Skipped`.
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

    /// Verifies guards are skipped when status is not Done.
    ///
    /// When agent declares Retry, guards shouldn't run — returns Skipped.
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

    /// Verifies guards actually run when status is Done.
    ///
    /// When agent declares Done, guards must run and return their actual outcome.
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
