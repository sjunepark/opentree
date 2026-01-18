---
status: complete
created_at: 2026-01-18T04:47:16Z
---

# Task ii

## Focus

Define consistent run outcome classification for reporting and regression detection.

## Inputs

- `docs/project/cli.md` (runner exit codes: 0/1/2/3)
- Sub-plan C/i (`checks.json` schema)

## Work

1. Define the run outcome enum (example):
   - `success` (runner complete + all checks pass)
   - `fail` (runner complete but checks fail)
   - `stuck` (runner exit code 3)
   - `error` (runner exit code 1 or harness error)
   - `skipped` (missing toolchain, if we choose to support this)
2. Encode the outcome in `meta.json` and print a single-line run summary.
3. Add tests for outcome mapping from (runner exit code, check results).

## Output

- Added `Outcome` enum + classification logic in `eval/src/outcome.rs` with tests for exit code mapping.
- Wired outcome storage into `meta.json` via `eval/src/results.rs` (`update_outcome`).

## Handoff

Advance to Sub-plan D (CLI + reporting + docs).
