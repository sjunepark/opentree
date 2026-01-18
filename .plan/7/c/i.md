---
status: complete
created_at: 2026-01-18T04:47:16Z
---

# Task i

## Focus

Implement deterministic post-run checks and persist check results for later aggregation.

## Inputs

- `PROPOSAL.md` (check types)
- `docs/project/cli.md` (runner exit codes; stuck vs complete)

## Work

1. Implement check types:
   - `file_exists { path }`
   - `command_succeeds { cmd }` (cwd = workspace, capture stdout/stderr + exit code)
   - `runner_completed` (define precisely; likely runner exit code `0` + tree complete)
2. Define the `Judgment` output schema (`checks.json`) with:
   - pass/fail
   - per-check detail (errors, exit codes, short output excerpts if useful)
3. Make command checks bounded (timeout, output cap) and record truncation explicitly.
4. Add tests covering pass/fail behavior for each check type.

## Output

- Implemented checks + judgment serialization in `eval/src/judge.rs` (file_exists, command_succeeds with timeout/output caps, runner_completed).
- Added `checks.json` writer and structured outcome details (exit code, timeout, truncation flags).
- Added tests covering pass/fail and truncation behavior in `eval/src/judge.rs`.

## Handoff

Proceed to outcome classification in Task ii.
