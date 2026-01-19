# Problem: Decomposition Prompting Gap

## Status (2026-01-19)

Resolved by introducing a dedicated **decomposer agent** and moving routing into the task tree via
`node.next`. The decomposer outputs child specs (including each child’s `next`), and the runner
writes them into `.runner/state/tree.json` deterministically.

## Summary

Previously, decomposition relied on a combined decision+decompose agent and implicit routing. This
blurred responsibilities and made it unclear where decomposition should happen or how children were
defined.

## Current State

**What the runner does now:**

- Uses `node.next` to decide whether to run the decomposer or executor
- Decomposer outputs child specs (`title`, `goal`, `acceptance`, `next`)
- Runner applies child specs to the selected node and enforces invariants

**What the prompt says (decomposer):**

- Decompose the selected node into child specs
- Provide `next` for each child (`execute` or `decompose`)
- Do not edit repository files

## Impact

Decomposition no longer depends on the executor mutating `tree.json`. The runner owns all tree
writes, and routing is explicit in the tree schema.

## Testing

- Decomposer outputs are schema-validated and must include `next`
- Step tests ensure decomposition creates children and selection proceeds deterministically
