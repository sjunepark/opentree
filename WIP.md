# WIP

## Now

- Completed `.plan/4` (`runner step` one full iteration, context/prompt/executor/guards/run state/logging).
- Added runner init scaffolding for state/context/iterations and new run state + prompt builder adapters.
- Added step orchestration tests for `done`/`retry`/`decomposed`.

## Next

- Resolve run-id strategy (deterministic input vs generated at run start).
- Decide next plan milestone after `runner step` stabilization.

## Notes

- Plan CLI needs `pyyaml` when run via `uv run --with pyyaml`.
