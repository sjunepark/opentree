//! Shared deterministic types for runner core logic.
//!
//! These types define stable contracts between core components. They should not
//! depend on external state or I/O and must remain deterministic across runs.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

use crate::tree::NodeNext;

/// Agent-declared status for the selected node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AgentStatus {
    Done,
    Retry,
    Decomposed,
}

/// Minimal child specification produced by the decomposer agent.
///
/// The runner fills in mechanical fields deterministically:
/// - `id`, `order`, `passes`, `attempts`, `max_attempts`, `children`
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TreeChildSpec {
    pub title: String,
    pub goal: String,
    #[serde(default)]
    pub acceptance: Vec<String>,
    pub next: NodeNext,
}

/// Structured output produced by the decomposer agent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecompositionOutput {
    pub summary: String,
    pub children: Vec<TreeChildSpec>,
}

/// Structured output produced by an agent session.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentOutput {
    pub status: AgentStatus,
    pub summary: String,
}

/// Result of running guards after an iteration.
///
/// Guards are deterministic: a given command and inputs always yield the same
/// outcome classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GuardOutcome {
    /// Guards completed successfully.
    Pass,
    /// Guards ran and failed.
    Fail,
    /// Guards were not run (e.g., agent did not declare `status: done`, or the runner errored
    /// before guards could complete).
    Skipped,
}

/// Summary of runner-owned state updates applied after a step.
///
/// Lists must be recorded in deterministic order (lexicographic node id) to
/// keep serialized outputs stable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateUpdateSummary {
    /// Agent-declared status that drove the update rules for this step.
    pub status: AgentStatus,
    /// Guard outcome associated with this step.
    pub guard_outcome: GuardOutcome,
    /// Node ids that were marked `passes=true` by the runner.
    pub passes_set: Vec<String>,
    /// Node ids whose attempt counters were incremented by the runner.
    pub attempts_incremented: Vec<String>,
    /// Node ids whose `passes` were derived from child completion.
    pub derived_passes_set: Vec<String>,
}
