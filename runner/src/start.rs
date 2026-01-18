//! Orchestration for starting a new run.
//!
//! A "run" is a single execution session identified by a stable `run_id`. Starting
//! a run: creates `runner/<run-id>` branch, stamps `GOAL.md` with the id, and
//! commits the bootstrap. Subsequent `runner step` invocations must be on this
//! branch with matching ids.

use std::path::Path;

use anyhow::{Context, Result, anyhow};
use tracing::{debug, info};

use crate::io::git::Git;
use crate::io::goal::{ensure_goal_id, read_goal_id, validate_id};
use crate::io::init::{InitOptions, init_runner};
use crate::io::run_state::{RunState, load_run_state, write_run_state};

/// Outcome of `runner start`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StartOutcome {
    pub run_id: String,
    pub branch: String,
}

/// Start (or resume) a run in `root`.
///
/// - Ensures `.runner/` scaffolding exists (runs `init` if missing).
/// - Ensures `.runner/GOAL.md` contains a stable `id` in YAML frontmatter.
/// - Creates/checks out `runner/<run-id>` branch (allowed from `main`/`master`).
/// - Commits runner bootstrap changes.
pub fn start_run(root: &Path) -> Result<StartOutcome> {
    debug!(root = %root.display(), "starting run");
    let git = Git::new(root);

    // Refuse to proceed if the repo has non-runner changes, to avoid committing user work
    // as part of loop bootstrap. `.runner/` changes are allowed.
    git.ensure_clean_except_prefixes(&[".runner/"])?;

    let runner_dir = root.join(".runner");
    let goal_path = runner_dir.join("GOAL.md");
    let state_dir = runner_dir.join("state");
    let run_state_path = state_dir.join("run_state.json");

    // If a goal id already exists, prefer it; otherwise generate a new run id.
    let existing_goal_id = if goal_path.exists() {
        read_goal_id(&goal_path)?
    } else {
        None
    };
    let run_id = match existing_goal_id {
        Some(id) => {
            debug!(run_id = %id, "using existing goal id");
            validate_id(&id)?;
            id
        }
        None => {
            let id = generate_run_id(&git)?;
            info!(run_id = %id, "generated new run id");
            id
        }
    };

    let branch = format!("runner/{run_id}");

    // Create/checkout run branch before writing/committing any runner-owned files.
    let current = git.current_branch()?;
    if current != branch {
        if git.branch_exists(&branch)? {
            debug!(branch = %branch, "checking out existing branch");
            git.checkout_branch(&branch)
                .with_context(|| format!("checkout existing branch {branch}"))?;
        } else {
            info!(branch = %branch, "creating new branch");
            git.checkout_new_branch(&branch)
                .with_context(|| format!("create branch {branch}"))?;
        }
    }

    if !runner_dir.exists() {
        init_runner(root, &InitOptions { force: false }).context("runner init")?;
    }

    ensure_runner_gitignore(&runner_dir.join(".gitignore"))?;

    if !goal_path.exists() {
        return Err(anyhow!(
            "missing {} (expected runner init to create it)",
            goal_path.display()
        ));
    }
    ensure_goal_id(&goal_path, &run_id)?;

    let mut run_state = if run_state_path.exists() {
        load_run_state(&run_state_path)?
    } else {
        RunState::default()
    };

    // If the persisted run_id differs, treat this as a new run and reset iteration bookkeeping.
    if run_state.run_id.as_deref() != Some(&run_id) {
        run_state = RunState::default();
        run_state.run_id = Some(run_id.clone());
    }

    write_run_state(&run_state_path, &run_state)?;

    git.add_all()?;
    let _committed = git.commit_staged(&format!("chore(loop): start run {run_id}"))?;

    info!(run_id = %run_id, branch = %branch, "run started");
    Ok(StartOutcome { run_id, branch })
}

fn generate_run_id(git: &Git) -> Result<String> {
    let sha = git.head_short_sha(8)?;
    let base = format!("run-{sha}");

    // Ensure uniqueness against existing local branches `runner/<id>`.
    for suffix in 1..=999u32 {
        let id = if suffix == 1 {
            base.clone()
        } else {
            format!("{base}-{suffix}")
        };
        validate_id(&id)?;
        let branch = format!("runner/{id}");
        if !git.branch_exists(&branch)? {
            return Ok(id);
        }
    }

    Err(anyhow!(
        "unable to generate unique run id from base '{base}' (too many existing branches)"
    ))
}

fn ensure_runner_gitignore(path: &Path) -> Result<()> {
    const REQUIRED_LINES: [&str; 2] = ["context/", "iterations/"];

    let mut existing = String::new();
    if path.exists() {
        existing =
            std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    }

    let mut lines: Vec<String> = existing
        .lines()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect();

    for req in REQUIRED_LINES {
        if !lines.iter().any(|l| l == req) {
            lines.push(req.to_string());
        }
    }

    // Stable ordering.
    lines.sort();
    lines.dedup();

    let mut out = lines.join("\n");
    out.push('\n');

    if out != existing {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("create {}", parent.display()))?;
        }
        std::fs::write(path, out).with_context(|| format!("write {}", path.display()))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::TestRepo;
    use std::process::Command;

    fn capture(root: &Path, args: &[&str]) -> String {
        let out = Command::new(args[0])
            .args(&args[1..])
            .current_dir(root)
            .output()
            .expect("run command");
        assert!(out.status.success(), "command failed: {args:?}");
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    }

    #[test]
    fn start_creates_branch_sets_goal_id_and_commits() {
        let repo = TestRepo::new().expect("repo");
        let root = repo.root();

        let outcome = repo.start_run().expect("start");
        let branch = capture(root, &["git", "rev-parse", "--abbrev-ref", "HEAD"]);
        assert_eq!(branch, outcome.branch);

        let goal = std::fs::read_to_string(root.join(".runner/GOAL.md")).expect("read goal");
        assert!(goal.contains(&format!("id: {}", outcome.run_id)));

        let run_state =
            load_run_state(&root.join(".runner/state/run_state.json")).expect("load run_state");
        assert_eq!(run_state.run_id, Some(outcome.run_id.clone()));

        let last_msg = capture(root, &["git", "log", "-1", "--pretty=%B"]);
        assert!(last_msg.contains(&format!("start run {}", outcome.run_id)));
    }
}
