//! Git adapter for runner commands.
//!
//! The runner enforces git safety and commits deterministically, so we keep a
//! small, explicit wrapper around `git` subprocess calls.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use anyhow::{Context, Result, anyhow};
use tracing::{debug, instrument, warn};

/// Parsed `git status --porcelain` entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusEntry {
    /// 2-letter XY code, or "??" for untracked.
    pub code: String,
    /// Path for the changed file.
    pub path: String,
}

/// Wrapper for executing git commands in a working directory.
#[derive(Debug, Clone)]
pub struct Git {
    workdir: PathBuf,
}

impl Git {
    pub fn new(workdir: impl Into<PathBuf>) -> Self {
        Self {
            workdir: workdir.into(),
        }
    }

    pub fn workdir(&self) -> &Path {
        &self.workdir
    }

    /// Return the current branch name (errors on detached HEAD).
    #[instrument(skip_all)]
    pub fn current_branch(&self) -> Result<String> {
        let out = self.run_capture(&["rev-parse", "--abbrev-ref", "HEAD"])?;
        let name = out.trim().to_string();
        if name == "HEAD" {
            warn!("detached HEAD detected");
            return Err(anyhow!("detached HEAD (refuse to run)"));
        }
        debug!(branch = %name, "current branch");
        Ok(name)
    }

    /// Return the current HEAD short SHA (stable given repo state).
    pub fn head_short_sha(&self, len: usize) -> Result<String> {
        let arg = format!("--short={len}");
        let out = self.run_capture(&["rev-parse", &arg, "HEAD"])?;
        Ok(out.trim().to_string())
    }

    /// Get status entries (including untracked) in porcelain format.
    pub fn status_porcelain(&self) -> Result<Vec<StatusEntry>> {
        let out = self.run_capture(&["status", "--porcelain=v1", "-uall"])?;
        let mut entries = Vec::new();
        for line in out.lines() {
            if line.trim().is_empty() {
                continue;
            }
            entries.push(parse_status_line(line)?);
        }
        Ok(entries)
    }

    /// Ensure the worktree is clean, allowing entries with any of the given prefixes.
    #[instrument(skip_all)]
    pub fn ensure_clean_except_prefixes(&self, allowed_prefixes: &[&str]) -> Result<()> {
        let entries = self.status_porcelain()?;
        let mut disallowed = Vec::new();
        for entry in entries {
            if allowed_prefixes
                .iter()
                .any(|prefix| entry.path.starts_with(prefix))
            {
                continue;
            }
            disallowed.push(entry);
        }
        if disallowed.is_empty() {
            debug!("worktree is clean");
            return Ok(());
        }
        warn!(disallowed_count = disallowed.len(), "worktree not clean");
        let mut msg = String::new();
        msg.push_str("working tree not clean (disallowed changes):\n");
        for entry in disallowed {
            msg.push_str(&format!("{} {}\n", entry.code, entry.path));
        }
        Err(anyhow!(msg.trim_end().to_string()))
    }

    /// Ensure the worktree is fully clean (including untracked files).
    pub fn ensure_clean(&self) -> Result<()> {
        self.ensure_clean_except_prefixes(&[])
    }

    /// Check whether a local branch exists.
    pub fn branch_exists(&self, branch: &str) -> Result<bool> {
        let status = self
            .run(&[
                "show-ref",
                "--verify",
                "--quiet",
                &format!("refs/heads/{branch}"),
            ])?
            .status;
        Ok(status.success())
    }

    /// Create and checkout a new branch at current HEAD.
    #[instrument(skip_all, fields(branch))]
    pub fn checkout_new_branch(&self, branch: &str) -> Result<()> {
        debug!(branch, "creating and checking out new branch");
        self.run_checked(&["checkout", "-b", branch])?;
        Ok(())
    }

    /// Checkout an existing branch.
    #[instrument(skip_all, fields(branch))]
    pub fn checkout_branch(&self, branch: &str) -> Result<()> {
        debug!(branch, "checking out branch");
        self.run_checked(&["checkout", branch])?;
        Ok(())
    }

    /// Stage all changes (respects .gitignore).
    pub fn add_all(&self) -> Result<()> {
        self.run_checked(&["add", "-A"])?;
        Ok(())
    }

    /// True if there is anything staged for commit.
    pub fn has_staged_changes(&self) -> Result<bool> {
        let out = self.run(&["diff", "--cached", "--name-only"])?;
        Ok(!String::from_utf8_lossy(&out.stdout).trim().is_empty())
    }

    /// Commit staged changes with a message.
    ///
    /// If there are no staged changes, this returns Ok(false) and does nothing.
    #[instrument(skip_all)]
    pub fn commit_staged(&self, message: &str) -> Result<bool> {
        if !self.has_staged_changes()? {
            debug!("no staged changes, skipping commit");
            return Ok(false);
        }
        debug!("committing staged changes");
        self.run_checked(&["commit", "-m", message])?;
        Ok(true)
    }

    fn run_capture(&self, args: &[&str]) -> Result<String> {
        let output = self.run_checked(args)?;
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    fn run_checked(&self, args: &[&str]) -> Result<Output> {
        let output = self.run(args)?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("git {} failed: {}", args.join(" "), stderr.trim()));
        }
        Ok(output)
    }

    fn run(&self, args: &[&str]) -> Result<Output> {
        Command::new("git")
            .args(args)
            .current_dir(&self.workdir)
            .output()
            .with_context(|| format!("spawn git {}", args.join(" ")))
    }
}

fn parse_status_line(line: &str) -> Result<StatusEntry> {
    if let Some(path) = line.strip_prefix("?? ") {
        return Ok(StatusEntry {
            code: "??".to_string(),
            path: path.trim().to_string(),
        });
    }
    if line.len() < 4 {
        return Err(anyhow!("unexpected porcelain line: '{line}'"));
    }
    let code = line[..2].to_string();
    let mut path = line[3..].trim().to_string();
    if let Some((_, new)) = path.split_once("->") {
        path = new.trim().to_string();
    }
    Ok(StatusEntry { code, path })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_untracked_line() {
        let e = parse_status_line("?? foo.txt").expect("parse");
        assert_eq!(
            e,
            StatusEntry {
                code: "??".to_string(),
                path: "foo.txt".to_string()
            }
        );
    }

    #[test]
    fn parses_modified_line() {
        let e = parse_status_line(" M src/main.rs").expect("parse");
        assert_eq!(
            e,
            StatusEntry {
                code: " M".to_string(),
                path: "src/main.rs".to_string()
            }
        );
    }

    #[test]
    fn parses_rename_line_uses_new_path() {
        let e = parse_status_line("R  old.txt -> new.txt").expect("parse");
        assert_eq!(e.path, "new.txt");
    }
}
