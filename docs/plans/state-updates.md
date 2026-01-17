# Plan: State Updates & Runner-Owned Fields

**Priority:** High
**Target:** `docs/project/state-updates.md`

## Why This Matters

The state update logic is central to determinism but only visible in tests. Understanding how `passes`/`attempts` are managed is essential for:

- Debugging unexpected behavior
- Understanding why agent edits to runner-owned fields are ignored
- Knowing when nodes become "stuck"

## Topics to Cover

### Runner-Owned Field Reset

- `reset_runner_owned_fields()` reindexes previous tree's state
- Agent-provided values for `passes`/`attempts` are **overwritten**
- New nodes (not in prev tree) get defaults: `passes=false`, `attempts=0`

### State Transitions

| Condition | `passes` | `attempts` |
|-----------|----------|------------|
| `done` + guards pass | `true` | unchanged |
| `done` + guards fail | unchanged | `+1` (if < max) |
| `retry` (guards skipped) | unchanged | `+1` (if < max) |
| `decomposed` | unchanged | unchanged |

### Attempt Saturation

- Increment only if `attempts < max_attempts`
- Prevents overflow; defines "stuck" state
- Runner determines stuck, not agent

### Derived Internal Passes

- `derive_internal_passes()` computes parent passes bottom-up
- Leaf passes only via agent + guards
- Parent passes iff **all children pass** (recursive)

## Source Files

- `runner/src/core/state_update.rs` — main logic
- `runner/src/core/types.rs` — `StateUpdateSummary`
