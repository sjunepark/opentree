//! Shared deterministic types for runner core logic.
//!
//! These types define stable contracts between core components. They should not
//! depend on external state or I/O and must remain deterministic across runs.

#![allow(dead_code)]

/// Iteration classification derived from changed paths.
///
/// The runner must classify deterministically: the same list of paths always
/// yields the same `Mode`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// Only `.runner/` paths changed; agent decomposed work without executing.
    Decompose,
    /// Any non-`.runner/` path changed; agent executed work.
    Execute,
}

/// Result of running guards after an EXECUTE iteration.
///
/// Guards are deterministic: a given command and inputs always yield the same
/// outcome classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuardOutcome {
    /// Guards completed successfully.
    Pass,
    /// Guards completed with failures.
    Fail,
    /// Guards were not run (e.g., DECOMPOSE mode).
    Skipped,
}

/// Summary of runner-owned state updates applied after a step.
///
/// Lists must be recorded in deterministic order (lexicographic node id) to
/// keep serialized outputs stable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateUpdateSummary {
    /// Mode that drove the update rules for this step.
    pub mode: Mode,
    /// Guard outcome associated with this step.
    pub guard_outcome: GuardOutcome,
    /// Node ids that were marked `passes=true` by the runner.
    pub passes_set: Vec<String>,
    /// Node ids whose attempt counters were incremented by the runner.
    pub attempts_incremented: Vec<String>,
    /// Node ids whose `passes` were derived from child completion.
    pub derived_passes_set: Vec<String>,
}
