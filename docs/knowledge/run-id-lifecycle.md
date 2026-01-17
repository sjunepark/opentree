# Run ID / Goal ID Lifecycle

The runner implements **deterministic, branch-isolated runs** where each run is identified by a unique ID stored in multiple locations that must match.

## Identity Invariant

```mermaid
flowchart LR
    subgraph identity["Run Identity (must all match)"]
        G[".runner/GOAL.md<br/>id: run-abc123"]
        S[".runner/state/run_state.json<br/>run_id: run-abc123"]
        B["git branch<br/>runner/run-abc123"]
    end

    G <-.->|"enforced by<br/>runner step"| S
    S <-.->|"enforced by<br/>runner step"| B
```

This design ensures:

- **Isolation** — Each run gets its own git branch for clean rollback
- **Resumability** — Re-running `runner start` on same goal ID resumes work
- **Intentionality** — Changing the goal requires explicit ID update

## `runner start` Flow

`runner start` initializes or resumes a run by ensuring all three identity sources are consistent.

```mermaid
flowchart TD
    START["runner start"] --> CLEAN{"Working tree clean?<br/>(except .runner/)"}
    CLEAN -->|No| ERR1["Error: uncommitted changes"]
    CLEAN -->|Yes| READ["Read GOAL.md frontmatter"]

    READ --> HAS_ID{id exists?}
    HAS_ID -->|Yes| VALIDATE["Validate id format<br/>[A-Za-z0-9._-]+"]
    HAS_ID -->|No| GEN["Generate run-&lt;sha8&gt;[-suffix]"]

    GEN --> UNIQUE{"Branch runner/&lt;id&gt;<br/>exists?"}
    UNIQUE -->|Yes| SUFFIX["Try next suffix"]
    SUFFIX --> UNIQUE
    UNIQUE -->|No| USE_ID

    VALIDATE --> USE_ID["Use id as run_id"]

    USE_ID --> BRANCH{"On runner/&lt;id&gt;<br/>branch?"}
    BRANCH -->|Yes| SKIP_BRANCH["Already on correct branch"]
    BRANCH -->|No| CHECK_BRANCH{"Branch exists?"}

    CHECK_BRANCH -->|Yes| CHECKOUT["git checkout runner/&lt;id&gt;"]
    CHECK_BRANCH -->|No| CREATE["git checkout -b runner/&lt;id&gt;"]

    CHECKOUT --> INIT
    CREATE --> INIT
    SKIP_BRANCH --> INIT

    INIT{"runner dir<br/>exists?"} -->|No| RUN_INIT["runner init"]
    INIT -->|Yes| WRITE_ID
    RUN_INIT --> WRITE_ID

    WRITE_ID["ensure_goal_id()<br/>Write id to GOAL.md frontmatter"]
    WRITE_ID --> LOAD_STATE["Load run_state.json"]

    LOAD_STATE --> MATCH{run_id matches?}
    MATCH -->|No| RESET["Reset iteration state<br/>run_state = default"]
    MATCH -->|Yes| KEEP["Keep existing state"]

    RESET --> SAVE
    KEEP --> SAVE

    SAVE["Write run_state.json<br/>run_id = id"] --> COMMIT["git commit<br/>'chore(loop): start run &lt;id&gt;'"]

    COMMIT --> DONE["StartOutcome { run_id, branch }"]

    style ERR1 fill:#8b0000,color:#fff
    style DONE fill:#2d5a27,color:#fff
```

### ID Generation

When no ID exists in GOAL.md, `runner start` generates one:

```text
run-<HEAD-sha-8-chars>[-suffix]
```

Example: `run-abc12345`, or `run-abc12345-2` if the first branch already exists.

## `runner step` Enforcement

Before executing any iteration, `runner step` validates all three identity sources match.

```mermaid
flowchart TD
    STEP["runner step"] --> BRANCH_CHECK{"On main/master?"}
    BRANCH_CHECK -->|Yes| ERR1["Error: refuse to run on protected branch<br/>(run 'runner start')"]
    BRANCH_CHECK -->|No| CLEAN{"Working tree clean?"}

    CLEAN -->|No| ERR2["Error: uncommitted changes"]
    CLEAN -->|Yes| GITIGNORE{".runner/.gitignore<br/>exists?"}

    GITIGNORE -->|No| ERR3["Error: missing .gitignore<br/>(run 'runner start')"]
    GITIGNORE -->|Yes| LOAD["Load run_state.json"]

    LOAD --> HAS_RUN_ID{run_id set?}
    HAS_RUN_ID -->|No| ERR4["Error: missing run id<br/>(run 'runner start')"]
    HAS_RUN_ID -->|Yes| GOAL_CHECK["Read GOAL.md id"]

    GOAL_CHECK --> GOAL_MATCH{run_state.run_id<br/>== GOAL.md id?}
    GOAL_MATCH -->|No| ERR5["Error: run id mismatch<br/>(run 'runner start')"]
    GOAL_MATCH -->|Yes| BRANCH_MATCH{"On runner/&lt;run_id&gt;<br/>branch?"}

    BRANCH_MATCH -->|No| ERR6["Error: wrong branch<br/>(run 'runner start')"]
    BRANCH_MATCH -->|Yes| PROCEED["Proceed with iteration"]

    style ERR1 fill:#8b0000,color:#fff
    style ERR2 fill:#8b0000,color:#fff
    style ERR3 fill:#8b0000,color:#fff
    style ERR4 fill:#8b0000,color:#fff
    style ERR5 fill:#8b0000,color:#fff
    style ERR6 fill:#8b0000,color:#fff
    style PROCEED fill:#2d5a27,color:#fff
```

All error messages point to `(run 'runner start')` as the fix.

## Goal ID Storage

The ID lives in YAML frontmatter of `.runner/GOAL.md`:

```markdown
---
id: run-abc12345
---

# Goal

Implement the feature...
```

### Frontmatter Operations

| Operation | Behavior |
|-----------|----------|
| `read_goal_id()` | Parse frontmatter, return `Some(id)` or `None` |
| `ensure_goal_id()` | Upsert `id:` line, preserve other keys |
| `validate_id()` | Ensure `[A-Za-z0-9._-]+` (branch-safe) |

## Starting a New Run After Goal Edit

To start fresh after modifying the goal:

```mermaid
sequenceDiagram
    participant U as User
    participant G as GOAL.md
    participant R as runner start
    participant B as Git Branch

    U->>G: Edit goal content
    U->>G: Change/remove id in frontmatter

    U->>R: runner start
    R->>G: Read id (or generate new one)
    R->>B: Create/checkout runner/<new-id>
    R->>G: Write new id to frontmatter
    R-->>U: New run started on new branch

    Note over B: Old branch remains<br/>intact for rollback
```

Steps:

1. Edit `.runner/GOAL.md` — change the `id:` value or remove it entirely
2. Run `runner start`
   - If ID removed: auto-generates new ID from HEAD sha
   - Creates new `runner/<new-id>` branch
   - Resets iteration state

The old branch remains intact for reference or rollback.

## Key Implementation Locations

| Component | File | Line | Purpose |
|-----------|------|------|---------|
| `start_run()` | `runner/src/start.rs` | 25 | Orchestrates run initialization |
| `generate_run_id()` | `runner/src/start.rs` | 99 | Creates unique run-<sha>[-suffix] |
| `enforce_git_policy_pre_step()` | `runner/src/step.rs` | 233 | Refuses main/master, requires clean tree |
| `enforce_run_id_matches_goal()` | `runner/src/step.rs` | 264 | Validates run_state vs GOAL.md |
| `enforce_on_run_branch()` | `runner/src/step.rs` | 280 | Validates current branch name |
| `read_goal_id()` | `runner/src/io/goal.rs` | 12 | Parses frontmatter id |
| `ensure_goal_id()` | `runner/src/io/goal.rs` | 20 | Writes id to frontmatter |
| `validate_id()` | `runner/src/io/goal.rs` | 28 | Validates id format |
