# Reviews

## Resolved

- `max_attempts`: keep per-node `max_attempts` (v1 stays as-is; a separate global cap can be added later if needed).
- Ralph loop comparison: this runner is stricter and more deterministic (task tree is source of truth, selection is “leftmost open leaf”, runner-owned `passes=true`, clean/guarded iterations). Baseline notes: `docs/knowledge/ralph-*.md`.
- Rust doc comments: present in `runner/src/main.rs` and `runner/src/tree.rs`.
- JSON Schema vs structs: intentional schema-first validation of `.runner/tree.json` plus Rust structs for typed access; schema lives at `schemas/task_tree/v1.schema.json` (embedded only to write on `runner init`).
- `just ci`: now runs Rust `fmt --check`, `clippy`, `test` + `rumdl check`.
