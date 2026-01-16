# Ralph Pin

This is the persistent “pin” for the repo. Keep it short and practical.

## Repo Map

- Entry points:
- Key directories:
- Key files:

## Architecture Doc

- Required: `.ralph/architecture.md` (keep ≤ 250 lines; no `RALPH_TODO` placeholders)

## Build/Test/Lint Commands

- Primary gates: `./.ralph/gates.sh`

## Runner Sync Behavior

- Default: container clones once at job start; no automatic syncing mid-run.
- If using `RALPH_SYNC_MODE=rebase`: the job fetches and rebases before each iteration; pushes may require `git push --force-with-lease`.

## Conventions / Gotchas

- Keep root `AGENTS.md` tiny (~70 lines target overall); only edit the `## Ralph Loop` section; move long notes to `.ralph/progress.md` or `.ralph/skills/*.md`.
- Conventional Commits required.
- Prefer small, verifiable user stories (one iteration each).
- Keep `.ralph/prd.json` `platformConstraints` accurate (especially if CI OS differs from supported OS).
