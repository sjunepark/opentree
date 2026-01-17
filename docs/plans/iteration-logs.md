# Plan: Iteration Log Structure

**Priority:** Low
**Target:** `docs/project/iteration-logs.md`

## Why This Matters

Iteration logs are the audit trail for debugging. Understanding their structure helps diagnose failures and track run history.

## Topics to Cover

### Directory Structure

```text
.runner/iterations/{run-id}/{iter-n}/
├── meta.json         ← timing, node id, outcome
├── output.json       ← agent's status + summary
├── guard.log         ← guard stdout/stderr (if ran)
├── failure.log       ← failure output (guard failures or runner errors)
├── executor.log      ← executor output (if ran)
├── tree.before.json  ← tree snapshot pre-iteration
└── tree.after.json   ← tree snapshot post-iteration
```

### meta.json Format

```json
{
  "run_id": "run-abc123",
  "iteration": 1,
  "selected_id": "node-xyz",
  "status": "done|retry|decomposed",
  "guard": "pass|fail|skipped",
  "duration_ms": 12345,
  "started_at": null,
  "ended_at": null
}
```

Note: `started_at`/`ended_at` are placeholders for future timestamp support.

### Gitignore

- `.runner/iterations/` is gitignored
- Local-only; doesn't pollute repo history
- Enables clean working tree invariant

## Source Files

- `runner/src/io/iteration_log.rs`
- `runner/src/step.rs` — timing measurement
