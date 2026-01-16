# Vision: Goal-Driven Agent Looping System

This repository is pivoting from a “PRD/story runner” into a **general-purpose system for looping agents**.

The end state is a **deterministic local runner** that repeatedly spawns **fresh, isolated agent sessions** to:

1. Take a **Goal**.
2. Build and refine a **sequential task tree** (no parallel workflows).
3. Execute the **single next actionable leaf**.
4. Run **automatic validation** (format/lint/test/etc) as a feedback signal.
5. Learn from failures and continue until completion.

UI is explicitly secondary. A future browser tree view is desirable, but not part of the core vision.

---

## Core Principles

### Deterministic runner, flexible agent

- The **runner** is deterministic: selection, limits, validation, and invariants are enforced consistently.
- The **agent** can be creative during planning and execution, but only within the runner’s rules.

### Strict schemas and machine-validated state

- The **task tree is the source of truth** and must conform to strict schemas.
- The runner validates the tree **every iteration**.
- The runner enforces invariants like “no `passes=true` unless guards pass”.
- The node schema is fixed and intentionally small:
  - no ad-hoc extension
  - **do not add fields to node objects**
  - represent new needs via new nodes or runner-owned sidecar artifacts (not schema drift)

### Fresh context per iteration

- Each iteration starts in a new agent session/context (Ralph-style).
- Continuity is provided by **repo artifacts** (tree + memory docs + git history), not by long contexts.

### Sequential progress (no parallel workflows)

- Only one leaf is worked on at a time.
- The tree expansion loop is **just-in-time**: only the highest-priority leaf is decomposed/executed.

### Always proceed; record uncertainty

- The loop does not stop for questions.
- The agent must proceed with assumptions, and write down:
  - what needs human input/discussion
  - what assumptions were made
  - what might be wrong about them

### Validation is the feedback loop

Formatting, linting, and tests are not “nice to have”; they are the core mechanism that makes the
system self-correcting.

---

## Canonical Artifacts

### `GOAL.md` (human-facing, thorough)

`GOAL.md` is the thorough documentation of the top-level goal/spec (the “what”, not the “how”).

It should include:

- The desired outcome and success criteria.
- Scope and non-goals.
- Constraints (platform, safety, repo rules).
- Any upfront, large decisions (frameworks/libraries), if applicable.

### Tree file (machine-facing, strict)

The tree is the canonical state the runner uses to plan and execute work. It references `GOAL.md`
and contains ordered nodes representing the plan.

Other Markdown files exist for memory/logging, but are not the source of truth.

---

## Task Tree Model (MVP)

### Goal-oriented nodes (declarative)

For the MVP, nodes represent **goals**, not imperative “do X” steps.
Implementation details belong in code and in memory docs, not in the node schema.

### Node lifecycle

- Nodes start as **open** with `passes=false`.
- A node can be **decomposed** into ordered children if it is not single-context executable.
- A node can be **executed** if it is single-context executable.
- A node becomes **passed** only after the runner’s guards succeed, at which point `passes=true`.

### Immutability rule (critical)

- Nodes with `passes=true` are immutable:
  - The runner rejects any edits to their node record (values and structural placement).
  - Improvements/refactors affecting past work happen via **new nodes** (e.g., “Refactor X”).
- Nodes with `passes=false` may be edited and/or decomposed.

---

## Loop Mechanics (Deterministic)

Each iteration:

1. **Load** the task tree and validate schema.
2. **Select** the next node deterministically:
   - choose the highest-priority open **leaf** (lowest `order`, tie-break by `id`)
3. Decide mode:
   - If the node is not “single context executable”: run a **DECOMPOSE** iteration.
   - Otherwise: run an **EXECUTE** iteration.
4. **Run** the agent (fresh session) with only the needed context.
5. **Validate**:
   - Runner executes guards (below) and records results.
6. **Update state** (strictly):
   - Mark `passes=true` only if guards pass.
   - Increment attempts on failure.
7. **Persist**:
   - Write logs/memory files.
   - Commit changes (exact commit policy is TBD; defaults should remain automation-friendly).

### Completion condition

The run is complete when there are **no open nodes remaining** (i.e., all nodes have `passes=true`).

### Leaf iteration limit

- Each leaf has an iteration budget (`N` attempts).
- If the leaf fails its budget, the agent must either:
  - **rewrite** the leaf (change its plan/acceptance at the leaf level), or
  - **expand** it into children (it was too large/unclear), or
  - do both, as long as determinism and schemas are preserved.

---

## Guards / Gates (Automatic Feedback)

The runner owns validation; it does not rely on the agent “remembering” to run checks.

MVP direction:

- Prefer a deterministic guard entrypoint like `just ci` (or an equivalent configured command).
- Allow the agent to edit the guard implementation (e.g., update `just ci` / scripts) as needed for the repo.
- The runner enforces:
  - schema validity
  - clean working tree / commit invariants (as configured)
  - “no pass without green guards”

---

## Memory, Feedback, and Human Questions

The system must accumulate learning without bloating the tree schema.

These are Markdown (human-facing, flexible), and can be updated freely:

- `FEEDBACK_LOG.md` — mistakes, failure modes, what to do differently next iteration.
- `HUMAN_QUESTIONS.md` — missing product intent, unclear constraints, etc. (not syntax-level issues).
- `ASSUMPTIONS.md` — assumptions made to keep execution moving.
- `IMPROVEMENTS.md` — tools/prompts/approach that would have helped (e.g., Playwright, Chrome DevTools, PDF readers,
  MCPs, domain knowledge packs).

The runner should ensure these are discoverable and consistently written/updated.

---

## Tooling and Capability Growth

The system should make it easy for the agent to use the tools required to validate and ship:

- Local commands for formatting/linting/testing.
- Optional tool installation during execution (when needed).
- Optional integrations (Playwright, browser automation, devtools, doc readers, MCP servers).

Large architectural choices (frameworks/libraries) should be decided upfront in `GOAL.md`.
Smaller implementation-level decisions can be made by the agent during execution.

---

## Non-Goals (Near Term)

- Parallel execution or multi-agent concurrency.
- Remote execution / CI-as-runner as the default.
- A polished UI (TUI/web) before the core loop is correct and deterministic.
- Auto-push, PR creation, or GitHub integrations.

---

## Glossary

- **Runner**: deterministic CLI that selects nodes, enforces schemas/invariants, runs guards, and orchestrates agent runs.
- **Agent**: the LLM tool invoked by the runner (fresh session per iteration).
- **Task Tree**: ordered node tree that represents the plan; source of truth for progress and selection.
- **Leaf**: a node with no children (at the moment) that is selected for decomposition or execution.
