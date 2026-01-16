# Migration Index (Archived)

This folder preserves the original “migration pack” that seeded this repository.

The repository has since been **unpacked**:

- `VISION.md` remains at the repo root (repo-level vision).
- Per-project run docs (goal + memory) now live under `.runner/` (e.g., `.runner/GOAL.md`).
- Legacy reference material has been moved under `docs/legacy/`, `prompts/legacy/`, `schemas/legacy/`,
  `templates/legacy/`, and `legacy/`.

## What This Is

This directory is a curated “carry-forward” snapshot for the repo pivot described in `VISION.md`:
a **deterministic goal-driven agent loop runner** (fresh context per iteration, strict schemas, sequential
task tree, and automatic validation gates).

## Read Order (Fresh Agent Session)

1. `VISION.md` — principles and end-state.
2. `.runner/GOAL.md` — execution-ready goal spec + MVP success criteria.
3. `docs/loom/lessons.md` — patterns to borrow (state machines, hooks, tool boundaries, guard entrypoint).
4. `.runner/HUMAN_QUESTIONS.md` — open decisions that shape architecture.
5. `.runner/ASSUMPTIONS.md` — assumptions made to keep execution moving.
6. `.runner/FEEDBACK_LOG.md` — mistakes, failure modes, and prevention.
7. `.runner/IMPROVEMENTS.md` — tool/prompt/system improvements to consider.
8. `.runner/GLOSSARY.md` — stable shared terminology.

## Layout (Current)

- Repo root — `VISION.md` (repo-level vision).
- `.runner/` — per-project run docs (`GOAL.md` and memory docs).
- `docs/loom/` — Loom study notes.
- `docs/migration/` — this archived migration pack (how the reboot was bootstrapped).
- `docs/legacy/` — legacy docs snapshot (verbatim reference).
- `prompts/legacy/` — legacy prompt templates snapshot (verbatim reference).
- `schemas/legacy/` — legacy JSON schemas snapshot (verbatim reference).
- `templates/legacy/` — legacy templates snapshot (verbatim reference).
- `legacy/code/` — legacy Go implementation snapshot (reference only).
- `legacy/root/` — legacy root file snapshot (reference only).

## Legacy Entry Points (Current Locations)

- `docs/migration/COPYLIST.md` — authoritative checklist of what was preserved.
- `docs/migration/MAPPING.md` — draft mapping from legacy artifacts to the new repo structure.
- `docs/legacy/INDEX.md` — legacy docs index.
- `prompts/legacy/INDEX.md` — legacy prompt templates index.
- `schemas/legacy/INDEX.md` — legacy schemas index.
- `templates/legacy/INDEX.md` — legacy templates index.
- `legacy/code/NOTES.md` — legacy Go implementation notes.
- `legacy/root/INDEX.md` — legacy root files index.

## Provenance Convention

Snapshot commit (source repo): `ae73510a682078e189735b26f412ef53fee1503c`

- **Copied verbatim:** files are copied into the corresponding `docs/legacy/`, `prompts/legacy/`,
  `schemas/legacy/`, `templates/legacy/`, `legacy/code/`, and `legacy/root/` folders and retain their original
  relative path *within the source root for that category*.
  - Example: `docs/knowledge/ralph-loop-mechanics.md` → `docs/legacy/knowledge/ralph-loop-mechanics.md`
- **Summarized / excerpted:** use a `NOTES.md` (or similarly named) file near the preserved artifacts with
  explicit `Source: <path>` pointers and rationale.

## Next

See [`WIPE_CHECKLIST.md`](WIPE_CHECKLIST.md) for historical “wipe-to-migration-only” instructions.
