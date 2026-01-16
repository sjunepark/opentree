# Legacy Go Code Notes (Pre-Pivot)

This directory preserves the prior Go implementation as **reference only**. The repo reboot aims to build a
new deterministic goal-driven runner (task tree + DECOMPOSE/EXECUTE), but the legacy code contains useful
patterns worth reusing conceptually.

Snapshot commit (source repo): `ae73510a682078e189735b26f412ef53fee1503c`

## What the legacy code did

The legacy system is a Ralph-style loop runner for PRDs and boards:

- `ralph run` loops until all stories in `.ralph/prd.json` have `passes=true`.
- `ralph board run` loops until all cards in `.ralph/board.json` are `done`.

It enforces “clean working tree per iteration” and expects each successful iteration to create a commit.

## Key entrypoints / packages

### CLI

- Source: `cmd/ralph/main.go`
  - Commands: `run`, `board run`
  - Flags: `--max-iterations`, `--executor codex|claude`, `--model`, git branch controls

### Loop orchestration

- Source: `internal/app/app.go`
  - Requires running from git repo root containing `.ralph/`.
  - Loads merged config (defaults + global + project + env overrides).
  - Branch strategy: refuses to run on forbidden branches; creates a `ralph/*` branch unless overridden.
  - Iteration loop:
    - requires clean tree at start,
    - runs the executor with a prompt template,
    - requires clean tree at end (expects executor committed),
    - checks completion condition (`AllStoriesPass` / `AllDone`),
    - errors if “exit 0 but no commit” (no progress).
  - Writes per-iteration logs under `.ralph/logs/<run-id>/iter-XXX.log`.

### Executor abstraction (Codex / Claude)

- Source: `internal/executor/executor.go`
  - `Executor.FromConfig` builds an argv for:
    - Codex: `codex exec --sandbox danger-full-access -`
    - Claude: `claude -p --permission-mode acceptEdits --settings <temp.json> [--model ...] <prompt>`
  - Model override support (executor-specific).
  - Captures a practical pattern for “agent session invocation” behind an interface.

### Strict parsing / determinism patterns

- Source: `internal/board/board.go`
  - Uses `json.Decoder.DisallowUnknownFields()` and an explicit version (`version=1`).
  - Validates IDs, required fields, status enum, and “at most one in_progress”.
  - This strict-validation style maps well to the new repo’s “task tree schema is source of truth”.

- Source: `internal/config/config.go`
  - Strict YAML decoding via `KnownFields(true)` and explicit config versioning.
  - Merge layers: defaults → global config → project config → env overrides.

### Completion checks

- Source: `internal/prd/prd.go`
  - Minimal completion predicate: “all stories pass”.

## Preserved Snapshot Files

These are the verbatim copies preserved under `legacy/code/`:

- [`cmd/ralph/main.go`](cmd/ralph/main.go)
- [`internal/app/app.go`](internal/app/app.go)
- [`internal/board/board.go`](internal/board/board.go)
- [`internal/board/board_test.go`](internal/board/board_test.go)
- [`internal/config/config.go`](internal/config/config.go)
- [`internal/config/config_test.go`](internal/config/config_test.go)
- [`internal/executor/executor.go`](internal/executor/executor.go)
- [`internal/git/git.go`](internal/git/git.go)
- [`internal/prd/prd.go`](internal/prd/prd.go)
- [`internal/prompts/prompts.go`](internal/prompts/prompts.go)
- [`internal/prompts/templates/iteration_run.md`](internal/prompts/templates/iteration_run.md)
- [`internal/prompts/templates/iteration_board.md`](internal/prompts/templates/iteration_board.md)
- [`internal/prompts/templates/prd_generate_unattended.md`](internal/prompts/templates/prd_generate_unattended.md)

## How this maps to the new pivot

- The new runner’s **deterministic loop** can borrow:
  - strict decoding + explicit versioning (board/config),
  - per-iteration log structure,
  - hard invariants like “no progress without commit” (if the new commit policy chooses this).
- The new runner’s **executor interface** can reuse the concept of “provider kind → argv template”, but should
  enforce stronger isolation boundaries (workspace confinement, timeouts, output limits) per `VISION.md`.
