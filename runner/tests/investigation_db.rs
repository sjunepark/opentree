//! Investigation tests for database interactions.
//!
//! These tests are ignored by default because they require external services
//! (a running database) and local credentials/config.
//!
//! Run with:
//!
//! ```bash
//! cargo test -p runner --test investigation_db -- --ignored
//! ```

// Intentionally empty for now.
// Add DB-backed tests under `runner/tests/investigation_db/` and wire them
// via `#[path = "investigation_db/<name>.rs"] mod <name>;`.
