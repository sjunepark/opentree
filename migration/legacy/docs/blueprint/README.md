# Blueprint: Local-Only Ralph CLI (No Docker)

This folder is the spec pack for the `ralph` CLI in this repository:

- Runs **only inside an existing local git repo** (run from repo root)
- Requires a `.ralph/` directory in that repo
- Runs Ralph loops until `.ralph/prd.json` has all stories `passes: true` (or max iterations)
- Uses **Codex CLI** or **Claude Code** (`claude`) as an executor
- Never allows running on `main`/`master`
- Requires a clean working tree before starting
- Auto-commits each successful iteration (no pushes)

Docs:

- `CLI_SPEC.md`: commands and behavior
- `FILE_CONTRACTS.md`: `.ralph/` contract (including Kanban board mode)
- `CONFIG.md`: global + project config model (strict)
- `prompts/`: prompt templates to feed into the executor
- `schemas/board.schema.json`: strict JSON schema for `.ralph/board.json`
- `examples/`: example `config.yaml`, `board.json`, and PRD for building the tool

Next: see `NEW_REPO_CHECKLIST.md` for a contributor/bootstrap checklist.
