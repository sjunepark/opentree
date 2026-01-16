# Prompt: iteration_board

You are running inside a Ralph-style loop WITH a Kanban board.

Board file: `.ralph/board.json`
Per-card PRDs: `.ralph/prds/<card-id>/prd.json`
Active PRD file: `.ralph/prd.json` (copy of the active card PRD)
Per-card progress: `.ralph/prds/<card-id>/progress.md`

Before selecting a story:

1) Read `.ralph/board.json`.
2) If there is a card with `status: "in_progress"`, keep working that card.
3) Otherwise:
   - If any `todo` cards have numeric `priority`, choose the lowest numeric priority (tie-break by `id`) and set ONLY that card to `status: "in_progress"`.
   - Else (all `todo` cards have `priority: null`), choose a `todo` card yourself and set ONLY its `status: "in_progress"`.
   - Do NOT delete cards.
4) Ensure the active card has a canonical PRD at `.ralph/prds/<card-id>/prd.json`.
   - If missing, generate it unattended from the card title/description (right-sized stories, verifiable acceptance criteria).
5) Copy `.ralph/prds/<card-id>/prd.json` to `.ralph/prd.json`.

Then run a normal single-story Ralph iteration using `.ralph/prd.json`:

- Pick the highest-priority story with `passes: false`.
- Implement ONLY that story.
- Run gates.
- Update `.ralph/prd.json` completion fields.
- Copy updated `.ralph/prd.json` back to `.ralph/prds/<card-id>/prd.json` to keep canonical in sync.
- Append progress to `.ralph/prds/<card-id>/progress.md`.
- Commit changes.
- Do NOT push.

When all stories in the active card PRD have `passes: true`:

- Mark the active card `status: "done"` in `.ralph/board.json`.
- Commit.
- On the next iteration, move to the next card (if any).

If all cards are `done`, print exactly:
<promise>COMPLETE</promise>
