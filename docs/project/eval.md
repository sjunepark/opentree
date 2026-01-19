# Evaluation Framework

The `eval` crate runs real runner loops against declarative test cases for local experimentation.

## Quick Start

```bash
just eval-list                    # List available cases
just eval-run calculator-go       # Run a case once
just eval-run calculator-go --runs 5  # Run multiple times
just eval-clean calculator-go     # Remove workspace and results
```

## Case Format

Cases live in `eval/cases/*.toml`. Each case defines a goal for the agent and checks to verify
the result.

```toml
[case]
id = "calculator-go"
goal = """
Create a very basic calculator CLI using Golang.
It should support add, subtract, multiply, divide.
Usage: ./calculator <num1> <op> <num2>
"""

[config]
max_iterations = 30
max_attempts_default = 3

[config.guard]
command = ["just", "ci"]

[env]
SOME_VAR = "value"

[[checks]]
type = "file_exists"
path = "main.go"

[[checks]]
type = "command_succeeds"
cmd = ["go", "build", "."]

[[checks]]
type = "runner_completed"
```

### Sections

| Section | Required | Description |
|---------|----------|-------------|
| `[case]` | Yes | `id` (slug, `[a-z0-9_-]`) and `goal` (task description) |
| `[config]` | No | Runner configuration overrides |
| `[config.guard]` | No | Custom guard command (default: `["just", "ci"]`) |
| `[env]` | No | Environment variables passed to runner |
| `[[checks]]` | Yes | At least one check required |

### Check Types

| Type | Fields | Passes when |
|------|--------|-------------|
| `file_exists` | `path` | File exists in workspace |
| `command_succeeds` | `cmd` (array) | Command exits 0 within timeout |
| `runner_completed` | — | Runner loop exited 0 |

## Execution Flow

```text
1. cargo build -p runner
2. Create isolated workspace
   └── Fresh git repo with generated justfile
3. runner start
4. Write case goal to .runner/GOAL.md
5. Apply config overrides to .runner/state/config.toml
6. git commit (clean starting point)
7. runner loop
8. Capture artifacts to eval/results/<case-id>/<eval-run-id>/
9. Run checks against workspace
10. Classify outcome
```

## Workspace Isolation

Each eval run gets a fresh, isolated git repository:

```text
eval/workspaces/
  calculator-go_20260118_120000_abc123/
    .git/
    justfile      # Generated from command_succeeds checks
    README.txt    # Case metadata
    .runner/      # Created by runner start
```

The `justfile` is auto-generated from `command_succeeds` checks so the runner's guard (`just ci`)
exercises the same verification commands:

```just
set shell := ["bash", "-eu", "-o", "pipefail", "-c"]

ci:
  @go build .
```

## Outcome Classification

Outcome depends on runner exit code and check results:

| Runner exit | Checks | Outcome | Meaning |
|-------------|--------|---------|---------|
| 0 | All pass | `success` | Goal achieved, all checks pass |
| 0 | Any fail | `fail` | Runner finished but checks don't pass |
| 3 | (any) | `stuck` | Runner hit max_attempts on a node |
| 1, 2, None | (any) | `error` | Runner internal error or crash |

Exit code semantics match the runner (see `docs/project/cli.md`):

- 0 = completed (root passes)
- 1 = internal error
- 2 = validation error
- 3 = stuck (max_attempts reached)

## Results Structure

Results are captured under `eval/results/<case-id>/<eval-run-id>/`:

```text
eval/results/calculator-go/eval-20260118_120000/
  meta.json           # Run metadata
  checks.json         # Check outcomes
  tree.json           # Final task tree
  run_state.json      # Runner state
  iterations/         # Full iteration logs
    run-abc123/
      1/
        output.json
        guard.log
        meta.json
  runner.start.log    # stdout/stderr from runner start
  runner.loop.log     # stdout/stderr from runner loop
```

### meta.json

```json
{
  "case_id": "calculator-go",
  "eval_run_id": "eval-20260118_120000",
  "case_hash": "sha256...",
  "runner_git_sha": "abc123...",
  "runner_binary": "/path/to/runner",
  "runner_run_id": "run-xyz789",
  "outcome": "success",
  "start_time": "2026-01-18T12:00:00Z",
  "end_time": "2026-01-18T12:05:30Z",
  "duration_secs": 330.5,
  "exit_code": 0,
  "workspace": "/path/to/workspace",
  "errors": []
}
```

### checks.json

```json
{
  "checks": [
    {
      "type": "file_exists",
      "path": "main.go",
      "passed": true
    },
    {
      "type": "command_succeeds",
      "cmd": ["go", "build", "."],
      "passed": true,
      "exit_code": 0,
      "timed_out": false,
      "stdout": "...",
      "stderr": "...",
      "stdout_truncated": false,
      "stderr_truncated": false
    },
    {
      "type": "runner_completed",
      "passed": true,
      "exit_code": 0
    }
  ]
}
```

## Command Limits

Commands in `command_succeeds` checks run with:

- **Timeout**: 60 seconds (hard kill after timeout)
- **Output limit**: 50KB per stream (stdout/stderr truncated beyond)

These limits prevent runaway commands from blocking evaluation.

## Design Rationale

1. **Real loops, not mocks**: Integration testing reveals issues that unit tests miss.

2. **Declarative cases**: TOML format is easy to write and version control.

3. **Workspace isolation**: Fresh git repos prevent cross-run interference and ensure
   reproducibility.

4. **Structured results**: JSON artifacts enable automated analysis and regression tracking.

5. **justfile generation**: The workspace guard (`just ci`) mirrors the checks, so agents see
   the same verification commands during execution that eval uses for final judgment.

## Open Questions

These are not yet decided and may affect future behavior:

1. **Missing toolchains**: If a case requires `go` but it's not installed, should the run be:
   - `failed` (check fails as expected), or
   - `skipped` (preflight detects missing deps and records skip)?

2. **Multiple runs strategy**: Should `eval run --runs N` create:
   - N isolated workspaces (current behavior, full isolation), or
   - 1 workspace reset between runs (faster, less disk)?
