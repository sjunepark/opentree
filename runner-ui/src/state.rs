//! Shared application state for the UI server.

use std::path::PathBuf;
use std::sync::Arc;

use tokio::sync::broadcast;

/// Events broadcast to SSE clients when files change.
#[derive(Debug, Clone)]
pub enum ChangeEvent {
    TreeChanged,
    RunStateChanged,
    IterationAdded {
        run_id: String,
        iter: u32,
    },
    IterationCompleted {
        run_id: String,
        iter: u32,
    },
    /// JSONL stream file updated (new events written).
    StreamUpdated {
        run_id: String,
        iter: u32,
    },
    ConfigChanged,
    AssumptionsChanged,
    QuestionsChanged,
}

/// Shared state accessible from all request handlers.
#[derive(Clone)]
pub struct AppState {
    /// Root directory of the project (contains .runner/).
    pub project_dir: PathBuf,
    /// Broadcast sender for file change events.
    pub event_tx: Arc<broadcast::Sender<ChangeEvent>>,
}

impl AppState {
    pub fn new(project_dir: PathBuf) -> Self {
        let (event_tx, _) = broadcast::channel(64);
        Self {
            project_dir,
            event_tx: Arc::new(event_tx),
        }
    }

    /// Path to .runner/state/ directory.
    pub fn state_dir(&self) -> PathBuf {
        self.project_dir.join(".runner").join("state")
    }

    /// Path to .runner/iterations/ directory.
    pub fn iterations_dir(&self) -> PathBuf {
        self.project_dir.join(".runner").join("iterations")
    }

    /// Path to tree.json.
    pub fn tree_path(&self) -> PathBuf {
        self.state_dir().join("tree.json")
    }

    /// Path to run_state.json.
    pub fn run_state_path(&self) -> PathBuf {
        self.state_dir().join("run_state.json")
    }

    /// Path to config.toml.
    pub fn config_path(&self) -> PathBuf {
        self.state_dir().join("config.toml")
    }

    /// Path to assumptions.md.
    pub fn assumptions_path(&self) -> PathBuf {
        self.state_dir().join("assumptions.md")
    }

    /// Path to questions.md.
    pub fn questions_path(&self) -> PathBuf {
        self.state_dir().join("questions.md")
    }
}
