# Tree Validation & Invariants

The tree is the source of truth. Validation ensures determinism and prevents subtle corruption. Three layers: **schema** (structural), **invariants** (semantic), and **behavioral** (status + immutability).

## Validation Pipeline

Validation runs at two points in the step flow:

1. **At load time** (before agent runs): schema + invariants
2. **Post-execution** (after agent modifies tree): schema + invariants + immutability + status

```text
run_step()
├── load_tree()           ← schema + invariants
│   ├── validate_schema()
│   └── validate_invariants()
├── execute_and_load()    ← agent runs
├── load_tree()           ← schema + invariants (on modified tree)
├── validate_post_exec_tree()  ← immutability
└── validate_status()     ← status invariants
```

All validators collect ALL errors before returning (no early exit). Errors are deterministic and sorted for stability.

## 1. Schema Validation (Structural Layer)

**Schema location:** `schemas/task_tree/v1.schema.json` (source), `.runner/state/schema.json` (deployed)

**Implementation:** `io/tree_store.rs` → `validate_schema()`

Uses `jsonschema` crate to validate tree JSON against JSON Schema Draft 2020-12.

### What It Validates

| Constraint | Description |
|------------|-------------|
| Required fields | `id`, `order`, `title`, `goal`, `acceptance`, `passes`, `attempts`, `max_attempts`, `children` |
| Type correctness | Strings, booleans, integers, arrays as specified |
| Numeric bounds | `max_attempts >= 0`, `attempts >= 0` |
| No extra properties | `additionalProperties: false` |
| Recursive structure | Children are valid nodes |

### Error Format

```text
tree schema validation failed: <error1>; <error2>; ...
```

Errors are joined with "; " separator.

## 2. Semantic Invariants (Business Logic Layer)

**Implementation:** `core/invariants.rs` → `validate_invariants()`

Checks not expressible via JSON Schema. Traverses depth-first, building path context.

### Invariant Checks

| Invariant | Check | Error Message |
|-----------|-------|---------------|
| Unique IDs | `HashSet` tracking during DFS | `duplicate id '{id}' at {path}` |
| Max attempts positive | `max_attempts > 0` | `{path}: max_attempts must be > 0` |
| Attempts bounds | `attempts <= max_attempts` | `{path}: attempts {n} exceeds max_attempts {m}` |
| Children sorted | Pairwise `(order, id)` comparison | `{path}: children must be sorted by (order,id)` |

### Error Format

```text
tree invariants failed: <error1>; <error2>; ...
```

## 3. Immutability Enforcement (Passed-Node Integrity)

**Implementation:** `core/immutability.rs` → `check_passed_node_immutability()`

Nodes with `passes=true` are immutable. Runs post-execution to detect agent tampering.

### Per-Node Checks

For each node with `passes=true` in the previous tree:

| Requirement | Error Message |
|-------------|---------------|
| Node exists in next tree | `passed node '{id}' missing in next tree` |
| Same parent | `passed node '{id}' moved from parent '{old}' to '{new}'` |
| Identical by value | `passed node '{id}' changed in next tree` |

Comparison uses `PartialEq` trait (all fields including children). Errors sorted by node ID for stability.

### What's Allowed

Open nodes (`passes=false`) can be freely:

- Edited (goal, title, acceptance, etc.)
- Removed from tree
- Moved to different parent

### Error Format

```text
immutability failed: <error1>; <error2>; ...
```

## 4. Status Invariants (Agent Consistency)

**Implementation:** `core/status_validator.rs` → `validate_status_invariants()`

Ensures agent's declared status aligns with tree changes. Compares child count of selected node between prev and next trees.

### Constraints

| Agent Status | Tree Constraint | Rationale |
|--------------|-----------------|-----------|
| `done` | Selected node must NOT gain children | Work complete, not decomposed |
| `retry` | Selected node must NOT gain children | Retry = try again, not decompose |
| `decomposed` | Selected node MUST gain children | Decomposition requires subtasks |

### Error Format

```text
status=decomposed but selected node 'auth' did not gain children (prev=0, next=0)
status=done but selected node 'auth' gained children (prev=0, next=2)
```

Violation is treated as malformed iteration → triggers retry with error context.

## Canonicalization

**Implementation:** `tree.rs` → `Node::sort_children()`

```rust
fn sort_children(&mut self) {
    self.children.sort_by(|a, b|
        a.order.cmp(&b.order)
            .then_with(|| a.id.cmp(&b.id))
    );
    for child in &mut self.children {
        child.sort_children();
    }
}
```

Called on every tree write to ensure:

- Deterministic comparison between trees
- Immutability checks work via value equality
- Stable diffs for debugging

### Write Locations That Canonicalize

| Location | When |
|----------|------|
| `io/tree_store.rs` → `write_tree()` | Main tree persistence |
| `core/state_update.rs` → `apply_state_updates()` | After state changes |
| `io/iteration_log.rs` → `write_tree()` | Tree snapshots |
| `io/init.rs` | Initial tree creation |

## TreeStore: Load and Write

**Implementation:** `io/tree_store.rs`

### `load_tree(schema_path, tree_path)`

1. Read tree JSON from disk
2. Parse JSON to `serde_json::Value`
3. **Validate schema** → compile schema, validate, collect errors
4. **Validate invariants** → call `validate_invariants()`
5. Deserialize to `Node` struct
6. Return tree or error

### `write_tree(path, tree)`

1. Clone tree
2. Call `sort_children()` for canonicalization
3. Serialize to pretty JSON with trailing newline
4. Write atomically to disk

## Error Recovery: Repair Tree Mode

When tree validation fails after an agent session, the runner enters **REPAIR TREE** mode:

- Does NOT select a normal leaf
- Prompts agent to restore schema validity
- Commits the repair attempt
- Keeps loop automation-first (deterministic recovery)

Note: This is architecturally designed but not fully implemented in MVP.

## Error Reporting

All validation errors propagate as `anyhow::Error` to `run_step()` caller:

- Written to `runner_error.log` if iteration fails internally
- Does NOT increment node `attempts` (runner-internal failure)
- Iteration logs still written (meta.json, tree snapshots, output.json)

## Source Files

| File | Purpose |
|------|---------|
| `runner/src/core/invariants.rs` | `validate_invariants()` |
| `runner/src/core/immutability.rs` | `check_passed_node_immutability()` |
| `runner/src/core/status_validator.rs` | `validate_status_invariants()` |
| `runner/src/io/tree_store.rs` | `load_tree()`, `write_tree()`, `validate_schema()` |
| `runner/src/step.rs` | Validation orchestration (lines 92-114, 177-178) |
| `runner/src/tree.rs` | `Node::sort_children()` |
| `schemas/task_tree/v1.schema.json` | Schema definition |
