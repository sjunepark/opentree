# Refactoring Proposal: Node `next` Routing + Decomposer

## Problem

The current `TreeAgent` both decides **whether** to decompose and **how** to decompose in the same response.
That makes routing implicit and ties selection policy to prompt heuristics instead of explicit tree state.

## Goal

Move routing into the task tree and keep the runner deterministic:

- Add `next` to each node (`execute` | `decompose`)
- Runner **trusts `node.next`** to choose the agent (no decider)
- Decomposer outputs children *and* the `next` value for each child
- Executor remains unchanged

## Scope

### In Scope

- Add `next` to the task-tree schema and `Node`
- Replace `TreeAgent` with `DecomposerAgent`
- Route `runner step` by `node.next`
- Update schemas/prompts/tests/docs accordingly

### Out of Scope

- Guard system
- External APIs or CLI interface changes
- Parallel traversal or selection logic changes

## Constraints

- Deterministic execution must be preserved
- Existing tests updated to reflect new routing
- Node ID allocation scheme unchanged
- Step artifacts remain inspectable

## Affected Areas

- `runner/src/tree.rs` → `Node` gains `next`
- `schemas/task_tree/v1.schema.json` → add `next`
- `runner/src/agents/decomposer.rs` → new decomposer wrapper
- `runner/src/step.rs` → route by `node.next`
- Prompt templates + schemas for decomposer output
- Prompt Lab (inputs, schemas, UI types)

## Acceptance Criteria

- [ ] `node.next` exists on all nodes and is schema-validated
- [ ] Runner chooses decomposer/executor solely via `node.next`
- [ ] Decomposer outputs child specs with `next`
- [ ] Decomposer invariant violations hard-fail the step
- [ ] Tests and docs reflect updated flow
