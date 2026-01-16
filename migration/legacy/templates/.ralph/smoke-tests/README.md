# Smoke Tests

Manual smoke-test checklists live here when a story cannot be fully automated in CI (e.g., OS-specific UI, proprietary tooling, hardware).

Guidelines:

- Name files by story id (example: `US-012.md`).
- Include prerequisites, concrete steps, and expected results.
- Reference the checklist from `.ralph/prd.json` using the story field `manualSmokeTestChecklist`.
