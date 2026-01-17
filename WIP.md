# WIP

## Now

- Completed `.plan/4` (`runner step` one full iteration, context/prompt/executor/guards/run state/logging).
- Implemented `runner start` (auto run-id, `runner/<run-id>` branch checkout, bootstrap commit).
- Enforced git policy + deterministic per-iteration commits in `runner step`.

## Next

- Add missing MVP commands (`runner validate`, `runner select`) or keep CLI intentionally minimal.
- Add executor timeouts/output caps (shared per-iteration budget) and improve failure commits.

## Notes

- Plan CLI needs `pyyaml` when run via `uv run --with pyyaml`.
