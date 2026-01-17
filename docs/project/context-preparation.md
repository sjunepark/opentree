# Context Preparation

The "Prepare Context" phase (step 3 in the orchestration flow) has two parts:

1. **`write_context()`** — Write ephemeral files for agent reference
2. **`PromptBuilder.build()`** — Assemble the prompt sent to the executor

## 1. Ephemeral Context Files

These human-readable markdown files live in `.runner/context/` and are **cleared and rewritten every iteration**.

```text
.runner/context/
├── goal.md      ← Always written (current task)
├── history.md   ← Previous attempt summary (if retry)
└── failure.md   ← Failure output (if last attempt failed)
```

### What Gets Written

| File | Source | When Populated |
|------|--------|----------------|
| `goal.md` | Selected node's `title`, `goal`, `acceptance` | Always |
| `history.md` | `run_state.last_summary` | Only when `last_status == Retry` |
| `failure.md` | Previous iteration's `failure.log` (fallback: `guard.log`) | Only when `last_guard == Fail` |

### Example Files

**`goal.md`:**

```markdown
# Goal

title: Implement user authentication
goal: Add JWT-based auth to the API
acceptance:
- POST /login returns JWT token
- Protected routes require valid token
- Token expires after 24 hours
```

**`history.md` (on retry):**

```markdown
# History (previous attempt)

Implemented login endpoint but tests fail due to missing
bcrypt dependency. Need to add bcrypt to Cargo.toml.
```

**`failure.md` (after failure):**

```markdown
# Failure (previous attempt)

error[E0432]: unresolved import `bcrypt`
 --> src/auth.rs:3:5
  |
3 | use bcrypt::hash;
  |     ^^^^^^ use of undeclared crate
```

## 2. Prompt Pack Assembly

`PromptBuilder` assembles a single prompt string sent to the agent (via `codex exec`). It reads from multiple sources and enforces a byte budget (default 40KB).

### Section Order (Deterministic)

| # | Section | Required | Source |
|---|---------|----------|--------|
| 1 | Runner Contract | Yes | Hardcoded rules |
| 2 | Goal | Yes | `.runner/context/goal.md` |
| 3 | History | No | `.runner/context/history.md` |
| 4 | Failure | No | `.runner/context/failure.md` |
| 5 | Selected Node | Yes | Node metadata (path, id, title, goal, acceptance) |
| 6 | Tree Summary | No | Bounded summary of full tree |
| 7 | Assumptions | No | `.runner/state/assumptions.md` |
| 8 | Open Questions | No | `.runner/state/questions.md` |
| 9 | Output Contract | Yes | Hardcoded output instructions |

### Budget Enforcement

When total prompt exceeds budget, **droppable sections are removed in priority order**:

```text
Drop order (least → most important):
1. tree        ← First to go (can be large, agent has tree.json)
2. assumptions ← Historical context, not critical
3. questions   ← Historical context, not critical
4. history     ← Useful but not essential
5. failure     ← Important for debugging but droppable
```

Required sections (`contract`, `goal`, `selected`, `output`) are never dropped.

If still over budget after dropping all droppable sections, the last section gets truncated with `[truncated]` suffix.

## Data Flow

```text
                    ┌──────────────────┐
                    │   run_state.json │
                    │  (last_status,   │
                    │   last_summary,  │
                    │   last_guard)    │
                    └────────┬─────────┘
                             │
         ┌───────────────────┼───────────────────┐
         ▼                   ▼                   ▼
   ┌───────────┐      ┌───────────┐      ┌──────────────┐
   │ goal.md   │      │history.md │      │ failure.md   │
   │ (always)  │      │(if retry) │      │(if last fail) │
   └─────┬─────┘      └─────┬─────┘      └──────┬───────┘
         │                  │                   │
         └──────────────────┼───────────────────┘
                            ▼
                   ┌─────────────────┐
                   │  PromptInputs   │◄── tree_summary
                   │   .from_root()  │◄── selected_node
                   └────────┬────────┘◄── assumptions.md
                            │         ◄── questions.md
                            ▼
                   ┌─────────────────┐
                   │ PromptBuilder   │
                   │  .build()       │
                   │ (40KB budget)   │
                   └────────┬────────┘
                            ▼
                   ┌─────────────────┐
                   │   PromptPack    │
                   │   .render()     │──► Single string to executor
                   └─────────────────┘
```

## Key Design Decisions

1. **Ephemeral vs persistent separation** — `context/` cleared each iteration; `state/assumptions.md` and `state/questions.md` persist across iterations

2. **Deterministic ordering** — Sections always appear in the same order for reproducibility

3. **Graceful degradation** — Budget enforcement drops least-critical info first rather than failing

4. **Required bookends** — Contract at top, output instructions at bottom ensure agent always knows the rules

## Source Files

- `runner/src/io/context.rs` — `write_context()`, `ContextPayload`
- `runner/src/io/prompt.rs` — `PromptBuilder`, `PromptPack`, `PromptSection`
