# Migration Index (Start Here)

You plan to wipe this repository and keep only the `migration/` directory. This index is the entrypoint
that should remain usable after the wipe.

## What This Is

This directory is a curated “carry-forward” snapshot for the repo pivot described in `migration/VISION.md`:
a **deterministic goal-driven agent loop runner** (fresh context per iteration, strict schemas, sequential
task tree, and automatic validation gates).

## Read Order (Fresh Agent Session)

1. `migration/VISION.md` — principles and end-state.
2. `migration/GOAL.md` — execution-ready goal spec + MVP success criteria.
3. `migration/docs/loom/lessons.md` — patterns to borrow (state machines, hooks, tool boundaries, guard entrypoint).
4. `migration/HUMAN_QUESTIONS.md` — open decisions that shape architecture.
5. `migration/ASSUMPTIONS.md` — assumptions made to keep execution moving.
6. `migration/FEEDBACK_LOG.md` — mistakes, failure modes, and prevention.
7. `migration/IMPROVEMENTS.md` — tool/prompt/system improvements to consider.
8. `migration/GLOSSARY.md` — stable shared terminology.

## Layout

- `migration/` — new-repo seed documents (VISION/GOAL/etc).
- `migration/docs/` — additional research notes (e.g., Loom lessons).
- `migration/legacy/` — curated legacy artifacts copied from the pre-pivot repo:
  - `migration/legacy/docs/` — docs to preserve (from `docs/`).
  - `migration/legacy/prompts/` — prompt templates and related docs.
  - `migration/legacy/schemas/` — prior JSON schemas and contracts.
  - `migration/legacy/templates/` — prior project templates.
  - `migration/legacy/code/` — code notes and any preserved code snapshots.
  - `migration/legacy/root/` — key root-level files (e.g., `AGENTS.md`, `justfile`, `go.mod`).

## Legacy Entry Points

- `migration/COPYLIST.md` — authoritative checklist of what was preserved.
- `migration/MAPPING.md` — draft mapping from legacy artifacts to the new repo structure.
- `migration/legacy/docs/INDEX.md` — legacy `docs/` index.
- `migration/legacy/prompts/INDEX.md` — legacy prompt templates index.
- `migration/legacy/schemas/INDEX.md` — legacy schemas index.
- `migration/legacy/templates/INDEX.md` — legacy templates index.
- `migration/legacy/code/NOTES.md` — legacy Go implementation notes.
- `migration/legacy/root/INDEX.md` — legacy root files index.

## Provenance Convention

Snapshot commit (source repo): `ae73510a682078e189735b26f412ef53fee1503c`

- **Copied verbatim:** files are copied into `migration/legacy/<category>/` and retain their original
  relative path *within the source root for that category*.
  - Example: `docs/knowledge/ralph-loop-mechanics.md` → `migration/legacy/docs/knowledge/ralph-loop-mechanics.md`
- **Summarized / excerpted:** use a `NOTES.md` (or similarly named) file under `migration/legacy/<category>/`
  with explicit `Source: <path>` pointers and rationale.

## Next

Use [`migration/WIPE_CHECKLIST.md`](WIPE_CHECKLIST.md) to wipe the repo safely and continue from only
`migration/`.
