//! Runner binary building and execution.
//!
//! Handles building the runner crate and invoking `runner start`/`runner loop`.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};

use anyhow::{Context, Result, bail};
use tracing::{debug, error, info, instrument};

/// Build the runner binary and return its path.
#[instrument(skip_all)]
pub fn build_runner_binary(repo_root: &Path) -> Result<PathBuf> {
    debug!("cargo build -p runner starting");
    let output = Command::new("cargo")
        .arg("build")
        .arg("-p")
        .arg("runner")
        .current_dir(repo_root)
        .output()
        .context("build runner binary")?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        error!(stderr = %stderr.trim(), "cargo build failed");
        bail!("runner build failed: {}", stderr.trim());
    }
    let path = runner_binary_path(repo_root);
    info!(binary = %path.display(), "runner binary built");
    Ok(path)
}

/// Get the expected path to the runner binary.
pub fn runner_binary_path(repo_root: &Path) -> PathBuf {
    let binary = format!("runner{}", std::env::consts::EXE_SUFFIX);
    repo_root.join("target").join("debug").join(binary)
}

/// Run `runner start` in the workspace.
#[instrument(skip_all, fields(workdir = %workspace_root.display()))]
pub fn run_runner_start(
    runner_path: &Path,
    workspace_root: &Path,
    logs_dir: &Path,
    env_overrides: &BTreeMap<String, String>,
) -> Result<ExitStatus> {
    fs::create_dir_all(logs_dir)
        .with_context(|| format!("create logs dir {}", logs_dir.display()))?;

    let start_log = logs_dir.join("runner.start.log");
    debug!(log_path = %start_log.display(), "spawning runner start");
    let status = run_and_capture(
        runner_path,
        workspace_root,
        &["start"],
        env_overrides,
        &start_log,
    )?;
    debug!(exit_code = ?status.code(), "runner start finished");
    Ok(status)
}

/// Run `runner loop` in the workspace.
#[instrument(skip_all, fields(workdir = %workspace_root.display()))]
pub fn run_runner_loop(
    runner_path: &Path,
    workspace_root: &Path,
    logs_dir: &Path,
    env_overrides: &BTreeMap<String, String>,
) -> Result<ExitStatus> {
    fs::create_dir_all(logs_dir)
        .with_context(|| format!("create logs dir {}", logs_dir.display()))?;

    let loop_log = logs_dir.join("runner.loop.log");
    debug!(log_path = %loop_log.display(), "spawning runner loop");
    let status = run_and_capture(
        runner_path,
        workspace_root,
        &["loop"],
        env_overrides,
        &loop_log,
    )?;
    debug!(exit_code = ?status.code(), "runner loop finished");
    Ok(status)
}

fn run_and_capture(
    runner_path: &Path,
    workspace_root: &Path,
    args: &[&str],
    env_overrides: &BTreeMap<String, String>,
    log_path: &Path,
) -> Result<ExitStatus> {
    let mut command = Command::new(runner_path);
    command.args(args).current_dir(workspace_root);
    for (key, value) in env_overrides {
        command.env(key, value);
    }

    let output = command
        .output()
        .with_context(|| format!("run runner {:?}", args))?;

    let mut combined = output.stdout;
    combined.extend_from_slice(&output.stderr);
    fs::write(log_path, combined).with_context(|| format!("write log {}", log_path.display()))?;

    Ok(output.status)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runner_binary_path_is_deterministic() {
        let root = Path::new("/repo/root");
        let path = runner_binary_path(root);
        let expected = format!(
            "/repo/root/target/debug/runner{}",
            std::env::consts::EXE_SUFFIX
        );
        assert_eq!(path, PathBuf::from(expected));
    }
}
