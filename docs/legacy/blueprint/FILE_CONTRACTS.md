# File Contracts

## Repo Preconditions

- You run the CLI from the repo root.
- The repo root contains `.ralph/`.

## Required (Run Mode)

- `.ralph/prd.json`
  - JSON-first PRD (source of truth).

## Strongly Recommended

- `.ralph/pin.md`
  - Short persistent anchor for the repo.
- `.ralph/architecture.md`
  - Short, stable architecture notes (templates/gates expect it to exist and to have no placeholders).
- `.ralph/gates.sh`
  - Quality gates runner.

## Logs / State

- `.ralph/logs/`
  - CLI-created logs for runs.
- `.ralph/progress.md` (optional)
  - A global append-only log.

## Board Mode (Kanban)

- `.ralph/board.json`
  - Strict JSON schema (see `schemas/board.schema.json`).
- `.ralph/prds/<card-id>/prd.json`
  - Canonical per-card PRD.
- `.ralph/prds/<card-id>/progress.md`
  - Per-card append-only progress.
- `.ralph/prd.json`
  - Active PRD file used by the story loop.
  - In board mode, it is a copy of the canonical per-card PRD.

## Root Agent Instructions

- Repo root `AGENTS.md` should stay small.
- The agent should only edit a `## Ralph Loop` section and keep it high-signal.
