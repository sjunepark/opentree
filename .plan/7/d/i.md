---
status: complete
created_at: 2026-01-18T04:47:16Z
---

# Task i

## Focus

Implement the `eval` CLI entrypoints and flag surface for local experimentation.

## Inputs

- `PROPOSAL.md` (CLI interface)
- Existing project `justfile` conventions

## Work

1. Add the `eval` binary crate with clap commands:
   - `eval list`
   - `eval run <case-id> [--runs N]`
   - `eval report <case-id>`
   - `eval clean <case-id>`
2. Make output stable and greppable (plain text, minimal noise).
3. Ensure `eval run` returns non-zero on harness errors and prints where results were written.
4. Add CLI parsing tests for basic commands.

## Output

- Added CLI commands (`list`, `run`, `report`, `clean`) in `eval/src/main.rs` + `eval/src/cli.rs`.
- Implemented run flow wiring with stable output lines for run and report commands.
- `eval run` prints results directory and returns non-zero on failures via propagated errors.

## Handoff

Proceed to reporting aggregation in Task ii.
