# Proposal: Evaluation Framework for Runner

## Problem Statement

### The Challenge

The runner orchestrates AI agents (Codex) to achieve goals through iterative loops. However, we currently lack visibility into how well the system performs on real-world tasks:

1. **Non-determinism**: Agents produce different outputs for the same input. A single test run tells us little about actual reliability.

2. **No baseline**: We can't measure whether configuration changes (max_attempts, timeouts, prompt budgets) improve or degrade outcomes.

3. **Black box iteration**: When tweaking internal settings, we need repeated experiments to observe effects—currently done ad-hoc with no systematic capture.

4. **No regression detection**: If the runner "works" today, how do we know a code change doesn't break it tomorrow?

### Current Testing Gap

The existing test suite (`runner/tests/`) uses **mocked executors** for deterministic, fast unit/integration tests. This validates runner logic correctness but tells us nothing about:

- Does the system actually complete real goals?
- How many iterations does it take?
- What's the success rate across multiple runs?
- How do different settings affect outcomes?

We need a separate evaluation system that runs the runner against real goals with real agents.

## Goals

1. **Define reproducible test cases**: Specify goals, configurations, and success criteria in declarative files.

2. **Execute with real agents**: Run the full runner loop using Codex (not mocks) against actual tasks.

3. **Capture all artifacts**: Preserve tree states, agent outputs, guard logs, timing—everything needed for post-hoc analysis.

4. **Evaluate success**: Apply success criteria (file exists, command succeeds, runner completed) to judge outcomes.

5. **Aggregate across runs**: Compute statistics (success rate, avg iterations) across multiple runs of the same case.

6. **Enable iteration**: Make it easy to run experiments with different settings and compare results.

## Non-Goals

1. **CI integration**: This is for local experimentation only. Non-determinism makes CI unreliable.

2. **Parallelism**: Running multiple cases concurrently adds complexity without sufficient benefit.

3. **Automatic optimization**: We won't auto-tune settings—just measure and report.

4. **Coverage metrics**: We're measuring "does it work" not "how much code was exercised."

## Architecture

### Overview

```text
eval/                         # New crate
├── cases/                    # Test case definitions (TOML)
│   └── calculator-go.toml
├── workspaces/               # Execution workspaces (gitignored)
│   └── calculator-go_20240118_abc123/
├── results/                  # Captured artifacts (gitignored)
│   └── calculator-go/
│       └── abc123/
│           ├── meta.json     # Config, timing, outcome
│           ├── tree.json     # Final tree state
│           └── iterations/   # Per-iteration snapshots
└── src/
    ├── main.rs               # CLI entry point
    ├── case.rs               # Case definition + loader
    ├── harness.rs            # Execution harness
    ├── judge.rs              # Success criteria evaluation
    └── report.rs             # Aggregation + reporting
```

### Components

#### Test Case Definition

Declarative TOML files specifying what to test:

```toml
[case]
id = "calculator-go"
goal = """
Create a very basic calculator CLI using golang.
It should support add, subtract, multiply, divide.
Usage: ./calculator <num1> <op> <num2>
"""

[config]
max_iterations = 50
max_attempts_default = 3

[[checks]]
type = "file_exists"
path = "main.go"

[[checks]]
type = "command_succeeds"
cmd = ["go", "build", "."]

[[checks]]
type = "runner_completed"
```

#### Execution Harness

Orchestrates a single evaluation run:

1. Create isolated workspace directory
2. Initialize git repo and `.runner/` scaffold
3. Write goal to `.runner/GOAL.md`
4. Apply config overrides
5. Execute `runner start` then `runner loop` as subprocess
6. Capture exit code, timing, final state
7. Copy artifacts to results directory

**Key decision**: Invoke runner as subprocess (not library). Matches real usage and avoids coupling.

#### Judgment System

Evaluates success after runner completes:

```rust
pub enum Check {
    FileExists { path: PathBuf },        // Does this file exist?
    CommandSucceeds { cmd: Vec<String> }, // Does this command exit 0?
    RunnerCompleted,                      // Did runner finish without getting stuck?
}
```

Each check produces a `Judgment` with pass/fail and optional detail.

#### Reporting

Aggregates results across runs:

```text
Case: calculator-go
Runs: 5

┌──────────┬─────────┬───────────┬──────────┐
│ Outcome  │ Count   │ Avg Iter  │ Avg Time │
├──────────┼─────────┼───────────┼──────────┤
│ Success  │ 4 (80%) │ 12.3      │ 4m 23s   │
│ Stuck    │ 1 (20%) │ 30.0      │ 15m 00s  │
└──────────┴─────────┴───────────┴──────────┘

Checks:
  file_exists(main.go):     5/5 (100%)
  command_succeeds(go build): 4/5 (80%)
  runner_completed:         4/5 (80%)
```

### CLI Interface

```bash
eval run <case-id>      # Execute case, print result
eval list               # List available cases
eval report <case-id>   # Show aggregate stats
eval clean <case-id>    # Delete workspaces and results
```

## Execution Flow

```text
eval run calculator-go
        │
        ├─1─ Load cases/calculator-go.toml
        │
        ├─2─ Create workspace: workspaces/calculator-go_20240118_abc123/
        │     ├── git init
        │     ├── runner init
        │     └── Write GOAL.md
        │
        ├─3─ Apply config overrides
        │
        ├─4─ Execute: runner start && runner loop
        │     └── Real Codex execution (takes minutes)
        │
        ├─5─ Capture artifacts
        │     ├── Copy .runner/iterations/*
        │     ├── Copy .runner/state/*
        │     └── Record timing + exit code
        │
        ├─6─ Evaluate checks
        │     ├── file_exists(main.go) → pass
        │     ├── command_succeeds(go build) → pass
        │     └── runner_completed → pass
        │
        ├─7─ Write results to results/calculator-go/abc123/
        │
        └─8─ Print summary
             ✓ calculator-go: PASSED (12 iterations, 4m 23s)
```

## Implementation Details

### Dependencies

Same stack as runner for consistency:

- `anyhow` - Error handling
- `clap` - CLI parsing
- `serde` + `toml` - Case definition parsing
- `tracing` - Logging
- `chrono` - Timestamps for run IDs

### Workspace Management

- Location: `eval/workspaces/{case_id}_{timestamp}_{short_hash}/`
- Persistent by default (user runs `eval clean` to remove)
- Full git repo with `.runner/` scaffold
- Isolated from main project

### Results Storage

```text
results/calculator-go/abc123/
├── meta.json           # { config, start_time, end_time, exit_code, outcome }
├── tree.json           # Final .runner/state/tree.json
├── run_state.json      # Final .runner/state/run_state.json
├── checks.json         # Judgment results
└── iterations/         # Copy of .runner/iterations/{run-id}/
    ├── 001/
    │   ├── tree.before.json
    │   ├── tree.after.json
    │   └── output.json
    └── ...
```

### Config Override Mechanism

Case defines overrides; harness merges with runner defaults:

```toml
# In case file
[config]
max_iterations = 50
max_attempts_default = 5
```

Harness writes merged config to `.runner/state/config.toml` before starting.

## Key Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Separate crate | `eval/` sibling to `runner/` | Different concern; avoid bloating runner |
| Subprocess invocation | Yes | Matches real usage; simpler than library coupling |
| TOML case format | Yes | Consistent with runner config; human-readable |
| Persistent workspaces | Yes | Enables inspection; explicit cleanup |
| No parallelism | Correct | Complexity not justified for local use |
| No CI | Correct | Non-determinism makes CI flaky |

## Future Extensions (Not in Scope)

- **Variants**: Run same case with different config variants for A/B comparison
- **Custom validators**: Script-based checks for complex success criteria
- **Baseline comparison**: Compare current run against saved "golden" run
- **Agent swapping**: Use Claude instead of Codex for comparison

## Files to Create

1. `eval/Cargo.toml`
2. `eval/src/main.rs`
3. `eval/src/case.rs`
4. `eval/src/harness.rs`
5. `eval/src/judge.rs`
6. `eval/src/report.rs`
7. `eval/src/logging.rs`
8. `eval/cases/calculator-go.toml`
9. `eval/.gitignore`
10. Update root `Cargo.toml` (workspace member)
11. Update `justfile` (eval commands)
