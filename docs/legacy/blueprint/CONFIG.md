# Config Model

## Files

- Project config: `.ralph/config.yaml`
- Global config (macOS default): `~/.config/ralph/config.yaml`

## Precedence

`flags > env > .ralph/config.yaml > global config > defaults`

## Strictness

- Unknown fields should be rejected (fail fast).

## Executor

Use a typed executor with an optional argv override:

```yaml
executor:
  kind: codex # or claude
  command: ["codex", "exec", "--sandbox", "danger-full-access", "-"] # optional override
```

Default behavior:

- `codex`: `codex exec --sandbox danger-full-access -` (prompt via stdin)
- `claude`: `claude -p --permission-mode acceptEdits ...` with `--settings` enabling sandboxing

Rationale:

- `kind` enables safe defaults and validation.
- `command` enables forward compatibility when tool flags change.

## Env Overrides

Environment variables can override config values (precedence still applies):

- `RALPH_EXECUTOR_KIND`
- `RALPH_EXECUTOR_COMMAND` (JSON array of argv strings)
- `RALPH_MAX_ITERATIONS`
- `RALPH_GIT_REQUIRE_CLEAN`
- `RALPH_GIT_BRANCH_PREFIX`
- `RALPH_STATE_LOGS_DIR`

## Example

See `examples/config.yaml`.
