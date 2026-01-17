//! Validation helpers for `.runner/` layout and run identity.

use std::path::Path;

use anyhow::{Context, Result, anyhow};

use crate::io::config::load_config;
use crate::io::git::Git;
use crate::io::goal::read_goal_id;
use crate::io::init::RunnerPaths;
use crate::io::run_state::load_run_state;
use crate::io::tree_store::load_tree;

/// Run identity validation result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunValidation {
    /// No run id present; run not started yet.
    NotStarted,
    /// Run id exists and matches GOAL.md + branch.
    Ok { run_id: String, branch: String },
}

/// High-level validation outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidateOutcome {
    pub run: RunValidation,
}

/// Validate `.runner/` layout, config, tree, and run identity.
pub fn validate_runner(root: &Path) -> Result<ValidateOutcome> {
    let paths = RunnerPaths::new(root);

    ensure_dir(&paths.runner_dir)?;
    ensure_dir(&paths.state_dir)?;
    ensure_dir(&paths.context_dir)?;
    ensure_dir(&paths.iterations_dir)?;

    ensure_file(&paths.gitignore_path)?;
    ensure_file(&paths.goal_path)?;
    ensure_file(&paths.tree_path)?;
    ensure_file(&paths.schema_path)?;
    ensure_file(&paths.config_path)?;
    ensure_file(&paths.assumptions_path)?;
    ensure_file(&paths.questions_path)?;
    ensure_file(&paths.run_state_path)?;
    ensure_file(&paths.context_goal_path)?;
    ensure_file(&paths.context_history_path)?;
    ensure_file(&paths.context_failure_path)?;

    ensure_gitignore(&paths.gitignore_path)?;

    load_config(&paths.config_path).with_context(|| "load config.toml")?;
    load_tree(&paths.schema_path, &paths.tree_path).with_context(|| "load tree.json")?;

    let run_state = load_run_state(&paths.run_state_path).with_context(|| "load run_state.json")?;
    let run_id = match run_state.run_id {
        Some(run_id) => run_id,
        None => {
            return Ok(ValidateOutcome {
                run: RunValidation::NotStarted,
            });
        }
    };

    let goal_id = read_goal_id(&paths.goal_path)
        .with_context(|| "read GOAL.md")?
        .ok_or_else(|| anyhow!("missing goal id in {}", paths.goal_path.display()))?;
    if goal_id != run_id {
        return Err(anyhow!(
            "run id mismatch: run_state has '{run_id}' but GOAL.md has '{goal_id}'"
        ));
    }

    let git = Git::new(root);
    let branch = git
        .current_branch()
        .with_context(|| "read current branch")?;
    let expected = format!("runner/{run_id}");
    if branch != expected {
        return Err(anyhow!("expected to be on '{expected}' but on '{branch}'"));
    }

    Ok(ValidateOutcome {
        run: RunValidation::Ok { run_id, branch },
    })
}

fn ensure_dir(path: &Path) -> Result<()> {
    if !path.exists() {
        return Err(anyhow!("missing directory {}", path.display()));
    }
    if !path.is_dir() {
        return Err(anyhow!("expected directory {}", path.display()));
    }
    Ok(())
}

fn ensure_file(path: &Path) -> Result<()> {
    if !path.exists() {
        return Err(anyhow!("missing file {}", path.display()));
    }
    if !path.is_file() {
        return Err(anyhow!("expected file {}", path.display()));
    }
    Ok(())
}

fn ensure_gitignore(path: &Path) -> Result<()> {
    let contents =
        std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    for required in ["context/", "iterations/"] {
        if !contents.lines().any(|line| line.trim() == required) {
            return Err(anyhow!("missing '{}' in {}", required, path.display()));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::git::Git;
    use crate::io::goal::ensure_goal_id;
    use crate::io::init::{InitOptions, init_runner};
    use crate::io::run_state::{RunState, write_run_state};
    use crate::test_support::TestRepo;

    #[test]
    fn validate_ok_when_run_not_started() {
        let repo = TestRepo::new().expect("repo");
        init_runner(repo.root(), &InitOptions { force: false }).expect("init");

        let outcome = validate_runner(repo.root()).expect("validate");
        assert_eq!(outcome.run, RunValidation::NotStarted);
    }

    #[test]
    fn validate_ok_when_run_started_and_matches() {
        let repo = TestRepo::new().expect("repo");
        init_runner(repo.root(), &InitOptions { force: false }).expect("init");

        let run_id = "run-abc123";
        let run_state = RunState {
            run_id: Some(run_id.to_string()),
            ..RunState::default()
        };
        let paths = RunnerPaths::new(repo.root());
        write_run_state(&paths.run_state_path, &run_state).expect("write run_state");
        ensure_goal_id(&paths.goal_path, run_id).expect("set goal id");

        let git = Git::new(repo.root());
        git.checkout_new_branch(&format!("runner/{run_id}"))
            .expect("checkout branch");

        let outcome = validate_runner(repo.root()).expect("validate");
        assert_eq!(
            outcome.run,
            RunValidation::Ok {
                run_id: run_id.to_string(),
                branch: format!("runner/{run_id}")
            }
        );
    }

    #[test]
    fn validate_errors_on_goal_id_mismatch() {
        let repo = TestRepo::new().expect("repo");
        init_runner(repo.root(), &InitOptions { force: false }).expect("init");

        let run_id = "run-1";
        let paths = RunnerPaths::new(repo.root());
        write_run_state(
            &paths.run_state_path,
            &RunState {
                run_id: Some(run_id.to_string()),
                ..RunState::default()
            },
        )
        .expect("write run_state");
        ensure_goal_id(&paths.goal_path, "run-2").expect("set goal id");

        let git = Git::new(repo.root());
        git.checkout_new_branch(&format!("runner/{run_id}"))
            .expect("checkout branch");

        let err = validate_runner(repo.root()).expect_err("validate should fail");
        assert!(err.to_string().contains("run id mismatch"));
    }

    #[test]
    fn validate_errors_on_branch_mismatch() {
        let repo = TestRepo::new().expect("repo");
        init_runner(repo.root(), &InitOptions { force: false }).expect("init");

        let run_id = "run-2";
        let paths = RunnerPaths::new(repo.root());
        write_run_state(
            &paths.run_state_path,
            &RunState {
                run_id: Some(run_id.to_string()),
                ..RunState::default()
            },
        )
        .expect("write run_state");
        ensure_goal_id(&paths.goal_path, run_id).expect("set goal id");

        let err = validate_runner(repo.root()).expect_err("validate should fail");
        assert!(
            err.to_string()
                .contains(&format!("expected to be on 'runner/{run_id}'"))
        );
    }

    #[test]
    fn validate_errors_on_missing_layout() {
        let temp = tempfile::tempdir().expect("tempdir");
        let err = validate_runner(temp.path()).expect_err("validate should fail");
        assert!(err.to_string().contains("missing directory"));
    }
}
