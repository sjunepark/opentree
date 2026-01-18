//! Investigation tests for external CLI behavior.
//!
//! These tests verify behavior of external tools (like Codex CLI) and are
//! excluded from regular CI runs because they require external dependencies
//! and API credentials.
//!
//! Run with: `cargo test --test investigation -- --ignored`

#[path = "investigation/codex_output_schema.rs"]
mod codex_output_schema;
