//! Workspace creation and management.
//!
//! Each eval run gets an isolated git repository with a generated justfile.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};
use chrono::Utc;
use rand::{Rng, distributions::Alphanumeric};

use crate::case::Check;

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
pub fn commit_all(root: &Path, message: &str) -> Result<()> {
    run_git(root, &["add", "."])?;
    run_git(root, &["commit", "-m", message])?;
    Ok(())
}

/// Create an isolated workspace for running a case.
///
/// The workspace is a fresh git repository with:
/// - A generated `justfile` from `command_succeeds` checks
/// - A `README.txt` with case metadata
pub fn create_workspace(base_dir: &Path, case_id: &str, checks: &[Check]) -> Result<Workspace> {
    fs::create_dir_all(base_dir)
        .with_context(|| format!("create workspace dir {}", base_dir.display()))?;

    let timestamp = generate_timestamp();
    let short_id = generate_short_id();
    let name = build_workspace_name(case_id, &timestamp, &short_id);
    let root = base_dir.join(&name);
    fs::create_dir_all(&root)
        .with_context(|| format!("create workspace root {}", root.display()))?;

    run_git(&root, &["init"])?;
    run_git(&root, &["config", "user.name", "Runner Eval"])?;
    run_git(
        &root,
        &["config", "user.email", "runner-eval@local.invalid"],
    )?;

    let justfile = render_justfile(checks);
    fs::write(root.join("justfile"), justfile)
        .with_context(|| format!("write justfile {}", root.display()))?;

    let seed = format!("case_id: {case_id}\ncreated_at: {timestamp}\n");
    fs::write(root.join("README.txt"), seed)
        .with_context(|| format!("write seed {}", root.display()))?;

    run_git(&root, &["add", "."])?;
    run_git(&root, &["commit", "-m", "chore(eval): bootstrap workspace"])?;

    let status = run_git(&root, &["status", "--porcelain"])?;
    if !status.trim().is_empty() {
        bail!("workspace has uncommitted changes after bootstrap");
    }

    Ok(Workspace { root, name })
}

pub fn build_workspace_name(case_id: &str, timestamp: &str, short_id: &str) -> String {
    format!("{case_id}_{timestamp}_{short_id}")
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

pub fn render_justfile(checks: &[Check]) -> String {
    let mut lines = Vec::new();
    lines.push("set shell := [\"bash\", \"-eu\", \"-o\", \"pipefail\", \"-c\"]".to_string());
    lines.push(String::new());
    lines.push("ci:".to_string());

    let mut has_command = false;
    for check in checks {
        if let Check::CommandSucceeds { cmd } = check {
            has_command = true;
            lines.push(format!("  @{}", render_command(cmd)));
        }
    }

    if !has_command {
        lines.push("  @true".to_string());
    }

    lines.push(String::new());
    lines.join("\n")
}

fn render_command(cmd: &[String]) -> String {
    cmd.iter()
        .map(|arg| shell_escape(arg))
        .collect::<Vec<_>>()
        .join(" ")
}

fn shell_escape(input: &str) -> String {
    if input
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.' | '/' | ':'))
    {
        return input.to_string();
    }
    let mut escaped = String::from("'");
    for ch in input.chars() {
        if ch == '\'' {
            escaped.push_str("'\"'\"'");
        } else {
            escaped.push(ch);
        }
    }
    escaped.push('\'');
    escaped
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
    fn renders_justfile_from_commands() {
        let checks = vec![
            Check::FileExists {
                path: "main.go".into(),
            },
            Check::CommandSucceeds {
                cmd: vec!["go".to_string(), "build".to_string(), ".".to_string()],
            },
        ];
        let expected = r#"set shell := ["bash", "-eu", "-o", "pipefail", "-c"]

ci:
  @go build .
"#;
        assert_eq!(render_justfile(&checks), expected);
    }

    #[test]
    fn renders_noop_ci_when_no_commands() {
        let checks = vec![Check::RunnerCompleted];
        let expected = r#"set shell := ["bash", "-eu", "-o", "pipefail", "-c"]

ci:
  @true
"#;
        assert_eq!(render_justfile(&checks), expected);
    }

    #[test]
    fn escapes_shell_arguments() {
        let checks = vec![Check::CommandSucceeds {
            cmd: vec!["echo".to_string(), "hello world".to_string()],
        }];
        let expected = r#"set shell := ["bash", "-eu", "-o", "pipefail", "-c"]

ci:
  @echo 'hello world'
"#;
        assert_eq!(render_justfile(&checks), expected);
    }

    #[test]
    fn workspace_name_uses_expected_format() {
        let name = build_workspace_name("case", "20260118_120000", "abc123");
        assert_eq!(name, "case_20260118_120000_abc123");
    }

    #[test]
    fn creates_workspace_with_clean_git_repo() {
        let temp = tempdir().expect("tempdir");
        let checks = vec![Check::RunnerCompleted];
        let workspace = create_workspace(temp.path(), "case", &checks).expect("workspace");

        assert!(workspace.root.join(".git").exists());
        assert!(workspace.root.join("justfile").exists());

        let status = run_git(&workspace.root, &["status", "--porcelain"]).expect("status");
        assert!(status.trim().is_empty());
    }
}
