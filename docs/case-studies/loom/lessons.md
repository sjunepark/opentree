# Lessons From Loom (../loom)

These notes summarize concrete patterns from the Loom codebase that are directly relevant to this
repo’s pivot toward a deterministic looping-agent runner.

Loom context (high-level):

- Rust workspace with many crates (`crates/`), plus a SvelteKit web UI (`web/`).
- “Spec pack” lives in `specs/` with an index (`specs/README.md`) and verification docs that compare
  specs vs the implementation (`verification.md`, various `*review*.md`).

## 1) Spec packs are useful, but only if “code reality” is enforced

Loom explicitly treats “specs” as intent and the codebase as the source of truth:

- `AGENTS.md` includes a warning: specs may describe planned features that are not implemented; you
  must search the codebase before assuming.
- `verification.md` tracks spec-vs-implementation match rates crate-by-crate.

Why this matters for our pivot:

- We’re going to create strict schemas and deterministic runner semantics. That will naturally
  produce docs that can drift.
- A lightweight “verification” document (or tests that encode the same invariants) is the practical
  mechanism that prevents drift.

Actionable adaptation:

- Keep a “spec index” for the runner (schemas, invariants, prompts, guard contract).
- Add a verification checklist that maps “must be true” requirements to:
  - where they are enforced in code, and/or
  - which tests cover them.

## 2) Determinism via explicit state machines (event → action)

Loom’s agent uses an explicit state machine (`loom-common-core/src/state.rs`, `agent.rs`):

- State transitions are driven by a closed set of events (`AgentEvent`).
- The state machine is synchronous/pure-ish and returns an `AgentAction` to be performed by an
  orchestrator (the caller owns I/O).
- This makes behavior testable and replayable: given the same event sequence, you get the same
  action sequence.

Why this matters for our pivot:

- Our runner is explicitly required to be deterministic.
- We’re introducing node attempt budgets and mode switches (decompose vs execute vs rewrite/expand).
  Those are “state machine” problems more than “prompt” problems.

Actionable adaptation:

- Model the runner loop as an explicit state machine:
  - load/validate tree → select leaf → decide mode → run agent → run guards → update tree → persist
- Keep “deterministic decisions” in code, and push “creative decisions” into the agent prompts.

## 3) “Post hook” infrastructure: keep commits/validation out of the agent’s toolset

Loom’s auto-commit system is not a tool exposed to the LLM:

- Mutating tools (like `edit_file` / `bash`) trigger a `PostToolsHook` state.
- The orchestrator runs auto-commit after tool execution, then continues the loop.
- Commit messages are generated from diffs using a small, dedicated model; this keeps the main agent
  context clean and makes commits consistent.

Why this matters for our pivot:

- You explicitly want an automatic feedback loop where the runner enforces validation and prevents
  “passes=true” unless guards are green.
- That enforcement should live in the runner, not be “optional behavior” inside prompts.

Actionable adaptation:

- Treat “run guards” as a post-execute hook owned by the runner.
- Treat “commit policy” similarly (exact “when to commit” is TBD, but enforcement should be in the
  runner, not dependent on prompt compliance).

## 4) Tool systems need: schema + sandbox boundary + timeouts/output limits

Loom’s tool system is built around:

- JSON Schema for tool input (so the model can call tools structurally).
- A hard workspace boundary: file paths are canonicalized and rejected if they escape the workspace
  root (`PathOutsideWorkspace`).
- Operational limits: command timeouts, output truncation, max bytes read, etc.

Why this matters for our pivot:

- “Agents should have access to necessary tools” is a product requirement.
- “Determinism and safety boundaries” requires that tool access is constrained and observable.

Actionable adaptation:

- In MVP, rely on a single explicit guard entrypoint (ex: `just ci`) for feedback and avoid
  heuristic/auto-detected checks.
- As we add more first-class tools, encode:
  - strict input schemas,
  - explicit allow/deny boundaries,
  - timeouts and output limits,
  - typed errors that the runner can classify.

## 5) Typed errors + bounded retries beat “keep trying forever”

Loom distinguishes error origins (LLM vs tool vs I/O) and uses bounded retries/backoff for
transient failures.

Why this matters for our pivot:

- Your node policy (“leaf can run N iterations, then rewrite/expand”) is exactly “bounded retry with
  state transition on exhaustion”.

Actionable adaptation:

- Represent “attempt count” and “retry exhaustion strategy” as runner state, not prompt convention.
- Classify failures (guard failure vs executor failure vs schema corruption) and handle them
  deterministically.

## 6) Quality gates as a single entrypoint

Loom’s `Makefile` provides a single `make check` entrypoint (format + lint + build + test).

Why this matters for our pivot:

- A single guard entrypoint keeps the runner deterministic and makes “passes=true” verifiable.
- A single entrypoint is also “agent-editable”: agents can update the guard command implementation
  if the repo changes.

Actionable adaptation:

- Standardize on one guard entrypoint (you suggested `just ci`) and make it required for passing.

## 7) Scope caution: Loom is a product; our runner MVP should stay small

Loom includes server-side LLM proxying, web UI, thread persistence/sync, Kubernetes “weavers”, ACP
editor integration, etc.

Takeaway:

- Loom demonstrates *patterns* (determinism, schemas, hooks, verification), but we should avoid
  importing its full product surface area during the MVP of a local-only runner.

## 8) Modularity comes from stable interfaces + layer boundaries

Loom is aggressively interface-driven (Rust traits) and organized into layers (common/core types,
CLI concerns, server concerns, web concerns). Even without copying the crate layout, the underlying
pattern is:

- define small, stable interfaces at the “bottom”
- keep higher-level orchestration at the “top”
- avoid dependency cycles; keep boundaries crisp

Why this matters for our pivot:

- We need to support multiple “executors” (LLM runners), multiple guard strategies, and potentially
  multiple storage backends for the tree/logs over time.

Actionable adaptation:

- Define explicit Go interfaces for:
  - executor invocation (Codex/Claude/other)
  - guard runner (ex: `just ci`)
  - tree storage (read/validate/write atomically)

## 9) UI integration works better as a protocol boundary

Loom’s ACP implementation demonstrates a useful strategy: put the “agent engine” behind a protocol
boundary so editors/UIs can drive it without coupling to internal details.

Why this matters for our pivot:

- You eventually want a browser view of the task tree. That UI becomes far easier if the runner can
  expose state/events via a stable boundary (even if the first implementation is “just render JSON
  files”).

Actionable adaptation:

- Keep the runner’s persisted tree as a strict, documented contract.
- Consider an eventual read-only mode for consuming the tree + iteration logs (CLI JSON output,
  simple local HTTP, etc.) without embedding UI assumptions into the core loop.
