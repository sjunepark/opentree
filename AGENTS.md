# AGENTS.md

## Session Continuity

- **Always read `WIP.md` at session start** and keep it current. It reflects ongoing work so agents without prior context can resume immediately.
- **Read `VISION.md` and `ARCHITECTURE.md`** to understand the project's principles and technical design.

## Documentation Structure

Root markdown files (`ARCHITECTURE.md`, `VISION.md`, `DECISIONS.md`) should stay **concise and scannable**:

- Keep root docs focused on high-level concepts and decisions
- Move detailed explanations (data flows, examples, deep dives) to `docs/project/`
- Link to detailed docs with relative paths: `[topic](docs/project/topic.md)`
- Readers should be able to understand the system from root docs alone, then drill down as needed

This enables progressive disclosure: quick orientation from root docs, detailed reference when needed.

## Commands

- Use `just` for common workflows (`just --list` to see all); prefer `just ci` before committing.

## Rust

- Prefer using `cargo` cli, rather than directly editing `Cargo.toml`.
- Use Rust LSP (rust-analyzer) when text search is insufficient: tracing trait implementations, finding all usages before refactoring, resolving generic type bounds, or navigating cross-crate definitions. Prefer `Grep`/`Glob` for simple pattern matching.
- In Rust, treat comments as first-class documentation: write them well and keep them current. Skip trivial docs (e.g., things inferable from names or signatures).
