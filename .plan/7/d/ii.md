---
status: complete
created_at: 2026-01-18T04:47:16Z
---

# Task ii

## Focus

Aggregate and display results across runs for the same case.

## Inputs

- `eval/results/<case-id>/<run-id>/meta.json`
- `eval/results/<case-id>/<run-id>/checks.json`

## Work

1. Implement `eval report <case-id>`:
   - read all runs in `eval/results/<case-id>/`
   - compute success rate + averages (duration, iterations if available)
   - compute per-check pass rates
2. Ensure reporting degrades gracefully when runs are partial/corrupt (skip with warning + count).
3. Add tests using fixture result dirs.

## Output

- Implemented aggregation + reporting in `eval/src/report.rs` (success/fail/stuck/error counts, avg duration, per-check pass rates).
- Added warnings for partial/corrupt runs and tests with fixture directories.

## Handoff

Finalize docs, example case, and `just` integration in Task iii.
