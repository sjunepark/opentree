# Ralph: Loop Mechanics

## The Fundamental Loop

At its simplest, Ralph is a bash while-loop:

```bash
while :; do cat PROMPT.md | claude -p "Follow the instructions in stdin."; done
```

This continuously feeds a prompt to the AI agent. Each iteration starts fresh, but the agent sees continuity through:

- Git history (commits and diffs from previous iterations)
- Modified files in the working directory
- Progress tracking files (`.ralph/progress.md`, PRD status)

## Context Window as an Array

A mental model that helps understand Ralph: treat the context window as an **array** with limited slots.

```text
┌─────────────────────────────────────────────┐
│ Slot 0: Harness prompt (Claude Code's base) │
│ Slot 1: agents.md (auto-injected)           │
│ Slot 2: Current task/prompt                 │
│ Slot 3-N: Working context (files, outputs)  │
└─────────────────────────────────────────────┘
```

Key implications:

- **Slot 0 and 1 are "always allocated"**—they consume context every iteration
- The more permanent allocations you have, the less room for actual work
- As context fills, you enter the "dumb zone" where outputs degrade

## Fresh Context Per Iteration

Each Ralph iteration spawns a **fresh context window**. This is intentional:

### Benefits

- Avoids context rot (accumulated noise degrading outputs)
- Prevents compaction (lossy summarization that loses critical details)
- Each iteration has maximum available context for its single task

### How Continuity Works

Since each iteration is fresh, continuity must be external:

| Mechanism | Purpose |
|-----------|---------|
| Git history | Shows what was done, what changed |
| `.ralph/progress.md` | Append-only log of learnings and patterns |
| PRD/prd.json | Tracks which stories are complete |
| Architecture doc | Stable high-level model + invariants (kept short) |
| AGENTS.md | Accumulated patterns and gotchas |

## Stop Conditions

Ralph needs to know when to stop. Common approaches:

### 1. Completion Promise

The agent outputs a specific string when done:

```text
<promise>COMPLETE</promise>
```

A "Stop Hook" blocks exit until this promise appears, forcing the agent to keep working.

### 2. PRD Completion

All user stories in prd.json have `passes: true`:

```json
{
  "userStories": [
    { "id": "US-001", "passes": true },
    { "id": "US-002", "passes": true }
  ]
}
```

### 3. Max Iterations

Safety limit to prevent runaway costs:

```bash
/ralph-loop "task" --max-iterations 20
```

Always set this. Exact string matching for completion promises can be unreliable.

## Context Rot and Compaction

### Context Rot

As iterations accumulate within a single context (non-Ralph approach), quality degrades:

- Earlier instructions get "forgotten"
- Conflicting information accumulates
- The agent starts hallucinating or inventing

### Compaction (The "Devil")

When context fills, systems attempt to compress:

- Summarization is **lossy**
- Critical details ("the pin") can be lost
- Outcomes become unpredictable

Ralph's fresh-context-per-iteration approach sidesteps both problems.

## The "Pin" Concept

A "pin" is a persistent reference document that anchors the system:

- Evolving specification built through conversations
- The frame of reference for current functionality
- Prevents the system from inventing features

Instead of injecting everything into context:

- Use **lookup tables** that point to relevant parts
- Include synonyms/hints to improve search hit rates
- Better retrieval = less hallucination

## Separate Arrays for Separate Goals

After generating specs, it's tempting to continue in the same context window. Don't.

**Spec creation** and **implementation** should use separate sessions:

- Each context has one goal
- Keeps each session clean and focused
- Prevents cross-contamination of concerns

Practical workflow:

1. Session A: Create/refine specs (keep open for reference)
2. Session B: Implementation loop
3. Session C: Testing/verification loop

## Practical Loop Commands

### Basic Ralph (Claude Code Plugin)

```bash
/ralph-loop "Your task description" --max-iterations 10
```

### Manual Bash Loop

```bash
while :; do
  cat PROMPT.md | claude --dangerously-skip-permissions
done
```

### With Iteration Counter

```bash
MAX=20
COUNT=0
while [ $COUNT -lt $MAX ]; do
  cat PROMPT.md | claude -p "Follow the instructions in stdin."
  COUNT=$((COUNT + 1))
done
```

## Permissions and Safety

To operate autonomously, Ralph requires `--dangerously-skip-permissions`:

- Asking for approval on every tool call breaks the loop
- This bypasses the permission system entirely

**Safety boundary**: Use sandboxing, isolated environments, or careful PRD constraints to limit what the agent can do.

## Monitoring

Start **attended** before going unattended:

- Watch for unexpected behavior
- Cancel and adjust prompts as needed
- Even bad output becomes fuel for another loop to refactor

Track costs:

- A 50-iteration loop can cost $50-100+
- Monitor API usage carefully during initial runs
