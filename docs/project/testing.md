# Testing

## Test Design Philosophy

### Unit vs Lifecycle Tests

**Unit tests** verify single-function behavior in isolation. Use when testing:

- Selection algorithms (e.g., `leftmost_open_leaf`, `is_stuck`)
- Single-step logic with clear inputs and outputs
- Error handling and edge cases

**Lifecycle tests** verify emergent behavior across multiple iterations. Use when:

- Testing requires multiple `run_step` iterations
- Verifying component handoffs (executor → guards → state updates)
- Testing accumulated state (attempts incrementing, passes propagating)

### Deduplication Principle

Prefer a single test with multiple assertions over separate tests for related behaviors.
This reduces test setup overhead and documents related invariants together.

```rust
// Prefer: single test covering related cases
#[test]
fn leftmost_open_leaf_traversal_cases() {
    // All traversal edge cases in one function with clear assertions
}

// Avoid: separate tests duplicating setup
#[test] fn leftmost_open_leaf_single_unpassed() { ... }
#[test] fn leftmost_open_leaf_all_passed() { ... }
```

## Layers

- Pure logic tests: selector, invariants, state updates, immutability checks.
- Loop-level harness tests: run `run_step` against a temp git repo with scripted executor/guards.
- (Future) CLI smoke tests: only when a non-codex executor backend exists.

## Harness usage

Use `crate::test_support::TestRepo` to create a hermetic temp repo with an initial commit, then
call `start_run()` to scaffold `.runner/`. For multi-iteration scenarios, pair it with
`ScriptedExecutor` and `ScriptedGuardRunner` to queue deterministic outputs and guard outcomes.

Rules:

- Do not spawn `codex` in tests.
- Do not access the network.
- Keep iteration logs/assertions anchored under `.runner/iterations/{run-id}/{iter}/`.

## Investigation tests (external deps)

Some checks require external dependencies (LLM tools, databases) and are intentionally excluded from CI.

### LLM / CLI

- Location: `runner/tests/investigation/` (wired via `runner/tests/investigation_llm.rs`)
- Marked with `#[ignore]` so they only run when explicitly requested.
- Run with: `cargo test -p runner --test investigation_llm -- --ignored` or `just investigate-llm`
  (alias: `just investigate`).

These tests may access the network and require local credentials.

### DB

- Location: `runner/tests/investigation_db/` (wired via `runner/tests/investigation_db.rs`)
- Marked with `#[ignore]` so they only run when explicitly requested.
- Run with: `cargo test -p runner --test investigation_db -- --ignored` or `just investigate-db`.

## Fixtures

Fixtures live under `runner/tests/fixtures/` and are intentionally small:

- Prefer Rust builders (e.g., `test_support::node`) for happy-path logic tests.
- Use JSON/TOML fixtures for format/invariant edge cases and canonicalization checks.
- Update fixtures alongside schema changes, keeping them deterministic.

Helpers in `runner/src/test_support.rs` load and validate fixtures for tests.
