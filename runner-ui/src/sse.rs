//! Server-Sent Events stream and file watcher.

use std::convert::Infallible;
use std::time::Duration;

use axum::extract::State;
use axum::response::sse::{Event, Sse};
use futures::stream::Stream;
use notify::{Event as NotifyEvent, EventKind, PollWatcher, RecursiveMode, Watcher};
use serde::Serialize;
use tokio::sync::broadcast;
use tokio::sync::mpsc;
use tracing::{debug, info, warn};

use crate::state::{AppState, ChangeEvent};

#[derive(Serialize)]
struct SsePayload {
    #[serde(rename = "type")]
    event_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    run_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    iter: Option<u32>,
}

impl From<&ChangeEvent> for SsePayload {
    fn from(event: &ChangeEvent) -> Self {
        match event {
            ChangeEvent::TreeChanged => SsePayload {
                event_type: "tree_changed".to_string(),
                run_id: None,
                iter: None,
            },
            ChangeEvent::RunStateChanged => SsePayload {
                event_type: "run_state_changed".to_string(),
                run_id: None,
                iter: None,
            },
            ChangeEvent::IterationAdded { run_id, iter } => SsePayload {
                event_type: "iteration_added".to_string(),
                run_id: Some(run_id.clone()),
                iter: Some(*iter),
            },
            ChangeEvent::IterationCompleted { run_id, iter } => SsePayload {
                event_type: "iteration_completed".to_string(),
                run_id: Some(run_id.clone()),
                iter: Some(*iter),
            },
            ChangeEvent::StreamUpdated { run_id, iter } => SsePayload {
                event_type: "stream_updated".to_string(),
                run_id: Some(run_id.clone()),
                iter: Some(*iter),
            },
            ChangeEvent::ConfigChanged => SsePayload {
                event_type: "config_changed".to_string(),
                run_id: None,
                iter: None,
            },
            ChangeEvent::AssumptionsChanged => SsePayload {
                event_type: "assumptions_changed".to_string(),
                run_id: None,
                iter: None,
            },
            ChangeEvent::QuestionsChanged => SsePayload {
                event_type: "questions_changed".to_string(),
                run_id: None,
                iter: None,
            },
        }
    }
}

/// SSE endpoint handler.
pub async fn events_handler(
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let mut rx = state.event_tx.subscribe();

    let stream = async_stream::stream! {
        // Send initial connected event
        yield Ok(Event::default().event("connected").data("{}"));

        loop {
            match rx.recv().await {
                Ok(change_event) => {
                    let payload = SsePayload::from(&change_event);
                    if let Ok(json) = serde_json::to_string(&payload) {
                        yield Ok(Event::default().event("change").data(json));
                    }
                }
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    warn!(skipped = n, "SSE client lagged, some events dropped");
                }
                Err(broadcast::error::RecvError::Closed) => {
                    break;
                }
            }
        }
    };

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("ping"),
    )
}

/// Start the file watcher in a background task.
pub fn start_file_watcher(state: AppState) {
    tokio::spawn(async move {
        if let Err(e) = run_file_watcher(state).await {
            warn!(error = %e, "file watcher failed");
        }
    });
}

async fn run_file_watcher(state: AppState) -> anyhow::Result<()> {
    let (tx, mut rx) = mpsc::channel::<NotifyEvent>(100);

    let tx_clone = tx.clone();
    let mut watcher = PollWatcher::new(
        move |res: Result<NotifyEvent, notify::Error>| {
            if let Ok(event) = res {
                let _ = tx_clone.try_send(event);
            }
        },
        notify::Config::default().with_poll_interval(Duration::from_millis(100)),
    )?;

    // Watch .runner/state/ and .runner/iterations/
    let state_dir = state.state_dir();
    let iter_dir = state.iterations_dir();

    if state_dir.exists() {
        watcher.watch(&state_dir, RecursiveMode::Recursive)?;
        info!(path = %state_dir.display(), "watching state directory");
    }
    if iter_dir.exists() {
        watcher.watch(&iter_dir, RecursiveMode::Recursive)?;
        info!(path = %iter_dir.display(), "watching iterations directory");
    }

    // Track known iterations to detect new ones
    let mut known_iterations = collect_known_iterations(&iter_dir);

    // Process in batches at a fixed interval to avoid starving updates while a file
    // (e.g. stream.jsonl) is being written continuously.
    let mut pending_events: Vec<NotifyEvent> = Vec::new();
    let mut flush_tick = tokio::time::interval(Duration::from_millis(100));
    flush_tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    loop {
        tokio::select! {
            Some(event) = rx.recv() => {
                pending_events.push(event);
            }
            _ = flush_tick.tick() => {
                if pending_events.is_empty() {
                    continue;
                }
                process_events(&state, &pending_events, &mut known_iterations);
                pending_events.clear();
            }
        }
    }
}

fn process_events(
    state: &AppState,
    events: &[NotifyEvent],
    known_iterations: &mut std::collections::HashSet<(String, u32)>,
) {
    let mut tree_changed = false;
    let mut run_state_changed = false;
    let mut config_changed = false;
    let mut assumptions_changed = false;
    let mut questions_changed = false;
    let mut new_iterations: Vec<(String, u32)> = Vec::new();
    let mut iteration_completions: std::collections::HashSet<(String, u32)> =
        std::collections::HashSet::new();
    let mut stream_updates: std::collections::HashSet<(String, u32)> =
        std::collections::HashSet::new();

    let tree_path = state.tree_path();
    let run_state_path = state.run_state_path();
    let config_path = state.config_path();
    let assumptions_path = state.assumptions_path();
    let questions_path = state.questions_path();
    let iter_dir = state.iterations_dir();

    for event in events {
        // Only care about create/modify events
        if !matches!(event.kind, EventKind::Create(_) | EventKind::Modify(_)) {
            continue;
        }

        for path in &event.paths {
            if path == &tree_path {
                tree_changed = true;
            } else if path == &run_state_path {
                run_state_changed = true;
            } else if path == &config_path {
                config_changed = true;
            } else if path == &assumptions_path {
                assumptions_changed = true;
            } else if path == &questions_path {
                questions_changed = true;
            } else if path.starts_with(&iter_dir) {
                let Some((run_id, iter)) = parse_iteration_path(&iter_dir, path) else {
                    continue;
                };
                let key = (run_id, iter);

                if !known_iterations.contains(&key) {
                    known_iterations.insert(key.clone());
                    new_iterations.push(key.clone());
                }

                match path.file_name().and_then(|n| n.to_str()) {
                    Some("stream.jsonl") => {
                        stream_updates.insert(key);
                    }
                    Some("meta.json") => {
                        iteration_completions.insert(key);
                    }
                    _ => {}
                }
            }
        }
    }

    // Broadcast events
    if tree_changed {
        debug!("broadcasting tree change");
        let _ = state.event_tx.send(ChangeEvent::TreeChanged);
    }
    if run_state_changed {
        debug!("broadcasting run state change");
        let _ = state.event_tx.send(ChangeEvent::RunStateChanged);
    }
    if config_changed {
        debug!("broadcasting config change");
        let _ = state.event_tx.send(ChangeEvent::ConfigChanged);
    }
    if assumptions_changed {
        debug!("broadcasting assumptions change");
        let _ = state.event_tx.send(ChangeEvent::AssumptionsChanged);
    }
    if questions_changed {
        debug!("broadcasting questions change");
        let _ = state.event_tx.send(ChangeEvent::QuestionsChanged);
    }

    new_iterations.sort();
    for (run_id, iter) in new_iterations {
        debug!(run_id = %run_id, iter = iter, "broadcasting new iteration");
        let _ = state
            .event_tx
            .send(ChangeEvent::IterationAdded { run_id, iter });
    }

    let mut completions: Vec<(String, u32)> = iteration_completions.into_iter().collect();
    completions.sort();
    for (run_id, iter) in completions {
        debug!(run_id = %run_id, iter = iter, "broadcasting iteration completed");
        let _ = state
            .event_tx
            .send(ChangeEvent::IterationCompleted { run_id, iter });
    }

    let mut updates: Vec<(String, u32)> = stream_updates.into_iter().collect();
    updates.sort();
    for (run_id, iter) in updates {
        debug!(run_id = %run_id, iter = iter, "broadcasting stream update");
        let _ = state
            .event_tx
            .send(ChangeEvent::StreamUpdated { run_id, iter });
    }
}

fn collect_known_iterations(
    iter_dir: &std::path::Path,
) -> std::collections::HashSet<(String, u32)> {
    let mut known = std::collections::HashSet::new();

    if !iter_dir.exists() {
        return known;
    }

    if let Ok(runs) = std::fs::read_dir(iter_dir) {
        for run_entry in runs.flatten() {
            let run_path = run_entry.path();
            if !run_path.is_dir() {
                continue;
            }
            let run_id = match run_path.file_name().and_then(|n| n.to_str()) {
                Some(name) => name.to_string(),
                None => continue,
            };

            if let Ok(iters) = std::fs::read_dir(&run_path) {
                for iter_entry in iters.flatten() {
                    let iter_path = iter_entry.path();
                    if iter_path.is_dir()
                        && let Some(iter_name) = iter_path.file_name().and_then(|n| n.to_str())
                        && let Ok(iter_num) = iter_name.parse::<u32>()
                    {
                        known.insert((run_id.clone(), iter_num));
                    }
                }
            }
        }
    }

    known
}

fn parse_iteration_path(
    iter_dir: &std::path::Path,
    path: &std::path::Path,
) -> Option<(String, u32)> {
    let rel = path.strip_prefix(iter_dir).ok()?;
    let mut components = rel.components();

    let run_id = components.next()?.as_os_str().to_str()?.to_string();
    let iter_str = components.next()?.as_os_str().to_str()?;
    let iter = iter_str.parse::<u32>().ok()?;

    Some((run_id, iter))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn modify_event(path: std::path::PathBuf) -> NotifyEvent {
        NotifyEvent {
            kind: EventKind::Modify(notify::event::ModifyKind::Any),
            paths: vec![path],
            attrs: Default::default(),
        }
    }

    #[test]
    fn stream_update_emits_iteration_added_and_stream_updated() {
        let project_dir = std::env::temp_dir()
            .join("runner-ui-tests")
            .join(format!("pid-{}", std::process::id()));
        let state = AppState::new(project_dir);
        let mut rx = state.event_tx.subscribe();
        let mut known = std::collections::HashSet::new();

        let path = state
            .iterations_dir()
            .join("run-x")
            .join("1")
            .join("stream.jsonl");

        process_events(&state, &[modify_event(path)], &mut known);

        let mut events = Vec::new();
        while let Ok(ev) = rx.try_recv() {
            events.push(ev);
        }

        assert_eq!(events.len(), 2);
        assert!(
            matches!(&events[0], ChangeEvent::IterationAdded { run_id, iter } if run_id == "run-x" && *iter == 1)
        );
        assert!(
            matches!(&events[1], ChangeEvent::StreamUpdated { run_id, iter } if run_id == "run-x" && *iter == 1)
        );
    }

    #[test]
    fn meta_json_emits_iteration_completed() {
        let project_dir = std::env::temp_dir()
            .join("runner-ui-tests")
            .join(format!("pid-{}", std::process::id()));
        let state = AppState::new(project_dir);
        let mut rx = state.event_tx.subscribe();
        let mut known = std::collections::HashSet::new();
        known.insert(("run-y".to_string(), 2));

        let path = state
            .iterations_dir()
            .join("run-y")
            .join("2")
            .join("meta.json");

        process_events(&state, &[modify_event(path)], &mut known);

        let mut events = Vec::new();
        while let Ok(ev) = rx.try_recv() {
            events.push(ev);
        }

        assert_eq!(events.len(), 1);
        assert!(
            matches!(&events[0], ChangeEvent::IterationCompleted { run_id, iter } if run_id == "run-y" && *iter == 2)
        );
    }
}
