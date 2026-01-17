# Goal-Driven Agent Loop Runner

Deterministic local runner for looping **fresh agent sessions** over a strict, goal-oriented **task tree**.

Start here:

- `VISION.md` — principles
- `ARCHITECTURE.md` — canonical technical reference
- `DECISIONS.md` — decision log (dated rationale)

Reference material:

- `docs/loom/` — Loom lessons (state machines, hooks, guard entrypoints, tool boundaries).

Run context:

- `.runner/` — per-project “memory + spec” artifacts in the **target project repo root**.

Guards:

- `just ci` (runs Rust `fmt --check`, `clippy`, `test` + `rumdl check`).

## Quickstart (MVP)

Install (from this repo):

```bash
cargo install --path runner
```

From a target project repo root:

```bash
runner init
runner validate
runner select
```

Notes:

- `runner init --force` overwrites `.runner/tree.json`, empties `.runner/*.md` placeholders, and rewrites the schema file.
- `runner select` prints the selected leaf node id to stdout.

## File Contracts

- `.runner/tree.json` — canonical task tree (v1) written in stable order.
- `schemas/task_tree/v1.schema.json` — JSON Schema for v1 task trees.
