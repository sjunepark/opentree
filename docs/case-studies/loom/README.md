# Loom Case Study

Notes from studying [ghuntley/loom](https://github.com/ghuntley/loom), a Rust-based AI coding agent.

## Why Loom?

Loom is relevant as a reference because it has:

- **Explicit state machine**: Event-driven FSM with typed states, events, and actions. State
  transitions are deterministic; caller owns I/O.
- **Spec verification workflow**: Tracks spec-vs-implementation drift with item-level status (✅/⚠️/❌).
- **Post-hook infrastructure**: Auto-commit as runner-owned hook, not LLM-visible tool.
- **Workspace boundaries**: Path canonicalization + containment checks for file tools.
- **Bounded retries**: Typed errors + exponential backoff + hard caps; graceful degradation on
  exhaustion.
- **Thread persistence**: Single JSON document with monotonic versioning and optimistic concurrency.

## What to Borrow (patterns)

- Determinism via explicit state machine: keep creative decisions in prompts, deterministic
  decisions in code.
- Post-hook infrastructure for validation/commits: runner enforces, not prompt compliance.
- Spec verification workflow: item-by-item audit with actionable recommendations.
- Typed errors + bounded retries: classify failures, transition on exhaustion (don't crash).
- Snapshot state for resumption: persist enough to render "what was happening", not full state.

## What to Avoid (scope)

Loom is a full product (~50 crates): server, auth, analytics, weavers, web UI. The valuable
reference is `loom-common-core` (agent state machine + tool abstractions). Everything else is
platform that we don't need for a local-only runner MVP.

## Docs

- `lessons.md`: 12 patterns with detailed explanations and actionable adaptations
