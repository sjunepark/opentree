# Human Questions / Open Decisions

This is the running list of product/architecture questions that should be answered by a human (or explicitly
decided in `GOAL.md`) during the reboot.

If a question has been decided, remove it from this file and ensure the decision is captured in
`VISION.md` and/or `GOAL.md`.

## Product / Workflow

- What is the “agent session” implementation target (Codex CLI, Claude Code, other)?
- Is the runner intended to be provider-agnostic (pluggable executors) from day one?
- Should the runner require *fresh* git commits per iteration, or allow dirty working trees until pass?
- Should DECOMPOSE output be constrained to a strict “tree patch” format, or can it edit the tree file directly?

## Data Model

- What is the canonical tree file format (JSON vs YAML), and what is the schema versioning strategy?
- Do we need stable node IDs across rewrites/decompositions? If so, how are IDs generated?
- How do we represent leaf attempt budgets and failure classifications in the tree schema?

## Determinism & Safety

- What are the deterministic tie-break rules beyond `order` then `id` (if needed)?
- What is the workspace isolation story (temp workdir per iteration, git worktree, etc.)?
- What tool access boundary is required for safety (allowed commands/files, network)?
- Do we need timeouts/output limits at the runner level in MVP?

## Guard / Validation

- What is the single canonical guard entrypoint (`just ci`?) and what should it include (fmt/lint/test/build)?
- What artifacts should the runner record from guard runs (stdout/stderr, exit code, duration)?

## Persistence & Observability

- Where should iteration logs live and what is the retention policy?
- What is the minimum “memory doc set” required in MVP (feedback log, human questions, improvement ideas)?
- Should the runner emit machine-readable events (JSONL) for future UI integration?
