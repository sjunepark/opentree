# AGENTS.md

## Session Continuity

- **Read `VISION.md` and `ARCHITECTURE.md`** to understand the project's principles and technical design.

## Documentation Structure

- **Root docs** (`ARCHITECTURE.md`, `VISION.md`, `DECISIONS.md`): Goals and vision; may describe desired future state
  - ⚠️ `ARCHITECTURE.md` is aspirational — verify against code before assuming features exist
- **Project docs** (`docs/project/`): Implementation details; **must be accurate and current**

## Commands

- Use `just` for common workflows (`just --list` to see all); prefer `just ci` before committing.

## Frontend

- Use `bun` for frontend development (install, run, test, build).
- When working with Svelte code, always use the Svelte MCP to read Svelte 5 docs.

## Rust

- Prefer using `cargo` cli, rather than directly editing `Cargo.toml`.
- Use Rust LSP (rust-analyzer) when text search is insufficient: tracing trait implementations, finding all usages before refactoring, resolving generic type bounds, or navigating cross-crate definitions. Prefer `Grep`/`Glob` for simple pattern matching.
- In Rust, treat comments as first-class documentation: write them well and keep them current. Skip trivial docs (e.g., things inferable from names or signatures).
- Avoid macros when possible; prefer functions, generics, or traits. When a macro seems warranted, discuss with the user first—macros obscure control flow and complicate debugging.

## Logging

- Use `tracing` crate. Control via `RUST_LOG` env (e.g., `RUST_LOG=runner=debug`).
- **Log levels**: `info!` for milestones (run start, branch created), `debug!` for diagnostics (state transitions, file ops), `warn!` for non-fatal issues (timeouts, guard failures), `error!` for failures.
- **Structured fields**: Use `field = %value` for Display, `field = ?value` for Debug. Example: `info!(workdir = %path, "starting")`
- Use `#[instrument(skip_all, fields(...))]` for function-level tracing with relevant context.
- **What/where to log**: Log at boundaries and decision points, not in utility functions or hot paths. Focus on state changes, external calls, and branch decisions. Let callers log—callees should be quiet unless they have unique context.
