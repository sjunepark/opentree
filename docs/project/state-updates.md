# State Updates

Runner-owned state transitions for task trees.

## Overview

The runner owns `passes` and `attempts` fields—agent edits are ignored. After
each step, state updates apply in order:

1. Reset runner-owned fields from previous tree
2. Apply transition rules to selected node
3. Derive internal passes bottom-up
4. Sort children for determinism

Entry point: `apply_state_updates()` in `runner/src/core/state_update.rs:13-69`

## Runner-Owned Field Reset

`reset_runner_owned_fields()` overwrites `next` tree values with `prev` tree
values, enforcing runner ownership.

| Node Status | `passes` | `attempts` |
|-------------|----------|------------|
| Existing    | from prev | from prev |
| New (added by agent) | `false` | `0` |

Code: `runner/src/core/state_update.rs:84-96`

## State Transitions

Applied to the selected node based on `AgentStatus` and `GuardOutcome`:

| Status | Guard | `passes` | `attempts` |
|--------|-------|----------|------------|
| Done | Pass | `true` | unchanged |
| Done | Fail | unchanged | `+1` if < max |
| Done | Skipped | unchanged | unchanged |
| Retry | (skipped) | unchanged | `+1` if < max |
| Decomposed | — | unchanged | unchanged |

Code: `runner/src/core/state_update.rs:34-59`

## Attempt Saturation

Attempts increment only when `attempts < max_attempts`. This:

- Prevents overflow
- Defines "stuck" state (attempts == max_attempts)
- Ensures runner, not agent, decides when node is stuck

## Derived Internal Passes

`derive_internal_passes()` propagates pass status bottom-up after transitions:

- **Leaf nodes**: keep existing `passes` value
- **Parent nodes**: `passes = all_children_passed`
- Only recorded in summary when transitioning to `true`

This ensures parent completion reflects actual child completion, regardless of
what the agent claims.

Code: `runner/src/core/state_update.rs:98-118`

## Runner Error Recovery

When step execution fails (timeout, output parse error, etc.), the runner enters
a recovery path instead of propagating the error immediately:

| Condition | `status` | `guard_outcome` | `attempts` |
|-----------|----------|-----------------|------------|
| Executor timeout | Retry | Skipped | unchanged |
| Output parse error | Retry | Skipped | unchanged |
| Guard spawn/I/O error | Retry | Skipped | unchanged |
| Other runner error | Retry | Skipped | unchanged |

Recovery behavior:

1. Error message written to `runner_error.log` under the iteration directory (for human debugging)
2. Tree persisted without consuming an attempt
3. Iteration logged normally (meta.json, tree snapshots)
4. Error propagated after persistence (step returns error, but state is consistent)

Attempts only increment from successful agent outputs via `apply_state_updates()`.
Runner-internal failures never increment attempts.

Runner-internal errors are not fed back to the node agent via `.runner/context/history.md` or
`.runner/context/failure.md`.

Code: `runner/src/step.rs` `run_step()` recovery `match attempt { ... }`

## Flow Summary

```text
apply_state_updates(prev, next, selected_id, status, guard)
  │
  ├─ index_runner_owned(prev)        → HashMap<id, (passes, attempts)>
  ├─ reset_runner_owned_fields(next) → overwrite from prev, defaults for new
  ├─ find selected node
  ├─ apply transition rules          → update passes/attempts per table
  ├─ derive_internal_passes(root)    → propagate passes bottom-up
  └─ sort_children()                 → deterministic output
```
