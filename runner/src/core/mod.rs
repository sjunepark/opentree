//! Deterministic, pure logic shared by the runner core.
//!
//! Core modules must be free of I/O side effects. They operate on in-memory
//! data structures and return deterministic outputs suitable for tests.

pub mod classifier;
pub mod immutability;
pub mod invariants;
pub mod selector;
pub mod state_update;
pub mod status_validator;
pub mod types;
