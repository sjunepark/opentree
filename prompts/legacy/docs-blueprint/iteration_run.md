# Prompt: iteration_run

You are running inside a Ralph-style loop.

Per iteration:

1) Read `.ralph/prd.json`, `.ralph/pin.md`, and `.ralph/architecture.md` (if present).
2) Maintain repo root `AGENTS.md`:
   - Only edit/add a `## Ralph Loop` section.
   - Keep it tiny and high-signal; move long notes to `.ralph/progress.md`.
3) Select the single highest-priority user story with `"passes": false`.
4) Implement ONLY that story end-to-end.
5) Run quality gates:
   - If `./.ralph/gates.sh` exists, run it and make it pass.
   - Otherwise infer the best available gates (typecheck/lint/tests) and make them pass.
6) Update `.ralph/prd.json`:
   - Set `"passes": true` ONLY for the story you completed.
   - Fill `completion.summary`, `completion.retro.*`, `completion.scopeAppropriate`, `completion.autoCompactOccurred`.
   - If the story requires manual validation, set `manualSmokeTestChecklist` and create/update the checklist file.
7) Append to `.ralph/progress.md` (if present; otherwise create it) with what changed, commands run, and gotchas.
8) Commit ALL changes with a Conventional Commit message.
9) Do NOT push.

Hard rules:

- Do NOT implement multiple stories in one iteration.
- Do NOT mark a story passing unless gates pass.
- Do NOT add secrets/credentials.

If there are no remaining stories with `"passes": false`:

- If any passed story has missing/empty `completion`, backfill completion fields (use `N/A` or `"null"` as plain text where needed), run gates, and commit.
- Otherwise, do not make changes. Print exactly:
<promise>COMPLETE</promise>
