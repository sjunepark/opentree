# Brainstorm: Deterministic Goal-Driven Agent Loop Runner (MVP)

This document captures the architecture and product decisions we made while bootstrapping this repo into a
**deterministic local runner** that orchestrates **fresh agent sessions** to build and execute a **strict,
goal-oriented task tree** (no parallel workflows).

Repo layout note: `VISION.md` is the canonical project-level vision doc at the repo root. This file is an
explicit, human-approved exception requested during the bootstrap discussion.

Date of decisions: 2026-01-16.

---

## 1) What We’re Building (In One Sentence)

A deterministic CLI runner that repeatedly:

1. Loads + validates a strict JSON task tree (source of truth).
2. Deterministically selects the **single next open leaf**.
3. Spawns a **fresh agent session** (Codex or Claude) to either plan (tree edits) or implement (code edits).
4. Runs a single deterministic guard entrypoint (`just ci`) after implementation.
5. Updates the tree deterministically (`passes=true` only on green guards), logs everything, and commits the
   iteration so the next iteration starts clean.

---

## 2) Non-Negotiable Constraints / Invariants

From `VISION.md` and agreed decisions:

- **Determinism over convenience**
  - The runner must make the same choices given the same repo state.
  - Stable ordering, explicit tie-breaks, stable serialization.
- **Strict schemas**
  - The task tree JSON is machine-facing and strictly validated every iteration.
  - No schema drift; no “add a field mid-run”.
- **Tree is the source of truth**
  - `.runner/*.md` are memory/spec aids only; progress state lives only in the tree.
- **Fresh context per iteration**
  - Every iteration is a new agent session; continuity via repo artifacts + git history.
- **Sequential only**
  - One actionable leaf at a time; no parallel execution / multi-agent concurrency.
- **No pass without green guards**
  - `passes=true` can only be recorded by the runner after `just ci` succeeds.
- **Immutability**
  - Any node with `passes=true` is immutable (record + structural placement).
  - Improvements/refactors happen via **new nodes**, not edits to passed nodes.
- **Always proceed**
  - Agents proceed with assumptions, record assumptions in `.runner/ASSUMPTIONS.md`,
    and open decisions in `.runner/HUMAN_QUESTIONS.md`.

---

## 3) Locked Decisions (2026-01-16)

These decisions are “locked” for the MVP unless explicitly revised.

### 3.1 Language / Runtime

- MVP runner language: **Rust**.
- UI is explicitly secondary; future Tauri UI is a plausible direction.

### 3.2 Canonical State Format

- Canonical tree format: **JSON-only**.
- Runner is allowed to **canonicalize formatting on write** (and should do so deterministically).

### 3.3 Ordering / Selection Semantics

- `order` is **per-sibling** (not global priority).
- Selection is deterministic “leftmost open leaf” using a stable traversal with explicit sorting:
  - Sort siblings by `(order ASC, id ASC)` and traverse depth-first.
- Tie-break: `id` lexical ordering.

### 3.4 Mode Control (Implicit)

- There is **no explicit `mode` field** in the node schema.
- Runner decides “DECOMPOSE vs EXECUTE” implicitly (see Section 7).

### 3.5 Workspace + Git Policy

- Workspace isolation: **in-place** (no worktrees/temp dirs for code).
- Runner refuses to run on `main` / `master`.
- Runner defaults to a dedicated branch: `runner/<run-id>`.
- Cleanliness definition: `git status --porcelain` must be **empty including untracked** at iteration start.
- Commit policy: **commit every iteration**, regardless of guard success.
  - Commits are history only; they do **not** represent `passes=true`.
  - Tree remains the sole progress truth.

### 3.6 Logs

- `.runner/iterations/` should exist and be **gitignored** (persist locally for debugging).

### 3.7 Timeouts / Limits

- Per-iteration wall-clock timeout: **30 minutes**, covering **agent + guards combined**.
- If an iteration hits the timeout: stop the whole process (do not continue).
- Additional loop limits (max iterations, max output) still required, but values are TBD.

### 3.8 Tree Editing Permission

- Agents may edit **any open node(s)** (future plan), not just the selected leaf subtree.
- Passed nodes remain immutable.

---

## 4) Canonical Artifacts / Directory Contract

### 4.1 Source-of-truth state

- `.runner/tree.json`: canonical, strict machine state.
- `schemas/task_tree/v1.schema.json`: the versioned schema that defines the allowed shape.

### 4.2 Run context (memory/spec)

These Markdown files are provided to each iteration (fresh session) and are not the source of truth:

- `.runner/GOAL.md`: thorough human-facing goal/spec.
- `.runner/HUMAN_QUESTIONS.md`: open product/architecture questions.
- `.runner/ASSUMPTIONS.md`: assumptions made to proceed.
- `.runner/FEEDBACK_LOG.md`: mistakes/failure modes/lessons.
- `.runner/IMPROVEMENTS.md`: “tools/prompting/approach would have helped”.

### 4.3 Iteration logs (local-only)

- `.runner/iterations/<run-id>/<iter-n>/...` (gitignored)

Implementation note: add `.runner/iterations/` to `.gitignore` in the MVP.

---

## 5) Architecture Overview

### 5.1 High-level components (MVP)

- **TreeStore**
  - Load tree JSON.
  - Validate against schema + invariants.
  - Canonicalize + atomic write.
- **Selector**
  - Deterministically selects the next open leaf.
- **PromptBuilder**
  - Builds a stable prompt pack for the executor (fresh session).
- **Executor**
  - Stable interface; backends: Codex CLI, Claude CLI.
- **GuardRunner**
  - Runs `just ci`, captures outputs + timing.
- **StateUpdater**
  - Applies deterministic updates to the tree (attempts, passes, derived statuses).
- **GitManager**
  - Enforces branch + cleanliness policies.
  - Commits each iteration with deterministic commit message.
- **IterationLogger**
  - Writes per-iteration logs + metadata.

### 5.2 Core loop (state machine view)

Deterministic runner “step”:

1. Ensure on safe branch (`runner/<run-id>`, not `main`/`master`).
2. Ensure clean working tree (incl. untracked).
3. Load `tree.json` → validate schema/invariants.
4. Select next open leaf deterministically.
5. Build prompt pack and run executor (fresh session) with a 30m cap (shared with guards).
6. Re-load/validate tree (and invariants, including immutability).
7. Classify iteration as DECOMPOSE/EXECUTE (implicit).
8. If EXECUTE: run `just ci`; capture outputs.
9. Update tree deterministically:
   - `passes=true` only when guards pass (runner-owned).
   - Attempts increment on failure (runner-owned).
   - Optionally derive internal node pass state (see Section 6.6).
10. Persist:
    - write tree atomically (canonical),
    - write iteration logs,
    - commit iteration.
11. Stop when no open nodes remain.

---

## 6) Task Tree (v1) – Data Model Draft

### 6.1 Design goals

- Goal-oriented, declarative nodes (not imperative “click this / run that”).
- Strict + minimal schema (no extension fields).
- Supports:
  - deterministic selection
  - pass/fail gating with guards
  - attempt budgets
  - immutability of passed nodes

### 6.2 Proposed file location

- Canonical path: `.runner/tree.json`

### 6.3 Proposed top-level shape

We want a true tree (nested children) because selection is per-sibling ordered and traversal-based:

```json
{
  "version": 1,
  "root": {
    "id": "root",
    "order": 0,
    "title": "Root",
    "goal": "Satisfy .runner/GOAL.md",
    "acceptance": [],
    "passes": false,
    "attempts": 0,
    "max_attempts": 1,
    "children": []
  }
}
```

Notes:

- `goal_path` can be implicit (`.runner/GOAL.md`) to avoid schema bloat.
- Root is a coordination node and will usually decompose into children immediately.

### 6.4 Node fields (draft)

Required for every node:

- `id: string`
  - globally unique in the entire tree.
  - does **not** need to be human-readable (ULID/UUID/etc acceptable).
- `order: integer`
  - per-sibling ordering key.
- `title: string`
  - short label.
- `goal: string`
  - declarative goal statement.
- `acceptance: string[]`
  - acceptance criteria in human-readable bullets (can be empty).
- `passes: boolean`
  - runner-owned “this node is done and immutable” flag.
- `attempts: integer`
  - runner-owned attempt counter for leaf execution failures.
- `max_attempts: integer`
  - budget for a leaf before deterministic “rewrite/expand” policy triggers.
  - exact default value is TBD; but must be stable/configurable.
- `children: Node[]`
  - ordered children (may be empty).

### 6.5 Canonical ordering and determinism

- The runner treats sibling order as:
  - primary: `order ASC`
  - tie-break: `id ASC` (string lexicographic)
- The runner should canonicalize the on-disk tree by sorting `children` arrays accordingly when writing.

### 6.6 Passing semantics (proposed)

We need a deterministic completion condition without requiring agents to “manually pass” internal goals.

Proposed rule:

- **Leaf nodes** pass only when:
  - iteration is classified as EXECUTE, and
  - `just ci` succeeds (exit code 0), and
  - tree invariants pass.
- **Non-leaf nodes** pass when **all children pass**.
  - Runner can derive this deterministically after every update.

Completion condition:

- Run completes when `root.passes == true` (equivalently, no open leaves remain).

If we reject derived internal pass state, we must keep internal nodes open forever, which is undesirable.
So “derived internal passes” is recommended unless it conflicts with later UX needs.

### 6.7 Immutability enforcement (critical)

Rule:

- If a node has `passes=true` in the prior committed tree, then in the next tree:
  - the node must exist with the same `id`,
  - the entire node record must be identical (all fields),
  - its placement in the tree must be identical (same parent, same sibling ordering keys).

Implementation approach (deterministic):

- Load previous tree as canonical form.
- Load new tree as canonical form.
- For each `id` where `prev.passes=true`:
  - compare canonical serialized node (or a stable hash) for exact equality.
  - compare parent `id` and sibling set membership.

### 6.8 Runner-owned vs agent-editable fields

We want agents to be able to edit open nodes freely, but not to “cheat” passing.

Proposed policy:

- Agents may edit any fields of nodes where `passes=false`, including `children`.
- Agents must not set `passes=true`.
  - Runner will treat any agent-changed `passes` as invalid unless set by runner after green guards.
- Runner owns updates to `attempts` and any derived pass propagation.

(Exact enforcement choice is a product detail: “hard reject if agent changes runner-owned fields” is
preferred for strictness.)

---

## 7) Implicit Mode: DECOMPOSE vs EXECUTE

We intentionally do not add a `mode` field to the schema.

### 7.1 Deterministic classification rule (proposal)

After the agent session ends, the runner classifies the iteration as:

- **EXECUTE** if any non-`.runner/` file changed (code/work changes).
- **DECOMPOSE** if only `.runner/` files changed (tree/memory updates only).

Why this rule:

- Simple and deterministic (based on git diff).
- Matches the intent: DECOMPOSE is planning-only; EXECUTE is implementation.
- Still allows the agent to edit future open nodes during EXECUTE (tree edits are allowed).

### 7.2 What happens in each classification

- DECOMPOSE:
  - No guards run.
  - Tree changes are validated and committed.
  - Selected leaf remains open unless the runner can deterministically derive that it’s “done”
    (generally not possible without execution + guards, so typically it stays open).
- EXECUTE:
  - Runner runs `just ci` and captures stdout/stderr + duration.
  - If guards pass, runner marks selected leaf `passes=true` (and derives internal passes).
  - If guards fail, runner increments attempt counters deterministically.

---

## 8) Executor Integration (Codex + Claude)

### 8.1 Requirements

- “Fresh agent session” per iteration.
- Both Codex and Claude supported behind a stable interface.
- Executor should be replaceable without touching the core loop/state machine.

### 8.2 Proposed runner interface

Conceptual Rust trait:

- `Executor::run(ctx, prompt_pack) -> ExecutorResult`
  - `ctx`: repo root, timeouts, output caps, executor kind/config.
  - `prompt_pack`: stable set of files/strings to provide.
  - `result`: exit status, stdout/stderr capture pointers, duration.

### 8.3 CLI backends (showing intent, not final flags)

Codex CLI (example pattern):

- `codex exec --sandbox danger-full-access -`
  - prompt text via stdin.

Claude CLI (example pattern):

- `claude -p --permission-mode acceptEdits --settings <temp.json> <prompt>`
  - prompt as a CLI arg (legacy Ralph style) or via stdin if supported.

Important: flags will be pinned in code/config so behavior is stable.

### 8.4 Prompt pack contents (stable order)

Each iteration prompt should include, in stable order:

1. Runner contract / invariants (do not touch passed nodes, do not set passes manually, etc).
2. Goal (`.runner/GOAL.md`).
3. Current tree summary + the selected leaf (include full path from root).
4. Relevant `.runner/` memory docs (assumptions/questions/feedback).
5. The guard contract: “runner will run `just ci`”.
6. Instructions:
   - You may edit any open nodes.
   - Record assumptions/questions in the appropriate `.runner/*.md`.
   - Keep `.runner/tree.json` strictly valid.

To keep prompts small and deterministic, the runner should include:

- The full selected leaf subtree.
- A minimal, deterministic summary of the rest of the tree (or a bounded depth).

---

## 9) Guards / Validation Contract

### 9.1 Single entrypoint

- Guard entrypoint: **mandatory `just ci`**.

### 9.2 What the runner records

At minimum:

- exit code
- duration
- captured stdout/stderr (raw, with output caps)

### 9.3 Relationship to `passes=true`

- `passes=true` can only be set by the runner when guard succeeds.
- No “manual pass”.

---

## 10) Git Rules (Detailed)

### 10.1 Branch safety

- Refuse `main` / `master`.
- Default branch name: `runner/<run-id>` (run-id is unique).

### 10.2 Cleanliness

- At iteration start: require `git status --porcelain` empty (including untracked files).

### 10.3 Commit every iteration

- The runner commits at the end of each iteration regardless of guard outcome.
- Logs under `.runner/iterations/` are gitignored and not committed.
- Commit message should be deterministic and conform to Conventional Commits.

Proposed commit message format:

- Type: `chore`
- Scope: `loop`
- Description: `run <run-id> iter <iter-n> node <node-id> <decompose|execute> <guard=pass|fail|skipped>`

Example:

- `chore(loop): run 01J... iter 0007 node 01J... execute guard=fail`

### 10.4 Why we commit failures

- Keeps the invariant “clean tree at iteration start” without destructive resets.
- Preserves all intermediate states for debugging and learning.
- Ensures the loop can continue deterministically without manual cleanup.

---

## 11) Logging / Observability (MVP)

Goal: “simple, but debuggable”.

### 11.1 Directory structure (local-only)

`.runner/iterations/<run-id>/<iter-n>/`

Proposed files:

- `meta.json`:
  - run_id, iter_n
  - selected_leaf_id
  - selected_leaf_path (list of node ids from root)
  - classification: `decompose|execute`
  - executor_kind, exit_status, duration_ms
  - guard: `skipped|pass|fail`, exit_status, duration_ms
  - commit_sha
- `executor.log` (raw stdout/stderr combined)
- `guard.log` (raw stdout/stderr combined; present only for EXECUTE)
- `tree.before.json` and `tree.after.json` (canonical snapshots)

Optional (nice-to-have):

- `diff.patch` (for quick inspection without git commands)

### 11.2 Machine-readable events

We can defer `events.jsonl` until it materially helps a UI.
If we add it, it should be local-only and not fed to agents by default.

---

## 12) Limits / Budgets (MVP)

Locked:

- 30-minute per-iteration timeout (executor + guard combined); stop entire run on timeout.

Still required (values TBD):

- Global max iterations (hard stop to prevent infinite loops).
- Per-leaf max attempts (`max_attempts` default).
- Output caps (max bytes per log stream) to prevent runaway tool output.

Deterministic behavior on attempt exhaustion (proposal):

- If a selected leaf hits `attempts >= max_attempts`, the runner should:
  - refuse to run EXECUTE again for that leaf until the leaf is rewritten or decomposed, and
  - require the agent to either:
    - expand into children, and/or
    - rewrite acceptance/goal to be achievable.

Exact “how to detect rewrite” must be deterministic (e.g., require a structural change to the leaf node or
adding children).

---

## 13) Failure Handling / Recovery (Desired Behavior)

### 13.1 Guard failures

- Commit the iteration.
- Increment attempts deterministically.
- Next iteration selects next open leaf (likely the same one unless the tree was edited).

### 13.2 Executor failures (crash/non-zero)

- Treat as iteration failure.
- Commit whatever changed (still keeping tree valid if possible).
- Increment attempts for the selected leaf (runner-owned).

### 13.3 Tree invalid after agent edits

We should avoid “manual fix required” because the system is automation-first.

Proposed deterministic recovery mode:

- If `.runner/tree.json` is invalid after an iteration:
  - runner enters a special **REPAIR TREE** step that:
    - does not select a normal leaf,
    - prompts the agent to repair the tree back to schema validity,
    - preserves logs for the invalid attempt,
    - commits the repair attempt.

This keeps “always proceed” while keeping “tree must be valid” as a hard requirement.

---

## 14) Extensibility Without Schema Drift

We anticipate future features like “node-specific knowledge packs” and “node-specific tool access”.
We should NOT add ad-hoc fields to the node schema.

Pattern:

- Use deterministic, runner-owned sidecars keyed by node id:
  - `.runner/nodes/<node-id>/context.md`
  - `.runner/nodes/<node-id>/tools.json`
  - `.runner/nodes/<node-id>/artifacts/...`

This allows growth without changing the task tree schema.

---

## 15) MVP CLI Surface (Proposal)

We can keep commands minimal:

- `runner init`
  - scaffolds `.runner/` docs + initial `.runner/tree.json`
  - updates `.gitignore` to ignore `.runner/iterations/`
- `runner validate`
  - validates schema + invariants
- `runner step`
  - runs exactly one deterministic iteration
- `runner run`
  - loops until completion or stop condition

---

## 16) First Implementation Milestone (Sequential)

Ordered tasks (no parallelism):

1. Write the concrete “contracts” as specs:
   - tree path/name, ordering semantics, mode classification, git policy, logging layout, timeouts.
2. Define `task_tree v1` schema + invariants (incl. passed-node immutability).
3. Implement `validate` (strict decode + invariants) + canonical JSON writer (atomic write).
4. Implement deterministic selector (leftmost open leaf with stable sibling sort).
5. Implement executor trait + Codex/Claude CLI adapters (timeouts + output caps).
6. Implement guard runner (`just ci`) with capture + timing.
7. Implement git manager:
   - branch safety, clean checks, deterministic commits every iteration.
8. Implement iteration logger (`.runner/iterations/`) and ensure `.gitignore` ignores it.
9. Implement `step` then `run` loops (global max iterations + stop conditions).
10. Add tests for determinism + invariants (selection, immutability, schema strictness).

---

## 17) Remaining Open Questions

These are the “still TBD” items that should be written into `.runner/HUMAN_QUESTIONS.md` and/or decided in
`.runner/GOAL.md` as we start implementation:

- Default values:
  - global max iterations
  - `max_attempts` default for new leaves
  - output caps
- Exact enforcement rules for runner-owned fields:
  - hard reject if agent edits `passes` / `attempts` vs overwrite deterministically
- How strictly to constrain “attempt exhaustion” behavior:
  - what constitutes a “rewrite” vs “expand”
- Whether to emit `events.jsonl` from day 1 (local-only) or defer.
