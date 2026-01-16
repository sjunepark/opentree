# Goal-Driven Agent Loop Runner

Deterministic local runner for looping **fresh agent sessions** over a strict, goal-oriented **task tree**.

Start here:

- `VISION.md` — principles
- `ARCHITECTURE.md` — canonical technical reference
- `DECISIONS.md` — decision log (dated rationale)
- `.runner/GOAL.md` — project goal/spec
- `.runner/HUMAN_QUESTIONS.md` — open questions

Reference material:

- `docs/loom/` — Loom lessons (state machines, hooks, guard entrypoints, tool boundaries).

Run context:

- `.runner/` — per-project “memory + spec” artifacts the runner surfaces into each loop.

Guards:

- `just ci` (currently runs Markdown checks via `rumdl`).

## Quickstart (MVP)

From the repo root:

```bash
cargo run -p runner -- init
cargo run -p runner -- validate
cargo run -p runner -- select
```

Notes:

- `runner init --force` overwrites `.runner/tree.json` and rewrites the schema file.
- `runner select` prints the selected leaf node id to stdout.

## File Contracts

- `.runner/tree.json` — canonical task tree (v1) written in stable order.
- `schemas/task_tree/v1.schema.json` — JSON Schema for v1 task trees.
