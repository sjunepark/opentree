# Lessons From Loom

These notes summarize patterns from the [Loom](https://github.com/ghuntley/loom) codebase relevant
to deterministic, goal-driven agent runners.

Loom context (high-level):

- Rust workspace (~50 crates in `crates/`), SvelteKit web UI (`web/`).
- "Spec pack" in `specs/` with index (`specs/README.md`) + verification docs (`verification.md`).
- Proxy architecture: clients never store LLM API keys; all calls go through `loom-server`.
- Two execution modes: local CLI (trust-based) and "weavers" (Kubernetes sandboxing).

## 1) Spec packs are useful, but only if "code reality" is enforced

Loom treats `specs/` as design intent and Rust code as reality:

- `AGENTS.md` warns: specs may describe planned features; assume NOT implemented, check code first.
- `verification.md` tracks spec-vs-implementation match rates with item-level status:
  - ✅ matches spec
  - ⚠️ partial/minor discrepancy
  - ❌ missing/significantly different

Verification workflow:

1. Start from spec index (`specs/README.md`) with explicit "Spec → Code" mappings.
2. Structured item-by-item audit: enums, struct fields, defaults, CLI flags, endpoints.
3. Record match status + notes for each item.
4. Drift becomes actionable recommendations: "spec updates needed" vs "implementation fixes".

Concrete examples of caught drift:

- `ThreadSummary.message_count` became `u32` instead of `usize` (field type).
- Error variant names differ between spec and code (⚠️ naming).
- Route patterns differ from spec (e.g., `.git` suffix mismatch in SCM routes).

Anti-drift via generation: HTTP APIs use `utoipa` to generate OpenAPI from Rust types—spec is
derived from implementation, not maintained separately.

Actionable adaptation:

- Keep a "spec index" for the runner (schemas, invariants, prompts, guard contract).
- Add a verification checklist mapping "must be true" requirements to code/tests.
- Consider deriving API docs from types where possible (reduces manual sync burden).

## 2) Determinism via explicit state machines (event → action)

Loom's agent is an event-driven FSM: consumes `AgentEvent`, mutates state, returns `AgentAction`.
The state machine does NOT do I/O—caller performs I/O and feeds results back as events.

**States** (each variant carries its context, not global flags):

- `WaitingForUserInput { conversation }` — idle at turn end
- `CallingLlm { conversation, retries }` — LLM request in flight (streaming deltas arrive here)
- `ProcessingLlmResponse { conversation, response }` — decision point: tools or back to waiting
- `ExecutingTools { conversation, executions: Vec<ToolExecutionStatus> }` — parallel tool tracking
- `PostToolsHook { conversation, pending_llm_request, completed_tools }` — auto-commit hook point
- `Error { conversation, error, retries, origin }` — recoverable, retry depends on origin
- `ShuttingDown` — terminal

**Events**:

- `UserInput(Message)` — user submitted
- `LlmEvent` — streaming: `TextDelta`, `ToolCallDelta`, `Completed(LlmResponse)`, `Error(LlmError)`
- `ToolCompleted { call_id, outcome }` — tool finished (success or error)
- `PostToolsHookCompleted { action_taken }` — post-tool hooks finished
- `RetryTimeoutFired` — external backoff timer
- `ShutdownRequested` — graceful shutdown from any state

**Actions** (what caller should do):

- `SendLlmRequest(request)`, `ExecuteTools(Vec<ToolCall>)`, `RunPostToolsHook`, `WaitForInput`
- `DisplayMessage(String)`, `DisplayError(String)`, `Shutdown`

Key design: state transitions decide *what*; caller decides *how* (async execution, timers, etc.).

**State persistence**: Loom converts internal state to compact `AgentStateSnapshot` (kind + retries +
last_error + pending tool calls) for thread persistence—UI renders "what the agent is doing" without
embedding all internal details.

Actionable adaptation:

- Model runner loop as explicit state machine: load tree → select leaf → decide mode → run agent →
  run guards → update tree → persist
- Keep "deterministic decisions" in code; push "creative decisions" into prompts.
- Consider snapshotting runner state for resumption/debugging (not full state, but enough to render
  "what was happening").

## 3) "Post hook" infrastructure: keep commits/validation out of the agent's toolset

Loom's auto-commit is runner infrastructure, NOT an LLM-visible tool:

**Trigger conditions**:

- Enabled by default, only runs after tool execution (not plain chat).
- Trigger = at least one "trigger tool" (`edit_file`, `bash`) completed successfully.
- Additional gates: workspace must be a git repo, `git diff` must be non-empty.
- Opt-out: `LOOM_AUTO_COMMIT_DISABLE=true|1|yes`.

**Commit message generation**:

- Uses small, dedicated model (`claude-3-haiku`) with `max_tokens=256`, `temperature=0.3`.
- Prompt enforces conventional commit format (`<type>(<scope>): <desc>`).
- Diff truncated at 32KB (UTF-8 boundary, newline preferred) with `[TRUNCATED: ...]` notice.
- Fail-open fallback: if LLM fails, uses `chore: auto-commit from loom`.

**State machine integration**:

1. Tools finish executing.
2. If any mutating tool succeeded → transition to `PostToolsHook`, emit `RunPostToolsHook`.
3. Orchestrator runs auto-commit, sends `PostToolsHookCompleted`.
4. Agent transitions back to `CallingLlm` with *already-prepared* `pending_llm_request`.

Key insight: auto-commit doesn't alter LLM conversation history—the next request was built at tool
completion time, before the hook runs.

Why this matters:

- Automatic feedback loops (validation, commits) should live in runner, not be "optional behavior"
  inside prompts.
- Using a smaller/cheaper model for mechanical tasks (commit messages) keeps main agent context clean.

Actionable adaptation:

- Treat "run guards" as a post-execute hook owned by the runner.
- Treat "commit policy" similarly (enforcement in runner, not dependent on prompt compliance).
- Consider dedicated, smaller model for mechanical generation tasks (diff summaries, etc.).

## 4) Tool systems need: schema + sandbox boundary + timeouts/output limits

Loom's tool execution model: LLM emits tool calls → registry dispatches → tool runs with
`ToolContext` containing `workspace_root` → returns JSON output.

**Workspace boundary enforcement** (filesystem tools):

- Resolve requested path (join with `workspace_root` if relative).
- Canonicalize (resolve symlinks, `..`, etc.).
- Require `canonical_path.starts_with(canonicalized_workspace_root)`.
- Tests explicitly cover attempts like `/etc/passwd` or `../../../etc/passwd`.

`edit_file` special case: for non-existent targets (file creation), ensures parent directory (if it
exists) canonicalizes inside workspace—"best effort" vs a real OS sandbox.

**Bash tool** (`sh -c <command>`):

- `cwd` must canonicalize under `workspace_root`.
- Timeout: default 60s, max 300s.
- Output truncation: 256KB per stream.
- **NOT a sandbox**: can do anything user account can do (network, secrets, etc.); boundary is
  enforced on `cwd`, not on what the command references.

**Size limits**:

- `read_file`: 1MB max (DEFAULT_MAX_BYTES) with truncation.
- `bash`: output truncation at 256KB.

**Two execution planes**:

1. Local CLI: trust-based, workspace boundary via path validation + timeouts + truncation.
2. Weavers (Kubernetes): actual sandbox—ephemeral pods, non-root execution,
   `allowPrivilegeEscalation: false`, all capabilities dropped, TTL cleanup (4h default, 48h max).

Weaver hardening gap: spec says `readOnlyRootFilesystem: true`, implementation currently uses
`false`—meaningful drift between spec and code.

Actionable adaptation:

- In MVP, rely on single explicit guard entrypoint (`just ci`) for feedback.
- As tools expand, encode: strict input schemas, allow/deny boundaries, timeouts/output limits,
  typed errors for runner classification.
- Document which safety guarantees are "best effort" vs "enforced" (like local bash vs weaver
  sandboxing).

## 5) Typed errors + bounded retries beat "keep trying forever"

Loom has two distinct retry systems sharing the same philosophy: classify errors, retry with backoff,
hard-cap attempts.

**Error classification**:

- Top-level: `AgentError` with categories `Llm`, `Tool`, `Io`, `Timeout`.
- LLM-specific: `LlmError` variants `Timeout`, `RateLimited`, `InvalidResponse`.
- HTTP retryability via `RetryableError` trait: timeouts, connect errors, certain 4xx/5xx.

**HTTP retry wrapper**:

- Exponential backoff: `base_delay * backoff_factor^attempt`, capped at `max_delay`.
- Optional jitter: delay × random factor in [0.5, 1.5].
- Default max attempts: 3 (enforced via `RetryConfig.max_attempts`).
- Config validation: `retry.max_attempts` must be 1..=20.

**Agent-level LLM retry**:

- On LLM error in `CallingLlm`: increment retries, transition to `Error { origin: ErrorOrigin::Llm }`.
- Wait for external `RetryTimeoutFired` event (backoff timer managed by caller).
- On timer: reconstruct request, return to `CallingLlm` with preserved retry count.
- Default `max_retries`: 3 (so 3 total attempts; 3rd failure is terminal).

**What happens when retries exhaust**:

- HTTP wrapper: returns last error after logging `"max retry attempts exhausted"`.
- Agent LLM retry: gives up, transitions to `WaitingForUserInput`, emits `DisplayError(...)`.
- User can continue/retry manually—graceful degradation, not crash.

Actionable adaptation:

- Represent "attempt count" and "retry exhaustion strategy" as runner state, not prompt convention.
- Classify failures (guard failure vs executor failure vs schema corruption) deterministically.
- On exhaustion: transition to a recoverable state (user intervention, rewrite/expand), not crash.

## 6) Quality gates as a single entrypoint

Loom’s `Makefile` provides a single `make check` entrypoint (format + lint + build + test).

Why this matters for our pivot:

- A single guard entrypoint keeps the runner deterministic and makes “passes=true” verifiable.
- A single entrypoint is also “agent-editable”: agents can update the guard command implementation
  if the repo changes.

Actionable adaptation:

- Standardize on one guard entrypoint (you suggested `just ci`) and make it required for passing.

## 7) Scope caution: Loom is a product; our runner MVP should stay small

Loom's full scope (~50 crates):

- **Core**: agent state machine, tool registry, conversation management.
- **CLI**: REPL, thread commands, weaver commands, auto-commit, ACP editor integration.
- **Server**: HTTP API, SQLite persistence, LLM proxying, auth (GitHub/Google/Okta/Magic Link/Device
  Code), SCIM, feature flags, analytics, job queuing.
- **LLM**: proxy architecture + provider crates (Anthropic, OpenAI, Vertex).
- **Weavers**: Kubernetes orchestration, WireGuard tunneling, eBPF syscall auditing (planned).
- **Web**: SvelteKit UI with its own XState machine mirroring backend states.

Takeaway:

- Loom demonstrates *patterns* (determinism, schemas, hooks, verification), but we should avoid
  importing its full product surface area during MVP.
- The core agent loop (`loom-common-core`) is the valuable reference; everything else is platform.

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

- Keep the runner's persisted tree as a strict, documented contract.
- Consider an eventual read-only mode for consuming the tree + iteration logs (CLI JSON output,
  simple local HTTP, etc.) without embedding UI assumptions into the core loop.

## 10) Thread persistence: single JSON document + versioning

Loom persists conversation threads as single JSON documents containing:

- `conversation`: array of message snapshots
- `agent_state`: state-machine snapshot (kind, retries, pending tool calls, last error)
- Metadata: title, tags, pinned, git/workspace metadata, timestamps
- Monotonic `version` counter for optimistic concurrency

**Local persistence** (CLI):

- One file per thread: `{threadId}.json` under `$XDG_DATA_HOME/loom/threads`.
- State restoration = load thread JSON, rehydrate conversation + agent state snapshot.

**Server persistence** (SQLite):

- Denormalized columns (title, message_count, last_activity_at) + full JSON blob (`full_json`).
- FTS indexes for search.
- `Thread::touch()` updates timestamps and increments `version` on every change.

**Thread syncing**:

- CLI uses `SyncingThreadStore` wrapper: save locally first, sync to server in background.
- Private sessions (`thread.is_private = true`) never sync.
- Sync = HTTP `PUT /api/threads/{id}` with full document.
- Optimistic concurrency: `If-Match: <version>` header, 409 on mismatch.
- Pending sync queue: failed upserts/deletes persisted to `$XDG_STATE_HOME/loom/sync/pending.json`.

Actionable adaptation:

- Consider single-document persistence with monotonic versioning for tree state.
- `version` enables optimistic concurrency and conflict detection.
- Private/local-only mode is valuable for development/testing.

## 11) Secret handling: redacted types + exposure control

Loom uses `Secret<T>` / `SecretString` wrappers with redacted `Debug`/`Display`/`Serialize`
implementations:

- Secrets can't accidentally appear in logs or serialized output.
- Explicit `.expose()` required to access inner value.
- Verified in `verification.md` (drift would be caught).

Server query layer (`query_security.rs`) has additional protections:

- Query size limits, per-session limits, timeout ranges.
- Path rules: blocked prefixes (`/etc`, `/root`, `/sys`).
- `PathSanitizer`: rejects absolute paths, null bytes, enforces canonicalized workspace containment.

Weaver secrets (planned design):

- No env-var injection (too easy to leak).
- Short-lived identity tokens + ABAC policies instead.

Actionable adaptation:

- If handling API keys or credentials, use wrapper types with redacted serialization.
- Make exposure explicit (no accidental `.to_string()` leaks).

## 12) Invalid transitions: log and continue, don't crash

Loom's state machine has a safety net for unhandled `(state, event)` combinations:

- Logs a warning (`"invalid state transition"`).
- Returns `AgentAction::WaitForInput`.
- State remains unchanged.

This prevents crashes from unexpected event sequences (e.g., tool progress events that aren't wired
into transitions yet) while making the issue visible in logs.

Actionable adaptation:

- Explicit catch-all for invalid transitions: log, don't crash.
- Make invalid transitions visible for debugging without halting the system.
- Useful during development when event handling isn't fully wired up.
