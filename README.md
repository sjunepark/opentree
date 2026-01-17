# Goal-Driven Agent Loop Runner

Deterministic local runner for looping **fresh agent sessions** over a strict, goal-oriented **task tree**.

Start here:

- `VISION.md` — principles
- `ARCHITECTURE.md` — canonical technical reference
- `DECISIONS.md` — decision log (dated rationale)

Reference material:

- `docs/loom/` — Loom lessons (state machines, hooks, guard entrypoints, tool boundaries).
- `docs/project/cli.md` — CLI command behavior and exit codes.

Run context:

- `.runner/` — per-project “memory + spec” artifacts in the **target project repo root**.

Guards:

- `just ci` (runs Rust `fmt --check`, `clippy`, `test` + `rumdl check`).

## Testing

- `just test` for Rust unit/integration tests, `just ci` for full checks.
- Harness helpers live in `runner/src/test_support.rs`.
- Fixtures live in `runner/tests/fixtures/` and stay small/deterministic.
- Tests must not spawn `codex` or access the network.

## Quickstart (MVP)

Install (from this repo):

```bash
cargo install --path runner
```

From a target project repo root:

```bash
runner start
runner validate
runner select
runner step
```

Notes:

- `runner start` creates/checks out `runner/<run-id>`, writes `id: <run-id>` into `.runner/GOAL.md`, and commits bootstrap changes.
- `runner step` refuses to run without `runner start` (missing `run_id`).
- `runner init --force` overwrites runner-owned `.runner/` artifacts (including `.runner/GOAL.md`).

## File Contracts

Runner-owned root docs:

- `.runner/GOAL.md` — project-level goal/spec used to seed the root node (YAML frontmatter includes `id`).
- `.runner/.gitignore` — ignores runner-ephemeral dirs (`context/`, `iterations/`).

Runner-owned state (long-lived):

- `.runner/state/tree.json` — canonical task tree (v1) written in stable order.
- `.runner/state/schema.json` — JSON Schema for v1 task trees.
- `.runner/state/config.toml` — runner configuration (guards, defaults, limits).
- `.runner/state/run_state.json` — run/iteration bookkeeping (runner-owned).
- `.runner/state/assumptions.md` — accumulated assumptions (agent may append).
- `.runner/state/questions.md` — open questions for human review (agent may append).
- `.runner/state/agent_output.schema.json` — JSON Schema for agent output.

Ephemeral context (rewritten each iteration):

- `.runner/context/goal.md` — current node goal + acceptance criteria.
- `.runner/context/history.md` — previous attempt summary (retry only).
- `.runner/context/failure.md` — guard output (when guards failed).

Iteration logs (append-only, gitignored):

- `.runner/iterations/{run-id}/{iter-n}/output.json`
- `.runner/iterations/{run-id}/{iter-n}/guard.log`
- `.runner/iterations/{run-id}/{iter-n}/executor.log`
- `.runner/iterations/{run-id}/{iter-n}/meta.json`
- `.runner/iterations/{run-id}/{iter-n}/tree.before.json`
- `.runner/iterations/{run-id}/{iter-n}/tree.after.json`
