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
- `docs/legacy/` — legacy docs snapshot (verbatim reference).
- `prompts/legacy/`, `schemas/legacy/`, `templates/legacy/` — legacy snapshots (verbatim reference).
- `legacy/` — legacy Go implementation + root file snapshot (reference only).

Run context:

- `.runner/` — per-project “memory + spec” artifacts the runner surfaces into each loop.

Guards:

- `just ci` (currently runs Markdown checks via `rumdl`).
