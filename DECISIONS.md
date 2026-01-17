# Decisions: Deterministic Goal-Driven Agent Loop Runner

This file is a dated decision log (what we decided and why).

- Principles: `VISION.md`
- Canonical technical reference: `ARCHITECTURE.md`

When adding a decision, record a dated section with status, decision, rationale, consequences, and references.

---

## 2026-01-16 — Non-negotiable invariants

- Status: accepted
- Decision: the runner enforces these invariants across the MVP.
- Rationale: determinism and safety require the runner (not the agent) to own validation and state
  transitions.
- Consequences: implementation must bias toward explicit, stable rules and strict state validation.
- References: `VISION.md`, `ARCHITECTURE.md`

Invariants:

- Determinism over convenience (stable ordering, explicit tie-breaks, stable serialization).
- Strict schemas (no schema drift during a run).
- Tree is the source of truth (supporting docs are context, not progress state).
- Fresh context per iteration (continuity via repo artifacts + git history).
- Sequential only (one actionable leaf at a time).
- No pass without green guards (runner-owned `passes=true`).
- Immutability (passed nodes are immutable; follow-up work becomes new nodes).
- Always proceed (record assumptions and open questions instead of blocking).

## 2026-01-16 — Runner language: Rust

- Status: accepted
- Decision: implement the MVP runner in Rust.
- Rationale: strong fit for a deterministic CLI with strict types, schema validation, and safe file I/O.
- Consequences: core components, adapters, and tests target a Rust workspace.
- References: `ARCHITECTURE.md`

## 2026-01-16 — UI priority

- Status: accepted
- Decision: UI is explicitly secondary; focus on the core loop first.
- Rationale: correctness and determinism matter more than presentation during MVP.
- Consequences: any UI work is deferred until the runner loop and contracts are stable.
- References: `VISION.md`

## 2026-01-16 — Canonical state format: JSON-only

- Status: accepted
- Decision: the task tree is stored as JSON only, and the runner may canonicalize formatting on write.
- Rationale: strict machine state + deterministic serialization avoids “format wars” and schema drift.
- Consequences: schema validation + canonical JSON writing are foundational MVP capabilities.
- References: `ARCHITECTURE.md`

## 2026-01-16 — Deterministic selection semantics (“leftmost open leaf”)

- Status: accepted
- Decision: selection is deterministic depth-first traversal of siblings sorted by `(order ASC, id ASC)`.
- Rationale: makes “what happens next” predictable and testable without ad-hoc priority rules.
- Consequences: tree writes must canonicalize sibling ordering to prevent selection ambiguity.
- References: `ARCHITECTURE.md`

## 2026-01-16 — Implicit mode classification (no `mode` field)

- Status: accepted
- Decision: do not add a `mode` field to the node schema; classify iterations implicitly.
- Rationale: keep the schema minimal and avoid embedding runner orchestration concerns into node records.
- Consequences: runner determines DECOMPOSE vs EXECUTE based on deterministic rules (e.g., diff of changed
  paths).
- References: `ARCHITECTURE.md`

## 2026-01-16 — Workspace + git policy

- Status: accepted
- Decision:
  - workspace isolation is in-place (no worktrees/temp dirs for code).
  - refuse to run `runner step` on `main` / `master` (but allow `runner start` to branch off immediately).
  - default branch naming is `runner/<run-id>`.
  - require a clean working tree (including untracked) at iteration start.
  - commit every iteration (including failures) with deterministic Conventional Commit messages.
- Rationale: keeping the worktree clean and historyful enables automation-first looping without manual
  cleanup or destructive resets.
- Consequences: git policy is part of the runner contract and must be enforced before/after each
  iteration.
- References: `ARCHITECTURE.md`

## 2026-01-16 — Iteration logs are local-only

- Status: accepted
- Decision: keep `.runner/iterations/` as a local-only directory and gitignore it.
- Rationale: logs are essential for debugging, but should not pollute the repo history or break
  cleanliness invariants.
- Consequences: runner writes logs there, and repo tooling must ignore it by default.
- References: `ARCHITECTURE.md`

## 2026-01-16 — Per-iteration timeout

- Status: accepted
- Decision: default 30-minute wall-clock timeout per iteration covering agent + guards (configurable
  via `.runner/state/config.toml`); stop the whole run on timeout.
- Rationale: bounds runaway tool output and prevents non-terminating loops.
- Consequences: executor and guard runner share a single deterministic budget.
- References: `ARCHITECTURE.md`

## 2026-01-16 — Tree editing permission (for open nodes)

- Status: accepted
- Decision: agents may edit any open node(s), not only the selected leaf; passed nodes remain immutable.
- Rationale: planning often requires reshaping adjacent open work while preserving history of completed
  work.
- Consequences: invariants must distinguish “open and editable” from “passed and immutable”.
- References: `ARCHITECTURE.md`

## 2026-01-17 — Runner-owned state transition semantics

- Status: accepted
- Decision:
  - Increment `attempts` on `done` + guards fail, or on `retry` (guards skipped).
  - `attempts` only increments from successful agent outputs via `apply_state_updates()`;
    runner-internal failures (timeouts, parse errors, guard errors, git failures) never increment.
  - Attempt exhaustion saturates at `max_attempts` (no hard error in MVP).
  - Runner overwrites runner-owned fields (`passes`, `attempts`) deterministically from the previous
    tree before applying updates (agent edits are ignored).
- Rationale: keeps state updates deterministic, preserves "no pass without green guards," and avoids
  introducing new policy branching during MVP.
- Consequences: runner will ignore agent edits to runner-owned fields for open nodes, and retries
  continue without raising a hard error when max attempts is reached.
- References: `ARCHITECTURE.md`

## 2026-01-17 — Agent-declared status model

- Status: accepted
- Decision:
  - Agents must output a JSON file with `status` (`done`/`retry`/`decomposed`) and `summary` fields.
  - Guards only run when `status == done`.
  - A node passes only when agent declares `done` AND guards pass.
  - `retry` skips guards, increments attempts, persists summary for next iteration.
  - `decomposed` validates tree gained children, accepts changes, progresses to next node.
  - `stuck` is not an agent status; runner determines stuck when `attempts == max_attempts`.
- Rationale: guards verify syntactic correctness (tests pass, lint clean) but cannot assess semantic
  completeness. Agents know when work is partial. Combining both signals prevents false positives.
- Consequences:
  - Agent output format is a contract; executor must enforce structured output.
  - Runner must validate decompose invariants (children added) and done/retry invariants (no children added).
  - Guard cycles saved when agent knows work is incomplete.
- References: `ARCHITECTURE.md`

## 2026-01-17 — Directory structure split (state / context / iterations)

- Status: accepted
- Decision:
  - `.runner/state/` — runner-owned canonical state (tree.json, schema.json, config, assumptions, questions).
  - `.runner/context/` — ephemeral, cleared and rewritten each iteration by runner for agent consumption.
  - `.runner/iterations/` — append-only immutable log of all iterations.
- Rationale: separates concerns; agents read context but don't manage it; runner controls what context
  agents see; iterations provide audit trail without polluting working state.
- Consequences:
  - Runner clears `context/` at iteration start, writes goal/history/failure info.
  - Agent reads `context/` and `state/tree.json` (read-only for state semantics).
  - Agent may append to `state/assumptions.md` and `state/questions.md`.
  - Agent writes `iterations/{id}/output.json` with status + summary.
  - Machine state is JSON; human notes (context/assumptions/questions) are Markdown.
- References: `ARCHITECTURE.md`
