---
status: pending
created_at: 2026-01-18T06:07:29Z
---

# Task iii

## Focus

Add a low-complexity observability path (“Option A.5”) and make the recommendation/migration triggers
more testable and less subjective.

## Inputs

- `PROPOSAL2.md`
- `runner/src/io/executor.rs` (current executor contract)
- `runner/src/io/process.rs` (process timeout/output handling)

## Work

1. Add “Option A.5: Improve observability while staying on `codex exec`” with concrete ideas (e.g.,
   streaming/teeing subprocess output to logs, richer failure metadata, etc.), clearly scoped as a
   smaller step than `app-server`.
2. Rewrite migration triggers into testable conditions (what we’d measure/observe to justify moving).
3. Add/adjust “Prepare” action items (record Codex version + invocation, define what event stream data
   we’d persist if we adopt `app-server`).

## Output

## Handoff
