//! Context writer for `.runner/context/` (ephemeral per-iteration).

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use tracing::debug;

/// Data to write into `.runner/context/` for the current iteration.
#[derive(Debug, Clone)]
pub struct ContextPayload {
    /// Current node's goal and acceptance criteria.
    pub goal: String,
    /// Summary from the previous attempt (set on retry).
    pub history: Option<String>,
    /// Guard output from the previous attempt (set when guards failed).
    pub failure: Option<String>,
}

/// Resolved paths for context files.
#[derive(Debug, Clone)]
pub struct ContextPaths {
    pub dir: PathBuf,
    pub goal_path: PathBuf,
    pub history_path: PathBuf,
    pub failure_path: PathBuf,
}

impl ContextPaths {
    pub fn new(root: &Path) -> Self {
        let dir = root.join(".runner").join("context");
        Self {
            dir: dir.clone(),
            goal_path: dir.join("goal.md"),
            history_path: dir.join("history.md"),
            failure_path: dir.join("failure.md"),
        }
    }
}

/// Clear `.runner/context/` and write fresh ephemeral context files.
pub fn write_context(root: &Path, payload: &ContextPayload) -> Result<ContextPaths> {
    debug!(root = %root.display(), "writing context");
    let paths = ContextPaths::new(root);
    clear_context_dir(&paths.dir)?;

    write_file(&paths.goal_path, &render_goal(&payload.goal))?;
    write_file(
        &paths.history_path,
        &render_optional("History (previous attempt)", payload.history.as_deref()),
    )?;
    write_file(
        &paths.failure_path,
        &render_optional("Failure (guard output)", payload.failure.as_deref()),
    )?;

    debug!(
        has_history = payload.history.is_some(),
        has_failure = payload.failure.is_some(),
        "context written"
    );
    Ok(paths)
}

fn clear_context_dir(dir: &Path) -> Result<()> {
    if dir.exists() {
        debug!(dir = %dir.display(), "clearing context dir");
        fs::remove_dir_all(dir).with_context(|| format!("remove context dir {}", dir.display()))?;
    }
    fs::create_dir_all(dir).with_context(|| format!("create context dir {}", dir.display()))
}

fn write_file(path: &Path, contents: &str) -> Result<()> {
    fs::write(path, contents).with_context(|| format!("write {}", path.display()))
}

fn render_goal(body: &str) -> String {
    format!("# Goal\n\n{}\n", body.trim())
}

fn render_optional(title: &str, body: Option<&str>) -> String {
    let content = body
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("None.");
    format!("# {}\n\n{}\n", title, content)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verifies that write_context clears stale files from the context directory.
    ///
    /// Creates a stale file, writes new context, and confirms the stale file is gone
    /// while fresh context files exist.
    #[test]
    fn context_rewrite_clears_previous_files() {
        let temp = tempfile::tempdir().expect("tempdir");
        let root = temp.path();
        let dir = root.join(".runner").join("context");
        fs::create_dir_all(&dir).expect("create context dir");
        fs::write(dir.join("stale.txt"), "stale").expect("write stale");

        let payload = ContextPayload {
            goal: "Do the thing".to_string(),
            history: None,
            failure: None,
        };

        let paths = write_context(root, &payload).expect("write context");

        assert!(paths.goal_path.is_file());
        assert!(paths.history_path.is_file());
        assert!(paths.failure_path.is_file());
        assert!(!dir.join("stale.txt").exists());
    }

    /// Verifies optional sections render correctly.
    ///
    /// When history is Some, it should appear in the file. When failure is None,
    /// the file should contain "None." placeholder text.
    #[test]
    fn context_renders_optional_sections() {
        let temp = tempfile::tempdir().expect("tempdir");
        let root = temp.path();
        let payload = ContextPayload {
            goal: "Goal body".to_string(),
            history: Some("Did work".to_string()),
            failure: None,
        };

        let paths = write_context(root, &payload).expect("write context");
        let history = fs::read_to_string(&paths.history_path).expect("read history");
        let failure = fs::read_to_string(&paths.failure_path).expect("read failure");

        assert!(history.contains("Did work"));
        assert!(failure.contains("None."));
    }
}
