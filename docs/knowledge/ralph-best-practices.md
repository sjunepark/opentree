# Ralph Best Practices

## When Ralph Works Best

Ralph excels at tasks with **clear completion criteria** and **mechanical execution**:

### Ideal Use Cases

- **Large refactors**: Class to functional components, callback to async/await
- **Framework migrations**: Jest to Vitest, Express to Fastify
- **TDD workflows**: You write failing tests, Ralph makes them pass
- **Batch operations**: Update 50 files with the same pattern
- **Greenfield builds**: New features with clear specs
- **Code coverage**: Add tests until coverage threshold met

### Common Success Pattern

If you can describe "done" precisely and verify it programmatically, Ralph can iterate toward it.

## When to Avoid Ralph

### Poor Fit

- **Ambiguous requirements**: "Make it better" has no clear endpoint
- **Architectural decisions**: Requires judgment and tradeoffs
- **Security-sensitive code**: Needs careful human review
- **Exploratory work**: When you don't know what you want yet
- **Novel problems**: No training data, no patterns to follow

### Warning Signs

- You can't write acceptance criteria
- "Done" is subjective
- Human judgment needed at multiple steps
- The domain is poorly understood

## Quality Gates Are Mandatory

Never commit broken code. Each iteration must pass gates before committing:

### Required Gates

- **Typecheck**: Non-negotiable for typed languages
- **Lint**: Catches style and potential bugs
- **Tests**: Existing tests must stay green

### Optional Gates

- **Browser verification**: For UI stories
- **API testing**: For backend changes
- **Coverage thresholds**: For test-writing tasks

### Gate Philosophy

The best setup blocks commits unless everything passes. Ralph can't declare victory if the tests are red.

## Start Attended, Go Unattended

### Initial Runs

1. Watch the first few iterations closely
2. Look for unexpected behavior patterns
3. Cancel and adjust prompts when needed
4. Build intuition for how your codebase responds

### Gradual Trust

- Increase iteration limits as confidence grows
- Set up monitoring and alerts
- Review commits periodically even in unattended mode

### Bad Output Is Still Useful

Even failed iterations become fuel for another Ralph loop:

- Security review loop
- Code style alignment loop
- Test coverage loop

## Multi-Phase Planning

Large features exceed single-context capacity. Use phases:

### Phase Structure

```text
Phase 1: Database schema and migrations
Phase 2: API endpoints and services
Phase 3: UI components
Phase 4: Integration and testing
```

### Per-Phase Prompts

Write different prompts for each phase:

- "Implement the database schema per specs/data-model.md"
- "Add API endpoints per specs/api.md"
- "Build UI components per specs/ui.md"

### Phase Boundaries

Each phase is a separate Ralph loop with:

- Its own PRD or task list
- Clear completion criteria
- Quality gates

## Iteration Limits and Cost Awareness

### Always Set Limits

```bash
--max-iterations 20
```

Without limits:

- Impossible tasks burn unlimited budget
- Stuck loops spiral endlessly
- Costs compound quickly

### Cost Expectations

| Iterations | Approximate Cost |
|------------|------------------|
| 10 | $10-20 |
| 20 | $20-40 |
| 50 | $50-100+ |

Costs vary significantly based on:

- Model used (Opus vs Sonnet vs Haiku)
- Task complexity
- File sizes being processed

### Monitoring

Track costs during initial runs before going unattended.

## Screwdriver First, Jackhammer Later

### The Progression

1. **Manual**: Understand the concepts by doing them by hand
2. **Assisted**: Use basic loops with close supervision
3. **Automated**: Full autonomous operation with proper guardrails

### Why This Matters

- Build intuition for what works
- Understand failure modes
- Craft better prompts through experience

### Anti-Pattern

Jumping straight to full automation without understanding fundamentals leads to:

- Wasted API costs
- Poor results
- Frustration and abandonment

## Feedback Loop Speed

> "The rate at which you can get feedback is your speed limit. Never outrun your headlights."

### Fast Feedback

- Quick tests give rapid iteration
- Immediate type errors guide corrections
- Lint failures catch issues early

### Slow Feedback = Problems

- Long test suites delay iterations
- Complex builds slow the loop
- Integration tests as only verification

### Optimize for Speed

- Unit tests over integration tests (for Ralph)
- Incremental builds
- Fast type checking

## Separate Arrays for Separate Goals

### One Context, One Purpose

| Context | Purpose |
|---------|---------|
| Spec context | Create and refine specifications |
| Impl context | Execute implementation |
| Test context | Add test coverage |
| Review context | Security/style review |

### Why Separate

- Prevents context contamination
- Each session stays focused
- Avoids compaction from mixed concerns

### Practical Workflow

1. Spec session generates requirements
2. Implementation session executes against specs
3. Review session validates output
4. Each can reference artifacts from others

## Progress Tracking

### `.ralph/progress.md` Format

```markdown
## [Date/Time] - US-001
Thread: [link if available]
- What was implemented
- Files changed
- **Learnings for future iterations:**
  - Patterns discovered
  - Gotchas encountered
---
```

### Codebase Patterns Section

At the top of `.ralph/progress.md`, consolidate reusable learnings:

```markdown
# Codebase Patterns
- API routes follow /api/v1/{resource} convention
- All database access through repository pattern
- Tests use factory functions for fixtures
```

### AGENTS.md Updates

After iterations, update relevant AGENTS.md files with:

- Only durable, high-signal guidance (keep it small ~70 lines; prefer updating only a dedicated `## Ralph Loop` section; move long notes to progress logs or lazily-loaded "skills")
- API patterns specific to that module
- Gotchas or non-obvious requirements
- File dependencies
- Testing approaches

## Common Anti-Patterns

### 1. Huge Prompts

- Trying to explain everything upfront
- Better: Minimal guidance + iteration

### 2. No Quality Gates

- Letting broken code accumulate
- Compounds problems across iterations

### 3. Vague Completion Criteria

- "Make it work" instead of specific conditions
- Loop never knows when to stop

### 4. Skipping Specs

- Jumping straight to implementation
- Results in wandering behavior

### 5. Single Massive Context

- Not splitting into phases
- Hits context limits, triggers compaction

### 6. Ignoring Failures

- Not tuning prompts when loops fail
- Same failures repeat indefinitely

## Verification Checklist

Before running Ralph loops:

- [ ] PRD/specs are complete and specific
- [ ] Each story fits in one context window
- [ ] Acceptance criteria are objectively verifiable
- [ ] Quality gates are configured
- [ ] Max iterations limit is set
- [ ] Cost monitoring is in place
- [ ] You've done at least one attended run

## The Right Mindset

### Old Model

"I use AI to help me code"

### Ralph Model

"I design systems and let AI execute"

The human becomes the **locomotive engineer**:

- Keeping the system on track
- Designing routes and constraints
- Intervening when derailment threatens

The mechanical work of writing code becomes automated. The engineering work of design, safety, and oversight becomes primary.
