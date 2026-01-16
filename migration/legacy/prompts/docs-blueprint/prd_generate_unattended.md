# Prompt: prd_generate_unattended

You are generating a per-card Ralph PRD unattended.

Input: a single card from `.ralph/board.json` with fields `id`, `title`, `description`.

Task:

- Create `.ralph/prds/<card-id>/prd.json`.
- Use the JSON-first PRD format:
  - `project`, `branchName`, `description`, `platformConstraints`, `userStories[]`.
- Write right-sized user stories (each fits in one context window).
- Each story must include verifiable acceptance criteria and quality gates (typecheck/lint/tests where applicable).
- Set all `passes: false` initially.
- Do NOT implement code.
- Commit the new PRD file.
- Do NOT push.
