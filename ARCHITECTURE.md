# Architecture: Deterministic Goal-Driven Agent Loop Runner (MVP)

This document is the canonical technical reference for the **deterministic local runner**.

- Vision principles: `VISION.md`
- Decision log (dated rationale): `DECISIONS.md`
- Project-level goal/spec provided to each iteration: `.runner/GOAL.md`

## 1) Goals & Non-Negotiable Invariants

The runner exists to make progress **deterministically** while agents remain flexible.

Non-negotiables (source: `VISION.md` + decisions captured in `DECISIONS.md`):

- **Determinism over convenience**
  - Given the same repo state and inputs, the runner makes the same choices.
  - Any tie-breaks are explicit and stable.
- **Strict schemas**
  - The task tree is machine-facing and strictly validated.
  - No schema drift during a run.
- **Tree is the source of truth**
  - Progress state lives only in the tree.
  - Markdown “memory” docs are supporting context only.
- **Fresh context per iteration**
  - Every iteration is a new agent session.
  - Continuity is provided by committed repo artifacts + iteration logs.
- **Sequential only**
  - One actionable leaf at a time. No parallel workflows.
- **No pass without green guards**
  - `passes=true` can only be recorded by the runner after the guard entrypoint succeeds.
- **Immutability**
  - Any node with `passes=true` is immutable: record and structural placement.
  - Improvements/refactors happen via **new nodes**, not edits to passed nodes.

## 2) System Boundary (Runner vs Agent)

The runner is the deterministic orchestrator; the agent is a tool invoked by the runner.

Runner responsibilities:

- Enforce git safety (branch + cleanliness).
- Load/validate/update/persist the task tree deterministically.
- Decide which leaf to work on next (deterministic selection).
- Build and pass a stable prompt pack to the executor.
- Run guards (`just ci`) and record results.
- Apply runner-owned state transitions (`passes`, `attempts`, derived internal passes).
- Write iteration logs and commit each iteration with a deterministic message.

Agent responsibilities:

- Edit code and/or open nodes to satisfy goals.
- Record uncertainty in `.runner/ASSUMPTIONS.md` and open questions in `.runner/HUMAN_QUESTIONS.md`.
- Keep `.runner/tree.json` strictly valid when editing it.
- Never mutate passed nodes; never set `passes=true` (runner-owned).

## 3) Canonical Artifacts (File/Dir Contracts)

### 3.1 Source of truth

- `.runner/tree.json`: canonical task tree (strict JSON, canonicalized formatting on write).
- `schemas/task_tree/v1.schema.json`: versioned schema for `.runner/tree.json`.

### 3.2 Memory / spec (provided to each iteration; not source of truth)

- `.runner/GOAL.md`
- `.runner/HUMAN_QUESTIONS.md`
- `.runner/ASSUMPTIONS.md`
- `.runner/FEEDBACK_LOG.md`
- `.runner/IMPROVEMENTS.md`

### 3.3 Iteration logs (local-only; gitignored)

- `.runner/iterations/<run-id>/<iter-n>/...`

The runner must ensure `.runner/iterations/` is gitignored so future iterations can still start from a
clean working tree.

### 3.4 Extensibility sidecars (no schema drift)

Future features attach to nodes via deterministic sidecars keyed by node id (not by adding schema fields):

- `.runner/nodes/<node-id>/context.md`
- `.runner/nodes/<node-id>/tools.json`
- `.runner/nodes/<node-id>/artifacts/...`

## 4) Task Tree Data Model (v1)

Canonical format: **JSON-only**.

### 4.1 Node fields (conceptual)

Every node:

- `id: string` (globally unique)
- `order: integer` (per-sibling ordering key)
- `title: string`
- `goal: string` (declarative)
- `acceptance: string[]` (human-readable bullets)
- `passes: boolean` (**runner-owned**)
- `attempts: integer` (**runner-owned**)
- `max_attempts: integer`
- `children: Node[]`

### 4.2 Ordering and deterministic selection

Sibling order is defined as sorting by `(order ASC, id ASC)` and then traversing depth-first to pick the
**leftmost open leaf**.

The runner canonicalizes the on-disk tree by sorting `children` arrays accordingly when writing.

### 4.3 Passing semantics

- Leaf nodes pass only when:
  - the iteration is classified as **EXECUTE**, and
  - guards succeed (`just ci` exit code 0), and
  - tree invariants pass.
- Non-leaf nodes pass when **all children pass** (derived by the runner).

### 4.4 Immutability enforcement

If a node has `passes=true` in the previous committed tree, then in the next tree it must:

- exist with the same `id`
- be byte-for-byte identical in canonical form (all fields)
- appear in the same structural position (same parent + sibling ordering keys)

## 5) Core Loop as a Deterministic State Machine

Each `step` is one deterministic iteration:

1. Ensure repo safety:
   - refuse `main`/`master`
   - ensure clean working tree (including untracked)
2. Load + validate `.runner/tree.json` against schema + invariants.
3. Deterministically select next open leaf.
4. Build a stable prompt pack.
5. Invoke the executor (fresh agent session) with a 30-minute wall clock budget shared with guards.
6. Re-load + validate `.runner/tree.json` (including passed-node immutability).
7. Classify the iteration:
   - **EXECUTE** if any non-`.runner/` file changed
   - **DECOMPOSE** if only `.runner/` files changed
8. If EXECUTE: run `just ci` and capture stdout/stderr + timing.
9. Apply deterministic state updates:
   - `passes=true` only on green guards (runner-owned)
   - increment attempts on failure (runner-owned)
   - derive internal node passes
10. Persist:
    - write tree atomically (canonical form)
    - write iteration logs
    - commit the iteration
11. Stop when `root.passes == true` (no open leaves remain).

### 5.1 Deterministic recovery: invalid tree

If the tree is invalid after an agent session, the runner enters a special **REPAIR TREE** step that:

- does not select a normal leaf
- prompts the agent to restore schema validity
- commits the repair attempt (keeping the loop automation-first)

## 6) Component Architecture (MVP)

Keep deterministic decision-making separate from I/O. This makes behavior testable and reduces
non-deterministic failure modes.

### 6.1 “Core” (pure-ish logic; deterministic)

- **Selector**
  - Computes the next open leaf id/path (leftmost open leaf).
- **Classifier**
  - Determines DECOMPOSE vs EXECUTE from a list of changed paths.
- **InvariantChecker**
  - Schema validation (strict) + invariants (immutability, id uniqueness, ordering, etc.).
- **StateUpdater**
  - Applies runner-owned updates (`passes`, `attempts`, derived internal passes).

### 6.2 “Adapters” (I/O boundary; observable side effects)

- **TreeStore**
  - Loads tree JSON.
  - Canonicalizes and atomically writes tree JSON.
- **PromptBuilder**
  - Produces a stable prompt pack (stable ordering and bounded size).
- **Executor**
  - Stable interface; backends: Codex CLI, Claude CLI.
- **GuardRunner**
  - Runs `just ci` with timeouts/output caps.
- **GitManager**
  - Branch policy, clean checks, diffs for classification, deterministic commits.
- **IterationLogger**
  - Writes local-only `.runner/iterations/...` logs and metadata.

## 7) Determinism Rules (Implementation Guardrails)

Determinism is enforced by design constraints plus implementation guardrails:

- Stable sorting everywhere:
  - nodes: `(order ASC, id ASC)`
  - file lists: lexicographic
- Canonical JSON:
  - stable key order (struct field order, not hash map iteration)
  - canonical whitespace/indentation
  - children arrays sorted on write
- No implicit “now” in decisions:
  - timestamps are allowed in logs, but must not affect selection or updates
  - run-id should be an explicit input (or generated once at run start and treated as an input)
- Output caps + timeouts:
  - limit stdout/stderr capture sizes
  - enforce the shared 30-minute iteration budget deterministically

## 8) Executor & Prompt Pack

Executor requirements:

- “Fresh agent session” per iteration.
- Replaceable backends behind a stable interface.
- Timeouts/output caps applied by the runner.

Prompt pack (stable order, minimal but sufficient):

1. Runner contract / invariants (especially: immutability, runner-owned fields).
2. `.runner/GOAL.md`.
3. Selected leaf path + selected leaf subtree (full).
4. Deterministic summary of the rest of the tree (bounded).
5. `.runner/ASSUMPTIONS.md`, `.runner/HUMAN_QUESTIONS.md`, `.runner/FEEDBACK_LOG.md`, `.runner/IMPROVEMENTS.md`.
6. Guard contract (“runner will run `just ci` after EXECUTE”).

## 9) Git Policy (MVP)

Locked MVP policy (per `DECISIONS.md`):

- Refuse to run on `main`/`master`.
- Require clean working tree at iteration start (`git status --porcelain` empty, including untracked).
- Commit every iteration (including failures).
- Commit messages are deterministic and use Conventional Commits:
  - `chore(loop): run <run-id> iter <iter-n> node <node-id> <decompose|execute> guard=<pass|fail|skipped>`

## 10) Observability (Iteration Logs)

Local-only logs live under:

`.runner/iterations/<run-id>/<iter-n>/`

Minimum recommended contents:

- `meta.json`
- `executor.log`
- `guard.log` (EXECUTE only)
- `tree.before.json` / `tree.after.json` (canonical snapshots)

Optional:

- `diff.patch`

## 11) Open Decisions (Need Human Input)

These are not settled yet and materially affect implementation details:

- Defaults:
  - global max iterations
  - default `max_attempts` for new leaves
  - output caps for executor/guards
- Enforcement details for runner-owned fields:
  - hard reject if agent edits `passes`/`attempts` vs overwrite deterministically
- Attempt exhaustion policy:
  - what counts as “rewrite” vs “expand” deterministically
- Whether to emit machine-readable events (ex: `events.jsonl`) in MVP or defer.
