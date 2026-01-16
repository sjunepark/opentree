# WIP

## Roadmap

- Next: implement `runner step` (one deterministic iteration end-to-end).
- Enforce runner policies: git safety (clean branch + refuse `main`/`master`), deterministic commits, local iteration logs.
- Add guard runner + state updates: run `just ci`, set `passes=true` on green, increment attempts / derive internal passes.
- Implement prompt pack builder + executor backends (Codex CLI / Claude CLI).
- Add extensibility sidecars + UX polish (events, output caps/defaults, `runner run` for multi-step looping).

## Current status

- Phase: MVP foundations (tree schema + CLI scaffolding).
- Implemented: v1 task tree schema, canonical JSON writing, invariant checks, deterministic leaf selection.
- Commands: `runner init|validate|select` (init creates `.runner/` placeholders + schema/tree).
- Guards: `just ci` currently runs Markdown checks via `rumdl`.
