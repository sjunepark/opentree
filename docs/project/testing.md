# Testing

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

## Fixtures

Fixtures live under `runner/tests/fixtures/` and are intentionally small:

- Prefer Rust builders (e.g., `test_support::node`) for happy-path logic tests.
- Use JSON/TOML fixtures for format/invariant edge cases and canonicalization checks.
- Update fixtures alongside schema changes, keeping them deterministic.

Helpers in `runner/src/test_support.rs` load and validate fixtures for tests.
