# WIP

## Now

- Completed `.plan/4` (`runner step` one full iteration, context/prompt/executor/guards/run state/logging).
- Implemented `runner start` (auto run-id, `runner/<run-id>` branch checkout, bootstrap commit).
- Enforced git policy + deterministic per-iteration commits in `runner step`.
- Implemented TOML runner config + shared per-iteration timeout/output caps; `runner step` commits even on post-start failures.
- Clarified attempt semantics: runner-internal failures do not increment `attempts` (only successful agent outputs do).

## Next

- Land current changes:
  - Split changes into Conventional Commits and commit.
  - Ensure docs match behavior (`config.toml`, budgets, failure-commit semantics).
- Land Plan 5 testing harness + fixtures + docs (commit + CI).
- ~~Define + document stuck-node policy~~ ✓ hard-stop decided, documented in `DECISIONS.md` + `ARCHITECTURE.md`.
- Add missing MVP commands:
  - `runner validate`:
    - Validate `.runner/` layout + required files.
    - Load + validate `.runner/state/config.toml`.
    - Validate `.runner/state/tree.json` against schema + semantic invariants (no duplicate ids, attempts bounds, sorted children).
    - Validate run identity when present: `GOAL.md id == run_state.run_id == current branch runner/<id>`.
  - `runner select`:
    - Print deterministic next leaf (id + path + attempts/max_attempts), or exit non-zero if complete.
    - Detect “stuck” leaf (attempts == max_attempts) and surface it clearly.

## Notes

- Plan CLI needs `pyyaml` when run via `uv run --with pyyaml`.
