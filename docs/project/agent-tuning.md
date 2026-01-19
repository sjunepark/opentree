# Agent Tuning

Learnings from iterating on agent behavior. This doc captures failure patterns and fixes discovered through eval runs.

## Principles

### Defense in Depth

Agent behavior is shaped at multiple layers:

| Layer | Purpose | Example |
|-------|---------|---------|
| **Prompt guidance** | Teach correct behavior | "Only use `next=decompose` for multi-phase work" |
| **Prompt constraints** | Forbid bad behavior | "MUST NOT change `next`, `passes`, or `attempts`" |
| **Tool/script hooks** | Shape environment | Pre-commit checks, schema validation |
| **Runtime enforcement** | Catch violations | `reset_runner_owned_fields()` restores from snapshot |

Earlier layers prevent issues; later layers catch what slips through.

### Eval-Driven Iteration

1. Run eval case, observe failure
2. Diagnose root cause from logs (`executor.log`, `planner_output.json`, `agent_error.log`)
3. Hypothesize fix (prompt change, tool/script, runtime protection, or combination)
4. Re-run eval, compare metrics

Key metrics: iteration count, retry rate, tree depth, completion status.

## Failure Patterns

### Over-Decomposition

**Symptom:** Decomposer creates deep trees (3+ levels) for simple goals, never reaches implementation.

**Example:** "Build CLI calculator" decomposed into:

```text
├── Define CLI behavior (decompose)
│   ├── Define input format (decompose)
│   │   └── ...spec drilling forever
```

**Root cause:** No sizing guidance. Decomposer treated "write specs" as decomposable work.

**Fix:** Added task sizing section to decomposer prompt (`runner/src/io/prompts/decomposer.md`):

- Each child completable in one agent session (~30 min)
- "Writing is execution": specs, docs, code, tests are all `next=execute`
- Anti-patterns: don't split implement/test/format, don't create deep trees

**Result:** Calculator task decomposed into 2 children (impl+tests, docs) instead of 14+.

### Field Ownership Confusion

**Symptom:** Executor sets `next="done"`, causing schema validation error.

**Example log:**

```text
item_10 (reasoning): "I'm preparing to mark the node's next status as 'done'..."
item_11 (command): python3 ... child['next']='done' ...
```

**Root cause:** Executor confused routing field (`next`) with completion status. Prompt said "MAY edit open nodes" without specifying which fields.

**Fix (prompt):** Explicit field ownership in executor prompt (`runner/src/io/prompts/executor.md`):

```markdown
- You MAY edit open nodes (title, goal, acceptance), but:
  - MUST NOT change `next`, `passes`, or `attempts` (runner-owned fields)
```

**Fix (runtime):** Extended `reset_runner_owned_fields()` to restore `next` from previous snapshot (`runner/src/core/state_update.rs`).

**Result:** No more "done is not one of execute/decompose" errors.

## Runner-Owned Fields

Fields the runner controls exclusively:

| Field | Purpose | Agent Can Modify? |
|-------|---------|-------------------|
| `passes` | Completion status | No (derived from guard) |
| `attempts` | Retry counter | No (incremented on fail/retry) |
| `next` | Routing decision | No (set by decomposer, immutable after) |

New nodes from decomposition keep their `next` value (decomposer sets it correctly). Existing nodes have `next` restored from previous snapshot (executor can't change routing).

## Tuning Mechanisms

### Prompts

#### Decomposer Prompt

Key sections:

1. **Contract** - What decomposer must/must not do
2. **Task Sizing** - When to use `execute` vs `decompose`
3. **Selected Node** - Current node to decompose
4. **Tree Summary** - Context for planning

Critical guidance:

- Default to `next=execute` for most children
- Use `next=decompose` only for genuinely multi-phase work
- Avoid spec-drilling (decomposing "define behavior" into sub-specs)

#### Executor Prompt

Key sections:

1. **Contract** - Field ownership, output format
2. **Planner Notes** - Context from decomposer
3. **Selected Node** - Current task to execute
4. **History/Failure** - Previous attempt context (for retries)

Critical constraints:

- Do not modify passed nodes
- Do not set `passes=true`
- Do not change runner-owned fields (`next`, `passes`, `attempts`)
- Do not add children to any node

### Tools and Scripts

(To be added: schema validators, pre-commit hooks, guard scripts, etc.)

### Runtime Enforcement

See `runner/src/core/state_update.rs` for field protection logic. Key function:
`reset_runner_owned_fields()` restores `passes`, `attempts`, and `next` from
previous tree snapshot, preventing executor tampering.

## Eval Comparison

Before/after metrics for calculator-go:

| Metric | Before Fix | After Fix |
|--------|------------|-----------|
| Iterations | 4 (incomplete) | 3 (complete) |
| Final status | retry (error) | done + pass |
| Retries | Multiple (exhausted) | 0 |
| Tree depth | 2 | 2 |
| Children | 3 | 2 |
| `next="done"` error | Yes | No |

## Source Files

- `runner/src/io/prompts/decomposer.md` - Decomposer prompt template
- `runner/src/io/prompts/executor.md` - Executor prompt template
- `runner/src/core/state_update.rs` - Runtime field protection
- `runner/prompt_lab/prompts/decomposer/baseline.md` - Prompt lab variant
