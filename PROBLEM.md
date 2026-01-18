# Problem: Decomposition Prompting Gap

## Status (2026-01-18)

This is addressed by introducing a dedicated **tree agent** that outputs structured
decomposition decisions (`decision=execute|decompose` + optional child specs). The
runner applies child nodes to `.runner/state/tree.json` deterministically, so
decomposition no longer relies on the executor agent discovering/editing tree files.

## Summary

Previously, when an agent declared `status: "decomposed"`, it was expected to add children to `tree.json`. The prompt did not tell the agent:

1. Where `tree.json` is located (`.runner/state/tree.json`)
2. The JSON schema to use
3. How to structure child nodes

## Current State

**What the prompt says:**

```text
Runner contract:
- Do not modify passed nodes.
- Do not set `passes=true` (runner-owned).
- Only add children when declaring `decomposed`.
- Output must be structured JSON with `status` and `summary`.
```

**What's missing:**

- Path to tree file
- Tree schema reference
- Example of valid child node structure

## Evidence

In `runner/src/step.rs:204`:

```rust
let next_tree = load_tree(&schema_path, &tree_path)?;
```

The runner loads `tree.json` after agent execution, expecting the agent may have modified it. But the agent isn't told where or how.

## Impact

Decomposition likely doesn't work reliably. The agent either:

- Discovers the file path organically (fragile)
- Ignores the instruction (decomposition broken)
- Fails with invalid tree structure

## Proposed Fix

Add to prompt:

1. Tree path: `.runner/state/tree.json`
2. Schema path: `.runner/state/schema.json`
3. Example child node structure
4. Clear instruction: "When decomposing, read tree.json, add children to the selected node, write back"

## Testing

Create an eval case that:

1. Gives a goal requiring decomposition
2. Verifies `tree.json` has children after completion
3. Validates children conform to schema
