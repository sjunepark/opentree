# Plan

## Background

This repository implements a local-only “Ralph” development-loop CLI. The spec pack lives under `docs/blueprint/`, and `.ralph/` starter scaffolding lives under `templates/.ralph/`.

The initial scaffold has been promoted into the repo root (`cmd/`, `internal/`, `schemas/`), and legacy migration packaging has been removed.

Key constraints: runs from an existing repo root that contains `.ralph/`, refuses to run on forbidden branches (`main`/`master`), requires a clean working tree by default, uses Codex CLI or Claude Code as the executor (fresh invocation per iteration), writes logs to `.ralph/logs/`, and never pushes.

## Intent and Goals

- Deliver a runnable `ralph` Go CLI implementing `ralph run` and `ralph board run` per the blueprint.
- Keep the loop orchestrator local-only, safe-by-default (branch + clean-tree preflight), and automation-friendly (no mandatory interactive prompts).
- Maintain strict config loading (global + project) with clear precedence and fail-fast validation.
- Provide clear documentation and examples for integrating `.ralph/` into target repos.

## Reasoning and Approach

We will build on the existing minimal Go scaffold to avoid framework lock-in and keep behavior explicit. The CLI will orchestrate iterations and validate file/contracts; the executor (Codex/Claude) will perform the actual “one story per iteration” work using embedded prompt templates.

For board mode we will keep the behavior agent-driven (the prompt manages card selection/state and per-card PRDs), while the CLI enforces schema-level validity (strict JSON + invariant checks) and loop stop conditions.

## Scope

- In:
  - Implement the canonical `ralph` CLI at repo root (`cmd/`, `internal/`, `schemas/`).
  - Implement/polish v0.1 commands: `ralph run`, `ralph board run`.
  - Strict config (global + project) with precedence, overrides, and validation.
  - Board JSON strict validation matching `schemas/board.schema.json`.
  - Documentation updates and minimal unit tests.
- Out:
  - Multi-repo orchestration, parallel execution, or remote CI runners.
  - Auto-push, PR creation, or GitHub integration.
  - Fully attended PRD authoring “interview” flows.
  - Container isolation (relies on executor sandboxing).

## Assumptions

- Target runtime is macOS (primary) with Go 1.22 available; Linux is “best effort”.
- Target repos provide `.ralph/prd.json` (and `.ralph/board.json` for board mode) and a repo-specific `.ralph/gates.sh` when needed.
- Executors are installed and authenticated locally; defaults are acceptable unless overridden via config.
- We will keep board-mode card selection/state changes in the agent prompts (not re-implement in Go).
- Canonical module path is `github.com/sjunepark/ralph`.
- Default branch behavior is to always create a new `ralph/*` branch (no interactive selection by default).

## Risks and Edge Cases

- Interactive branch-selection can block unattended runs; must provide a non-interactive path (flags/config defaults).
- Executor CLI flags/STDIN behavior can drift (especially `claude`); must keep argv override and provide clear errors.
- “No-progress” iterations (executor exits non-zero or makes no changes) can spin until max iterations; should detect and surface.
- Strict file validation: missing required JSON fields may silently decode unless explicitly checked; must enforce required/non-empty and patterns.
- Running in subdirectories or worktrees: repo-root detection must be correct and error messages actionable.

## Action Items

[x] Promote starter implementation to repo root (`cmd/`, `internal/`, `schemas/`) and move spec/docs to `docs/`.
[x] Update `go.mod` module path to `github.com/sjunepark/ralph` and update all Go imports accordingly.
[x] Replace root `README.md` to describe the actual `ralph` CLI (install, commands, config, `.ralph/` contract, safety rules), and add a brief “design/spec” pointer to `docs/blueprint/`.
[x] Add a small root `AGENTS.md` that matches the `.ralph/` contract (reserve a `## Ralph Loop` section and keep it high-signal).
[x] Fix config merging so booleans can be explicitly set false (e.g., `git.require_clean: false`) without being overwritten by defaults; keep strict YAML decoding.
[x] Implement config precedence completely: `flags > env > project .ralph/config.yaml > global ~/.config/ralph/config.yaml > defaults` (define env var names and document them).
[x] Make branch selection automation-friendly: default to always create a new `ralph/*` branch; add flags for “use current branch” (opt-in) and optional explicit branch name; keep interactive prompt only as a fallback.
[x] Harden `.ralph/board.json` strictness to match `schemas/board.schema.json` (required fields, non-empty strings, id regex, priority bounds, and exactly-one-in-progress rule).
[x] Improve error messages for missing `.ralph/`, missing `.ralph/prd.json`, missing `.ralph/board.json`, and executor-not-found cases; ensure failures are actionable.
[x] Add “no-progress” protection in the loop (detect unchanged `HEAD` after a successful executor run) to avoid silent spinning.
[x] Add unit tests for config precedence/merging and board validation; keep tests fast and local (`go test ./...`).
[x] Add developer commands/docs for formatting and checks (`gofmt`, `go test`), and ensure CI-ready behavior.
[x] Remove legacy `migration/` packaging from the repo.

## Validation

- Run `gofmt ./...` and `go test ./...` (and `go vet ./...` if enabled) from repo root.
- Manual smoke: run `ralph run` and `ralph board run` in a small dummy git repo containing a minimal `.ralph/` setup; verify branch/clean-tree refusal and log creation under `.ralph/logs/`.

## Dependencies

- Go 1.22 toolchain.
- `git` on PATH.
- Executor CLI installed/authenticated: `codex` and/or `claude` (with config `executor.command` override when needed).

## Open Questions

- None
