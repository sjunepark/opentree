//! Iteration logging helpers for `.runner/iterations/`.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Serialize;

use crate::core::types::{AgentOutput, AgentStatus, GuardOutcome};
use crate::tree::Node;

#[derive(Debug, Clone, Serialize)]
pub struct IterationMeta {
    pub run_id: String,
    pub iter: u32,
    pub node_id: String,
    pub status: AgentStatus,
    pub guard: GuardOutcome,
    pub started_at: Option<String>,
    pub ended_at: Option<String>,
    pub duration_ms: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct IterationPaths {
    pub dir: PathBuf,
    pub meta_path: PathBuf,
    pub output_path: PathBuf,
    pub guard_log_path: PathBuf,
    pub tree_before_path: PathBuf,
    pub tree_after_path: PathBuf,
}

impl IterationPaths {
    pub fn new(root: &Path, run_id: &str, iter: u32) -> Self {
        let dir = root
            .join(".runner")
            .join("iterations")
            .join(run_id)
            .join(iter.to_string());
        Self {
            dir: dir.clone(),
            meta_path: dir.join("meta.json"),
            output_path: dir.join("output.json"),
            guard_log_path: dir.join("guard.log"),
            tree_before_path: dir.join("tree.before.json"),
            tree_after_path: dir.join("tree.after.json"),
        }
    }
}

pub struct IterationWriteRequest<'a> {
    pub root: &'a Path,
    pub run_id: &'a str,
    pub iter: u32,
    pub meta: &'a IterationMeta,
    pub output: &'a AgentOutput,
    pub guard_log: Option<&'a str>,
    pub tree_before: &'a Node,
    pub tree_after: &'a Node,
}

pub fn write_iteration(request: &IterationWriteRequest<'_>) -> Result<IterationPaths> {
    let paths = IterationPaths::new(request.root, request.run_id, request.iter);
    fs::create_dir_all(&paths.dir)
        .with_context(|| format!("create iteration dir {}", paths.dir.display()))?;

    // Write in deterministic order to keep logs stable.
    write_json(&paths.meta_path, request.meta)?;
    write_json(&paths.output_path, request.output)?;
    if let Some(log) = request.guard_log {
        write_text(&paths.guard_log_path, log)?;
    }
    write_tree(&paths.tree_before_path, request.tree_before)?;
    write_tree(&paths.tree_after_path, request.tree_after)?;

    Ok(paths)
}

fn write_text(path: &Path, contents: &str) -> Result<()> {
    fs::write(path, contents).with_context(|| format!("write {}", path.display()))
}

fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    let mut buf = serde_json::to_string_pretty(value)?;
    buf.push('\n');
    write_text(path, &buf)
}

fn write_tree(path: &Path, tree: &Node) -> Result<()> {
    let mut cloned = tree.clone();
    cloned.sort_children();
    write_json(path, &cloned)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::AgentStatus;
    use crate::tree::default_tree;

    #[test]
    fn iteration_paths_are_stable() {
        let temp = tempfile::tempdir().expect("tempdir");
        let paths = IterationPaths::new(temp.path(), "run-1", 3);

        assert!(paths.dir.ends_with(Path::new(".runner/iterations/run-1/3")));
        assert!(paths.meta_path.ends_with("meta.json"));
        assert!(paths.output_path.ends_with("output.json"));
        assert!(paths.guard_log_path.ends_with("guard.log"));
        assert!(paths.tree_before_path.ends_with("tree.before.json"));
        assert!(paths.tree_after_path.ends_with("tree.after.json"));
    }

    #[test]
    fn writes_iteration_logs_with_guard_failure() {
        let temp = tempfile::tempdir().expect("tempdir");
        let root = temp.path();
        let tree_before = default_tree();
        let tree_after = default_tree();
        let meta = IterationMeta {
            run_id: "run-9".to_string(),
            iter: 1,
            node_id: "node-1".to_string(),
            status: AgentStatus::Done,
            guard: GuardOutcome::Fail,
            started_at: None,
            ended_at: None,
            duration_ms: None,
        };
        let output = AgentOutput {
            status: AgentStatus::Done,
            summary: "summary".to_string(),
        };

        let paths = write_iteration(&IterationWriteRequest {
            root,
            run_id: "run-9",
            iter: 1,
            meta: &meta,
            output: &output,
            guard_log: Some("guard output"),
            tree_before: &tree_before,
            tree_after: &tree_after,
        })
        .expect("write logs");

        assert!(paths.meta_path.is_file());
        assert!(paths.output_path.is_file());
        assert!(paths.guard_log_path.is_file());
        assert!(paths.tree_before_path.is_file());
        assert!(paths.tree_after_path.is_file());
    }
}
