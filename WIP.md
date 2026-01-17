# WIP

## Now

- Completed `.plan/4` (`runner step` one full iteration, context/prompt/executor/guards/run state/logging).
- Completed `.plan/5` (testing harness + fixtures + deterministic scenario coverage).
- Implemented `runner start` (auto run-id, `runner/<run-id>` branch checkout, bootstrap commit).
- Enforced git policy + deterministic per-iteration commits in `runner step`.
- Implemented TOML runner config + shared per-iteration timeout/output caps; `runner step` commits even on post-start failures.
- Clarified attempt semantics: runner-internal failures do not increment `attempts` (only successful agent outputs do).
- Decided stuck-node policy: hard-stop (documented in `DECISIONS.md` + `ARCHITECTURE.md`).
- Enforced stuck-node hard-stop in `runner step` (exits non-zero when `attempts == max_attempts`).

## Next

- Execute `.plan/6` (MVP commands: `runner validate` + `runner select`).

## Notes

- Plan CLI needs `pyyaml` when run via `uv run --with pyyaml`.
