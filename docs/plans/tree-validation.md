# Plan: Tree Validation & Invariants

**Priority:** Medium
**Target:** `docs/project/tree-validation.md`

## Why This Matters

The tree is the source of truth. Validation ensures determinism and prevents subtle corruption. Two layers: schema (structural) and invariants (semantic).

## Topics to Cover

### Schema Validation

- JSON Schema at `.runner/state/schema.json`
- Validates structure, required fields, types
- Run on every tree load

### Semantic Invariants (`core/invariants.rs`)

| Invariant | Check |
|-----------|-------|
| Unique IDs | No duplicate `id` across all nodes |
| Attempts bounds | `0 <= attempts <= max_attempts` |
| Max attempts positive | `max_attempts > 0` |
| Children sorted | By `(order ASC, id ASC)` |

### Immutability (`core/immutability.rs`)

Nodes with `passes=true` in previous tree must:

- Exist with same `id` in next tree
- Be byte-for-byte identical in canonical form
- Appear in same structural position (parent + sibling order)

### Canonicalization

- `sort_children()` on every write
- Enables deterministic comparison
- Required for immutability checks

### Status Invariants (`core/status_validator.rs`)

| Status | Tree Constraint |
|--------|-----------------|
| `done` | No new children on selected node |
| `retry` | No new children on selected node |
| `decomposed` | Must have added children |

## Source Files

- `runner/src/core/invariants.rs`
- `runner/src/core/immutability.rs`
- `runner/src/core/status_validator.rs`
- `runner/src/io/tree_store.rs` — canonicalization
