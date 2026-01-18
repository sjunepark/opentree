---
status: complete
created_at: 2026-01-18T04:47:16Z
---

# Task ii

## Focus

Make workspaces compatible with the runner’s default guard command (`just ci`) without requiring
manual steps.

## Inputs

- `runner/src/io/config.rs` (default `guard.command = ["just","ci"]`)
- `PROPOSAL.md` (evaluation harness requirements)
- `justfile` (this repo’s `ci` semantics; inspiration for workspace `ci`)

## Work

1. Decide the workspace guard strategy (pick one and encode it in the case schema):
   - A) Always generate a minimal workspace `justfile` with a `ci` recipe derived from `checks`
   - B) Require each case to provide explicit `justfile` content
   - C) Override runner `guard.command` per case (and avoid `just ci` entirely)
2. If using A or B, define exact `justfile` output (including any needed shell safety flags).
3. Ensure the harness writes the `justfile` before `runner start` so “done” can ever pass.
4. Decide whether guard should run all `command_succeeds` checks, or only a subset (and why).
5. Add deterministic tests for workspace scaffolding:
   - `justfile` content matches expectation
   - runner config remains stable (or is overridden explicitly by case)

## Output

- Chose strategy A: always generate a minimal workspace `justfile` with `ci` derived from `command_succeeds` checks.
- Implemented deterministic `justfile` rendering + shell escaping in `eval/src/workspace.rs` with unit tests.
- Added config-override merge helper + tests to keep default guard stable unless explicitly overridden (`eval/src/config.rs`).
- Documented guard behavior in `eval/cases/README.md`.

## Handoff

Move to workspace creation (git bootstrap) in Sub-plan B; wire `render_justfile` into the harness before `runner start`.
