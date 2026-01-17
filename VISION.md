# Vision: Deterministic Goal-Driven Agent Loop Runner

This document defines the product vision and **principles**.

- Canonical technical reference: `ARCHITECTURE.md`
- Decision log (dated rationale): `DECISIONS.md`

## What we’re building

A deterministic local runner that loops fresh agent sessions over a strict, goal-oriented task tree to make
sequential, validated progress.

## Principles

### Deterministic runner, flexible agent

- The runner is deterministic: selection, limits, validation, and invariants are enforced consistently.
- The agent can be creative during planning/execution, but only within the runner’s rules.

### Tree-first, schema-first state

- The task tree is the machine-facing **source of truth** and is strictly validated every iteration.
- The node schema stays intentionally small: no ad-hoc extension or “just add a field”.
- New needs are represented via new nodes or runner-owned sidecars, not schema drift.

### Fresh context per iteration

- Every iteration is a fresh agent session/context.
- Continuity comes from repo artifacts (tree + memory docs + git history), not long-lived agent context.

### Sequential progress only

- Exactly one actionable leaf at a time (no parallel workflows).
- Planning and execution happen just-in-time, driven by the next leaf.

### Always proceed; record uncertainty

- The loop does not stop for questions.
- Uncertainty is recorded explicitly as assumptions and open questions for later resolution.

### Agents assess, guards verify

- Guards (CI, tests, linting) verify syntactic correctness but cannot assess semantic completeness.
- Agents know when work is partial or complete; they declare status explicitly.
- A node passes only when agent declares "done" AND guards pass.
- This two-signal model prevents false positives (guards pass but work is incomplete).

### Validation is the feedback loop

- Formatting/linting/tests are not "nice to have"; they are the mechanism that makes the system
  self-correcting.

### Automation-first, UI-secondary

- All workflows should be automatable end-to-end by the runner.
- UI is explicitly secondary; correctness and determinism come first.

## Non-goals (near term)

- Parallel execution or multi-agent concurrency.
- Remote execution / CI-as-runner as the default.
- A polished UI before the core loop is correct and deterministic.
- Auto-push, PR creation, or hosted GitHub integrations.

## Glossary

- Runner: deterministic CLI that selects work, enforces invariants, runs validation, and persists state.
- Agent: the LLM tool invoked by the runner (fresh session per iteration).
- Task tree: ordered goal tree that represents the plan and progress state.
- Leaf: a node with no children that is selected for decomposition or execution.
