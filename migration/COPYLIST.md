# Copy List (Legacy → `migration/legacy/`)

This is the authoritative checklist of what we preserve from the *current* repo before wiping it down to
only `migration/`.

Rule of thumb: prefer **copy verbatim** for small, high-signal artifacts (docs/schemas/templates), and use
**notes/excerpts** only when copying would add noise (binaries, caches, generated output).

## Docs (copy verbatim)

- [x] `docs/blueprint/` → `migration/legacy/docs/blueprint/` (design docs + file contracts)
- [x] `docs/knowledge/` → `migration/legacy/docs/knowledge/` (Ralph methodology references)

## Prompts (copy verbatim)

- [x] `docs/blueprint/prompts/` → `migration/legacy/prompts/docs-blueprint/`
- [x] `internal/prompts/templates/` → `migration/legacy/prompts/internal-templates/`

## Schemas (copy verbatim)

- [x] `schemas/board.schema.json` → `migration/legacy/schemas/board.schema.json`

## Templates (copy verbatim)

- [x] `templates/.ralph/` → `migration/legacy/templates/.ralph/` (legacy Ralph loop scaffold)

## Root Files (copy verbatim)

- [x] `AGENTS.md` → `migration/legacy/root/AGENTS.md`
- [x] `README.md` → `migration/legacy/root/README.md`
- [x] `PLAN.md` → `migration/legacy/root/PLAN.md`
- [x] `justfile` → `migration/legacy/root/justfile`
- [x] `go.mod` / `go.sum` → `migration/legacy/root/`
- [x] `.gitignore` → `migration/legacy/root/.gitignore`

## Code (copy + notes)

These are legacy implementation references; the new project is a reboot, so treat these as “read-only
reference” unless explicitly reused.

- [x] Copy `cmd/` → `migration/legacy/code/cmd/`
- [x] Copy `internal/` → `migration/legacy/code/internal/`
- [x] Write `migration/legacy/code/NOTES.md` summarizing the key packages and what is reusable conceptually.

## Explicit Excludes (do NOT copy)

- `.git/`
- `.plan/`
- `.rumdl_cache/`
- `.DS_Store`
- Built binaries and build outputs (e.g., repo-root `ralph`, `bin/`, `dist/`)
- Language/package caches (`node_modules/`, `vendor/`, `__pycache__/`, `.venv/`)
