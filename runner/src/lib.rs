//! Deterministic goal-driven agent loop runner.
//!
//! This crate implements a task-tree execution model where an agent iteratively
//! works through goals until the tree is complete or stuck. The architecture
//! enforces a strict separation:
//!
//! - **[`core`]**: Pure, deterministic logic (selection, validation, state updates).
//!   No I/O, fully testable in isolation.
//! - **[`io`]**: Side-effecting operations (filesystem, git, process execution).
//!   Isolated to enable mocking in tests.
//!
//! Orchestration modules ([`step`], [`start`], [`select`], [`validate`]) coordinate
//! core logic with I/O to implement CLI commands.

pub mod core;
pub mod exit_codes;
pub mod io;
pub mod logging;
pub mod select;
pub mod start;
pub mod step;
#[cfg(any(test, feature = "test-support"))]
pub mod test_support;
pub mod tree;
pub mod validate;
