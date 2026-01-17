# WIP

## Now

- Completed `.plan/3` runner core-first work (immutability + state updates).
- Documented agent-declared status model in DECISIONS.md, ARCHITECTURE.md, VISION.md.
- Migrated runner CLI paths to `.runner/state/` + `.runner/context/` (init/validate/select + README/.gitignore).
- Added agent-declared output types (`AgentStatus`, `AgentOutput`) + status invariant validator.
- Updated runner-owned state updater to use `AgentStatus` (`retry` increments attempts; `done` + guards mark pass).

## Next

Refactor for agent-declared status model:

1. **Context writer** — clear and write `context/` each iteration (goal, history, failure)
2. **Core loop** — read agent output, validate status, conditional guard execution

## Notes

- Plan CLI needs `pyyaml` when run via `uv run --with pyyaml`.
