//! Result capture and persistence.
//!
//! Captures runner artifacts (tree, iteration logs) and metadata to the
//! results directory for later analysis.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, anyhow};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tracing::{debug, instrument, warn};

use runner::io::goal::read_goal_id;

use crate::outcome::Outcome;

/// Input for capturing results from a completed run.
#[derive(Debug)]
pub struct CaptureInput<'a> {
    pub case_id: &'a str,
    pub case_path: &'a Path,
    pub eval_run_id: &'a str,
    pub runner_binary: &'a Path,
    pub runner_exit_code: Option<i32>,
    pub started_at: DateTime<Utc>,
    pub finished_at: DateTime<Utc>,
    pub workspace_root: &'a Path,
    pub repo_root: &'a Path,
}

/// Metadata for an eval run, persisted to `meta.json`.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EvalMeta {
    pub case_id: String,
    pub eval_run_id: String,
    /// SHA-256 hash of the case file for reproducibility tracking.
    pub case_hash: String,
    /// Git SHA of the runner repo at time of run.
    pub runner_git_sha: Option<String>,
    pub runner_binary: String,
    /// Run ID from the runner (from GOAL.md frontmatter).
    pub runner_run_id: Option<String>,
    pub outcome: Option<Outcome>,
    pub start_time: String,
    pub end_time: String,
    pub duration_secs: f64,
    pub exit_code: Option<i32>,
    pub workspace: String,
    /// Non-fatal errors encountered during capture.
    pub errors: Vec<String>,
}

/// Capture results from a completed run to the results directory.
///
/// Copies tree, run_state, and iteration logs. Writes metadata.
#[instrument(skip_all, fields(case_id = %input.case_id, eval_run_id = %input.eval_run_id))]
pub fn capture_results(base_dir: &Path, input: &CaptureInput<'_>) -> Result<PathBuf> {
    let results_dir = results_dir(base_dir, input.case_id, input.eval_run_id);
    fs::create_dir_all(&results_dir)
        .with_context(|| format!("create results dir {}", results_dir.display()))?;

    let mut errors = Vec::new();

    let case_hash = match file_sha256(input.case_path) {
        Ok(hash) => hash,
        Err(err) => {
            errors.push(format!("case hash: {err}"));
            String::new()
        }
    };

    let runner_git_sha = match git_rev_parse(input.repo_root) {
        Ok(sha) => Some(sha),
        Err(err) => {
            errors.push(format!("runner git sha: {err}"));
            None
        }
    };

    let runner_run_id = match read_goal_id(&input.workspace_root.join(".runner/GOAL.md")) {
        Ok(id) => id,
        Err(err) => {
            errors.push(format!("runner run id: {err}"));
            None
        }
    };

    let runner_state_dir = input.workspace_root.join(".runner/state");
    copy_if_exists(
        &runner_state_dir.join("tree.json"),
        &results_dir.join("tree.json"),
        "tree.json",
        &mut errors,
    );
    copy_if_exists(
        &runner_state_dir.join("run_state.json"),
        &results_dir.join("run_state.json"),
        "run_state.json",
        &mut errors,
    );

    if let Some(run_id) = &runner_run_id {
        let src = input.workspace_root.join(".runner/iterations").join(run_id);
        let dst = results_dir.join("iterations").join(run_id);
        if let Err(err) = copy_dir_recursive(&src, &dst) {
            errors.push(format!("iterations: {err}"));
        }
    } else {
        errors.push("iterations: missing run id".to_string());
    }

    if !errors.is_empty() {
        warn!(errors = ?errors, "artifact capture had errors");
    }

    let duration = input.finished_at - input.started_at;
    let meta = EvalMeta {
        case_id: input.case_id.to_string(),
        eval_run_id: input.eval_run_id.to_string(),
        case_hash,
        runner_git_sha,
        runner_binary: input.runner_binary.display().to_string(),
        runner_run_id,
        outcome: None,
        start_time: input.started_at.to_rfc3339(),
        end_time: input.finished_at.to_rfc3339(),
        duration_secs: duration.num_milliseconds() as f64 / 1000.0,
        exit_code: input.runner_exit_code,
        workspace: input.workspace_root.display().to_string(),
        errors,
    };

    write_meta(&results_dir.join("meta.json"), &meta)?;
    debug!(results_dir = %results_dir.display(), "results captured");
    Ok(results_dir)
}

pub fn update_outcome(results_dir: &Path, outcome: Outcome) -> Result<()> {
    let meta_path = results_dir.join("meta.json");
    let mut meta: EvalMeta = serde_json::from_str(
        &fs::read_to_string(&meta_path).with_context(|| format!("read {}", meta_path.display()))?,
    )
    .context("parse meta")?;
    meta.outcome = Some(outcome);
    write_meta(&meta_path, &meta)?;
    Ok(())
}

pub fn results_dir(base_dir: &Path, case_id: &str, eval_run_id: &str) -> PathBuf {
    base_dir.join(case_id).join(eval_run_id)
}

fn write_meta(path: &Path, meta: &EvalMeta) -> Result<()> {
    let contents = serde_json::to_string_pretty(meta).context("serialize meta")?;
    fs::write(path, format!("{contents}\n"))
        .with_context(|| format!("write meta {}", path.display()))?;
    Ok(())
}

fn file_sha256(path: &Path) -> Result<String> {
    let contents = fs::read(path).with_context(|| format!("read {}", path.display()))?;
    let mut hasher = Sha256::new();
    hasher.update(contents);
    let digest = hasher.finalize();
    Ok(hex::encode(digest))
}

fn git_rev_parse(repo_root: &Path) -> Result<String> {
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(repo_root)
        .output()
        .context("git rev-parse")?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("git rev-parse failed: {}", stderr.trim()));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn copy_if_exists(src: &Path, dst: &Path, label: &str, errors: &mut Vec<String>) {
    if !src.exists() {
        errors.push(format!("{label}: missing {}", src.display()));
        return;
    }
    if let Some(parent) = dst.parent()
        && let Err(err) = fs::create_dir_all(parent)
    {
        errors.push(format!("{label}: create dir failed: {err}"));
        return;
    }
    if let Err(err) = fs::copy(src, dst) {
        errors.push(format!("{label}: copy failed: {err}"));
    }
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    if !src.exists() {
        return Err(anyhow!("missing {}", src.display()));
    }
    fs::create_dir_all(dst).with_context(|| format!("create {}", dst.display()))?;
    for entry in fs::read_dir(src).with_context(|| format!("read {}", src.display()))? {
        let entry = entry.context("read entry")?;
        let path = entry.path();
        let target = dst.join(entry.file_name());
        if path.is_dir() {
            copy_dir_recursive(&path, &target)?;
        } else {
            fs::copy(&path, &target).with_context(|| format!("copy {}", path.display()))?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn results_dir_is_stable() {
        let base = Path::new("/tmp/results");
        let dir = results_dir(base, "case", "run-1");
        assert_eq!(dir, PathBuf::from("/tmp/results/case/run-1"));
    }

    #[test]
    fn copies_artifacts_when_present() {
        let temp = tempdir().expect("tempdir");
        let workspace = temp.path().join("workspace");
        let results = temp.path().join("results");
        fs::create_dir_all(workspace.join(".runner/state")).expect("state dir");
        fs::create_dir_all(workspace.join(".runner/iterations/run-123")).expect("iterations dir");
        fs::write(workspace.join(".runner/state/tree.json"), "{}").expect("tree");
        fs::write(workspace.join(".runner/state/run_state.json"), "{}").expect("run_state");
        fs::write(workspace.join(".runner/GOAL.md"), "---\nid: run-123\n---\n").expect("goal");
        fs::write(
            workspace.join(".runner/iterations/run-123/output.json"),
            "{}",
        )
        .expect("iter");

        let case_path = workspace.join("case.toml");
        let input = CaptureInput {
            case_id: "case",
            case_path: case_path.as_path(),
            eval_run_id: "eval-1",
            runner_binary: Path::new("/bin/runner"),
            runner_exit_code: Some(0),
            started_at: Utc::now(),
            finished_at: Utc::now(),
            workspace_root: &workspace,
            repo_root: temp.path(),
        };
        fs::write(
            &case_path,
            "[case]\nid='case'\ngoal='x'\n[[checks]]\ntype='runner_completed'\n",
        )
        .expect("case");

        let output_dir = capture_results(&results, &input).expect("capture");
        assert!(output_dir.join("tree.json").exists());
        assert!(output_dir.join("run_state.json").exists());
        assert!(output_dir.join("iterations/run-123/output.json").exists());
        assert!(output_dir.join("meta.json").exists());
    }
}
