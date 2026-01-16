# Copy List (Legacy → Archived Snapshot)

This is the authoritative checklist of what we preserved from the pre-pivot repo.

Rule of thumb: prefer **copy verbatim** for small, high-signal artifacts (docs/schemas/templates), and use
**notes/excerpts** only when copying would add noise (binaries, caches, generated output).

## Docs (copy verbatim)

- [x] `docs/blueprint/` → `docs/legacy/blueprint/` (design docs + file contracts)
- [x] `docs/knowledge/` → `docs/legacy/knowledge/` (Ralph methodology references)

## Prompts (copy verbatim)

- [x] `docs/blueprint/prompts/` → `prompts/legacy/docs-blueprint/`
- [x] `internal/prompts/templates/` → `prompts/legacy/internal-templates/`

## Schemas (copy verbatim)

- [x] `schemas/board.schema.json` → `schemas/legacy/board.schema.json`

## Templates (copy verbatim)

- [x] `templates/.ralph/` → `templates/legacy/.ralph/` (legacy Ralph loop scaffold)

## Root Files (copy verbatim)

- [x] `AGENTS.md` → `legacy/root/AGENTS.md`
- [x] `README.md` → `legacy/root/README.md`
- [x] `PLAN.md` → `legacy/root/PLAN.md`
- [x] `justfile` → `legacy/root/justfile`
- [x] `go.mod` / `go.sum` → `legacy/root/`
- [x] `.gitignore` → `legacy/root/.gitignore`

## Code (copy + notes)

These are legacy implementation references; the new project is a reboot, so treat these as “read-only
reference” unless explicitly reused.

- [x] Copy `cmd/` → `legacy/code/cmd/`
- [x] Copy `internal/` → `legacy/code/internal/`
- [x] Write `legacy/code/NOTES.md` summarizing the key packages and what is reusable conceptually.

## Current locations (after unpack)

- `migration/legacy/docs/*` → `docs/legacy/*`
- `migration/legacy/prompts/*` → `prompts/legacy/*`
- `migration/legacy/schemas/*` → `schemas/legacy/*`
- `migration/legacy/templates/*` → `templates/legacy/*`
- `migration/legacy/code/*` → `legacy/code/*`
- `migration/legacy/root/*` → `legacy/root/*`

## Explicit Excludes (do NOT copy)

- `.git/`
- `.plan/`
- `.rumdl_cache/`
- `.DS_Store`
- Built binaries and build outputs (e.g., repo-root `ralph`, `bin/`, `dist/`)
- Language/package caches (`node_modules/`, `vendor/`, `__pycache__/`, `.venv/`)
