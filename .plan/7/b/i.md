---
status: complete
created_at: 2026-01-18T04:47:16Z
---

# Task i

## Focus

Create an isolated, persistent workspace per run under `eval/workspaces/` that the runner can safely
operate on (git requirements satisfied, deterministic layout).

## Inputs

- `PROPOSAL.md` (workspace layout)
- `runner/src/start.rs` (git cleanliness + bootstrap commit expectations)
- `runner/src/io/git.rs` (runner git policy)

## Work

1. Define workspace naming scheme:
   - directory name: `{case_id}_{timestamp}_{short_id}/`
   - include the chosen `run_id` in `meta.json`
2. Implement workspace creation:
   - create directory
   - `git init`
   - set local `user.name`/`user.email`
   - create an initial commit (so `runner start` can generate ids if needed)
3. Write required workspace scaffolding:
   - case-specific `justfile` (per Sub-plan A decision)
   - any case-provided seed files (optional; keep minimal)
4. Ensure workspace is clean before invoking `runner start`.
5. Add unit/integration tests for workspace creation using temp dirs (no Codex execution).

## Output

- Implemented workspace bootstrap in `eval/src/workspace.rs` (naming scheme, git init/config, seed file, clean check).
- Added deterministic workspace name builder + tests and git-clean workspace creation test in `eval/src/workspace.rs`.
- Workspace `justfile` is written during bootstrap using `render_justfile` (per Sub-plan A guard strategy).

## Handoff

Proceed to building/invoking the runner subprocess in Task ii.
