# WIP

## Now

- Completed `.plan/4` (`runner step` one full iteration, context/prompt/executor/guards/run state/logging).
- Implemented `runner start` (auto run-id, `runner/<run-id>` branch checkout, bootstrap commit).
- Enforced git policy + deterministic per-iteration commits in `runner step`.
- Implemented TOML runner config + shared per-iteration timeout/output caps; `runner step` commits even on post-start failures.

## Next

- Land current changes:
  - Split changes into Conventional Commits and commit.
  - Ensure docs match behavior (`config.toml`, budgets, failure-commit semantics).
- Add missing MVP commands:
  - `runner validate`:
    - Validate `.runner/` layout + required files.
    - Load + validate `.runner/state/config.toml` (legacy `.runner/state/config.json` fallback).
    - Validate `.runner/state/tree.json` against schema + semantic invariants (no duplicate ids, attempts bounds, sorted children).
    - Validate run identity when present: `GOAL.md id == run_state.run_id == current branch runner/<id>`.
  - `runner select`:
    - Print deterministic next leaf (id + path + attempts/max_attempts), or exit non-zero if complete.
    - Detect “stuck” leaf (attempts == max_attempts) and surface it clearly.
- Define + implement stuck policy:
  - Decide what runner should do when selected leaf is stuck (hard-stop vs auto-decompose vs require human edit).
  - Encode as deterministic behavior + document in `DECISIONS.md`/`ARCHITECTURE.md`.

## Notes

- Plan CLI needs `pyyaml` when run via `uv run --with pyyaml`.
