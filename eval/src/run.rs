use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use chrono::Utc;

use crate::case::CaseFile;
use crate::config::apply_case_config;
use crate::harness::{build_runner_binary, run_runner_loop, run_runner_start};
use crate::judge::{CommandLimits, run_checks, write_judgment};
use crate::outcome::{Outcome, classify_outcome};
use crate::results::{CaptureInput, capture_results, update_outcome};
use crate::workspace::{commit_all, create_workspace, write_goal_file};

#[derive(Debug)]
pub struct RunOutcome {
    pub eval_run_id: String,
    pub results_dir: PathBuf,
    pub outcome: Outcome,
}

pub fn run_case(repo_root: &Path, case_path: &Path, case: &CaseFile) -> Result<RunOutcome> {
    let runner_binary = build_runner_binary(repo_root)?;
    if !runner_binary.exists() {
        bail!("runner binary not found at {}", runner_binary.display());
    }

    let workspace_base = repo_root.join("eval").join("workspaces");
    let workspace = create_workspace(&workspace_base, &case.case.id, &case.checks)
        .context("create workspace")?;

    let started_at = Utc::now();
    let eval_run_id = format!("eval-{}", Utc::now().format("%Y%m%d_%H%M%S"));
    let logs_dir = repo_root
        .join("eval")
        .join("results")
        .join(&case.case.id)
        .join(&eval_run_id);

    run_runner_start(&runner_binary, &workspace.root, &logs_dir, &case.env)
        .context("run runner start")?;

    write_goal_file(&workspace.root, &case.case.goal).context("write goal")?;

    let state_dir = workspace.root.join(".runner/state");
    std::fs::create_dir_all(&state_dir)
        .with_context(|| format!("create {}", state_dir.display()))?;

    let mut cfg = runner::io::config::RunnerConfig::default();
    cfg = apply_case_config(cfg, &case.config)?;
    runner::io::config::write_config(&state_dir.join("config.toml"), &cfg)
        .context("write runner config")?;

    commit_all(&workspace.root, "chore(eval): configure runner")?;

    let loop_status = run_runner_loop(&runner_binary, &workspace.root, &logs_dir, &case.env)
        .context("run runner loop")?;
    let finished_at = Utc::now();

    let exit_code = loop_status.code();

    let capture_input = CaptureInput {
        case_id: &case.case.id,
        case_path,
        eval_run_id: &eval_run_id,
        runner_binary: &runner_binary,
        runner_exit_code: exit_code,
        started_at,
        finished_at,
        workspace_root: &workspace.root,
        repo_root,
    };
    let results_dir = capture_results(&repo_root.join("eval").join("results"), &capture_input)
        .context("capture results")?;

    let limits = CommandLimits::default_limits();
    let judgment =
        run_checks(&case.checks, &workspace.root, exit_code, limits).context("run checks")?;
    write_judgment(&results_dir.join("checks.json"), &judgment).context("write checks")?;

    let outcome = classify_outcome(exit_code, &judgment);
    update_outcome(&results_dir, outcome).context("update outcome")?;

    Ok(RunOutcome {
        eval_run_id,
        results_dir,
        outcome,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eval_run_id_format() {
        let id = format!("eval-{}", Utc::now().format("%Y%m%d_%H%M%S"));
        assert!(id.starts_with("eval-"));
        assert!(id.len() > 10);
    }
}
