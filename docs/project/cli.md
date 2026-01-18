# Runner CLI (MVP)

This document specifies the user-facing behavior for MVP CLI commands.

## Exit Codes

- `0` — success / open leaf selected
- `1` — invalid layout/config/tree/run identity or other errors
- `2` — complete (no open leaf)
- `3` — stuck (attempts exhausted on selected leaf)

## Terminology

- **Iteration**: one `runner step` execution (tracked by `.runner/state/run_state.json:next_iter`).
  - The runner enforces a run-level cap via `.runner/state/config.toml:max_iterations`.
- **Attempt**: a per-node retry counter (`attempts`/`max_attempts` in the task tree).
  - Attempts increment only when the agent outputs `retry`, or when it outputs `done` but guards fail.
  - A leaf is **stuck** when `passes == false` and `attempts == max_attempts`.

## `runner validate`

Checks:

- `.runner/` layout and required files
- `.runner/.gitignore` contains `context/` and `iterations/`
- `config.toml` parses and validates
- `tree.json` parses and validates (schema + invariants)
- Run identity when `run_state.run_id` is set:
  - `GOAL.md` frontmatter `id` matches `run_state.run_id`
  - current branch is `runner/<run_id>`

Output (plain text, stable `key=value` fields):

```text
validate: layout=ok
validate: config=ok
validate: tree=ok
validate: run=not-started
```

If run identity is present:

```text
validate: run=ok id=run-abc123 branch=runner/run-abc123
```

Notes:

- `runner validate` does not treat stuck leaves as an error; use `runner select` to detect stuck.

## `runner select`

Loads the current tree and prints the deterministic next leaf.

Output:

```text
select: status=open id=<id> path=<root/...> attempts=<n>/<max>
```

If complete:

```text
select: status=complete
```

If stuck:

```text
select: status=stuck id=<id> path=<root/...> attempts=<n>/<max>
```

Exit codes follow the table above.

## `runner step` (stuck hard-stop)

`runner step` exits with code `3` and prints a hard-stop error when the selected
leaf is stuck (`attempts == max_attempts` and `passes == false`).

Recovery options:

- increase `max_attempts` on the node
- decompose the node into smaller children
- abandon the goal and replace the node with a new plan

## `runner loop`

Runs deterministic iterations (`runner step`) repeatedly until one of:

- the tree is complete (no open leaf) → exit `0`
- a stuck leaf is selected → exit `3`
- the run exceeds `.runner/state/config.toml:max_iterations` → exit `1`

Output:

```text
loop: step run=<run-id> iter=<n> node=<id> status=<status> guard=<guard>
...
loop: status=complete run=<run-id> steps=<k> started_at_iter=<n>
```

If stuck:

```text
loop: status=stuck run=<run-id> id=<id> path=<root/...> attempts=<n>/<max>
```

If iteration limit reached:

```text
loop: status=limit run=<run-id> next_iter=<n> max_iterations=<max> steps=<k> started_at_iter=<n>
```
