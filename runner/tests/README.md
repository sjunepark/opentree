# Integration Tests

Integration tests for the runner crate. Each `.rs` file compiles as a separate crate.

## Test Files

| File | Purpose |
|------|---------|
| `cli_select.rs` | CLI exit code behavior for `runner select` |
| `harness_lifecycle.rs` | Multi-iteration lifecycle scenarios via `run_step` |
| `investigation_llm.rs` | External CLI/LLM interaction tests (ignored by default) |
| `investigation_db.rs` | External DB interaction tests (ignored by default) |

## Test Categories

**CLI tests** (`cli_select.rs`): Spawn the runner binary and verify exit codes.
Use when testing user-facing CLI behavior.

**Lifecycle tests** (`harness_lifecycle.rs`): Drive `run_step` through multiple
iterations with scripted executor/guards. Use when testing accumulated state,
component handoffs, or loop termination.

**Investigation tests (LLM)** (`investigation_llm.rs`): Validate external tool behavior (Codex CLI / real LLM runs).
These are `#[ignore]`d and must be run explicitly with `cargo test -p runner --test investigation_llm -- --ignored`
or `just investigate-llm` (alias: `just investigate`).

**Investigation tests (DB)** (`investigation_db.rs`): Validate DB interactions against external services.
These are `#[ignore]`d and must be run explicitly with `cargo test -p runner --test investigation_db -- --ignored`
or `just investigate-db`.

## Fixtures

Test fixtures live in `fixtures/`. Prefer Rust builders (`test_support::node`)
for happy-path tests; use JSON fixtures for format edge cases.

## Guidelines

- Do not spawn `codex` in non-investigation tests
- Do not access the network in non-investigation tests
- Use `TestRepo` for hermetic temp repos with git
- Use `ScriptedExecutor`/`ScriptedGuardRunner` for deterministic outputs

See `docs/project/testing.md` for full testing philosophy.
