# agents.md Best Practices

## Why agents.md Exists

Repositories were getting cluttered with tool-specific instruction files (Jules.md, Claude.md, Cursor.md, etc.). The community converged on a single standard: `agents.md`.

### Brief History

- Multiple competing files created fragmentation
- OpenAI and Google standardized on `agents.md` (plural)
- Domain ownership drama eventually resolved (OpenAI bought agents.md)
- Tools now symlink their specific files to agents.md for compatibility

## The "Always Allocated" Problem

This is the critical insight: **agents.md is always injected into the context window**.

Using the context-as-array model:

```text
┌─────────────────────────────────────────────┐
│ Slot 0: Harness prompt (tool's base)        │
│ Slot 1: agents.md ← ALWAYS PRESENT          │
│ Slot 2-N: Available working context         │
└─────────────────────────────────────────────┘
```

Implications:

- Every token in agents.md reduces available working context
- Large agents.md files push you into the "dumb zone" faster
- Content should be minimal and high-value

## Target Size: ~70 Lines

Keep your agents.md to approximately **70 lines of text**.

Why this matters:

- Everything gets tokenized every iteration
- Huge files = less room for actual work
- Diminishing returns on extensive documentation

## What to Include

Focus on **minimal, high-utility content** that enables autonomous operation:

### Essential

- **How to build** the project
- **How to run tests**
- **How to lint/format**

### Optional (if small)

- Basic repo layout pointers
- Key architectural patterns

### Practical Test

If the agent can run loops without you intervening because basic commands are unclear, your agents.md is sufficient.

## What NOT to Include

### Specs and Requirements

- These change frequently
- Should be injected on-demand, not permanently
- Use separate spec files that get loaded when needed

### Detailed Deployment Info

- Only needed sometimes
- Better as a "skill" that's lazily loaded
- Keeps the always-allocated content minimal

### Growing Knowledge Dumps

Anti-pattern: Teams keep appending rules via PRs until agents.md becomes a sprawling document with unclear origins.

## The "Just Enough" Principle

Modern LLMs are capable. You don't need hyper-specific instructions.

### Provide Just Enough to Trigger Latent Behavior

**Bad**: Detailed step-by-step instructions for every scenario

```markdown
To check the web server status:
1. SSH into the server
2. Run `sudo systemctl status nginx`
3. Check the output for "active (running)"
4. If not running, run `sudo systemctl start nginx`
```

**Good**: Minimal cue that triggers correct behavior

```markdown
Use journalctl/journald for checking services.
```

The model infers systemctl, sudo, and correct syntax from this minimal guidance.

### Tuning Mindset

When the agent repeatedly fails tool calls, that's a signal to tune your prompts:

- **Wrong/missing guidance** = "cache miss" = wandering/searching behavior
- Add clarity incrementally until behavior improves
- Think of it as "tuning a guitar"

## Maintenance: Aggressive Pruning

Don't let agents.md grow unchecked.

### Regular Review

- Why was each section added?
- Is it still relevant?
- Can it be removed or moved to lazy-loaded content?

### Regeneration Strategy

If you need extensive guidance temporarily:

1. Add it to accomplish the task
2. Remove it afterward
3. Regenerate via prompts when needed again

## One File Doesn't Fit All Models

A complication: different LLMs respond differently to the same instructions.

### The Mismatch Problem

- Some models respond well to firm, uppercase instructions
- Others become "timid" with the same approach
- Model-specific guidance in a shared file creates noise

### Alternative Approach

Consider per-model files: `agent/<model-name>.md`

This lets you tune instructions for each model's characteristics while keeping a minimal shared agents.md for universal basics.

## Example Structure

```markdown
# agents.md

## Build
npm install && npm run build

## Test
npm test

## Lint
npm run lint -- --fix

## Key Patterns
- Components in src/components/
- API routes in src/api/
- Tests co-located with source files

## Notes
- Use TypeScript strict mode
- Run prettier before committing
```

Approximately 20 lines. Everything else is loaded on-demand.

## Skills: The Lazy Loading Alternative

For content needed only sometimes:

- Create separate markdown files as "skills"
- Load them when relevant tasks arise
- Keeps permanent allocation minimal

Example: Deployment instructions live in `skills/deploy.md`, loaded only when deploying.

This pattern gives you "high control of that malloc"—choosing exactly when to consume context for specific knowledge.
