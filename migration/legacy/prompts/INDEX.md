# Legacy Prompts Index

Prompt templates copied from the pre-pivot repo. These are **legacy Ralph/PRD-runner prompts**; treat them
as reference material rather than “drop-in” prompts for the new goal-driven task-tree runner.

## `docs/blueprint/prompts/` → `docs-blueprint/`

Files:

- [`docs-blueprint/iteration_run.md`](docs-blueprint/iteration_run.md) — single-story loop instructions (legacy: `.ralph/prd.json`).
- [`docs-blueprint/iteration_board.md`](docs-blueprint/iteration_board.md) — board-driven loop (legacy: `.ralph/board.json`).
- [`docs-blueprint/prd_generate_unattended.md`](docs-blueprint/prd_generate_unattended.md) — unattended PRD generation prompt.

Why keep:

- Captures practical loop mechanics, quality gates, and “one task per iteration” discipline.
- Useful as a starting point for writing the new runner’s DECOMPOSE/EXECUTE prompts.

## `internal/prompts/templates/` → `internal-templates/`

Files:

- [`internal-templates/iteration_run.md`](internal-templates/iteration_run.md)
- [`internal-templates/iteration_board.md`](internal-templates/iteration_board.md)
- [`internal-templates/prd_generate_unattended.md`](internal-templates/prd_generate_unattended.md)

Why keep:

- These are the in-repo “source of truth” templates used by the legacy Go implementation.
- Useful for understanding how the old system structured prompts and injected constraints.
