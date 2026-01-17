//! Deterministic classification of changed paths.

#![allow(dead_code)]

use crate::core::types::Mode;
use std::path::{Component, Path};

/// Classify changed paths into `DECOMPOSE` vs `EXECUTE`.
///
/// - `EXECUTE` if any changed path is outside `.runner/`.
/// - `DECOMPOSE` if all changed paths are under `.runner/`.
/// - Empty input defaults to `DECOMPOSE`.
pub fn classify_changed_paths<P: AsRef<Path>>(changed_paths: &[P]) -> Mode {
    if changed_paths
        .iter()
        .any(|path| !is_runner_path(path.as_ref()))
    {
        Mode::Execute
    } else {
        Mode::Decompose
    }
}

fn is_runner_path(path: &Path) -> bool {
    let mut components = path
        .components()
        .filter(|component| !matches!(component, Component::CurDir | Component::RootDir));

    match components.next() {
        Some(Component::Normal(name)) => name == ".runner",
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn classify_empty_paths_is_decompose() {
        let paths: Vec<PathBuf> = Vec::new();
        assert_eq!(classify_changed_paths(&paths), Mode::Decompose);
    }

    #[test]
    fn classify_only_runner_paths_is_decompose() {
        let paths = vec![
            PathBuf::from(".runner/GOAL.md"),
            PathBuf::from("./.runner/state/tree.json"),
        ];
        assert_eq!(classify_changed_paths(&paths), Mode::Decompose);
    }

    #[test]
    fn classify_mixed_paths_is_execute() {
        let paths = vec![
            PathBuf::from(".runner/GOAL.md"),
            PathBuf::from("src/lib.rs"),
        ];
        assert_eq!(classify_changed_paths(&paths), Mode::Execute);
    }

    #[test]
    fn classify_runner_subdir_not_prefix_is_execute() {
        let paths = vec![PathBuf::from("src/.runner/tree.json")];
        assert_eq!(classify_changed_paths(&paths), Mode::Execute);
    }

    #[test]
    fn classify_similar_prefix_is_execute() {
        let paths = vec![PathBuf::from(".runnerx/config")];
        assert_eq!(classify_changed_paths(&paths), Mode::Execute);
    }
}
