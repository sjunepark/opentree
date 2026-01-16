# Legacy → New Repo Mapping (Draft)

This mapping is a planning aid for the repo reboot. It does not imply we should keep the legacy structure;
it explains how preserved artifacts might translate into the new “goal-driven agent loop runner”.

## New Repo “Intentional” Top-Level (suggested)

- `GOAL.md` — product/spec anchor (from `migration/GOAL.md`).
- `schemas/` — strict, versioned task-tree schemas (new; see `migration/VISION.md`).
- `runner/` (or `cmd/` + `internal/`) — deterministic selection + orchestration + executor integration (new).
- `prompts/` — DECOMPOSE/EXECUTE prompt templates (new; seed from `migration/legacy/prompts/`).
- `docs/` — minimal, high-signal docs:
  - `docs/vision.md` (or keep `VISION.md` at root),
  - schema contract docs,
  - guard contract docs.
- `justfile` — canonical guard entrypoint (recommended: `just ci`).

## Preserved Legacy Artifacts → Intended Use

### Docs

- `migration/legacy/docs/blueprint/*`
  - Use: reference for file contracts, config patterns, and legacy CLI expectations.
  - Likely output: new `docs/` rewritten to match the new task-tree runner (do not cargo-cult the old spec).

- `migration/legacy/docs/knowledge/*`
  - Use: background on Ralph looping, fresh-context rationale, and prompt discipline.

### Prompts

- `migration/legacy/prompts/*`
  - Use: seed material for new runner prompts.
  - Likely output: `prompts/decompose.md` and `prompts/execute.md` (new), aligned with the new tree schema.

### Schemas

- `migration/legacy/schemas/board.schema.json`
  - Use: reference for strict schema discipline and versioning.
  - Likely output: new `schemas/task-tree.schema.json` (new) + tests enforcing invariants.

### Templates

- `migration/legacy/templates/.ralph/*`
  - Use: inspiration for guard entrypoint and smoke-test checklist structure.
  - Likely output: new `just ci` + optional `smoke-tests/` conventions.

### Code

- `migration/legacy/code/*`
  - Use: reference implementation for:
    - executor abstraction (codex/claude),
    - strict JSON/YAML validation patterns,
    - per-iteration logging and git cleanliness invariants.
  - Likely output: a new architecture (state machine) matching `migration/VISION.md`, not a fork.
