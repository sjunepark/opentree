# Iteration Log Structure

Iteration logs are the audit trail for debugging. They enable post-mortem analysis of run history and failure diagnosis.

## Directory Layout

```text
.runner/iterations/{run-id}/{iter}/
├── meta.json           ← iteration metadata (timing, node, outcome)
├── planner_output.json ← decomposer output (child specs)
├── planner_executor.log ← executor (codex) stdout/stderr for decomposer agent
├── output.json         ← iteration status + summary
├── executor.log        ← executor (codex) stdout/stderr for executor agent (execute only)
├── guard.log           ← guard stdout/stderr (only when status=done)
├── runner_error.log    ← runner-detected failures (runner-internal or agent contract violations)
├── tree.before.json    ← tree snapshot pre-iteration
└── tree.after.json     ← tree snapshot post-iteration
```

All logs are **local-only** and **gitignored**. They don't pollute repo history and preserve the clean working tree invariant.

## When Each File Is Written

| File | When Written | Trigger |
|------|--------------|---------|
| `planner_output.json` | During decomposer phase | Written by `execute_and_load_json()` after decomposer completes |
| `planner_executor.log` | After decomposer completes | `write_executor_log()` captures command output |
| `output.json` | At iteration end | Runner-written canonical output for the iteration (status + summary) |
| `executor.log` | After executor completes | Written only when the executor agent runs |
| `guard.log` | After guards complete | Only when `status=done`; guards skip on retry |
| `runner_error.log` | On failure | Runner-internal errors and agent contract violations that force retry |
| `tree.before.json` | At iteration end | Snapshot of tree before agent ran |
| `tree.after.json` | At iteration end | Snapshot after all updates applied |
| `meta.json` | At iteration end | First file written in `write_iteration()` |

Write order in `write_iteration()` is deterministic: meta → output → guard.log → tree snapshots.

## File Formats

### meta.json

Runner-written metadata capturing iteration context and outcome:

```json
{
  "run_id": "run-abc123",
  "iter": 1,
  "node_id": "node-xyz",
  "status": "done",
  "guard": "pass",
  "started_at": null,
  "ended_at": null,
  "duration_ms": 12345
}
```

| Field | Type | Description |
|-------|------|-------------|
| `run_id` | string | Current run identifier |
| `iter` | u32 | Iteration number (1-indexed) |
| `node_id` | string | Selected leaf node id |
| `status` | enum | Agent status: `done`, `retry`, `decomposed` |
| `guard` | enum | Guard outcome: `pass`, `fail`, `skipped` |
| `started_at` | string? | Reserved for future timestamp support |
| `ended_at` | string? | Reserved for future timestamp support |
| `duration_ms` | u64? | Wall-clock time for entire iteration |

Duration is captured via `Instant::now()` at step start, includes executor + guards + tree updates + commit.

### output.json

Agent's structured output:

```json
{
  "status": "done",
  "summary": "Implemented the auth middleware..."
}
```

### planner_output.json

Decomposer output:

```json
{
  "summary": "Split into two executable tasks.",
  "children": [
    {
      "title": "Implement login flow",
      "goal": "Add OAuth2 login endpoints and callback handling.",
      "acceptance": ["Login succeeds with Google"],
      "next": "execute"
    }
  ]
}
```

### executor.log

Executor (codex) command output in standardized format:

```text
=== stdout ===
... agent stdout ...

=== stderr ===
... agent stderr ...
```

- Truncated to `output_limit_bytes` from config
- Includes timeout notification if process exceeded time budget

### guard.log

Guard command output, same format as executor.log:

```text
=== stdout ===
... just ci stdout ...

=== stderr ===
... just ci stderr ...
```

- Only written when `status=done` (guards skip on retry)
- Truncated to 1MB default
- Used for failure feedback: read and propagated to `.runner/context/failure.md` on next retry

### runner_error.log

Runner-internal failures (not agent failures):

```text
runner error: immutability failed: passed node 'auth' changed in next tree
```

- Written when guard execution fails internally, state updates fail, or tree validation fails
- Does NOT increment node `attempts` (state preserved from pre-iteration)
- Not propagated to agent context (isolated from agent view)

### tree.before.json / tree.after.json

Canonical tree snapshots. Both are sorted by `(order, id)` before serialization via `sort_children()` for deterministic comparison.

## Error Handling

Log write failures are handled with context propagation:

- Each writer creates parent directories defensively (`fs::create_dir_all`)
- Errors include path context via `.with_context()`
- If ANY log write fails, `run_step()` returns error and runner terminates
- Guard log reading is non-fatal (`.ok()` suppresses missing file)

## Relationship to Context Files

Iteration logs are distinct from ephemeral context files in `.runner/context/`:

| Location | Lifetime | Purpose |
|----------|----------|---------|
| `.runner/iterations/` | Permanent (gitignored) | Audit trail, debugging |
| `.runner/context/` | Rewritten each iteration | Agent input for current iteration |

Failure propagation: `guard.log` from iteration N is read and written to `.runner/context/failure.md` for iteration N+1 when guards failed.

## Debugging & Post-Mortem Analysis

**Tree diffs:** Compare `tree.before.json` and `tree.after.json` to see what changed.

**Failure diagnosis:**

- Guard failures → check `guard.log` for test output
- Runner errors → check `runner_error.log` for validation failures
- Agent behavior → check `output.json` for declared status and reasoning

**Iteration reconstruction:** Complete history in `.runner/iterations/{run_id}/` preserves full audit trail for any iteration.

## Source Files

| File | Purpose |
|------|---------|
| `runner/src/io/iteration_log.rs` | `write_iteration()`, `IterationMeta`, `IterationPaths` |
| `runner/src/step.rs` | Orchestration, timing capture, error handling |
| `runner/src/io/executor.rs` | `write_executor_log()` |
| `runner/src/io/guards.rs` | `write_guard_log()` |
