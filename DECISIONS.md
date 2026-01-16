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
  - refuse to run on `main` / `master`.
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
- Decision: 30-minute wall-clock timeout per iteration covering agent + guards; stop the whole run on
  timeout.
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

## 2026-01-16 — Open questions

- Status: proposed
- Decision: these are intentionally left open for follow-up.
- Rationale: MVP needs a working loop, but some defaults and enforcement details can be deferred.
- Consequences: capture resolutions here as new dated decisions once settled.
- References: `.runner/HUMAN_QUESTIONS.md` (target project), `ARCHITECTURE.md`

Open questions (as of 2026-01-16):

- Default values: global max iterations, `max_attempts`, executor/guard output caps.
- Runner-owned fields enforcement: hard reject agent edits vs overwrite deterministically.
- Attempt exhaustion policy: define “rewrite” vs “expand” in a deterministic way.
- Whether to emit machine-readable events (e.g., `events.jsonl`) in MVP or defer.
