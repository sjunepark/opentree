---
status: complete
created_at: 2026-01-18T04:47:16Z
---

# Task ii

## Focus

Always run the `runner` binary built from this repo (not a PATH-installed runner), and invoke it as a
subprocess to match real usage.

## Inputs

- `Cargo.toml` workspace members
- `runner` CLI (`docs/project/cli.md`)
- `PROPOSAL.md` (subprocess decision)

## Work

1. Implement a helper to build runner:
   - `cargo build -p runner` (debug is fine for MVP)
   - resolve the binary path deterministically (`target/debug/runner` or platform equivalent)
2. Implement harness subprocess calls (cwd = workspace):
   - `runner start`
   - `runner loop`
3. Capture stdout/stderr to `eval/results/.../runner.{start,loop}.log` (and/or keep them in meta).
4. Propagate environment (inherit by default), plus allow case-defined env overrides (optional).
5. Add tests for runner binary resolution (no network/Codex; validate command construction only).

## Output

- Added runner build + subprocess harness in `eval/src/harness.rs` (`cargo build -p runner`, deterministic binary path, `runner start/loop` execution).
- Captured runner stdout/stderr to per-run logs via `runner.start.log` and `runner.loop.log`.
- Added env override support for runner subprocesses (inherit by default, apply case `env` map).
- Added deterministic test for runner binary path resolution in `eval/src/harness.rs`.

## Handoff

Implement results layout + artifact capture in Task iii.
