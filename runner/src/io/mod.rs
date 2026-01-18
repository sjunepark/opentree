//! I/O layer: all side effects live here.
//!
//! This module isolates filesystem, git, and process operations from pure logic.
//! Benefits:
//! - Core logic remains testable without mocking
//! - Side effects are explicit and auditable
//! - Test doubles can replace real I/O (see `test_support`)

pub mod config;
pub mod context;
pub mod executor;
pub mod git;
pub mod goal;
pub mod guards;
pub mod init;
pub mod iteration_log;
pub mod process;
pub mod prompt;
pub mod run_state;
pub mod tree_store;
