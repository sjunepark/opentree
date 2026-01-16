# ralph

A local-only CLI for running Ralph-style development loops inside an existing git repository.

## Install

```bash
go install github.com/sjunepark/ralph/cmd/ralph@latest
```

For local development:

```bash
go install ./cmd/ralph
```

## Usage

Run from a project repo root that contains a `.ralph/` directory:

```bash
ralph run
ralph board run
```

### Safety / Git Rules

- Refuses to run on `main` / `master`.
- Default behavior is to create a new `ralph/*` branch before looping.
  - Use `--use-current-branch` to stay on the current branch (still refuses on `main`/`master`).
  - Use `--branch <name>` to set an explicit new branch name.
- Requires a clean working tree before starting (configurable) and before each iteration.
- Never pushes.

## `.ralph/` Contract

- Required (run): `.ralph/prd.json`
- Required (board): `.ralph/board.json`
- Recommended: `.ralph/gates.sh`, `.ralph/pin.md`, `.ralph/architecture.md`
- Logs: `.ralph/logs/` (recommended to be gitignored)

Starter scaffolding for `.ralph/` is available under `templates/.ralph/`.

## Config

Config files:

- Project: `.ralph/config.yaml`
- Global (macOS default): `~/.config/ralph/config.yaml`

Precedence: `flags > env > project > global > defaults`

Environment overrides:

- `RALPH_EXECUTOR_KIND` (`codex` or `claude`)
- `RALPH_EXECUTOR_COMMAND` (JSON array of argv strings)
- `RALPH_MAX_ITERATIONS` (int)
- `RALPH_GIT_REQUIRE_CLEAN` (`true`/`false`)
- `RALPH_GIT_BRANCH_PREFIX` (string)
- `RALPH_STATE_LOGS_DIR` (string)

## Executors

- `codex` (default): uses `codex exec --sandbox danger-full-access -`
- `claude`: uses `claude -p` with `--permission-mode acceptEdits` and a temporary `--settings` file enabling sandboxing

Override argv in config via `executor.command` if you need custom flags.

## Docs

- `docs/blueprint/`: spec pack for commands/config/contracts/prompts
- `docs/knowledge/`: Ralph methodology notes
- `docs/loom/`: study notes and lessons from `../loom`
