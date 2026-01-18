---
status: complete
created_at: 2026-01-18T04:47:16Z
---

# Task iii

## Focus

Add minimal docs and example cases to make `eval` usable without tribal knowledge.

## Inputs

- `PROPOSAL.md` (files to create)
- `README.md` (project entrypoints)
- `justfile` (workflow)

## Work

1. Add `eval/cases/calculator-go.toml` (or a smaller starter case if preferred).
2. Add `eval/.gitignore` to ignore `workspaces/` and `results/` while keeping cases committed.
3. Update root `Cargo.toml` workspace members to include `eval`.
4. Add `just eval-*` recipes (ex: `just eval-list`, `just eval-run CASE`, `just eval-report CASE`).
5. Update `README.md` with a short “Evaluation” section and example commands.

## Output

- Added example case `eval/cases/calculator-go.toml`.
- Added `eval/.gitignore` for `workspaces/` and `results/`.
- Added `just` recipes (`eval-list`, `eval-run`, `eval-report`, `eval-clean`) in `justfile`.
- Updated `README.md` with an Evaluation section and example commands.

## Handoff

Run formatting/lint/tests and finalize plan.
