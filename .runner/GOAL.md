# GOAL: Deterministic Goal-Driven Agent Loop Runner (MVP)

This document turns `../VISION.md` into an execution-ready goal spec for the repository.
It lives under `.runner/` because it is part of the per-project run context the runner provides to each loop.

## Desired Outcome

Build a **deterministic local runner** that repeatedly spawns **fresh, isolated agent sessions** to:

1. Take a **Goal** (`GOAL.md`).
2. Build/refine a **sequential task tree** (no parallel workflows).
3. Execute the **single next actionable leaf**.
4. Run **automatic validation** (format/lint/test/etc) as feedback.
5. Persist state + learnings and continue until completion.

UI is explicitly secondary (nice-to-have later; not part of the MVP).

## Success Criteria (MVP)

- The runner can load a task tree file, validate it against a strict schema, and reject invalid state.
- The runner deterministically selects the next open leaf (lowest `order`, tie-break by `id`) every iteration.
- The runner supports at least two modes per leaf:
  - **DECOMPOSE**: agent expands the leaf into ordered children.
  - **EXECUTE**: agent performs the work for a single-context leaf.
- The runner executes a single deterministic “guard” entrypoint (e.g., `just ci`) after EXECUTE.
- `passes=true` is only recorded when guards succeed.
- Nodes with `passes=true` are immutable (runner rejects edits to their object values).
- The system persists:
  - the updated tree (atomically),
  - iteration logs,
  - “memory” markdown files (feedback log, human questions, assumptions, improvement ideas).
- The run completes when there are **no open nodes remaining**.

## Scope

### In scope (MVP)

- Strict, versioned schema for the task tree + validation on every iteration.
- Deterministic selection logic + state transitions.
- Leaf attempt budgets (`N` attempts) with deterministic handling on exhaustion:
  - rewrite leaf acceptance, and/or
  - expand into children.
- A single configured guard entrypoint (prefer `just ci`).
- A minimal executor interface to invoke “an agent session” (implementation TBD).
- Memory/log artifacts and discoverability conventions.

### Explicit non-goals (near term)

- Parallel execution / multi-agent concurrency.
- Polished UI (TUI/web) before correctness/determinism.
- Remote-first CI-as-runner.
- GitHub integrations (auto-push, PR creation) as a default behavior.

## Constraints / Invariants (from `../VISION.md`)

- **Deterministic runner, flexible agent**: creativity is allowed only within runner rules.
- **Strict schemas and machine-validated state**: tree is the source of truth.
- **Fresh context per iteration**: continuity via repo artifacts, not long prompts.
- **Sequential progress**: one leaf at a time, just-in-time expansion.
- **No pass without green guards**.
- **Immutability**: `passes=true` nodes cannot be edited; improvements happen via new nodes.

## Key Decisions (TBD)

These must be decided early because they shape the architecture:

- Language/runtime (Go vs Rust vs other).
- Tree file format and storage:
  - JSON vs YAML vs another strict format,
  - atomic writes strategy,
  - explicit schema version (no schema drift).
- Agent executor integration:
  - how to spawn “fresh session” deterministically,
  - what tools are allowed/blocked,
  - workspace isolation strategy.
- Commit policy:
  - when to commit (every iteration vs on pass vs configurable),
  - how to generate commit messages.
- Guard contract:
  - canonical command (`just ci`?) and failure reporting format.

## Related References

- `../VISION.md` — canonical vision and principles.
- `../docs/loom/lessons.md` — patterns from Loom worth borrowing (state machines, hooks, guard entrypoints, tool boundaries).
