---
status: complete
created_at: 2026-01-18T04:47:16Z
---

# Task i

## Focus

Define the `eval/cases/*.toml` schema and validation rules so cases are reproducible, debuggable, and
safe to run automatically.

## Inputs

- `PROPOSAL.md` (case format + goals)
- `runner/src/io/config.rs` (runner config model + defaults)
- `docs/project/cli.md` (runner exit codes)

## Work

1. Define Rust types for:
   - `case` metadata (id, goal)
   - runner config overrides (max_iterations, max_attempts_default, guard)
   - checks (`file_exists`, `command_succeeds`, `runner_completed`)
2. Decide required/optional fields and defaults; document them in code and in `eval/cases/README.md`
   (or in `eval` CLI help if you want to keep docs minimal).
3. Implement deterministic case discovery (`eval list`) by scanning `eval/cases/*.toml` and sorting by
   `case.id`.
4. Implement schema validation with actionable errors:
   - `case.id` path-safe
   - `checks[*]` well-formed and non-empty
   - `command_succeeds.cmd` non-empty
5. Add unit tests for:
   - parsing valid case
   - rejecting invalid ids
   - rejecting malformed checks

## Output

- Added case schema/types and validation in `eval/src/case.rs` (case/config/checks, list discovery).
- Implemented `eval list` command in `eval/src/main.rs` (sorted by `case.id`).
- Documented case schema in `eval/cases/README.md`.
- Added unit tests for valid case parsing, invalid ids, and malformed checks in `eval/src/case.rs`.

## Handoff

Proceed to guard/`just ci` workspace strategy in Task ii.
