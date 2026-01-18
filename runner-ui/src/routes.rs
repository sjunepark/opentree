//! HTTP route handlers for the UI API.

use std::fs;

use axum::Router;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::Json;
use axum::routing::get;
use serde::Serialize;
use serde_json::Value;

use crate::state::AppState;

/// Build the API router.
pub fn api_router() -> Router<AppState> {
    Router::new()
        .route("/health", get(health))
        .route("/tree", get(get_tree))
        .route("/run-state", get(get_run_state))
        .route("/iterations", get(list_iterations))
        .route("/iterations/{run_id}/{iter}", get(get_iteration))
        .route("/iterations/{run_id}/{iter}/guard.log", get(get_guard_log))
}

async fn health() -> &'static str {
    "ok"
}

/// GET /api/tree - returns tree.json contents.
async fn get_tree(State(state): State<AppState>) -> Result<Json<Value>, StatusCode> {
    let path = state.tree_path();
    read_json_file(&path)
}

/// GET /api/run-state - returns run_state.json contents.
async fn get_run_state(State(state): State<AppState>) -> Result<Json<Value>, StatusCode> {
    let path = state.run_state_path();
    read_json_file(&path)
}

#[derive(Serialize)]
struct IterationsResponse {
    runs: Vec<RunEntry>,
}

#[derive(Serialize)]
struct RunEntry {
    run_id: String,
    iterations: Vec<u32>,
}

/// GET /api/iterations - list all runs and their iterations.
async fn list_iterations(
    State(state): State<AppState>,
) -> Result<Json<IterationsResponse>, StatusCode> {
    let iter_dir = state.iterations_dir();

    if !iter_dir.exists() {
        return Ok(Json(IterationsResponse { runs: vec![] }));
    }

    let mut runs = Vec::new();

    let entries = fs::read_dir(&iter_dir).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let run_id = match path.file_name().and_then(|n| n.to_str()) {
            Some(name) => name.to_string(),
            None => continue,
        };

        let mut iterations = Vec::new();
        if let Ok(iter_entries) = fs::read_dir(&path) {
            for iter_entry in iter_entries.flatten() {
                let iter_path = iter_entry.path();
                if iter_path.is_dir() {
                    if let Some(iter_name) = iter_path.file_name().and_then(|n| n.to_str()) {
                        if let Ok(iter_num) = iter_name.parse::<u32>() {
                            iterations.push(iter_num);
                        }
                    }
                }
            }
        }

        iterations.sort();
        runs.push(RunEntry { run_id, iterations });
    }

    runs.sort_by(|a, b| a.run_id.cmp(&b.run_id));
    Ok(Json(IterationsResponse { runs }))
}

#[derive(Serialize)]
struct IterationDetail {
    meta: Value,
    output: Value,
}

/// GET /api/iterations/:run/:iter - get iteration details (meta + output merged).
async fn get_iteration(
    State(state): State<AppState>,
    Path((run_id, iter)): Path<(String, u32)>,
) -> Result<Json<IterationDetail>, StatusCode> {
    let iter_dir = state.iterations_dir().join(&run_id).join(iter.to_string());

    let meta_path = iter_dir.join("meta.json");
    let output_path = iter_dir.join("output.json");

    let meta = read_json_value(&meta_path)?;
    let output = read_json_value(&output_path)?;

    Ok(Json(IterationDetail { meta, output }))
}

/// GET /api/iterations/:run/:iter/guard.log - get guard log text.
async fn get_guard_log(
    State(state): State<AppState>,
    Path((run_id, iter)): Path<(String, u32)>,
) -> Result<String, StatusCode> {
    let log_path = state
        .iterations_dir()
        .join(&run_id)
        .join(iter.to_string())
        .join("guard.log");

    if !log_path.exists() {
        return Ok(String::new());
    }

    fs::read_to_string(&log_path).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

fn read_json_file(path: &std::path::Path) -> Result<Json<Value>, StatusCode> {
    let value = read_json_value(path)?;
    Ok(Json(value))
}

fn read_json_value(path: &std::path::Path) -> Result<Value, StatusCode> {
    if !path.exists() {
        return Err(StatusCode::NOT_FOUND);
    }
    let contents = fs::read_to_string(path).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let value: Value =
        serde_json::from_str(&contents).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(value)
}
