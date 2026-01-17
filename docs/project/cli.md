# Runner CLI (MVP)

This document specifies the user-facing behavior for MVP CLI commands.

## Exit Codes

- `0` — success / open leaf selected
- `1` — invalid layout/config/tree/run identity or other errors
- `2` — complete (no open leaf)
- `3` — stuck (attempts exhausted on selected leaf)

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
