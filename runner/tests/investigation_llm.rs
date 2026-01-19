//! Investigation tests for external CLI / LLM behavior.
//!
//! These tests verify behavior of external tools (like Codex CLI) and are
//! excluded from regular CI runs because they require external dependencies
//! and API credentials.
//!
//! Run with: `cargo test -p runner --test investigation_llm -- --ignored`

#[path = "investigation/codex.rs"]
mod codex;

#[path = "investigation/tree_agent.rs"]
mod tree_agent;
