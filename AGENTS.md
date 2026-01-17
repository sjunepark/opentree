# AGENTS.md

## Session Continuity

- **Always read `WIP.md` at session start** and keep it current. It reflects ongoing work so agents without prior context can resume immediately.
- **Read `VISION.md` and `ARCHITECTURE.md`** to understand the project's principles and technical design.

## Documentation Structure

- **Root docs** (`ARCHITECTURE.md`, `VISION.md`, `DECISIONS.md`): Goals and vision; may describe desired future state
  - ⚠️ `ARCHITECTURE.md` is aspirational — verify against code before assuming features exist
- **Project docs** (`docs/project/`): Implementation details; **must be accurate and current**

## Commands

- Use `just` for common workflows (`just --list` to see all); prefer `just ci` before committing.

## Rust

- Prefer using `cargo` cli, rather than directly editing `Cargo.toml`.
- Use Rust LSP (rust-analyzer) when text search is insufficient: tracing trait implementations, finding all usages before refactoring, resolving generic type bounds, or navigating cross-crate definitions. Prefer `Grep`/`Glob` for simple pattern matching.
- In Rust, treat comments as first-class documentation: write them well and keep them current. Skip trivial docs (e.g., things inferable from names or signatures).
