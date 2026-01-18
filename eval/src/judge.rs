//! Check execution and outcome recording.
//!
//! Runs verification checks after the runner loop completes and records
//! detailed outcomes including command output.

use std::fs;
use std::io::Read;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Duration;

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use tracing::{debug, instrument, warn};
use wait_timeout::ChildExt;

use crate::case::Check;

/// Limits for command execution in checks.
#[derive(Debug, Clone, Copy)]
pub struct CommandLimits {
    /// Maximum time before killing the command.
    pub timeout: Duration,
    /// Maximum bytes to capture from stdout/stderr.
    pub output_limit_bytes: usize,
}

impl CommandLimits {
    /// Default limits: 60s timeout, 50KB output.
    pub fn default_limits() -> Self {
        Self {
            timeout: Duration::from_secs(60),
            output_limit_bytes: 50_000,
        }
    }
}

/// Collected check outcomes for a run.
#[derive(Debug, Serialize, Deserialize)]
pub struct Judgment {
    pub checks: Vec<CheckOutcome>,
}

/// Result of running a single check.
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CheckOutcome {
    FileExists {
        path: String,
        passed: bool,
    },
    CommandSucceeds {
        cmd: Vec<String>,
        passed: bool,
        exit_code: Option<i32>,
        timed_out: bool,
        stdout: String,
        stderr: String,
        stdout_truncated: bool,
        stderr_truncated: bool,
    },
    RunnerCompleted {
        passed: bool,
        exit_code: Option<i32>,
    },
}

/// Run all checks and collect outcomes.
#[instrument(skip_all, fields(check_count = checks.len()))]
pub fn run_checks(
    checks: &[Check],
    workspace_root: &Path,
    runner_exit_code: Option<i32>,
    limits: CommandLimits,
) -> Result<Judgment> {
    let mut outcomes = Vec::with_capacity(checks.len());
    for check in checks {
        match check {
            Check::FileExists { path } => {
                let full_path = workspace_root.join(path);
                let passed = full_path.exists();
                debug!(check = "file_exists", path = %path.display(), passed, "check result");
                outcomes.push(CheckOutcome::FileExists {
                    path: path.display().to_string(),
                    passed,
                });
            }
            Check::CommandSucceeds { cmd } => {
                let outcome = run_command_check(cmd, workspace_root, limits)?;
                if let CheckOutcome::CommandSucceeds {
                    passed, timed_out, ..
                } = &outcome
                {
                    if *timed_out {
                        warn!(check = "command_succeeds", cmd = ?cmd, "check timed out");
                    } else {
                        debug!(check = "command_succeeds", cmd = ?cmd, passed, "check result");
                    }
                }
                outcomes.push(outcome);
            }
            Check::RunnerCompleted => {
                let passed = runner_exit_code == Some(0);
                debug!(check = "runner_completed", exit_code = ?runner_exit_code, passed, "check result");
                outcomes.push(CheckOutcome::RunnerCompleted {
                    passed,
                    exit_code: runner_exit_code,
                });
            }
        }
    }
    Ok(Judgment { checks: outcomes })
}

pub fn write_judgment(path: &Path, judgment: &Judgment) -> Result<()> {
    let contents = serde_json::to_string_pretty(judgment).context("serialize checks")?;
    fs::write(path, format!("{contents}\n"))
        .with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

fn run_command_check(
    cmd: &[String],
    workspace_root: &Path,
    limits: CommandLimits,
) -> Result<CheckOutcome> {
    if cmd.is_empty() {
        bail!("command_succeeds cmd must be non-empty");
    }

    let mut child = Command::new(&cmd[0])
        .args(&cmd[1..])
        .current_dir(workspace_root)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("spawn command {:?}", cmd))?;

    let mut timed_out = false;
    let status = match child.wait_timeout(limits.timeout)? {
        Some(status) => status,
        None => {
            timed_out = true;
            child.kill().ok();
            child.wait().context("wait after kill")?
        }
    };

    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    if let Some(mut out) = child.stdout.take() {
        out.read_to_end(&mut stdout).context("read stdout")?;
    }
    if let Some(mut err) = child.stderr.take() {
        err.read_to_end(&mut stderr).context("read stderr")?;
    }

    let stdout_truncated = truncate_output(&mut stdout, limits.output_limit_bytes);
    let stderr_truncated = truncate_output(&mut stderr, limits.output_limit_bytes);

    let exit_code = status.code();
    let passed = !timed_out && status.success();

    Ok(CheckOutcome::CommandSucceeds {
        cmd: cmd.to_vec(),
        passed,
        exit_code,
        timed_out,
        stdout: String::from_utf8_lossy(&stdout).to_string(),
        stderr: String::from_utf8_lossy(&stderr).to_string(),
        stdout_truncated,
        stderr_truncated,
    })
}

fn truncate_output(buf: &mut Vec<u8>, limit: usize) -> bool {
    if buf.len() > limit {
        buf.truncate(limit);
        return true;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::tempdir;

    #[test]
    fn file_exists_passes() {
        let temp = tempdir().expect("tempdir");
        let file = temp.path().join("hello.txt");
        fs::write(&file, "hi").expect("write");

        let checks = vec![Check::FileExists {
            path: PathBuf::from("hello.txt"),
        }];
        let result = run_checks(&checks, temp.path(), None, CommandLimits::default_limits())
            .expect("checks");

        match &result.checks[0] {
            CheckOutcome::FileExists { passed, .. } => assert!(*passed),
            _ => panic!("expected file_exists"),
        }
    }

    #[test]
    fn command_succeeds_passes_and_fails() {
        let temp = tempdir().expect("tempdir");
        let limits = CommandLimits {
            timeout: Duration::from_secs(5),
            output_limit_bytes: 1024,
        };

        let checks = vec![Check::CommandSucceeds {
            cmd: vec!["sh".to_string(), "-c".to_string(), "exit 0".to_string()],
        }];
        let result = run_checks(&checks, temp.path(), None, limits).expect("checks");
        match &result.checks[0] {
            CheckOutcome::CommandSucceeds { passed, .. } => assert!(*passed),
            _ => panic!("expected command_succeeds"),
        }

        let checks = vec![Check::CommandSucceeds {
            cmd: vec!["sh".to_string(), "-c".to_string(), "exit 1".to_string()],
        }];
        let result = run_checks(&checks, temp.path(), None, limits).expect("checks");
        match &result.checks[0] {
            CheckOutcome::CommandSucceeds { passed, .. } => assert!(!*passed),
            _ => panic!("expected command_succeeds"),
        }
    }

    #[test]
    fn command_output_is_truncated() {
        let temp = tempdir().expect("tempdir");
        let limits = CommandLimits {
            timeout: Duration::from_secs(5),
            output_limit_bytes: 4,
        };

        let checks = vec![Check::CommandSucceeds {
            cmd: vec![
                "sh".to_string(),
                "-c".to_string(),
                "printf 'abcdef'".to_string(),
            ],
        }];
        let result = run_checks(&checks, temp.path(), None, limits).expect("checks");
        match &result.checks[0] {
            CheckOutcome::CommandSucceeds {
                stdout_truncated,
                stdout,
                ..
            } => {
                assert!(*stdout_truncated);
                assert_eq!(stdout, "abcd");
            }
            _ => panic!("expected command_succeeds"),
        }
    }

    #[test]
    fn runner_completed_reflects_exit_code() {
        let checks = vec![Check::RunnerCompleted];
        let result = run_checks(
            &checks,
            Path::new("."),
            Some(0),
            CommandLimits::default_limits(),
        )
        .expect("checks");
        match &result.checks[0] {
            CheckOutcome::RunnerCompleted { passed, .. } => assert!(*passed),
            _ => panic!("expected runner_completed"),
        }
    }
}
