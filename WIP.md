# WIP

## Now

- Completed `.plan/4` (`runner step` one full iteration, context/prompt/executor/guards/run state/logging).
- Completed `.plan/5` (testing harness + fixtures + deterministic scenario coverage).
- Implemented `runner start` (auto run-id, `runner/<run-id>` branch checkout, bootstrap commit).
- Enforced git policy + deterministic per-iteration commits in `runner step`.
- Implemented TOML runner config + shared per-iteration timeout/output caps; `runner step` commits even on post-start failures.
- Added run-level iteration limit (`max_iterations`) + `runner loop` command.
- Clarified attempt semantics: runner-internal failures do not increment `attempts` (only successful agent outputs do).
- Decided stuck-node policy: hard-stop (documented in `DECISIONS.md` + `ARCHITECTURE.md`).
- Enforced stuck-node hard-stop in `runner step` (exits non-zero when `attempts == max_attempts`).
- Completed `.plan/6` (MVP commands: `runner validate` + `runner select`, shared helpers, docs).
- Completed `.plan/7` (evaluation framework: eval crate, cases, harness, reporting).
- Recorded Codex integration decision: keep `codex exec`; use `--json` for streaming if needed; defer `app-server`/`mcp-server`.

## Next

- Draft next plans (from `ARCHITECTURE.md` open decisions + `IMPENDING.md` research):
- Plan: decide/implement optional `events.jsonl` emission (or explicitly defer).
- Plan: keep executor codex-only for now; defer Claude backend until needed.

## Notes

- Plan CLI needs `pyyaml` when run via `uv run --with pyyaml`.
- Open questions (eval / `.plan/7`):
  - Missing toolchains (ex: `go`): should a run be `failed` (check fails) or `skipped` (preflight detects missing deps and records skip)?
  - `eval run --runs N`: should we create N isolated workspaces (current behavior) or reuse/reset one workspace per run for speed?
