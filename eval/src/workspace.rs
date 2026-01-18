//! Workspace creation and management.
//!
//! Each eval run gets an isolated git repository. Cases can optionally
//! provide a justfile for the `just ci` guard.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};
use chrono::Utc;
use rand::{Rng, distributions::Alphanumeric};
use tracing::{debug, info, instrument};

/// An isolated workspace for running a case.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Workspace {
    /// Absolute path to the workspace root.
    pub root: PathBuf,
    /// Workspace directory name (includes case id, timestamp, and random suffix).
    pub name: String,
}

/// Write the case goal to `.runner/GOAL.md`, preserving the run id.
pub fn write_goal_file(root: &Path, goal: &str) -> Result<String> {
    let goal_path = root.join(".runner").join("GOAL.md");
    let run_id = runner::io::goal::read_goal_id(&goal_path)?
        .ok_or_else(|| anyhow::anyhow!("missing run id in {}", goal_path.display()))?;
    let contents = format!("---\nid: {}\n---\n\n{}\n", run_id, goal.trim());
    fs::write(&goal_path, contents).with_context(|| format!("write {}", goal_path.display()))?;
    Ok(run_id)
}

/// Stage all files and commit with the given message.
/// Does nothing if there are no changes to commit.
pub fn commit_all(root: &Path, message: &str) -> Result<()> {
    run_git(root, &["add", "."])?;
    let status = run_git(root, &["status", "--porcelain"])?;
    if status.trim().is_empty() {
        debug!("no changes to commit, skipping");
        return Ok(());
    }
    run_git(root, &["commit", "-m", message])?;
    Ok(())
}

/// Create an isolated workspace for running a case.
///
/// The workspace is a fresh git repository with:
/// - An optional `justfile` if provided
/// - A `README.txt` with case metadata
#[instrument(skip_all, fields(case_id = %case_id))]
pub fn create_workspace(
    base_dir: &Path,
    case_id: &str,
    justfile: Option<&str>,
) -> Result<Workspace> {
    fs::create_dir_all(base_dir)
        .with_context(|| format!("create workspace dir {}", base_dir.display()))?;

    let timestamp = generate_timestamp();
    let short_id = generate_short_id();
    let name = build_workspace_name(case_id, &timestamp, &short_id);
    let root = base_dir.join(&name);
    fs::create_dir_all(&root)
        .with_context(|| format!("create workspace root {}", root.display()))?;

    debug!(path = %root.display(), "initializing git repo");
    run_git(&root, &["init"])?;
    run_git(&root, &["config", "user.name", "Runner Eval"])?;
    run_git(
        &root,
        &["config", "user.email", "runner-eval@local.invalid"],
    )?;

    if let Some(content) = justfile {
        fs::write(root.join("justfile"), content)
            .with_context(|| format!("write justfile {}", root.display()))?;
    }

    let seed = format!("case_id: {case_id}\ncreated_at: {timestamp}\n");
    fs::write(root.join("README.txt"), seed)
        .with_context(|| format!("write seed {}", root.display()))?;

    run_git(&root, &["add", "."])?;
    run_git(&root, &["commit", "-m", "chore(eval): bootstrap workspace"])?;

    let status = run_git(&root, &["status", "--porcelain"])?;
    if !status.trim().is_empty() {
        bail!("workspace has uncommitted changes after bootstrap");
    }

    info!(path = %root.display(), "workspace created");

    // Create/update _latest symlink for easy access
    update_latest_symlink(base_dir, case_id, &name)?;

    Ok(Workspace { root, name })
}

pub fn build_workspace_name(case_id: &str, timestamp: &str, short_id: &str) -> String {
    format!("{case_id}_{timestamp}_{short_id}")
}

/// Create or update a `<case_id>_latest` symlink pointing to the workspace.
fn update_latest_symlink(base_dir: &Path, case_id: &str, workspace_name: &str) -> Result<()> {
    let symlink_path = base_dir.join(format!("{case_id}_latest"));

    // Remove existing symlink if present
    if symlink_path.exists() || symlink_path.is_symlink() {
        fs::remove_file(&symlink_path)
            .with_context(|| format!("remove old symlink {}", symlink_path.display()))?;
    }

    #[cfg(unix)]
    std::os::unix::fs::symlink(workspace_name, &symlink_path)
        .with_context(|| format!("create symlink {}", symlink_path.display()))?;

    #[cfg(windows)]
    std::os::windows::fs::symlink_dir(workspace_name, &symlink_path)
        .with_context(|| format!("create symlink {}", symlink_path.display()))?;

    debug!(symlink = %symlink_path.display(), target = %workspace_name, "updated latest symlink");
    Ok(())
}

/// Resolve the latest workspace for a case via the `<case_id>_latest` symlink.
///
/// Returns an error if the symlink doesn't exist or points to an invalid workspace.
#[instrument(skip_all, fields(case_id = %case_id))]
pub fn resolve_latest_workspace(base_dir: &Path, case_id: &str) -> Result<Workspace> {
    let symlink_path = base_dir.join(format!("{case_id}_latest"));

    if !symlink_path.is_symlink() {
        bail!(
            "no latest workspace for case '{}': symlink {} does not exist",
            case_id,
            symlink_path.display()
        );
    }

    let target = fs::read_link(&symlink_path)
        .with_context(|| format!("read symlink {}", symlink_path.display()))?;

    // The symlink is relative to base_dir
    let root = base_dir.join(&target);
    let name = target
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("workspace name is not valid UTF-8"))?
        .to_string();

    if !root.exists() {
        bail!(
            "latest workspace for case '{}' is missing: {} does not exist",
            case_id,
            root.display()
        );
    }

    if !root.join(".git").exists() {
        bail!(
            "latest workspace for case '{}' is not a git repository: {} has no .git/",
            case_id,
            root.display()
        );
    }

    // Check that workspace is not empty (should have at least README.txt)
    let entries: Vec<_> = fs::read_dir(&root)
        .with_context(|| format!("read {}", root.display()))?
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name() != ".git")
        .collect();

    if entries.is_empty() {
        bail!(
            "latest workspace for case '{}' is empty (no files besides .git): {}",
            case_id,
            root.display()
        );
    }

    info!(path = %root.display(), "resolved latest workspace");
    Ok(Workspace { root, name })
}

fn generate_timestamp() -> String {
    Utc::now().format("%Y%m%d_%H%M%S").to_string()
}

fn generate_short_id() -> String {
    let mut rng = rand::thread_rng();
    std::iter::repeat_with(|| rng.sample(Alphanumeric))
        .map(char::from)
        .take(6)
        .collect::<String>()
        .to_lowercase()
}

fn run_git(root: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .with_context(|| format!("run git {:?}", args))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("git {:?} failed: {}", args, stderr.trim());
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn workspace_name_uses_expected_format() {
        let name = build_workspace_name("case", "20260118_120000", "abc123");
        assert_eq!(name, "case_20260118_120000_abc123");
    }

    #[test]
    fn creates_workspace_with_justfile() {
        let temp = tempdir().expect("tempdir");
        let justfile = "ci:\n  @echo hello\n";
        let workspace = create_workspace(temp.path(), "case", Some(justfile)).expect("workspace");

        assert!(workspace.root.join(".git").exists());
        assert!(workspace.root.join("justfile").exists());
        assert_eq!(
            std::fs::read_to_string(workspace.root.join("justfile")).unwrap(),
            justfile
        );

        let status = run_git(&workspace.root, &["status", "--porcelain"]).expect("status");
        assert!(status.trim().is_empty());
    }

    #[test]
    fn creates_workspace_without_justfile() {
        let temp = tempdir().expect("tempdir");
        let workspace = create_workspace(temp.path(), "case", None).expect("workspace");

        assert!(workspace.root.join(".git").exists());
        assert!(!workspace.root.join("justfile").exists());

        let status = run_git(&workspace.root, &["status", "--porcelain"]).expect("status");
        assert!(status.trim().is_empty());
    }
}
