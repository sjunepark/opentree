# Ralph: Overview

## What is Ralph?

Ralph is an autonomous AI coding methodology that runs AI agents in iterative loops until tasks are complete. Named after Ralph Wiggum from The Simpsons (known for his persistent optimism despite setbacks), the technique transforms AI coding tools from short-term assistants into long-running autonomous workers.

At its core, Ralph is deceptively simple:

```bash
while :; do cat PROMPT.md | claude -p "Follow the instructions in stdin."; done
```

This loop repeatedly feeds a prompt to an AI agent, letting it see its previous work (via git history and modified files), learn from failures, and iteratively improve until a defined completion state is reached.

## Origin

Ralph originated in May 2025 from Geoffrey Huntley, an Australian developer. What started as a five-line bash script became a widely adopted methodology after demonstrating that AI agents could complete substantial development work autonomously overnight.

The name captures the technique's philosophy: like Ralph Wiggum, the system maintains naive persistence—it keeps trying despite failures, eventually "dreaming" a correct solution just to escape the loop.

## Core Philosophy

### Naive Persistence

The power of Ralph lies in its "naive persistence"—the AI isn't protected from its own mistakes. It confronts its failures directly, forced to reason through errors and find solutions. This unsanitized feedback loop is what makes it effective.

### Deterministic Failures

A key insight: Ralph's failures are deterministic, not random. When the loop fails repeatedly in the same way, that's a signal to tune your prompts. The creator describes this as "tuning Ralph like a guitar"—adding clarity to prompts (like signs saying "SLIDE DOWN, DON'T JUMP") improves subsequent outputs.

### Software Development vs Software Engineering

Ralph reframes the distinction:

- **Software development** (the mechanical work of writing code) is now largely automatable
- **Software engineering** (designing systems, managing constraints, ensuring safety) becomes the human's primary role

The human shifts from "in the loop" to "on the loop"—designing and programming the automation itself rather than doing the mechanical work.

## The Economics Argument

Proponents claim Ralph fundamentally changes development economics:

- **$1042/hour**: Calculated by running Ralph loops against API costs over 24 hours
- **Case study**: A $50,000 contract completed for $297 in API costs
- **Throughput**: A single hour can output "multiple days worth of work if not weeks"

These numbers vary significantly based on task complexity, prompt quality, and operator skill. The methodology works best for tasks with clear completion criteria and mechanical execution.

## The Paradigm Shift

Traditional AI-assisted coding is "human-in-the-loop" (HITL):

- Give the AI a task
- Watch it work
- Intervene when it goes off-track

Ralph inverts this to autonomous operation:

- Define what "done" looks like
- Let the AI iterate until it gets there
- Human oversight focuses on design and safety

This shift requires thinking differently about specifications, quality gates, and feedback mechanisms—which the other documents in this knowledge base cover in detail.

## Key Implementations

- **Claude Code Plugin**: `/plugin install ralph-wiggum@claude-plugins-official`
- **snarktank/ralph**: GitHub repo with scripts, PRD templates, and Amp integration
- **Various forks**: ralph-claude-code, ralph-loop-agent (Vercel Labs)

## Further Reading

- `ralph-loop-mechanics.md` - How the loop works technically
- `agents-md-guide.md` - Best practices for agents.md files
- `prd-and-specs.md` - Writing effective specifications
- `ralph-best-practices.md` - Tips and anti-patterns
