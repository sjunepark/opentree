//! Run state storage for iteration bookkeeping.

use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::core::types::{AgentStatus, GuardOutcome};

/// Persisted bookkeeping for the current run (`.runner/state/run_state.json`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RunState {
    /// Identifier for the current execution run.
    pub run_id: Option<String>,
    /// Next iteration number (1-indexed, monotonically increasing).
    pub next_iter: u32,
    /// Status from the previous iteration's agent output.
    pub last_status: Option<AgentStatus>,
    /// Summary from the previous iteration's agent output.
    pub last_summary: Option<String>,
    /// Guard outcome from the previous iteration.
    pub last_guard: Option<GuardOutcome>,
}

impl Default for RunState {
    fn default() -> Self {
        Self {
            run_id: None,
            next_iter: 1,
            last_status: None,
            last_summary: None,
            last_guard: None,
        }
    }
}

/// Load run state from disk.
pub fn load_run_state(path: &Path) -> Result<RunState> {
    debug!(path = %path.display(), "loading run state");
    let contents =
        fs::read_to_string(path).with_context(|| format!("read run state {}", path.display()))?;
    let state: RunState = serde_json::from_str(&contents)
        .with_context(|| format!("parse run state {}", path.display()))?;
    debug!(run_id = ?state.run_id, next_iter = state.next_iter, "run state loaded");
    Ok(state)
}

/// Atomically write run state to disk (temp file + rename).
pub fn write_run_state(path: &Path, state: &RunState) -> Result<()> {
    debug!(path = %path.display(), run_id = ?state.run_id, next_iter = state.next_iter, "writing run state");
    let mut buf = serde_json::to_string_pretty(state)?;
    buf.push('\n');
    write_atomic(path, &buf)
}

fn write_atomic(path: &Path, contents: &str) -> Result<()> {
    let parent = path
        .parent()
        .with_context(|| format!("run state path missing parent {}", path.display()))?;
    fs::create_dir_all(parent).with_context(|| format!("create directory {}", parent.display()))?;
    let tmp_path = path.with_extension("json.tmp");
    fs::write(&tmp_path, contents)
        .with_context(|| format!("write temp run state {}", tmp_path.display()))?;
    fs::rename(&tmp_path, path).with_context(|| format!("replace run state {}", path.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verifies write → read preserves all fields.
    ///
    /// Writes a fully populated RunState, reads it back, and asserts equality.
    #[test]
    fn run_state_round_trips() {
        let temp = tempfile::tempdir().expect("tempdir");
        let path = temp.path().join("run_state.json");

        let state = RunState {
            run_id: Some("run-123".to_string()),
            next_iter: 5,
            last_status: Some(AgentStatus::Retry),
            last_summary: Some("summary".to_string()),
            last_guard: Some(GuardOutcome::Skipped),
        };

        write_run_state(&path, &state).expect("write");
        let loaded = load_run_state(&path).expect("load");
        assert_eq!(loaded, state);
    }

    /// Ensures default RunState serializes to a known, stable JSON format.
    ///
    /// Guards against accidental changes to the default values or field ordering.
    #[test]
    fn run_state_defaults_are_deterministic() {
        let temp = tempfile::tempdir().expect("tempdir");
        let path = temp.path().join("run_state.json");

        let state = RunState::default();
        write_run_state(&path, &state).expect("write");
        let contents = fs::read_to_string(&path).expect("read");
        let expected = "{\n  \"run_id\": null,\n  \"next_iter\": 1,\n  \"last_status\": null,\n  \"last_summary\": null,\n  \"last_guard\": null\n}\n";
        assert_eq!(contents, expected);
    }
}
