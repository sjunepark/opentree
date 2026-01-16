# CLI Spec

## Scope (v0.1)

- The CLI is run from a project’s repo root.
- The project repo must contain `.ralph/`.
- The loop is single-repo, single-process, sequential (no multi-repo orchestration).
- Each iteration uses a *fresh* agent invocation (Codex or Claude).
- Stop condition: all `.ralph/prd.json.userStories[*].passes` are `true`.

## Commands

### `ralph run`

Runs the Ralph story loop using `.ralph/prd.json`.

Preflight:

- Must be inside a git repo.
- Must be at repo root containing `.ralph/`.
- Must NOT be on `main` or `master`.
- Must have a clean working tree (`git status --porcelain` is empty).

Branch selection (prompted once at start):

- Default: create & checkout a new branch starting with `ralph/` (configurable prefix).
- `--use-current-branch` keeps the current branch (still refuses `main`/`master`).
- `--branch <name>` sets an explicit new branch name (must match the prefix).

Iteration contract (mirrors existing Ralph methodology):

- The agent:
  - picks the single highest-priority story with `passes: false`
  - implements only that story
  - runs gates (prefer `./.ralph/gates.sh`)
  - updates `.ralph/prd.json` (marking only that story passing + filling completion fields)
  - appends progress logs
  - commits changes with a Conventional Commit message
- The CLI:
  - repeats until all stories pass or max-iterations is reached
  - writes loop logs under `.ralph/logs/`
  - never pushes

Options (recommended):

- `--max-iterations N`
- `--executor codex|claude` (override config)
- `--model ...` (executor-specific passthrough)
- `--use-current-branch`
- `--branch <name>`

### `ralph board run`

Runs a Kanban-style outer loop over `.ralph/board.json` (strict schema).

- Exactly one card is `in_progress` at a time.
- Cards are never deleted by the loop (`todo|in_progress|done`).
- Card selection and state changes are handled by the agent prompt (see `prompts/iteration_board.md`); the CLI enforces schema validity and loop stop conditions.

Per-card PRDs:

- Canonical per-card PRD: `.ralph/prds/<card-id>/prd.json`
- Active PRD: `.ralph/prd.json` is a copy of the canonical PRD for the active card
- Per-card progress: `.ralph/prds/<card-id>/progress.md`

Unattended default:

- If `.ralph/prds/<card-id>/prd.json` does not exist, the loop generates it unattended from the card title/description and then starts implementing.

(Separate future command: “attended PRD authoring” via interview per card.)

### `ralph init` (optional in v0.1)

Scaffolds `.ralph/` in the current repo using embedded templates (copied from `templates/.ralph/` in this repo).
