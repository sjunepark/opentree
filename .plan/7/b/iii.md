---
status: complete
created_at: 2026-01-18T04:47:16Z
---

# Task iii

## Focus

Capture everything needed for post-hoc analysis in `eval/results/` while keeping `eval/workspaces/`
inspectable.

## Inputs

- `runner` artifacts (`.runner/state/*`, `.runner/iterations/*`)
- `PROPOSAL.md` (results layout)

## Work

1. Define stable results layout:
   - `eval/results/<case-id>/<eval-run-id>/`
2. Write `meta.json` including:
   - case id + case file hash
   - runner git SHA + runner binary path
   - timestamps + duration + runner exit code
   - workspace path
3. Copy artifacts from workspace into results:
   - `.runner/state/tree.json`
   - `.runner/state/run_state.json`
   - `.runner/iterations/<runner-run-id>/` (determine `<runner-run-id>` from state/GOAL)
4. Ensure failures are still captured (partial artifacts + error detail in meta).
5. Add tests for artifact copy semantics in temp dirs (no Codex).

## Output

- Added results capture utilities in `eval/src/results.rs` (layout, meta.json, artifact copy, error collection).
- Implemented case hashing + runner git SHA capture + iteration copy with partial-failure tolerance.
- Added tests for results layout and artifact copying in `eval/src/results.rs`.

## Handoff

Move to judgment system in Sub-plan C.
