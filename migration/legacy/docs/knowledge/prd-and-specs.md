# PRDs and Specifications for Ralph

## Specs Are the Foundation

The most fundamental requirement for Ralph: **you need specifications**.

Tool choice (Claude Code, Amp, Cursor, Codex) matters less than:

- Thinking from first principles
- Using conversation to drive spec creation
- Having clear, verifiable acceptance criteria

## The PRD Structure

Ralph uses a Product Requirements Document (PRD) to track tasks. The standard format is `prd.json`:

```json
{
  "project": "ProjectName",
  "branchName": "ralph/feature-name",
  "description": "High-level feature description",
  "platformConstraints": {
    "supportedOS": ["linux", "macos", "windows"],
    "ciOS": ["linux"],
    "runtimeDependencies": [],
    "nonSupportedOSBehavior": "N/A",
    "ciMismatchStrategy": "N/A"
  },
  "userStories": [
    {
      "id": "US-001",
      "title": "Story title",
      "description": "As a [user], I want [feature] so that [benefit]",
      "acceptanceCriteria": [
        "Verifiable criterion 1",
        "Verifiable criterion 2",
        "Typecheck passes"
      ],
      "manualSmokeTestChecklist": null,
      "priority": 1,
      "passes": false,
      "completion": {
        "summary": "",
        "retro": {
          "difficulties": "",
          "prdWeaknesses": "",
          "ralphImprovements": ""
        },
        "scopeAppropriate": null,
        "autoCompactOccurred": null
      }
    }
  ]
}
```

### Key Fields

#### Top-level fields

| Field | Purpose |
|-------|---------|
| `project` | Project/repo name |
| `branchName` | Branch Ralph should work on (e.g., `ralph/feature-name`) |
| `description` | 2–5 sentences describing the work |
| `platformConstraints` | Declares supported OS + CI OS and how to handle mismatches |
| `userStories` | Ordered list of focused stories |

#### Story fields

| Field | Purpose |
|-------|---------|
| `id` | Unique identifier (US-001, US-002...) |
| `title` | Short descriptive name |
| `description` | User story format explaining the why |
| `acceptanceCriteria` | Verifiable conditions for completion |
| `manualSmokeTestChecklist` | Optional path to a manual smoke-test checklist (required if the story has manual validation) |
| `priority` | Order of execution (1 = highest) |
| `passes` | Completion status (false → true when done) |
| `completion` | Completion metadata: summary + retro learnings + sizing signals |

### Platform Constraints

If the repo is OS-specific (or your CI runs on a different OS than the product/runtime), make that explicit in `platformConstraints` so every story can define the correct behavior in non-target environments.

### Manual Smoke Tests

If a story requires manual validation, include a checklist file (e.g., `.ralph/smoke-tests/US-012.md`) and reference it in `manualSmokeTestChecklist`.

### Story `completion` (Completion Retro)

When a story is marked `passes: true`, its `completion` should include:

- `completion.summary`: a short summary of what changed + key files touched
- `completion.retro.*`: retrospective learnings to improve future loops:
  - `difficulties` (or `N/A` / `"null"` as plain text)
  - `prdWeaknesses` (or `N/A` / `"null"`)
  - `ralphImprovements` (or `N/A` / `"null"`)
- `completion.scopeAppropriate`: a short judgement + reasoning about whether the story fit in a single context window (e.g. `Appropriate: yes — ...` / `Appropriate: no — ...`)
- `completion.autoCompactOccurred`: `true`/`false` (did the agent auto-compact during the iteration?)

## User Story Sizing

**Critical rule**: Each story must be completable in one context window.

### Right-Sized Stories

- Add a database column and migration
- Add a UI component to an existing page
- Update a server action with new logic
- Add a filter dropdown to a list
- Write tests for a single module

### Too Large (Needs Splitting)

| Bad | Split Into |
|-----|------------|
| "Build the entire dashboard" | Schema, queries, components, filters |
| "Add authentication" | Schema, middleware, login UI, session handling |
| "Refactor the API" | One story per endpoint |
| "Add user management" | CRUD operations as separate stories |

### Sizing Test

If you can't describe "done" in 3-5 acceptance criteria, the story is too big.

## The "Pin" Concept

A "pin" is an evolving specification document that anchors the system:

### Purpose

- Frame of reference for current functionality
- Prevents the agent from inventing features
- Built incrementally through conversations

### Implementation

- Create through conversational spec refinement
- Update as implementation reveals new requirements
- Reference in prompts to ground the agent

### Lookup Tables

Instead of injecting entire specs into context:

- Use lookup tables that point to relevant sections
- Include synonyms/hints to improve search accuracy
- Better retrieval = less hallucination

Example lookup entry:

```markdown
## User Authentication
- Location: specs/auth.md
- Also known as: login, signin, identity, session
- Key files: src/auth/*, src/middleware/auth.ts
```

## Architecture Doc (Agent-First)

For persistent, repo-specific context that should survive across iterations, keep a short architecture doc (e.g. `.ralph/architecture.md`):

- System overview, key components, critical flows
- Invariants/constraints that must not break
- Keep it small (size-capped by gates) and put detailed rationale close to code (comments)

## Writing Acceptance Criteria

Acceptance criteria must be **objectively verifiable**:

### Good Criteria

- "Typecheck passes with no errors"
- "All existing tests pass"
- "New component renders in browser at /dashboard"
- "API returns 200 with valid JSON for GET /users"
- "Form submits and shows success message"

### Bad Criteria

- "Code is clean" (subjective)
- "Performance is good" (unmeasurable)
- "User experience is improved" (vague)
- "Works correctly" (undefined)

### Always Include Quality Gates

Every story should include:

- "Typecheck passes"
- "Lint passes" (if applicable)
- "Tests pass" (if applicable)

## Building Specs Through Conversation

The "pottery wheel" method:

### 1. Start with Intent

```text
"I want to add product analytics like PostHog into the project.
Let's discuss and you can interview me."
```

### 2. Let the Model Interview You

- What does the model already know?
- What assumptions need validation?
- What constraints exist?

### 3. Shape Requirements Iteratively

Like clay on a pottery wheel:

- Add details when needed
- Remove complexity when possible
- Refine until the shape is clear

### 4. Capture in Specification

Document decisions as they're made:

- Why this approach over alternatives
- Constraints that affected the choice
- Open questions for future resolution

## Example: Analytics Spec Conversation

```text
User: I want to add analytics. Interview me.

Agent: What analytics platform?
User: PostHog-style, but internal.

Agent: What about privacy?
User: Collect everything, but wrap PII in a secret handler.

Agent: Data storage?
User: SQLite for now, migration path to Postgres later.

Agent: SDK needs?
User: Rust backend, TypeScript frontend.
```

This conversation produces:

- Clear scope boundaries
- Technical constraints
- Implementation guidance

## PRD Workflow

### 1. Create PRD from Conversation

Use plan mode or a PRD skill:

```text
Load the prd skill and create a PRD for [feature description]
```

### 2. Convert to Ralph Format

```text
Convert tasks/prd-feature.md to prd.json
```

### 3. Review Story Sizing

- Each story completable in one context?
- Dependencies ordered correctly?
- Acceptance criteria verifiable?

### 4. Execute Ralph Loop

Ralph picks highest priority incomplete story, implements it, marks complete, repeats.

## Linkage: Connecting Plans to Code

When creating implementation plans, establish strong linkage:

### Bullet Points with Citations

```markdown
## Implementation Steps
- Add `analytics_events` table (see specs/analytics.md#data-model)
- Create EventService in src/services/events.ts
- Update UserController (src/controllers/user.ts:45-60)
```

### Why Linkage Matters

- The "read tool works in hunks"
- Specific file:line references enable precise edits
- Reduces wandering/searching behavior

## Separate Sessions for Separate Goals

After generating specs, resist continuing in the same context:

| Session | Purpose |
|---------|---------|
| Session A | Spec creation and refinement |
| Session B | Implementation loop |
| Session C | Testing and verification |

Each context window has one goal. Mixing concerns leads to:

- Context rot
- Lost focus
- Degraded outputs
