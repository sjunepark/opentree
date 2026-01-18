# Eval Cases

Case files live under `eval/cases/*.toml` and are loaded by `eval list` / `eval run`.

## Schema

```toml
[case]
id = "calculator-go" # required, [a-z0-9_-] only
goal = ""               # required, non-empty

[config]
max_iterations = 30      # optional, > 0
max_attempts_default = 3 # optional, > 0

[config.guard]
command = ["just", "ci"] # optional override, non-empty

[env]
FOO = "bar"              # optional environment overrides for runner subprocesses

[[checks]]
type = "file_exists"
path = "main.go"

[[checks]]
type = "command_succeeds"
cmd = ["go", "build", "."]

[[checks]]
type = "runner_completed"
```

Notes:

- `checks` must be a non-empty array.
- `command_succeeds.cmd` must contain at least one entry.
- `case.id` must be path-safe and unique across case files.
- `env` values must be non-empty strings.
- The workspace `justfile` is auto-generated from `command_succeeds` checks and exposes a `ci` recipe.
