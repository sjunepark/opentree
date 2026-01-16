# Repo Checklist

Use this checklist to bootstrap local development and validate changes to the **local-only** `ralph` CLI (no Docker).

## 1) Toolchain

- Install Go (this repo targets `go 1.22`).
- Ensure `git` is available on PATH.
- Install at least one executor:
  - Codex CLI: `codex`
  - Claude Code: `claude`

## 2) Build

```bash
go install ./cmd/ralph
```

## 3) Run Checks

```bash
gofmt -w cmd internal
go test ./...
```

## 4) Manual Smoke (Suggested)

Create a small throwaway git repo with a minimal `.ralph/`:

- Copy `templates/.ralph/` into the repo’s `.ralph/`.
- Create a small `.ralph/prd.json` with at least one failing story.
- Run `ralph run` and verify:
  - It refuses on `main`/`master`.
  - It creates a new `ralph/*` branch by default.
  - It writes iteration logs under `.ralph/logs/`.

## 5) Spec Alignment

Keep these docs in sync with the implementation:

- `CLI_SPEC.md`
- `FILE_CONTRACTS.md`
- `CONFIG.md`
- `prompts/*`
- `schemas/*`
